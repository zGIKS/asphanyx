use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct DataApiTableAccessMetadataUpdateRequestResource {
    pub exposed: bool,
    pub read_enabled: bool,
    pub create_enabled: bool,
    pub update_enabled: bool,
    pub delete_enabled: bool,
    pub introspect_enabled: bool,
    pub authorization_mode: String,
}
