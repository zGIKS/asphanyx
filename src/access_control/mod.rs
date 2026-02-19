use std::sync::Arc;

use axum::Router;
use sqlx::PgPool;

use crate::{
    access_control::{
        application::{
            command_services::access_control_command_service_impl::AccessControlCommandServiceImpl,
            query_services::access_control_query_service_impl::AccessControlQueryServiceImpl,
        },
        infrastructure::persistence::repositories::postgres::{
            sqlx_authorization_decision_audit_repository_impl::SqlxAuthorizationDecisionAuditRepositoryImpl,
            sqlx_policy_rule_repository_impl::SqlxPolicyRuleRepositoryImpl,
            sqlx_role_assignment_repository_impl::SqlxRoleAssignmentRepositoryImpl,
        },
        interfaces::rest::controllers::access_control_rest_controller::{
            AccessControlRestControllerState, router,
        },
    },
    config::app_config::AppConfig,
};

pub mod application;
pub mod domain;
pub mod infrastructure;
pub mod interfaces;

pub async fn build_access_control_router(config: &AppConfig) -> Result<Router, String> {
    let admin_pool = PgPool::connect(&config.admin_database_url())
        .await
        .map_err(|e| e.to_string())?;

    let role_assignment_repository =
        Arc::new(SqlxRoleAssignmentRepositoryImpl::new(admin_pool.clone()));
    let policy_rule_repository = Arc::new(SqlxPolicyRuleRepositoryImpl::new(admin_pool.clone()));
    let audit_repository = Arc::new(SqlxAuthorizationDecisionAuditRepositoryImpl::new(
        admin_pool,
    ));

    let command_service = Arc::new(AccessControlCommandServiceImpl::new(
        role_assignment_repository.clone(),
        policy_rule_repository.clone(),
    ));
    let query_service = Arc::new(AccessControlQueryServiceImpl::new(
        policy_rule_repository,
        role_assignment_repository.clone(),
        audit_repository,
    ));

    Ok(router(AccessControlRestControllerState {
        command_service,
        query_service,
    }))
}
