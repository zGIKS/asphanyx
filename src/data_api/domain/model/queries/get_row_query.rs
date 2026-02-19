use crate::data_api::domain::model::{
    enums::{
        data_api_domain_error::DataApiDomainError, data_api_principal_type::DataApiPrincipalType,
    },
    value_objects::{
        api_version::ApiVersion, row_identifier::RowIdentifier, schema_name::SchemaName,
        table_name::TableName, tenant_id::TenantId,
    },
};

#[derive(Clone, Debug)]
pub struct GetRowQuery {
    api_version: ApiVersion,
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
    row_identifier: RowIdentifier,
    principal: String,
    principal_type: DataApiPrincipalType,
}

impl GetRowQuery {
    pub fn new(
        api_version: String,
        tenant_id: String,
        schema_name: String,
        table_name: String,
        row_identifier: String,
        principal: String,
        principal_type: DataApiPrincipalType,
    ) -> Result<Self, DataApiDomainError> {
        Ok(Self {
            api_version: ApiVersion::new(api_version)?,
            tenant_id: TenantId::new(tenant_id)?,
            schema_name: SchemaName::new(schema_name)?,
            table_name: TableName::new(table_name)?,
            row_identifier: RowIdentifier::new(row_identifier)?,
            principal,
            principal_type,
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
    pub fn row_identifier(&self) -> &RowIdentifier {
        &self.row_identifier
    }
    pub fn principal(&self) -> &str {
        &self.principal
    }
    pub fn principal_type(&self) -> DataApiPrincipalType {
        self.principal_type
    }
}
