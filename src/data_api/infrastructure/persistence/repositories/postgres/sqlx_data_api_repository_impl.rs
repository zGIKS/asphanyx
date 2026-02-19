use std::sync::Arc;

use serde_json::Value;
use sqlx::{PgPool, Postgres, QueryBuilder, Row};

use crate::data_api::{
    domain::model::{
        entities::table_schema_metadata::{TableColumnMetadata, TableSchemaMetadata},
        enums::data_api_domain_error::DataApiDomainError,
        value_objects::tenant_id::TenantId,
    },
    infrastructure::persistence::repositories::{
        data_api_repository::{
            CreateRowCriteria, DataApiRepository, DeleteRowCriteria, GetRowByPrimaryKeyCriteria,
            ListRowsCriteria, PatchRowCriteria,
        },
        tenant_connection_resolver_repository::TenantConnectionResolverRepository,
        tenant_pool_cache_repository::TenantPoolCacheRepository,
    },
};

pub struct SqlxDataApiRepositoryImpl {
    tenant_connection_resolver: Arc<dyn TenantConnectionResolverRepository>,
    tenant_pool_cache: Arc<dyn TenantPoolCacheRepository>,
}

impl SqlxDataApiRepositoryImpl {
    pub fn new(
        tenant_connection_resolver: Arc<dyn TenantConnectionResolverRepository>,
        tenant_pool_cache: Arc<dyn TenantPoolCacheRepository>,
    ) -> Self {
        Self {
            tenant_connection_resolver,
            tenant_pool_cache,
        }
    }

    async fn resolve_tenant_pool(
        &self,
        tenant_id: &TenantId,
    ) -> Result<PgPool, DataApiDomainError> {
        let database_url = self
            .tenant_connection_resolver
            .resolve_database_url(tenant_id)
            .await?;

        self.tenant_pool_cache
            .get_or_create_pool(&database_url)
            .await
    }

    fn quote_identifier(identifier: &str) -> Result<String, DataApiDomainError> {
        if identifier.is_empty()
            || !identifier
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
        {
            return Err(DataApiDomainError::InvalidQueryParameters);
        }

        Ok(format!("\"{}\"", identifier))
    }

    fn qualified_table(schema_name: &str, table_name: &str) -> Result<String, DataApiDomainError> {
        Ok(format!(
            "{}.{}",
            Self::quote_identifier(schema_name)?,
            Self::quote_identifier(table_name)?
        ))
    }
}

#[async_trait::async_trait]
impl DataApiRepository for SqlxDataApiRepositoryImpl {
    async fn introspect_table(
        &self,
        tenant_id: &TenantId,
        schema_name: &str,
        table_name: &str,
    ) -> Result<TableSchemaMetadata, DataApiDomainError> {
        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;

        let statement = r#"
            SELECT
                c.column_name,
                c.is_nullable,
                c.data_type,
                EXISTS (
                    SELECT 1
                    FROM information_schema.table_constraints tc
                    INNER JOIN information_schema.key_column_usage kcu
                        ON tc.constraint_name = kcu.constraint_name
                        AND tc.table_schema = kcu.table_schema
                    WHERE tc.table_schema = c.table_schema
                    AND tc.table_name = c.table_name
                    AND tc.constraint_type = 'PRIMARY KEY'
                    AND kcu.column_name = c.column_name
                ) AS is_primary_key
            FROM information_schema.columns c
            WHERE c.table_schema = $1 AND c.table_name = $2
            ORDER BY c.ordinal_position
        "#;

        let rows = sqlx::query(statement)
            .bind(schema_name)
            .bind(table_name)
            .fetch_all(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        if rows.is_empty() {
            return Err(DataApiDomainError::TableNotFound);
        }

        let columns = rows
            .into_iter()
            .map(|row| TableColumnMetadata {
                column_name: row.try_get::<String, _>("column_name").unwrap_or_default(),
                is_nullable: row
                    .try_get::<String, _>("is_nullable")
                    .unwrap_or_else(|_| "YES".to_string())
                    == "YES",
                data_type: row
                    .try_get::<String, _>("data_type")
                    .unwrap_or_else(|_| "text".to_string()),
                is_primary_key: row.try_get::<bool, _>("is_primary_key").unwrap_or(false),
            })
            .collect::<Vec<_>>();

        Ok(TableSchemaMetadata {
            schema_name: schema_name.to_string(),
            table_name: table_name.to_string(),
            columns,
        })
    }

    async fn list_rows(
        &self,
        tenant_id: &TenantId,
        criteria: ListRowsCriteria,
    ) -> Result<Value, DataApiDomainError> {
        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;
        let qualified_table = Self::qualified_table(&criteria.schema_name, &criteria.table_name)?;

        let selected_projection = if criteria.fields.is_empty() {
            "to_jsonb(t)".to_string()
        } else {
            let mut pairs = Vec::with_capacity(criteria.fields.len());
            for field in &criteria.fields {
                let quoted = Self::quote_identifier(field)?;
                pairs.push(format!("'{}', t.{}", field, quoted));
            }
            format!("jsonb_build_object({})", pairs.join(", "))
        };

        let mut builder = QueryBuilder::<Postgres>::new(format!(
            "SELECT COALESCE(jsonb_agg(payload), '[]'::jsonb) AS payload FROM (SELECT {} AS payload FROM {} AS t",
            selected_projection, qualified_table
        ));

        let mut has_where = false;
        for (column, value) in criteria.filters {
            let quoted = Self::quote_identifier(&column)?;
            if !has_where {
                builder.push(" WHERE ");
                has_where = true;
            } else {
                builder.push(" AND ");
            }
            builder.push(format!("t.{quoted}::text = "));
            builder.push_bind(value);
        }

        if let Some(order_by) = criteria.order_by {
            let quoted = Self::quote_identifier(&order_by)?;
            builder.push(format!(" ORDER BY t.{quoted} "));
            builder.push(if criteria.order_desc { "DESC" } else { "ASC" });
        }

        builder.push(" LIMIT ");
        builder.push_bind(criteria.limit);
        builder.push(" OFFSET ");
        builder.push_bind(criteria.offset);
        builder.push(") AS subq");

        let row = builder
            .build()
            .fetch_one(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        row.try_get("payload")
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))
    }

    async fn get_row_by_primary_key(
        &self,
        tenant_id: &TenantId,
        criteria: GetRowByPrimaryKeyCriteria,
    ) -> Result<Option<Value>, DataApiDomainError> {
        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;
        let qualified_table = Self::qualified_table(&criteria.schema_name, &criteria.table_name)?;
        let primary_key_column = Self::quote_identifier(&criteria.primary_key_column)?;

        let statement = format!(
            "SELECT to_jsonb(t) AS payload FROM {} AS t WHERE t.{}::text = $1",
            qualified_table, primary_key_column
        );

        let row = sqlx::query(&statement)
            .bind(criteria.primary_key_value)
            .fetch_optional(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        row.map(|r| {
            r.try_get("payload")
                .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))
        })
        .transpose()
    }

    async fn create_row(
        &self,
        tenant_id: &TenantId,
        criteria: CreateRowCriteria<'_>,
    ) -> Result<Value, DataApiDomainError> {
        if criteria.allowed_columns.is_empty() {
            return Err(DataApiDomainError::InvalidPayload);
        }

        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;
        let qualified_table = Self::qualified_table(criteria.schema_name, criteria.table_name)?;
        let columns_csv = criteria
            .allowed_columns
            .iter()
            .map(|c| Self::quote_identifier(c))
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        let statement = format!(
            "INSERT INTO {} AS t ({}) SELECT {} FROM jsonb_populate_record(NULL::{}, $1::jsonb) AS r RETURNING to_jsonb(t) AS payload",
            qualified_table,
            columns_csv,
            criteria
                .allowed_columns
                .iter()
                .map(|c| Self::quote_identifier(c))
                .collect::<Result<Vec<_>, _>>()?
                .iter()
                .map(|c| format!("r.{c}"))
                .collect::<Vec<_>>()
                .join(", "),
            qualified_table
        );

        let row = sqlx::query(&statement)
            .bind(criteria.payload)
            .fetch_one(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        row.try_get("payload")
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))
    }

    async fn patch_row(
        &self,
        tenant_id: &TenantId,
        criteria: PatchRowCriteria<'_>,
    ) -> Result<Option<Value>, DataApiDomainError> {
        if criteria.allowed_columns.is_empty() {
            return Err(DataApiDomainError::InvalidPayload);
        }

        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;
        let qualified_table = Self::qualified_table(criteria.schema_name, criteria.table_name)?;
        let primary_key_column = Self::quote_identifier(criteria.primary_key_column)?;

        let set_clause = criteria
            .allowed_columns
            .iter()
            .map(|c| {
                let quoted = Self::quote_identifier(c)?;
                Ok::<String, DataApiDomainError>(format!("{} = r.{}", quoted, quoted))
            })
            .collect::<Result<Vec<_>, _>>()?
            .join(", ");

        let statement = format!(
            "UPDATE {} AS t SET {} FROM jsonb_populate_record(NULL::{}, $1::jsonb) AS r WHERE t.{}::text = $2 RETURNING to_jsonb(t) AS payload",
            qualified_table, set_clause, qualified_table, primary_key_column
        );

        let row = sqlx::query(&statement)
            .bind(criteria.payload)
            .bind(criteria.primary_key_value)
            .fetch_optional(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        row.map(|r| {
            r.try_get("payload")
                .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))
        })
        .transpose()
    }

    async fn delete_row(
        &self,
        tenant_id: &TenantId,
        criteria: DeleteRowCriteria<'_>,
    ) -> Result<bool, DataApiDomainError> {
        let tenant_pool = self.resolve_tenant_pool(tenant_id).await?;
        let qualified_table = Self::qualified_table(criteria.schema_name, criteria.table_name)?;
        let primary_key_column = Self::quote_identifier(criteria.primary_key_column)?;

        let statement = format!(
            "DELETE FROM {} WHERE {}::text = $1",
            qualified_table, primary_key_column
        );

        let result = sqlx::query(&statement)
            .bind(criteria.primary_key_value)
            .execute(&tenant_pool)
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        Ok(result.rows_affected() > 0)
    }
}
