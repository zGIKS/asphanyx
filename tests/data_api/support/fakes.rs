use std::sync::Mutex;

use async_trait::async_trait;
use serde_json::{Value, json};
use swagger_axum_api::data_api::{
    domain::model::{
        entities::table_schema_metadata::{TableColumnMetadata, TableSchemaMetadata},
        enums::data_api_domain_error::DataApiDomainError,
        events::data_api_request_audited_event::DataApiRequestAuditedEvent,
        value_objects::{schema_name::SchemaName, tenant_id::TenantId},
    },
    infrastructure::persistence::repositories::{
        data_api_audit_log_repository::DataApiAuditLogRepository,
        data_api_repository::{
            CreateRowCriteria, DataApiRepository, DeleteRowCriteria, GetRowByPrimaryKeyCriteria,
            ListRowsCriteria, PatchRowCriteria,
        },
        tenant_schema_resolver_repository::TenantSchemaResolverRepository,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiAuthorizationCheckRequest,
    },
};

#[derive(Default)]
struct FakeDataApiRepositoryState {
    metadata: Option<TableSchemaMetadata>,
    create_calls: usize,
    patch_calls: usize,
    delete_calls: usize,
    list_calls: usize,
    get_calls: usize,
    introspect_calls: usize,
    last_tenant_for_create: Option<String>,
    last_tenant_for_list: Option<String>,
    last_list_criteria: Option<ListRowsCriteria>,
    create_should_fail: bool,
    patch_should_return_none: bool,
    get_should_return_none: bool,
}

pub struct FakeDataApiRepository {
    state: Mutex<FakeDataApiRepositoryState>,
}

impl FakeDataApiRepository {
    pub fn new() -> Self {
        let metadata = TableSchemaMetadata {
            schema_name: "public".to_string(),
            table_name: "productos".to_string(),
            columns: vec![
                TableColumnMetadata {
                    column_name: "id".to_string(),
                    is_nullable: false,
                    data_type: "integer".to_string(),
                    is_primary_key: true,
                },
                TableColumnMetadata {
                    column_name: "nombre".to_string(),
                    is_nullable: false,
                    data_type: "text".to_string(),
                    is_primary_key: false,
                },
                TableColumnMetadata {
                    column_name: "precio".to_string(),
                    is_nullable: false,
                    data_type: "numeric".to_string(),
                    is_primary_key: false,
                },
                TableColumnMetadata {
                    column_name: "image_url".to_string(),
                    is_nullable: true,
                    data_type: "text".to_string(),
                    is_primary_key: false,
                },
            ],
        };

        Self {
            state: Mutex::new(FakeDataApiRepositoryState {
                metadata: Some(metadata),
                ..FakeDataApiRepositoryState::default()
            }),
        }
    }

    pub fn set_create_should_fail(&self, value: bool) {
        self.state
            .lock()
            .expect("mutex poisoned")
            .create_should_fail = value;
    }

    pub fn set_patch_should_return_none(&self, value: bool) {
        self.state
            .lock()
            .expect("mutex poisoned")
            .patch_should_return_none = value;
    }

    pub fn set_get_should_return_none(&self, value: bool) {
        self.state
            .lock()
            .expect("mutex poisoned")
            .get_should_return_none = value;
    }

    pub fn set_without_primary_key(&self) {
        let mut state = self.state.lock().expect("mutex poisoned");
        if let Some(metadata) = &mut state.metadata {
            for column in &mut metadata.columns {
                column.is_primary_key = false;
            }
        }
    }

    pub fn create_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").create_calls
    }

    pub fn patch_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").patch_calls
    }

    pub fn last_tenant_for_create(&self) -> Option<String> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .last_tenant_for_create
            .clone()
    }

    pub fn last_list_criteria(&self) -> Option<ListRowsCriteria> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .last_list_criteria
            .clone()
    }

    pub fn last_tenant_for_list(&self) -> Option<String> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .last_tenant_for_list
            .clone()
    }
}

#[async_trait]
impl DataApiRepository for FakeDataApiRepository {
    async fn introspect_table(
        &self,
        _tenant_id: &TenantId,
        schema_name: &str,
        table_name: &str,
    ) -> Result<TableSchemaMetadata, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.introspect_calls += 1;

        let mut metadata = state
            .metadata
            .clone()
            .ok_or(DataApiDomainError::TableNotFound)?;
        metadata.schema_name = schema_name.to_string();
        metadata.table_name = table_name.to_string();
        Ok(metadata)
    }

    async fn list_rows(
        &self,
        tenant_id: &TenantId,
        criteria: ListRowsCriteria,
    ) -> Result<Value, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.list_calls += 1;
        state.last_tenant_for_list = Some(tenant_id.value().to_string());
        state.last_list_criteria = Some(criteria);
        Ok(json!([{"id": 1, "nombre": "producto demo"}]))
    }

    async fn get_row_by_primary_key(
        &self,
        _tenant_id: &TenantId,
        criteria: GetRowByPrimaryKeyCriteria,
    ) -> Result<Option<Value>, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.get_calls += 1;
        if state.get_should_return_none {
            return Ok(None);
        }

        Ok(Some(json!({
            criteria.primary_key_column: criteria.primary_key_value,
            "nombre": "producto demo"
        })))
    }

    async fn create_row(
        &self,
        tenant_id: &TenantId,
        criteria: CreateRowCriteria<'_>,
    ) -> Result<Value, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.create_calls += 1;
        state.last_tenant_for_create = Some(tenant_id.value().to_string());
        if state.create_should_fail {
            return Err(DataApiDomainError::InfrastructureError(
                "create failed".to_string(),
            ));
        }

        Ok(json!({
            "schema": criteria.schema_name,
            "table": criteria.table_name,
            "payload": criteria.payload,
            "columns": criteria.allowed_columns,
        }))
    }

    async fn patch_row(
        &self,
        _tenant_id: &TenantId,
        criteria: PatchRowCriteria<'_>,
    ) -> Result<Option<Value>, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.patch_calls += 1;
        if state.patch_should_return_none {
            return Ok(None);
        }

        Ok(Some(json!({
            "schema": criteria.schema_name,
            "table": criteria.table_name,
            "id": criteria.primary_key_value,
            "payload": criteria.payload,
        })))
    }

    async fn delete_row(
        &self,
        _tenant_id: &TenantId,
        _criteria: DeleteRowCriteria<'_>,
    ) -> Result<bool, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.delete_calls += 1;
        Ok(true)
    }
}

#[derive(Default)]
struct FakeTenantSchemaResolverState {
    calls: usize,
    last_tenant_id: Option<String>,
}

pub struct FakeTenantSchemaResolverRepository {
    schema_to_return: String,
    state: Mutex<FakeTenantSchemaResolverState>,
}

impl FakeTenantSchemaResolverRepository {
    pub fn new(schema_to_return: &str) -> Self {
        Self {
            schema_to_return: schema_to_return.to_string(),
            state: Mutex::new(FakeTenantSchemaResolverState::default()),
        }
    }

    pub fn calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").calls
    }
}

#[async_trait]
impl TenantSchemaResolverRepository for FakeTenantSchemaResolverRepository {
    async fn resolve_schema(
        &self,
        tenant_id: &TenantId,
        _requested_schema: Option<&str>,
    ) -> Result<SchemaName, DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.calls += 1;
        state.last_tenant_id = Some(tenant_id.value().to_string());
        SchemaName::new(self.schema_to_return.clone())
    }
}

#[derive(Clone, Debug)]
pub struct AccessCheckCall {
    pub tenant_id: String,
    pub principal: String,
    pub table_name: String,
    pub action_name: String,
    pub columns: Vec<String>,
    pub request_id: Option<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
}

#[derive(Default)]
struct FakeAccessControlState {
    calls: Vec<AccessCheckCall>,
    deny: bool,
}

pub struct FakeAccessControlFacade {
    state: Mutex<FakeAccessControlState>,
}

impl FakeAccessControlFacade {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakeAccessControlState::default()),
        }
    }

    pub fn calls(&self) -> Vec<AccessCheckCall> {
        self.state.lock().expect("mutex poisoned").calls.clone()
    }

    pub fn set_deny(&self, deny: bool) {
        self.state.lock().expect("mutex poisoned").deny = deny;
    }
}

#[async_trait]
impl AccessControlFacade for FakeAccessControlFacade {
    async fn check_table_permission(
        &self,
        request: DataApiAuthorizationCheckRequest,
    ) -> Result<(), DataApiDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.calls.push(AccessCheckCall {
            tenant_id: request.tenant_id,
            principal: request.principal_id,
            table_name: request.resource_name,
            action_name: request.action_name,
            columns: request.requested_columns,
            request_id: request.request_id,
            subject_owner_id: request.subject_owner_id,
            row_owner_id: request.row_owner_id,
        });

        if state.deny {
            return Err(DataApiDomainError::AccessDenied);
        }

        Ok(())
    }
}

pub struct FakeDataApiAuditLogRepository {
    events: Mutex<Vec<DataApiRequestAuditedEvent>>,
}

impl FakeDataApiAuditLogRepository {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn saved_events(&self) -> Vec<DataApiRequestAuditedEvent> {
        self.events.lock().expect("mutex poisoned").clone()
    }
}

#[async_trait]
impl DataApiAuditLogRepository for FakeDataApiAuditLogRepository {
    async fn save_event(
        &self,
        event: &DataApiRequestAuditedEvent,
    ) -> Result<(), DataApiDomainError> {
        self.events
            .lock()
            .expect("mutex poisoned")
            .push(event.clone());
        Ok(())
    }
}
