use async_trait::async_trait;

use crate::data_api::domain::model::{
    enums::data_api_domain_error::DataApiDomainError,
    value_objects::{schema_name::SchemaName, tenant_id::TenantId},
};

#[async_trait]
pub trait TenantSchemaResolverRepository: Send + Sync {
    async fn resolve_schema(
        &self,
        tenant_id: &TenantId,
        requested_schema: Option<&str>,
    ) -> Result<SchemaName, DataApiDomainError>;
}
