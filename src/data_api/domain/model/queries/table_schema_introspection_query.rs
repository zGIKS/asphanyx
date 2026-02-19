use crate::data_api::domain::model::{
    enums::data_api_domain_error::DataApiDomainError,
    value_objects::{schema_name::SchemaName, table_name::TableName, tenant_id::TenantId},
};

#[derive(Clone, Debug)]
pub struct TableSchemaIntrospectionQuery {
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
}

impl TableSchemaIntrospectionQuery {
    pub fn new(
        tenant_id: String,
        schema_name: String,
        table_name: String,
    ) -> Result<Self, DataApiDomainError> {
        Ok(Self {
            tenant_id: TenantId::new(tenant_id)?,
            schema_name: SchemaName::new(schema_name)?,
            table_name: TableName::new(table_name)?,
        })
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    pub fn schema_name(&self) -> &SchemaName {
        &self.schema_name
    }
    pub fn table_name(&self) -> &TableName {
        &self.table_name
    }
}
