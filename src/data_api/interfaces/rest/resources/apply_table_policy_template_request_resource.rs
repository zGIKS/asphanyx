use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct ApplyTablePolicyTemplateRequestResource {
    #[validate(length(min = 1))]
    pub template_name: String,
}
