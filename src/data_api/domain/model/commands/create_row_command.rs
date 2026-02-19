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
    request_id: Option<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
    payload: Value,
}

pub struct CreateRowCommandParts {
    pub api_version: String,
    pub tenant_id: String,
    pub schema_name: String,
    pub table_name: String,
    pub principal: String,
    pub principal_type: DataApiPrincipalType,
    pub request_id: Option<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub payload: Value,
}

impl CreateRowCommand {
    pub fn new(parts: CreateRowCommandParts) -> Result<Self, DataApiDomainError> {
        if !parts.payload.is_object() {
            return Err(DataApiDomainError::InvalidPayload);
        }

        Ok(Self {
            api_version: ApiVersion::new(parts.api_version)?,
            tenant_id: TenantId::new(parts.tenant_id)?,
            schema_name: SchemaName::new(parts.schema_name)?,
            table_name: TableName::new(parts.table_name)?,
            principal: parts.principal,
            principal_type: parts.principal_type,
            request_id: parts.request_id,
            subject_owner_id: parts.subject_owner_id,
            row_owner_id: parts.row_owner_id,
            payload: parts.payload,
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
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
    pub fn subject_owner_id(&self) -> Option<&str> {
        self.subject_owner_id.as_deref()
    }
    pub fn row_owner_id(&self) -> Option<&str> {
        self.row_owner_id.as_deref()
    }
    pub fn payload(&self) -> &Value {
        &self.payload
    }
}
