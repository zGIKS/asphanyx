use async_trait::async_trait;
use serde_json::Value;

use crate::data_api::domain::model::{
    enums::data_api_domain_error::DataApiDomainError,
    queries::{
        get_row_query::GetRowQuery, list_rows_query::ListRowsQuery,
        table_schema_introspection_query::TableSchemaIntrospectionQuery,
    },
};

#[async_trait]
pub trait DataApiQueryService: Send + Sync {
    async fn handle_list(&self, query: ListRowsQuery) -> Result<Value, DataApiDomainError>;
    async fn handle_get(&self, query: GetRowQuery) -> Result<Value, DataApiDomainError>;
    async fn handle_schema_introspection(
        &self,
        query: TableSchemaIntrospectionQuery,
    ) -> Result<Value, DataApiDomainError>;
}
