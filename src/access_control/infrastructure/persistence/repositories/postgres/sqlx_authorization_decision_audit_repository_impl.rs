use async_trait::async_trait;
use sqlx::PgPool;

use crate::access_control::{
    domain::model::{
        enums::access_control_domain_error::AccessControlDomainError,
        events::authorization_decision_audited_event::AuthorizationDecisionAuditedEvent,
    },
    infrastructure::persistence::repositories::authorization_decision_audit_repository::AuthorizationDecisionAuditRepository,
};

pub struct SqlxAuthorizationDecisionAuditRepositoryImpl {
    pool: PgPool,
}

impl SqlxAuthorizationDecisionAuditRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl AuthorizationDecisionAuditRepository for SqlxAuthorizationDecisionAuditRepositoryImpl {
    async fn save_decision(
        &self,
        event: &AuthorizationDecisionAuditedEvent,
    ) -> Result<(), AccessControlDomainError> {
        let statement = r#"
            INSERT INTO access_authorization_decision_audit (
                tenant_id,
                principal_id,
                request_id,
                resource_name,
                action_name,
                allowed,
                reason,
                occurred_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        "#;

        sqlx::query(statement)
            .bind(event.tenant_id)
            .bind(event.principal_id)
            .bind(&event.request_id)
            .bind(&event.resource_name)
            .bind(&event.action_name)
            .bind(event.allowed)
            .bind(&event.reason)
            .bind(event.occurred_at)
            .execute(&self.pool)
            .await
            .map_err(|e| AccessControlDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
