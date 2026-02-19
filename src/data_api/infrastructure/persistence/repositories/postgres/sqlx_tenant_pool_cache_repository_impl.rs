use std::{collections::HashMap, sync::Arc};

use async_trait::async_trait;
use sqlx::PgPool;
use tokio::sync::RwLock;

use crate::data_api::{
    domain::model::enums::data_api_domain_error::DataApiDomainError,
    infrastructure::persistence::repositories::tenant_pool_cache_repository::TenantPoolCacheRepository,
};

pub struct SqlxTenantPoolCacheRepositoryImpl {
    pools: Arc<RwLock<HashMap<String, PgPool>>>,
}

impl SqlxTenantPoolCacheRepositoryImpl {
    pub fn new() -> Self {
        Self {
            pools: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for SqlxTenantPoolCacheRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantPoolCacheRepository for SqlxTenantPoolCacheRepositoryImpl {
    async fn get_or_create_pool(&self, database_url: &str) -> Result<PgPool, DataApiDomainError> {
        {
            let read_guard = self.pools.read().await;
            if let Some(pool) = read_guard.get(database_url) {
                return Ok(pool.clone());
            }
        }

        let pool = PgPool::connect(database_url)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        let mut write_guard = self.pools.write().await;
        if let Some(existing) = write_guard.get(database_url) {
            return Ok(existing.clone());
        }

        write_guard.insert(database_url.to_string(), pool.clone());
        Ok(pool)
    }
}
