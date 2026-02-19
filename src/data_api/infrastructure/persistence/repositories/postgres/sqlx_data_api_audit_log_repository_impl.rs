use async_trait::async_trait;
use sqlx::PgPool;

use crate::data_api::{
    domain::model::{
        enums::data_api_domain_error::DataApiDomainError,
        events::data_api_request_audited_event::DataApiRequestAuditedEvent,
    },
    infrastructure::persistence::repositories::data_api_audit_log_repository::DataApiAuditLogRepository,
};

pub struct SqlxDataApiAuditLogRepositoryImpl {
    pool: PgPool,
}

impl SqlxDataApiAuditLogRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl DataApiAuditLogRepository for SqlxDataApiAuditLogRepositoryImpl {
    async fn save_event(
        &self,
        event: &DataApiRequestAuditedEvent,
    ) -> Result<(), DataApiDomainError> {
        let statement = r#"
            INSERT INTO data_api_audit_logs (
                tenant_id,
                request_id,
                schema_name,
                table_name,
                action,
                principal,
                success,
                status_code,
                details,
                occurred_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        "#;

        sqlx::query(statement)
            .bind(event.tenant_id)
            .bind(&event.request_id)
            .bind(&event.schema_name)
            .bind(&event.table_name)
            .bind(event.action.as_str())
            .bind(&event.principal)
            .bind(event.success)
            .bind(i32::from(event.status_code))
            .bind(&event.details)
            .bind(event.occurred_at)
            .execute(&self.pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}
