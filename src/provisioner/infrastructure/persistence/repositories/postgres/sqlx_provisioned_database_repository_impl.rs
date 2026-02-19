use async_trait::async_trait;
use chrono::{DateTime, Utc};
use sqlx::{PgPool, Row};
use std::str::FromStr;

use crate::provisioner::{
    domain::model::{
        entities::provisioned_database::ProvisionedDatabase,
        enums::{
            provisioned_database_status::ProvisionedDatabaseStatus,
            provisioner_domain_error::ProvisionerDomainError,
        },
        value_objects::{
            database_password_hash::DatabasePasswordHash, database_username::DatabaseUsername,
            provisioned_database_name::ProvisionedDatabaseName,
        },
    },
    infrastructure::persistence::repositories::provisioned_database_repository::ProvisionedDatabaseRepository,
};

pub struct SqlxProvisionedDatabaseRepositoryImpl {
    pool: PgPool,
}

impl SqlxProvisionedDatabaseRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    fn row_to_entity(
        row: sqlx::postgres::PgRow,
    ) -> Result<ProvisionedDatabase, ProvisionerDomainError> {
        let database_name_raw: String = row.try_get("database_name").map_err(map_infra_error)?;
        let username_raw: String = row.try_get("username").map_err(map_infra_error)?;
        let password_hash_raw: String = row.try_get("password_hash").map_err(map_infra_error)?;
        let status_raw: String = row.try_get("status").map_err(map_infra_error)?;
        let created_at: DateTime<Utc> = row.try_get("created_at").map_err(map_infra_error)?;

        let status = ProvisionedDatabaseStatus::from_str(&status_raw).map_err(|_| {
            ProvisionerDomainError::InfrastructureError("unknown status stored".to_string())
        })?;

        Ok(ProvisionedDatabase::restore(
            ProvisionedDatabaseName::new(database_name_raw)?,
            DatabaseUsername::new(username_raw)?,
            DatabasePasswordHash::new(password_hash_raw)?,
            status,
            created_at,
        ))
    }
}

#[async_trait]
impl ProvisionedDatabaseRepository for SqlxProvisionedDatabaseRepositoryImpl {
    async fn save(&self, database: &ProvisionedDatabase) -> Result<(), ProvisionerDomainError> {
        let statement = r#"
            INSERT INTO provisioned_databases (database_name, username, password_hash, status, created_at)
            VALUES ($1, $2, $3, $4, $5)
            ON CONFLICT (database_name)
            DO UPDATE SET
                username = EXCLUDED.username,
                password_hash = EXCLUDED.password_hash,
                status = EXCLUDED.status,
                created_at = EXCLUDED.created_at
        "#;

        sqlx::query(statement)
            .bind(database.database_name().value())
            .bind(database.username().value())
            .bind(database.password_hash().value())
            .bind(database.status().as_str())
            .bind(database.created_at())
            .execute(&self.pool)
            .await
            .map_err(map_infra_error)?;

        Ok(())
    }

    async fn find_by_name(
        &self,
        database_name: &ProvisionedDatabaseName,
    ) -> Result<Option<ProvisionedDatabase>, ProvisionerDomainError> {
        let statement = r#"
            SELECT database_name, username, password_hash, status, created_at
            FROM provisioned_databases
            WHERE database_name = $1
        "#;

        let maybe_row = sqlx::query(statement)
            .bind(database_name.value())
            .fetch_optional(&self.pool)
            .await
            .map_err(map_infra_error)?;

        maybe_row.map(Self::row_to_entity).transpose()
    }

    async fn list_all(&self) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError> {
        let statement = r#"
            SELECT database_name, username, password_hash, status, created_at
            FROM provisioned_databases
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(statement)
            .fetch_all(&self.pool)
            .await
            .map_err(map_infra_error)?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }

    async fn list_active_and_failed(
        &self,
    ) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError> {
        let statement = r#"
            SELECT database_name, username, password_hash, status, created_at
            FROM provisioned_databases
            WHERE status IN ('active', 'failed')
            ORDER BY created_at DESC
        "#;

        let rows = sqlx::query(statement)
            .fetch_all(&self.pool)
            .await
            .map_err(map_infra_error)?;

        rows.into_iter().map(Self::row_to_entity).collect()
    }
}

fn map_infra_error(error: sqlx::Error) -> ProvisionerDomainError {
    ProvisionerDomainError::InfrastructureError(error.to_string())
}
