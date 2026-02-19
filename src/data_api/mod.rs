use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use axum::Router;
use sqlx::PgPool;

use crate::{
    access_control::{
        application::{
            acl::access_control_facade_impl::AccessControlFacadeImpl,
            query_services::access_control_query_service_impl::AccessControlQueryServiceImpl,
        },
        infrastructure::persistence::repositories::postgres::{
            sqlx_authorization_decision_audit_repository_impl::SqlxAuthorizationDecisionAuditRepositoryImpl,
            sqlx_policy_rule_repository_impl::SqlxPolicyRuleRepositoryImpl,
            sqlx_role_assignment_repository_impl::SqlxRoleAssignmentRepositoryImpl,
        },
    },
    config::app_config::AppConfig,
    data_api::{
        application::{
            acl::access_control_facade_real_impl::AccessControlFacadeRealImpl,
            command_services::data_api_command_service_impl::DataApiCommandServiceImpl,
            query_services::data_api_query_service_impl::DataApiQueryServiceImpl,
        },
        infrastructure::persistence::repositories::postgres::{
            sqlx_data_api_audit_log_repository_impl::SqlxDataApiAuditLogRepositoryImpl,
            sqlx_data_api_repository_impl::SqlxDataApiRepositoryImpl,
            sqlx_tenant_connection_resolver_repository_impl::SqlxTenantConnectionResolverRepositoryImpl,
            sqlx_tenant_pool_cache_repository_impl::SqlxTenantPoolCacheRepositoryImpl,
            sqlx_tenant_schema_resolver_repository_impl::SqlxTenantSchemaResolverRepositoryImpl,
        },
        interfaces::rest::controllers::data_api_rest_controller::{
            DataApiRestControllerState, router,
        },
    },
};

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub async fn build_data_api_router(config: &AppConfig) -> Result<Router, String> {
    let admin_pool = PgPool::connect(&config.admin_database_url())
        .await
        .map_err(|e| e.to_string())?;

    let tenant_connection_resolver = Arc::new(SqlxTenantConnectionResolverRepositoryImpl::new(
        admin_pool.clone(),
        config.clone(),
    ));
    let tenant_pool_cache = Arc::new(SqlxTenantPoolCacheRepositoryImpl::new());
    let repository = Arc::new(SqlxDataApiRepositoryImpl::new(
        tenant_connection_resolver,
        tenant_pool_cache,
    ));
    let tenant_schema_resolver = Arc::new(SqlxTenantSchemaResolverRepositoryImpl::new());
    let audit_log_repository = Arc::new(SqlxDataApiAuditLogRepositoryImpl::new(admin_pool.clone()));
    let acl_role_assignment_repository =
        Arc::new(SqlxRoleAssignmentRepositoryImpl::new(admin_pool.clone()));
    let acl_policy_repository = Arc::new(SqlxPolicyRuleRepositoryImpl::new(admin_pool.clone()));
    let acl_audit_repository = Arc::new(SqlxAuthorizationDecisionAuditRepositoryImpl::new(
        admin_pool,
    ));
    let acl_query_service = Arc::new(AccessControlQueryServiceImpl::new(
        acl_policy_repository,
        acl_role_assignment_repository,
        acl_audit_repository,
    ));
    let access_control_facade = Arc::new(AccessControlFacadeRealImpl::new(Arc::new(
        AccessControlFacadeImpl::new(acl_query_service),
    )));

    let allowed_tables = read_allowed_tables();
    let editable_columns = read_editable_columns();

    let command_service = Arc::new(DataApiCommandServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver.clone(),
        access_control_facade.clone(),
        audit_log_repository.clone(),
        allowed_tables.clone(),
        editable_columns.clone(),
    ));
    let query_service = Arc::new(DataApiQueryServiceImpl::new(
        repository,
        tenant_schema_resolver,
        access_control_facade,
        audit_log_repository,
        allowed_tables,
    ));

    Ok(router(DataApiRestControllerState {
        command_service,
        query_service,
    }))
}

fn read_allowed_tables() -> HashSet<String> {
    std::env::var("DATA_API_ALLOWLIST_TABLES")
        .ok()
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<HashSet<_>>()
        })
        .unwrap_or_default()
}

fn read_editable_columns() -> HashMap<String, HashSet<String>> {
    std::env::var("DATA_API_EDITABLE_COLUMNS")
        .ok()
        .map(|raw| {
            raw.split(',')
                .filter_map(|entry| {
                    let (table, columns) = entry.split_once(':')?;
                    let cols = columns
                        .split('|')
                        .map(str::trim)
                        .filter(|c| !c.is_empty())
                        .map(|c| c.to_string())
                        .collect::<HashSet<_>>();
                    if table.trim().is_empty() {
                        None
                    } else {
                        Some((table.trim().to_string(), cols))
                    }
                })
                .collect::<HashMap<_, _>>()
        })
        .unwrap_or_default()
}
