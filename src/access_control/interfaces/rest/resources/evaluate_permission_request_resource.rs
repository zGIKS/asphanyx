use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct EvaluatePermissionRequestResource {
    #[validate(length(min = 1))]
    pub tenant_id: String,
    #[validate(length(min = 1))]
    pub principal_id: String,
    #[validate(length(min = 1))]
    pub resource_name: String,
    #[validate(length(min = 1))]
    pub action_name: String,
    pub requested_columns: Vec<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub request_id: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct EvaluatePermissionResponseResource {
    pub allowed: bool,
    pub reason: String,
}
