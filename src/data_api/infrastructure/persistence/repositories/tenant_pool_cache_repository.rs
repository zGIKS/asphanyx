use async_trait::async_trait;
use sqlx::PgPool;

use crate::data_api::domain::model::enums::data_api_domain_error::DataApiDomainError;

#[async_trait]
pub trait TenantPoolCacheRepository: Send + Sync {
    async fn get_or_create_pool(&self, database_url: &str) -> Result<PgPool, DataApiDomainError>;
}
