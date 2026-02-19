use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema, Validate)]
pub struct DataApiColumnAccessMetadataUpdateRequestResource {
    pub readable: bool,
    pub writable: bool,
}
