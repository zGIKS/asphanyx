use std::sync::Arc;

use axum::Router;
use sqlx::{PgPool, migrate};

use crate::{
    config::app_config::AppConfig,
    provisioner::{
        application::{
            command_services::database_provisioning_command_service_impl::DatabaseProvisioningCommandServiceImpl,
            query_services::database_provisioning_query_service_impl::DatabaseProvisioningQueryServiceImpl,
        },
        infrastructure::persistence::repositories::postgres::{
            sqlx_postgres_database_administration_repository_impl::SqlxPostgresDatabaseAdministrationRepositoryImpl,
            sqlx_provisioning_audit_event_repository_impl::SqlxProvisioningAuditEventRepositoryImpl,
            sqlx_provisioned_database_repository_impl::SqlxProvisionedDatabaseRepositoryImpl,
        },
        interfaces::rest::controllers::provisioner_rest_controller::{
            ProvisionerRestControllerState, router,
        },
    },
};

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub async fn build_provisioner_router(config: &AppConfig) -> Result<Router, String> {
    let admin_pool = PgPool::connect(&config.admin_database_url())
        .await
        .map_err(|e| e.to_string())?;

    migrate!("./migrations")
        .run(&admin_pool)
        .await
        .map_err(|e| e.to_string())?;

    let metadata_repository = Arc::new(SqlxProvisionedDatabaseRepositoryImpl::new(
        admin_pool.clone(),
    ));

    let postgres_administration_repository = Arc::new(
        SqlxPostgresDatabaseAdministrationRepositoryImpl::new(admin_pool.clone()),
    );
    let audit_event_repository = Arc::new(SqlxProvisioningAuditEventRepositoryImpl::new(
        admin_pool.clone(),
    ));

    let command_service = Arc::new(DatabaseProvisioningCommandServiceImpl::new(
        metadata_repository.clone(),
        postgres_administration_repository,
        audit_event_repository,
    ));
    let query_service = Arc::new(DatabaseProvisioningQueryServiceImpl::new(
        metadata_repository,
    ));

    Ok(router(ProvisionerRestControllerState {
        command_service,
        query_service,
    }))
}
