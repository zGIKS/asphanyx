use chrono::{DateTime, Utc};

#[derive(Clone, Debug)]
pub struct AuthorizationDecisionAuditedEvent {
    pub tenant_id: String,
    pub principal_id: String,
    pub request_id: Option<String>,
    pub resource_name: String,
    pub action_name: String,
    pub allowed: bool,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
}
