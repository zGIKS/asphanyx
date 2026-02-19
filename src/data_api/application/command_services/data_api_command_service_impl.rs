use std::{collections::HashSet, sync::Arc};

use async_trait::async_trait;
use chrono::Utc;
use serde_json::{Map, Value};
use uuid::Uuid;

use crate::data_api::{
    domain::{
        model::{
            commands::{
                create_row_command::CreateRowCommand, delete_row_command::DeleteRowCommand,
                patch_row_command::PatchRowCommand,
            },
            enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
            events::data_api_request_audited_event::DataApiRequestAuditedEvent,
        },
        services::data_api_command_service::DataApiCommandService,
    },
    infrastructure::persistence::repositories::{
        data_api_audit_log_repository::DataApiAuditLogRepository,
        data_api_repository::{
            CreateRowCriteria, DataApiRepository, DeleteRowCriteria, PatchRowCriteria,
        },
        tenant_schema_resolver_repository::TenantSchemaResolverRepository,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiAuthorizationBootstrapRequest, DataApiAuthorizationCheckRequest,
    },
};

const MAX_PAYLOAD_BYTES: usize = 64 * 1024;

pub struct DataApiCommandServiceImpl {
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

impl DataApiCommandServiceImpl {
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

    fn payload_columns(payload: &Value) -> Result<Vec<String>, DataApiDomainError> {
        let object = payload
            .as_object()
            .ok_or(DataApiDomainError::InvalidPayload)?;

        Ok(object.keys().cloned().collect::<Vec<_>>())
    }

    fn ensure_payload_size(payload: &Value) -> Result<(), DataApiDomainError> {
        if payload.to_string().len() > MAX_PAYLOAD_BYTES {
            return Err(DataApiDomainError::PayloadTooLarge);
        }

        Ok(())
    }

    fn ensure_editable_columns(
        editable: &HashSet<String>,
        payload_columns: &[String],
    ) -> Result<(), DataApiDomainError> {
        for column in payload_columns {
            if !editable.contains(column) {
                return Err(DataApiDomainError::NonEditableColumn(column.clone()));
            }
        }

        Ok(())
    }

    fn filter_allowed_payload(payload: &Value, allowed_columns: &[String]) -> Value {
        let mut map = Map::new();
        if let Some(object) = payload.as_object() {
            for column in allowed_columns {
                if let Some(value) = object.get(column) {
                    map.insert(column.clone(), value.clone());
                }
            }
        }
        Value::Object(map)
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
}

#[async_trait]
impl DataApiCommandService for DataApiCommandServiceImpl {
    async fn handle_create(&self, command: CreateRowCommand) -> Result<Value, DataApiDomainError> {
        Self::ensure_payload_size(command.payload())?;

        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(command.tenant_id(), Some(command.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(command.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.create_enabled, access_metadata.exposed)?;

        let requested_columns = Self::payload_columns(command.payload())?;
        let writable_columns = self
            .repository
            .list_writable_columns(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        Self::ensure_editable_columns(&writable_columns, &requested_columns)?;

        let metadata = self
            .repository
            .introspect_table(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        let allowed_columns = requested_columns
            .into_iter()
            .filter(|column| metadata.has_column(column))
            .collect::<Vec<_>>();

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                readable_columns: metadata
                    .columns
                    .iter()
                    .map(|column| column.column_name.clone())
                    .collect(),
                writable_columns: writable_columns.iter().cloned().collect(),
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                action_name: DataApiAction::Create.as_str().to_string(),
                requested_columns: allowed_columns.clone(),
                subject_owner_id: command.subject_owner_id().map(str::to_string),
                row_owner_id: command.row_owner_id().map(str::to_string),
                request_id: command.request_id().map(str::to_string),
            },
        )
        .await?;

        let filtered_payload = Self::filter_allowed_payload(command.payload(), &allowed_columns);
        let result = self
            .repository
            .create_row(
                command.tenant_id(),
                CreateRowCriteria {
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    payload: &filtered_payload,
                    allowed_columns: &allowed_columns,
                },
            )
            .await;

        match result {
            Ok(row) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Create,
                    principal: command.principal(),
                    success: true,
                    status_code: 201,
                    details: None,
                })
                .await;
                Ok(row)
            }
            Err(error) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Create,
                    principal: command.principal(),
                    success: false,
                    status_code: 500,
                    details: Some(error.to_string()),
                })
                .await;
                Err(error)
            }
        }
    }

    async fn handle_patch(&self, command: PatchRowCommand) -> Result<Value, DataApiDomainError> {
        Self::ensure_payload_size(command.payload())?;

        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(command.tenant_id(), Some(command.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(command.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.update_enabled, access_metadata.exposed)?;

        let requested_columns = Self::payload_columns(command.payload())?;
        let writable_columns = self
            .repository
            .list_writable_columns(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?
            .into_iter()
            .collect::<HashSet<_>>();
        Self::ensure_editable_columns(&writable_columns, &requested_columns)?;

        let metadata = self
            .repository
            .introspect_table(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        let primary_key = metadata
            .primary_key_column()
            .ok_or(DataApiDomainError::PrimaryKeyNotFound)?;

        let allowed_columns = requested_columns
            .into_iter()
            .filter(|column| metadata.has_column(column) && column != &primary_key.column_name)
            .collect::<Vec<_>>();

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                readable_columns: metadata
                    .columns
                    .iter()
                    .map(|column| column.column_name.clone())
                    .collect(),
                writable_columns: writable_columns.iter().cloned().collect(),
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                action_name: DataApiAction::Update.as_str().to_string(),
                requested_columns: allowed_columns.clone(),
                subject_owner_id: command.subject_owner_id().map(str::to_string),
                row_owner_id: command.row_owner_id().map(str::to_string),
                request_id: command.request_id().map(str::to_string),
            },
        )
        .await?;

        let filtered_payload = Self::filter_allowed_payload(command.payload(), &allowed_columns);

        let result = self
            .repository
            .patch_row(
                command.tenant_id(),
                PatchRowCriteria {
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    primary_key_column: &primary_key.column_name,
                    primary_key_value: command.row_identifier().value(),
                    payload: &filtered_payload,
                    allowed_columns: &allowed_columns,
                },
            )
            .await;

        match result {
            Ok(Some(row)) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Update,
                    principal: command.principal(),
                    success: true,
                    status_code: 200,
                    details: None,
                })
                .await;
                Ok(row)
            }
            Ok(None) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Update,
                    principal: command.principal(),
                    success: false,
                    status_code: 404,
                    details: Some("record not found".to_string()),
                })
                .await;
                Err(DataApiDomainError::RecordNotFound)
            }
            Err(error) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Update,
                    principal: command.principal(),
                    success: false,
                    status_code: 500,
                    details: Some(error.to_string()),
                })
                .await;
                Err(error)
            }
        }
    }

    async fn handle_delete(&self, command: DeleteRowCommand) -> Result<(), DataApiDomainError> {
        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(command.tenant_id(), Some(command.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(command.tenant_id(), schema_name.value())
            .await?;

        let access_metadata = self
            .repository
            .get_table_access_metadata(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        Self::ensure_action_allowed(access_metadata.delete_enabled, access_metadata.exposed)?;

        let metadata = self
            .repository
            .introspect_table(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        let primary_key = metadata
            .primary_key_column()
            .ok_or(DataApiDomainError::PrimaryKeyNotFound)?;

        self.enforce_acl_if_required(
            &access_metadata.authorization_mode,
            DataApiAuthorizationBootstrapRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                readable_columns: metadata
                    .columns
                    .iter()
                    .map(|column| column.column_name.clone())
                    .collect(),
                writable_columns: self
                    .repository
                    .list_writable_columns(
                        command.tenant_id(),
                        schema_name.value(),
                        command.table_name().value(),
                    )
                    .await?,
            },
            DataApiAuthorizationCheckRequest {
                tenant_id: command.tenant_id().value().to_string(),
                principal_id: command.principal().to_string(),
                resource_name: command.table_name().value().to_string(),
                action_name: DataApiAction::Delete.as_str().to_string(),
                requested_columns: vec![primary_key.column_name.clone()],
                subject_owner_id: command.subject_owner_id().map(str::to_string),
                row_owner_id: command.row_owner_id().map(str::to_string),
                request_id: command.request_id().map(str::to_string),
            },
        )
        .await?;

        match self
            .repository
            .delete_row(
                command.tenant_id(),
                DeleteRowCriteria {
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    primary_key_column: &primary_key.column_name,
                    primary_key_value: command.row_identifier().value(),
                },
            )
            .await
        {
            Ok(true) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Delete,
                    principal: command.principal(),
                    success: true,
                    status_code: 204,
                    details: None,
                })
                .await;
                Ok(())
            }
            Ok(false) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Delete,
                    principal: command.principal(),
                    success: false,
                    status_code: 404,
                    details: Some("record not found".to_string()),
                })
                .await;
                Err(DataApiDomainError::RecordNotFound)
            }
            Err(error) => {
                self.audit(AuditContext {
                    tenant_id: command.tenant_id().value(),
                    request_id: command.request_id().map(str::to_string),
                    schema_name: schema_name.value(),
                    table_name: command.table_name().value(),
                    action: DataApiAction::Delete,
                    principal: command.principal(),
                    success: false,
                    status_code: 500,
                    details: Some(error.to_string()),
                })
                .await;
                Err(error)
            }
        }
    }
}
