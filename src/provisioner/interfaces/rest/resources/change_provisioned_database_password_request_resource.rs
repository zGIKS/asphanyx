use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct ChangeProvisionedDatabasePasswordRequestResource {
    #[validate(length(min = 8))]
    pub password: String,
}
