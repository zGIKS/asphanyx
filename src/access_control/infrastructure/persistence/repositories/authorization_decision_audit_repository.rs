use async_trait::async_trait;

use crate::access_control::domain::model::{
    enums::access_control_domain_error::AccessControlDomainError,
    events::authorization_decision_audited_event::AuthorizationDecisionAuditedEvent,
};

#[async_trait]
pub trait AuthorizationDecisionAuditRepository: Send + Sync {
    async fn save_decision(
        &self,
        event: &AuthorizationDecisionAuditedEvent,
    ) -> Result<(), AccessControlDomainError>;
}
