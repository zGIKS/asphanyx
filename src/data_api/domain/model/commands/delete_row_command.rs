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
pub struct DeleteRowCommand {
    api_version: ApiVersion,
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
    row_identifier: RowIdentifier,
    principal: String,
    principal_type: DataApiPrincipalType,
    request_id: Option<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
}

pub struct DeleteRowCommandParts {
    pub api_version: String,
    pub tenant_id: String,
    pub schema_name: String,
    pub table_name: String,
    pub row_identifier: String,
    pub principal: String,
    pub principal_type: DataApiPrincipalType,
    pub request_id: Option<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
}

impl DeleteRowCommand {
    pub fn new(parts: DeleteRowCommandParts) -> Result<Self, DataApiDomainError> {
        Ok(Self {
            api_version: ApiVersion::new(parts.api_version)?,
            tenant_id: TenantId::new(parts.tenant_id)?,
            schema_name: SchemaName::new(parts.schema_name)?,
            table_name: TableName::new(parts.table_name)?,
            row_identifier: RowIdentifier::new(parts.row_identifier)?,
            principal: parts.principal,
            principal_type: parts.principal_type,
            request_id: parts.request_id,
            subject_owner_id: parts.subject_owner_id,
            row_owner_id: parts.row_owner_id,
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
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
    pub fn subject_owner_id(&self) -> Option<&str> {
        self.subject_owner_id.as_deref()
    }
    pub fn row_owner_id(&self) -> Option<&str> {
        self.row_owner_id.as_deref()
    }
}
