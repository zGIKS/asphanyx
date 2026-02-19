use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Clone, Deserialize, Serialize, Validate, ToSchema)]
pub struct DataApiPayloadResource {
    #[validate(custom(function = "validate_object_payload"))]
    pub payload: Value,
}

fn validate_object_payload(payload: &Value) -> Result<(), validator::ValidationError> {
    if payload.is_object() {
        Ok(())
    } else {
        Err(validator::ValidationError::new("payload_must_be_object"))
    }
}
