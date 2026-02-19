use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct AssignRoleRequestResource {
    #[validate(length(min = 1))]
    pub tenant_id: String,
    #[validate(length(min = 1))]
    pub principal_id: String,
    #[validate(length(min = 1))]
    pub role_name: String,
}
