use serde_json::Value;

use crate::data_api::domain::model::{
    enums::{
        data_api_domain_error::DataApiDomainError, data_api_principal_type::DataApiPrincipalType,
    },
    value_objects::{
        api_version::ApiVersion, schema_name::SchemaName, table_name::TableName,
        tenant_id::TenantId,
    },
};

#[derive(Clone, Debug)]
pub struct CreateRowCommand {
    api_version: ApiVersion,
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
    principal: String,
    principal_type: DataApiPrincipalType,
    payload: Value,
}

impl CreateRowCommand {
    pub fn new(
        api_version: String,
        tenant_id: String,
        schema_name: String,
        table_name: String,
        principal: String,
        principal_type: DataApiPrincipalType,
        payload: Value,
    ) -> Result<Self, DataApiDomainError> {
        if !payload.is_object() {
            return Err(DataApiDomainError::InvalidPayload);
        }

        Ok(Self {
            api_version: ApiVersion::new(api_version)?,
            tenant_id: TenantId::new(tenant_id)?,
            schema_name: SchemaName::new(schema_name)?,
            table_name: TableName::new(table_name)?,
            principal,
            principal_type,
            payload,
        })
    }

    pub fn api_version(&self) -> &ApiVersion {
        &self.api_version
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
    pub fn principal(&self) -> &str {
        &self.principal
    }
    pub fn principal_type(&self) -> DataApiPrincipalType {
        self.principal_type
    }
    pub fn payload(&self) -> &Value {
        &self.payload
    }
}
