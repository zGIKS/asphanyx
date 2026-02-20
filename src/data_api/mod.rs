use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    access_control::{
        application::{
            acl::access_control_facade_impl::AccessControlFacadeImpl,
            command_services::access_control_command_service_impl::AccessControlCommandServiceImpl,
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
            command_services::data_api_policy_template_command_service_impl::DataApiPolicyTemplateCommandServiceImpl,
            query_services::data_api_policy_template_query_service_impl::DataApiPolicyTemplateQueryServiceImpl,
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
    iam_integration::application::acl::grpc_iam_authentication_facade_impl::GrpcIamAuthenticationFacadeImpl,
    provisioner::infrastructure::persistence::repositories::postgres::sqlx_tenant_ownership_repository_impl::SqlxTenantOwnershipRepositoryImpl,
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
        admin_pool.clone(),
        tenant_connection_resolver,
        tenant_pool_cache,
    ));
    let tenant_schema_resolver = Arc::new(SqlxTenantSchemaResolverRepositoryImpl::new());
    let audit_log_repository = Arc::new(SqlxDataApiAuditLogRepositoryImpl::new(admin_pool.clone()));
    let acl_role_assignment_repository =
        Arc::new(SqlxRoleAssignmentRepositoryImpl::new(admin_pool.clone()));
    let acl_policy_repository = Arc::new(SqlxPolicyRuleRepositoryImpl::new(admin_pool.clone()));
    let acl_audit_repository = Arc::new(SqlxAuthorizationDecisionAuditRepositoryImpl::new(
        admin_pool.clone(),
    ));
    let tenant_ownership_repository =
        Arc::new(SqlxTenantOwnershipRepositoryImpl::new(admin_pool.clone()));
    let iam_authentication_facade = Arc::new(GrpcIamAuthenticationFacadeImpl::new(
        config.iam_grpc_endpoint.clone(),
        std::time::Duration::from_millis(config.iam_grpc_timeout_ms),
        std::time::Duration::from_secs(config.iam_token_cache_ttl_seconds),
        config.iam_grpc_circuit_breaker_failure_threshold,
        std::time::Duration::from_secs(config.iam_grpc_circuit_breaker_open_seconds),
    ));
    let acl_query_service = Arc::new(AccessControlQueryServiceImpl::new(
        acl_policy_repository.clone(),
        acl_role_assignment_repository.clone(),
        acl_audit_repository,
    ));
    let acl_command_service = Arc::new(AccessControlCommandServiceImpl::new(
        acl_role_assignment_repository.clone(),
        acl_policy_repository.clone(),
    ));
    let access_control_facade = Arc::new(AccessControlFacadeRealImpl::new(Arc::new(
        AccessControlFacadeImpl::new(acl_command_service, acl_query_service),
    )));

    let command_service = Arc::new(DataApiCommandServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver.clone(),
        access_control_facade.clone(),
        audit_log_repository.clone(),
    ));
    let query_service = Arc::new(DataApiQueryServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver.clone(),
        access_control_facade.clone(),
        audit_log_repository.clone(),
    ));
    let policy_template_command_service = Arc::new(DataApiPolicyTemplateCommandServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver,
        access_control_facade,
    ));
    let policy_template_query_service = Arc::new(DataApiPolicyTemplateQueryServiceImpl::new());

    Ok(router(DataApiRestControllerState {
        command_service,
        query_service,
        policy_template_command_service,
        policy_template_query_service,
        repository,
        iam_authentication_facade,
        tenant_ownership_repository,
    }))
}
