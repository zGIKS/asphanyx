use async_trait::async_trait;
use sqlx::{PgPool, Row};

use crate::{
    config::app_config::AppConfig,
    data_api::{
        domain::model::{
            enums::data_api_domain_error::DataApiDomainError, value_objects::tenant_id::TenantId,
        },
        infrastructure::persistence::repositories::tenant_connection_resolver_repository::TenantConnectionResolverRepository,
    },
};

pub struct SqlxTenantConnectionResolverRepositoryImpl {
    admin_pool: PgPool,
    config: AppConfig,
}

impl SqlxTenantConnectionResolverRepositoryImpl {
    pub fn new(admin_pool: PgPool, config: AppConfig) -> Self {
        Self { admin_pool, config }
    }
}

#[async_trait]
impl TenantConnectionResolverRepository for SqlxTenantConnectionResolverRepositoryImpl {
    async fn resolve_database_url(
        &self,
        tenant_id: &TenantId,
    ) -> Result<String, DataApiDomainError> {
        let statement = r#"
            SELECT database_name, status
            FROM provisioned_databases
            WHERE database_name = $1
        "#;

        let row = sqlx::query(statement)
            .bind(tenant_id.value())
            .fetch_optional(&self.admin_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?
            .ok_or(DataApiDomainError::TenantDatabaseNotFound)?;

        let database_name: String = row
            .try_get("database_name")
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;
        let status: String = row
            .try_get("status")
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        if status != "active" {
            return Err(DataApiDomainError::AccessDenied);
        }

        Ok(self.config.database_url_for(&database_name))
    }
}
