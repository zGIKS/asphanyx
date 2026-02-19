use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct ProvisionedDatabaseResource {
    pub id: String,
    pub database_name: String,
    pub username: String,
    pub status: String,
    pub created_at: String,
}
