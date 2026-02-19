use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct UpsertPolicyRuleRequestResource {
    #[validate(length(min = 1))]
    pub tenant_id: String,
    #[validate(length(min = 1))]
    pub role_name: String,
    #[validate(length(min = 1))]
    pub resource_name: String,
    #[validate(length(min = 1))]
    pub action_name: String,
    #[validate(length(min = 1))]
    pub effect: String,
    pub allowed_columns: Option<Vec<String>>,
    pub denied_columns: Option<Vec<String>>,
    pub owner_scope: bool,
}
