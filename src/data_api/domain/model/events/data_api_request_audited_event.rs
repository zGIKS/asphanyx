use chrono::{DateTime, Utc};

use crate::data_api::domain::model::enums::data_api_action::DataApiAction;

#[derive(Clone, Debug)]
pub struct DataApiRequestAuditedEvent {
    pub tenant_id: String,
    pub request_id: Option<String>,
    pub schema_name: String,
    pub table_name: String,
    pub action: DataApiAction,
    pub principal: String,
    pub success: bool,
    pub status_code: u16,
    pub details: Option<String>,
    pub occurred_at: DateTime<Utc>,
}

impl DataApiRequestAuditedEvent {
    pub fn with_occurred_at(mut self, occurred_at: DateTime<Utc>) -> Self {
        self.occurred_at = occurred_at;
        self
    }
}
