use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};
use uuid::Uuid;

use crate::data_api::{
    domain::{
        model::{
            enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
            events::data_api_request_audited_event::DataApiRequestAuditedEvent,
            queries::{
                get_row_query::GetRowQuery, list_rows_query::ListRowsQuery,
                table_schema_introspection_query::TableSchemaIntrospectionQuery,
            },
        },
        services::data_api_query_service::DataApiQueryService,
    },
    infrastructure::persistence::repositories::{
        data_api_audit_log_repository::DataApiAuditLogRepository,
        data_api_repository::{DataApiRepository, GetRowByPrimaryKeyCriteria, ListRowsCriteria},
        tenant_schema_resolver_repository::TenantSchemaResolverRepository,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiAuthorizationBootstrapRequest, DataApiAuthorizationCheckRequest,
    },
};

pub struct DataApiQueryServiceImpl {
    repository: Arc<dyn DataApiRepository>,
    tenant_schema_resolver: Arc<dyn TenantSchemaResolverRepository>,
    access_control_facade: Arc<dyn AccessControlFacade>,
    audit_log_repository: Arc<dyn DataApiAuditLogRepository>,
}

struct AuditContext<'a> {
    tenant_id: Uuid,
    request_id: Option<String>,
    schema_name: &'a str,
    table_name: &'a str,
    action: DataApiAction,
    principal: &'a str,
    success: bool,
    status_code: u16,
    details: Option<String>,
}

impl DataApiQueryServiceImpl {
    pub fn new(
        repository: Arc<dyn DataApiRepository>,
        tenant_schema_resolver: Arc<dyn TenantSchemaResolverRepository>,
        access_control_facade: Arc<dyn AccessControlFacade>,
        audit_log_repository: Arc<dyn DataApiAuditLogRepository>,
    ) -> Self {
        Self {
            repository,
            tenant_schema_resolver,
            access_control_facade,
            audit_log_repository,
        }
    }

    fn ensure_action_allowed(
        action_enabled: bool,
        table_exposed: bool,
    ) -> Result<(), DataApiDomainError> {
        if !table_exposed || !action_enabled {
            return Err(DataApiDomainError::TableNotAllowed);
        }

        Ok(())
    }

    async fn enforce_acl_if_required(
        &self,
        authorization_mode: &str,
        bootstrap_request: DataApiAuthorizationBootstrapRequest,
        request: DataApiAuthorizationCheckRequest,
    ) -> Result<(), DataApiDomainError> {
        if authorization_mode.eq_ignore_ascii_case("acl") {
            self.access_control_facade
                .bootstrap_table_access(bootstrap_request)
                .await?;
            self.access_control_facade
                .check_table_permission(request)
                .await?;
        }

        Ok(())
    }

    async fn audit(&self, context: AuditContext<'_>) {
        let _ = self
            .audit_log_repository
            .save_event(&DataApiRequestAuditedEvent {
                tenant_id: context.tenant_id,
                request_id: context.request_id,
                schema_name: context.schema_name.to_string(),
                table_name: context.table_name.to_string(),
                action: context.action,
                principal: context.principal.to_string(),
                success: context.success,
                status_code: context.status_code,
                details: context.details,
                occurred_at: Utc::now(),
            })
            .await;
    }
}

#[async_trait]
impl DataApiQueryService for DataApiQueryServiceImpl {
    async fn handle_list(&self, query: ListRowsQuery) -> Result<Value, DataApiDomainError> {
        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(query.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.read_enabled, access_metadata.exposed)?;

        let metadata = self
            .repository
            .introspect_table(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        let selected_fields = if query.select_fields().is_empty() {
            metadata
                .columns
                .iter()
                .map(|c| c.column_name.clone())
                .collect::<Vec<_>>()
        } else {
            query
                .select_fields()
                .iter()
                .filter(|field| metadata.has_column(field))
                .cloned()
                .collect::<Vec<_>>()
        };

        let mut filters = Vec::new();
        for (key, value) in query.filters() {
            if metadata.has_column(key) {
                filters.push((key.clone(), value.clone()));
            }
        }

        let order_by = query
            .order_by()
            .filter(|column| metadata.has_column(column))
            .map(str::to_string);

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                readable_columns: metadata
                    .columns
                    .iter()
                    .map(|column| column.column_name.clone())
                    .collect(),
                writable_columns: self
                    .repository
                    .list_writable_columns(
                        query.tenant_id(),
                        schema_name.value(),
                        query.table_name().value(),
                    )
                    .await?,
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                action_name: DataApiAction::Read.as_str().to_string(),
                requested_columns: selected_fields.clone(),
                subject_owner_id: query.subject_owner_id().map(str::to_string),
                row_owner_id: query.row_owner_id().map(str::to_string),
                request_id: query.request_id().map(str::to_string),
            },
        )
        .await?;

        let result = self
            .repository
            .list_rows(
                query.tenant_id(),
                ListRowsCriteria {
                    schema_name: schema_name.value().to_string(),
                    table_name: query.table_name().value().to_string(),
                    fields: selected_fields,
                    filters,
                    limit: query.limit(),
                    offset: query.offset(),
                    order_by,
                    order_desc: query.order_desc(),
                },
            )
            .await;

        match result {
            Ok(rows) => {
                self.audit(AuditContext {
                    tenant_id: query.tenant_id().value(),
                    request_id: query.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: query.table_name().value(),
                    action: DataApiAction::Read,
                    principal: query.principal(),
                    success: true,
                    status_code: 200,
                    details: None,
                })
                .await;
                Ok(rows)
            }
            Err(error) => {
                self.audit(AuditContext {
                    tenant_id: query.tenant_id().value(),
                    request_id: query.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: query.table_name().value(),
                    action: DataApiAction::Read,
                    principal: query.principal(),
                    success: false,
                    status_code: 500,
                    details: Some(error.to_string()),
                })
                .await;
                Err(error)
            }
        }
    }

    async fn handle_get(&self, query: GetRowQuery) -> Result<Value, DataApiDomainError> {
        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(query.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.read_enabled, access_metadata.exposed)?;

        let metadata = self
            .repository
            .introspect_table(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        let primary_key = metadata
            .primary_key_column()
            .ok_or(DataApiDomainError::PrimaryKeyNotFound)?;

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                readable_columns: metadata
                    .columns
                    .iter()
                    .map(|column| column.column_name.clone())
                    .collect(),
                writable_columns: self
                    .repository
                    .list_writable_columns(
                        query.tenant_id(),
                        schema_name.value(),
                        query.table_name().value(),
                    )
                    .await?,
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                action_name: DataApiAction::Read.as_str().to_string(),
                requested_columns: vec![],
                subject_owner_id: query.subject_owner_id().map(str::to_string),
                row_owner_id: query.row_owner_id().map(str::to_string),
                request_id: query.request_id().map(str::to_string),
            },
        )
        .await?;

        match self
            .repository
            .get_row_by_primary_key(
                query.tenant_id(),
                GetRowByPrimaryKeyCriteria {
                    schema_name: schema_name.value().to_string(),
                    table_name: query.table_name().value().to_string(),
                    primary_key_column: primary_key.column_name.clone(),
                    primary_key_value: query.row_identifier().value().to_string(),
                },
            )
            .await
        {
            Ok(Some(row)) => {
                self.audit(AuditContext {
                    tenant_id: query.tenant_id().value(),
                    request_id: query.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: query.table_name().value(),
                    action: DataApiAction::Read,
                    principal: query.principal(),
                    success: true,
                    status_code: 200,
                    details: None,
                })
                .await;
                Ok(row)
            }
            Ok(None) => {
                self.audit(AuditContext {
                    tenant_id: query.tenant_id().value(),
                    request_id: query.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: query.table_name().value(),
                    action: DataApiAction::Read,
                    principal: query.principal(),
                    success: false,
                    status_code: 404,
                    details: Some("record not found".to_string()),
                })
                .await;
                Err(DataApiDomainError::RecordNotFound)
            }
            Err(error) => {
                self.audit(AuditContext {
                    tenant_id: query.tenant_id().value(),
                    request_id: query.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: query.table_name().value(),
                    action: DataApiAction::Read,
                    principal: query.principal(),
                    success: false,
                    status_code: 500,
                    details: Some(error.to_string()),
                })
                .await;
                Err(error)
            }
        }
    }

    async fn handle_schema_introspection(
        &self,
        query: TableSchemaIntrospectionQuery,
    ) -> Result<Value, DataApiDomainError> {
        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(query.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.introspect_enabled, access_metadata.exposed)?;

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                readable_columns: vec![],
                writable_columns: vec![],
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal().to_string(),
                resource_name: query.table_name().value().to_string(),
                action_name: DataApiAction::Read.as_str().to_string(),
                requested_columns: vec![],
                subject_owner_id: query.subject_owner_id().map(str::to_string),
                row_owner_id: query.row_owner_id().map(str::to_string),
                request_id: query.request_id().map(str::to_string),
            },
        )
        .await?;

        let metadata = self
            .repository
            .introspect_table(
                query.tenant_id(),
                schema_name.value(),
                query.table_name().value(),
            )
            .await?;

        Ok(json!({
            "schema": metadata.schema_name,
            "table": metadata.table_name,
            "columns": metadata.columns.into_iter().map(|c| json!({
                "name": c.column_name,
                "data_type": c.data_type,
                "nullable": c.is_nullable,
                "primary_key": c.is_primary_key
            })).collect::<Vec<_>>()
        }))
    }
}
