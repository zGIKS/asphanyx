use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct PolicyTemplateCatalogResource {
    pub template_name: String,
    pub authorization_mode: String,
    pub read_enabled: bool,
    pub create_enabled: bool,
    pub update_enabled: bool,
    pub delete_enabled: bool,
    pub introspect_enabled: bool,
}
