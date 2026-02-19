use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Value, json};

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
    interfaces::acl::access_control_facade::AccessControlFacade,
};

pub struct DataApiQueryServiceImpl {
    repository: Arc<dyn DataApiRepository>,
    tenant_schema_resolver: Arc<dyn TenantSchemaResolverRepository>,
    access_control_facade: Arc<dyn AccessControlFacade>,
    audit_log_repository: Arc<dyn DataApiAuditLogRepository>,
    allowed_tables: HashSet<String>,
}

struct AuditContext<'a> {
    tenant_id: &'a str,
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
        allowed_tables: HashSet<String>,
    ) -> Self {
        Self {
            repository,
            tenant_schema_resolver,
            access_control_facade,
            audit_log_repository,
            allowed_tables,
        }
    }

    fn ensure_table_allowed(&self, table_name: &str) -> Result<(), DataApiDomainError> {
        if self.allowed_tables.is_empty() || !self.allowed_tables.contains(table_name) {
            return Err(DataApiDomainError::TableNotAllowed);
        }

        Ok(())
    }

    async fn audit(&self, context: AuditContext<'_>) {
        let _ = self
            .audit_log_repository
            .save_event(&DataApiRequestAuditedEvent {
                tenant_id: context.tenant_id.to_string(),
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
        self.ensure_table_allowed(query.table_name().value())?;

        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
            .await?;

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

        self.access_control_facade
            .check_table_permission(
                query.tenant_id(),
                query.principal(),
                query.table_name(),
                DataApiAction::Read,
                &selected_fields,
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
        self.ensure_table_allowed(query.table_name().value())?;

        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
            .await?;

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

        self.access_control_facade
            .check_table_permission(
                query.tenant_id(),
                query.principal(),
                query.table_name(),
                DataApiAction::Read,
                &[],
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
        self.ensure_table_allowed(query.table_name().value())?;

        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(query.tenant_id(), Some(query.schema_name().value()))
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
