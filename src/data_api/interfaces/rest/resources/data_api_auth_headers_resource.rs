use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

#[derive(Debug, Clone, Deserialize, Serialize, ToSchema)]
pub struct DataApiAuthHeadersResource {
    pub x_tenant_id: String,
    pub x_tenant_schema: Option<String>,
    pub x_api_key: Option<String>,
    pub authorization: Option<String>,
}
