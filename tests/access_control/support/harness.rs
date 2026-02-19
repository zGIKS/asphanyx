use std::{sync::Arc, time::Duration};

use swagger_axum_api::access_control::application::{
    command_services::access_control_command_service_impl::AccessControlCommandServiceImpl,
    query_services::access_control_query_service_impl::AccessControlQueryServiceImpl,
};

use super::fakes::{
    FakeAuthorizationDecisionAuditRepository, FakePolicyRuleRepository,
    FakeRoleAssignmentRepository,
};

pub struct AccessControlCommandHarness {
    pub role_repository: Arc<FakeRoleAssignmentRepository>,
    pub policy_repository: Arc<FakePolicyRuleRepository>,
    pub service: AccessControlCommandServiceImpl,
}

pub struct AccessControlQueryHarness {
    pub role_repository: Arc<FakeRoleAssignmentRepository>,
    pub policy_repository: Arc<FakePolicyRuleRepository>,
    pub audit_repository: Arc<FakeAuthorizationDecisionAuditRepository>,
    pub service: AccessControlQueryServiceImpl,
}

pub fn create_command_harness() -> AccessControlCommandHarness {
    let role_repository = Arc::new(FakeRoleAssignmentRepository::new());
    let policy_repository = Arc::new(FakePolicyRuleRepository::new());

    let service =
        AccessControlCommandServiceImpl::new(role_repository.clone(), policy_repository.clone());

    AccessControlCommandHarness {
        role_repository,
        policy_repository,
        service,
    }
}

pub fn create_query_harness(cache_ttl: Duration) -> AccessControlQueryHarness {
    let role_repository = Arc::new(FakeRoleAssignmentRepository::new());
    let policy_repository = Arc::new(FakePolicyRuleRepository::new());
    let audit_repository = Arc::new(FakeAuthorizationDecisionAuditRepository::new());

    let service = AccessControlQueryServiceImpl::new_with_cache_ttl(
        policy_repository.clone(),
        role_repository.clone(),
        audit_repository.clone(),
        cache_ttl,
    );

    AccessControlQueryHarness {
        role_repository,
        policy_repository,
        audit_repository,
        service,
    }
}
