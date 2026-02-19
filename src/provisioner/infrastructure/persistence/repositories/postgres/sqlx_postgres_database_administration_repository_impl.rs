use async_trait::async_trait;
use sqlx::PgPool;

use crate::provisioner::{
    domain::model::{
        enums::provisioner_domain_error::ProvisionerDomainError,
        value_objects::{
            database_password::DatabasePassword,
            database_username::DatabaseUsername,
            provisioned_database_name::ProvisionedDatabaseName,
        },
    },
    infrastructure::persistence::repositories::postgres_database_administration_repository::PostgresDatabaseAdministrationRepository,
};

pub struct SqlxPostgresDatabaseAdministrationRepositoryImpl {
    admin_pool: PgPool,
}

impl SqlxPostgresDatabaseAdministrationRepositoryImpl {
    pub fn new(admin_pool: PgPool) -> Self {
        Self { admin_pool }
    }

    async fn run_statement(&self, statement: &str) -> Result<(), ProvisionerDomainError> {
        sqlx::query(statement)
            .execute(&self.admin_pool)
            .await
            .map_err(|e| ProvisionerDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn run_statement_on_database(
        &self,
        database_name: &str,
        statement: &str,
    ) -> Result<(), ProvisionerDomainError> {
        let options = sqlx::postgres::PgConnectOptions::new()
            .host(&std::env::var("POSTGRES_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()))
            .port(
                std::env::var("POSTGRES_PORT")
                    .unwrap_or_else(|_| "5432".to_string())
                    .parse()
                    .unwrap_or(5432),
            )
            .username(&std::env::var("POSTGRES_USER").unwrap_or_else(|_| "postgres".to_string()))
            .password(&std::env::var("POSTGRES_PASSWORD").unwrap_or_else(|_| "admin".to_string()))
            .database(database_name);

        let db_pool = PgPool::connect_with(options)
            .await
            .map_err(|e| ProvisionerDomainError::InfrastructureError(e.to_string()))?;

        sqlx::query(statement)
            .execute(&db_pool)
            .await
            .map_err(|e| ProvisionerDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }
}

#[async_trait]
impl PostgresDatabaseAdministrationRepository for SqlxPostgresDatabaseAdministrationRepositoryImpl {
    async fn create_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
        password: &DatabasePassword,
        apply_seed_data: bool,
    ) -> Result<(), ProvisionerDomainError> {
        let db_identifier = database_name.value();
        let user_identifier = username.value();
        let escaped_password = password.value().replace('\'', "''");

        self.run_statement(&format!(
            "CREATE ROLE {user_identifier} LOGIN PASSWORD '{escaped_password}'"
        ))
        .await?;

        self.run_statement(&format!(
            "CREATE DATABASE {db_identifier} OWNER {user_identifier}"
        ))
        .await?;

        self.run_statement(&format!(
            "GRANT CONNECT ON DATABASE {db_identifier} TO {user_identifier}"
        ))
        .await?;

        self.run_statement_on_database(
            db_identifier,
            &format!("GRANT USAGE ON SCHEMA public TO {user_identifier}"),
        )
        .await?;

        self.run_statement_on_database(
            db_identifier,
            &format!(
                "GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO {user_identifier}"
            ),
        )
        .await?;

        self.run_statement_on_database(
            db_identifier,
            &format!(
                "ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO {user_identifier}"
            ),
        )
        .await?;

        let _ = apply_seed_data;

        Ok(())
    }

    async fn delete_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError> {
        let db_identifier = database_name.value();
        let user_identifier = username.value();

        self.run_statement(&format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{db_identifier}' AND pid <> pg_backend_pid()"
        ))
        .await?;

        self.run_statement(&format!("DROP DATABASE IF EXISTS {db_identifier}"))
            .await?;

        self.run_statement(&format!("DROP ROLE IF EXISTS {user_identifier}"))
            .await?;

        Ok(())
    }

    async fn rollback_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError> {
        self.delete_database_stack(database_name, username)
            .await
    }
}
