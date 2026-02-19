use async_trait::async_trait;

use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[derive(Clone, Debug)]
pub struct DataApiAuthorizationCheckRequest {
    pub tenant_id: String,
    pub principal_id: String,
    pub resource_name: String,
    pub action_name: String,
    pub requested_columns: Vec<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub request_id: Option<String>,
}

#[async_trait]
pub trait AccessControlFacade: Send + Sync {
    async fn check_table_permission(
        &self,
        request: DataApiAuthorizationCheckRequest,
    ) -> Result<(), DataApiDomainError>;
}
