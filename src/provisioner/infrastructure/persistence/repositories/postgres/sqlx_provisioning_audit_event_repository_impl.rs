use async_trait::async_trait;
use sqlx::PgPool;

use crate::provisioner::{
    domain::model::enums::provisioner_domain_error::ProvisionerDomainError,
    infrastructure::persistence::repositories::provisioning_audit_event_repository::{
        ProvisioningAuditEventRecord, ProvisioningAuditEventRepository,
    },
};

pub struct SqlxProvisioningAuditEventRepositoryImpl {
    pool: PgPool,
}

impl SqlxProvisioningAuditEventRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProvisioningAuditEventRepository for SqlxProvisioningAuditEventRepositoryImpl {
    async fn save_event(
        &self,
        event: &ProvisioningAuditEventRecord,
    ) -> Result<(), ProvisionerDomainError> {
        let statement = r#"
            INSERT INTO provisioning_audit_events (
                event_name,
                database_name,
                username,
                status,
                error_message,
                occurred_at
            )
            VALUES ($1, $2, $3, $4, $5, $6)
        "#;

        sqlx::query(statement)
            .bind(event.event_name())
            .bind(event.database_name())
            .bind(event.username())
            .bind(event.status())
            .bind(event.error_message())
            .bind(event.occurred_at())
            .execute(&self.pool)
            .await
            .map_err(|e| ProvisionerDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
