use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct CreateProvisionedDatabaseRequestResource {
    #[validate(length(min = 3, max = 63), regex(path = "*DATABASE_IDENTIFIER_REGEX"))]
    pub database_name: String,

    pub apply_seed_data: bool,
}

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ListProvisionedDatabasesQueryResource {
    pub include_deleted: Option<bool>,
}

lazy_static::lazy_static! {
    pub static ref DATABASE_IDENTIFIER_REGEX: regex::Regex = regex::Regex::new("^[a-z][a-z0-9_]{2,62}$").expect("valid regex");
}
