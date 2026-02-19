use std::collections::BTreeMap;

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
pub struct ListRowsQuery {
    api_version: ApiVersion,
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
    principal: String,
    principal_type: DataApiPrincipalType,
    request_id: Option<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
    select_fields: Vec<String>,
    filters: BTreeMap<String, String>,
    limit: i64,
    offset: i64,
    order_by: Option<String>,
    order_desc: bool,
}

pub struct ListRowsQueryParts {
    pub api_version: String,
    pub tenant_id: String,
    pub schema_name: String,
    pub table_name: String,
    pub principal: String,
    pub principal_type: DataApiPrincipalType,
    pub request_id: Option<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub select_fields: Vec<String>,
    pub filters: BTreeMap<String, String>,
    pub limit: i64,
    pub offset: i64,
    pub order_by: Option<String>,
    pub order_desc: bool,
}

impl ListRowsQuery {
    pub fn new(parts: ListRowsQueryParts) -> Result<Self, DataApiDomainError> {
        if parts.limit <= 0 || parts.limit > 500 || parts.offset < 0 {
            return Err(DataApiDomainError::InvalidQueryParameters);
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
            select_fields: parts.select_fields,
            filters: parts.filters,
            limit: parts.limit,
            offset: parts.offset,
            order_by: parts.order_by,
            order_desc: parts.order_desc,
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
    pub fn select_fields(&self) -> &[String] {
        &self.select_fields
    }
    pub fn filters(&self) -> &BTreeMap<String, String> {
        &self.filters
    }
    pub fn limit(&self) -> i64 {
        self.limit
    }
    pub fn offset(&self) -> i64 {
        self.offset
    }
    pub fn order_by(&self) -> Option<&str> {
        self.order_by.as_deref()
    }
    pub fn order_desc(&self) -> bool {
        self.order_desc
    }
}
