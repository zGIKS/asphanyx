use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DataApiTableAccessCatalogEntryResource {
    pub table_name: String,
    pub exposed: bool,
    pub read_enabled: bool,
    pub create_enabled: bool,
    pub update_enabled: bool,
    pub delete_enabled: bool,
    pub introspect_enabled: bool,
    pub authorization_mode: String,
    pub writable_columns: Vec<String>,
}
