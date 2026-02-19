use async_trait::async_trait;
use serde_json::Value;

use crate::data_api::domain::model::{
    entities::table_schema_metadata::TableSchemaMetadata,
    enums::data_api_domain_error::DataApiDomainError, value_objects::tenant_id::TenantId,
};

#[derive(Clone, Debug)]
pub struct ListRowsCriteria {
    pub schema_name: String,
    pub table_name: String,
    pub fields: Vec<String>,
    pub filters: Vec<(String, String)>,
    pub limit: i64,
    pub offset: i64,
    pub order_by: Option<String>,
    pub order_desc: bool,
}

#[derive(Clone, Debug)]
pub struct GetRowByPrimaryKeyCriteria {
    pub schema_name: String,
    pub table_name: String,
    pub primary_key_column: String,
    pub primary_key_value: String,
}

pub struct CreateRowCriteria<'a> {
    pub schema_name: &'a str,
    pub table_name: &'a str,
    pub payload: &'a Value,
    pub allowed_columns: &'a [String],
}

pub struct PatchRowCriteria<'a> {
    pub schema_name: &'a str,
    pub table_name: &'a str,
    pub primary_key_column: &'a str,
    pub primary_key_value: &'a str,
    pub payload: &'a Value,
    pub allowed_columns: &'a [String],
}

pub struct DeleteRowCriteria<'a> {
    pub schema_name: &'a str,
    pub table_name: &'a str,
    pub primary_key_column: &'a str,
    pub primary_key_value: &'a str,
}

#[async_trait]
pub trait DataApiRepository: Send + Sync {
    async fn introspect_table(
        &self,
        tenant_id: &TenantId,
        schema_name: &str,
        table_name: &str,
    ) -> Result<TableSchemaMetadata, DataApiDomainError>;

    async fn list_rows(
        &self,
        tenant_id: &TenantId,
        criteria: ListRowsCriteria,
    ) -> Result<Value, DataApiDomainError>;

    async fn get_row_by_primary_key(
        &self,
        tenant_id: &TenantId,
        criteria: GetRowByPrimaryKeyCriteria,
    ) -> Result<Option<Value>, DataApiDomainError>;

    async fn create_row(
        &self,
        tenant_id: &TenantId,
        criteria: CreateRowCriteria<'_>,
    ) -> Result<Value, DataApiDomainError>;

    async fn patch_row(
        &self,
        tenant_id: &TenantId,
        criteria: PatchRowCriteria<'_>,
    ) -> Result<Option<Value>, DataApiDomainError>;

    async fn delete_row(
        &self,
        tenant_id: &TenantId,
        criteria: DeleteRowCriteria<'_>,
    ) -> Result<bool, DataApiDomainError>;
}
