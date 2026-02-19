use async_trait::async_trait;

use crate::data_api::domain::model::{
    enums::data_api_domain_error::DataApiDomainError, value_objects::tenant_id::TenantId,
};

#[async_trait]
pub trait TenantConnectionResolverRepository: Send + Sync {
    async fn resolve_database_url(
        &self,
        tenant_id: &TenantId,
    ) -> Result<String, DataApiDomainError>;
}
