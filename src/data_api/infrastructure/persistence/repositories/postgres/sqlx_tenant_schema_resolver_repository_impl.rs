use async_trait::async_trait;

use crate::data_api::{
    domain::model::{
        enums::data_api_domain_error::DataApiDomainError,
        value_objects::{schema_name::SchemaName, tenant_id::TenantId},
    },
    infrastructure::persistence::repositories::tenant_schema_resolver_repository::TenantSchemaResolverRepository,
};

pub struct SqlxTenantSchemaResolverRepositoryImpl;

impl SqlxTenantSchemaResolverRepositoryImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for SqlxTenantSchemaResolverRepositoryImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TenantSchemaResolverRepository for SqlxTenantSchemaResolverRepositoryImpl {
    async fn resolve_schema(
        &self,
        _tenant_id: &TenantId,
        requested_schema: Option<&str>,
    ) -> Result<SchemaName, DataApiDomainError> {
        let schema = requested_schema.unwrap_or("public").to_string();
        SchemaName::new(schema)
    }
}
