use async_trait::async_trait;

use crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError;

#[derive(Clone, Debug)]
pub struct AccessControlPermissionRequest {
    pub tenant_id: String,
    pub principal_id: String,
    pub resource_name: String,
    pub action_name: String,
    pub requested_columns: Vec<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Clone, Debug)]
pub struct AccessControlPermissionDecision {
    pub allowed: bool,
    pub reason: String,
}

#[derive(Clone, Debug)]
pub struct DataApiAccessBootstrapRequest {
    pub tenant_id: String,
    pub principal_id: String,
    pub resource_name: String,
    pub readable_columns: Vec<String>,
    pub writable_columns: Vec<String>,
}

#[async_trait]
pub trait AccessControlFacade: Send + Sync {
    async fn check_permission(
        &self,
        request: AccessControlPermissionRequest,
    ) -> Result<AccessControlPermissionDecision, AccessControlDomainError>;

    async fn bootstrap_data_api_access(
        &self,
        request: DataApiAccessBootstrapRequest,
    ) -> Result<(), AccessControlDomainError>;
}
