use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DataApiAuthHeadersResource {
    pub x_tenant_id: String,
    pub x_tenant_schema: Option<String>,
    pub authorization: String,
    pub x_api_key: Option<String>,
    pub x_request_id: Option<String>,
    pub x_subject_owner_id: Option<String>,
    pub x_row_owner_id: Option<String>,
}
