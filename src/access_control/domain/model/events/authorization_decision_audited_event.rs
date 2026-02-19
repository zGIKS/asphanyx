use chrono::{DateTime, Utc};
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct AuthorizationDecisionAuditedEvent {
    pub tenant_id: Uuid,
    pub principal_id: Uuid,
    pub request_id: Option<String>,
    pub resource_name: String,
    pub action_name: String,
    pub allowed: bool,
    pub reason: String,
    pub occurred_at: DateTime<Utc>,
}
