use std::sync::Mutex;

use async_trait::async_trait;
use swagger_axum_api::access_control::{
    domain::model::{
        enums::access_control_domain_error::AccessControlDomainError,
        events::authorization_decision_audited_event::AuthorizationDecisionAuditedEvent,
        value_objects::{
            action_name::ActionName, principal_id::PrincipalId, resource_name::ResourceName,
            role_name::RoleName, tenant_id::TenantId,
        },
    },
    infrastructure::persistence::repositories::{
        authorization_decision_audit_repository::AuthorizationDecisionAuditRepository,
        policy_rule_repository::{PolicyRuleRecord, PolicyRuleRepository},
        role_assignment_repository::RoleAssignmentRepository,
    },
};

#[derive(Default)]
struct FakeRoleAssignmentState {
    assign_calls: usize,
    find_calls: usize,
    roles_by_principal: Vec<String>,
}

pub struct FakeRoleAssignmentRepository {
    state: Mutex<FakeRoleAssignmentState>,
}

impl FakeRoleAssignmentRepository {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakeRoleAssignmentState::default()),
        }
    }

    pub fn set_roles(&self, roles: Vec<String>) {
        self.state
            .lock()
            .expect("mutex poisoned")
            .roles_by_principal = roles;
    }

    pub fn assign_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").assign_calls
    }

    pub fn find_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").find_calls
    }
}

#[async_trait]
impl RoleAssignmentRepository for FakeRoleAssignmentRepository {
    async fn assign_role(
        &self,
        _tenant_id: &TenantId,
        _principal_id: &PrincipalId,
        _role_name: &RoleName,
    ) -> Result<(), AccessControlDomainError> {
        self.state.lock().expect("mutex poisoned").assign_calls += 1;
        Ok(())
    }

    async fn find_roles_by_principal(
        &self,
        _tenant_id: &TenantId,
        _principal_id: &PrincipalId,
    ) -> Result<Vec<String>, AccessControlDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.find_calls += 1;
        Ok(state.roles_by_principal.clone())
    }
}

#[derive(Default)]
struct FakePolicyRuleState {
    upsert_calls: usize,
    find_calls: usize,
    last_upsert: Option<PolicyRuleRecord>,
    rules_to_return: Vec<PolicyRuleRecord>,
}

pub struct FakePolicyRuleRepository {
    state: Mutex<FakePolicyRuleState>,
}

impl FakePolicyRuleRepository {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakePolicyRuleState::default()),
        }
    }

    pub fn set_rules(&self, rules: Vec<PolicyRuleRecord>) {
        self.state.lock().expect("mutex poisoned").rules_to_return = rules;
    }

    pub fn upsert_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").upsert_calls
    }

    pub fn find_calls(&self) -> usize {
        self.state.lock().expect("mutex poisoned").find_calls
    }

    pub fn last_upsert(&self) -> Option<PolicyRuleRecord> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .last_upsert
            .clone()
    }
}

#[async_trait]
impl PolicyRuleRepository for FakePolicyRuleRepository {
    async fn upsert_rule(&self, rule: PolicyRuleRecord) -> Result<(), AccessControlDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.upsert_calls += 1;
        state.last_upsert = Some(rule);
        Ok(())
    }

    async fn find_rules_for_roles(
        &self,
        _tenant_id: &TenantId,
        _resource_name: &ResourceName,
        _action_name: &ActionName,
        _role_names: &[String],
    ) -> Result<Vec<PolicyRuleRecord>, AccessControlDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.find_calls += 1;
        Ok(state.rules_to_return.clone())
    }
}

pub struct FakeAuthorizationDecisionAuditRepository {
    events: Mutex<Vec<AuthorizationDecisionAuditedEvent>>,
}

impl FakeAuthorizationDecisionAuditRepository {
    pub fn new() -> Self {
        Self {
            events: Mutex::new(Vec::new()),
        }
    }

    pub fn events(&self) -> Vec<AuthorizationDecisionAuditedEvent> {
        self.events.lock().expect("mutex poisoned").clone()
    }
}

#[async_trait]
impl AuthorizationDecisionAuditRepository for FakeAuthorizationDecisionAuditRepository {
    async fn save_decision(
        &self,
        event: &AuthorizationDecisionAuditedEvent,
    ) -> Result<(), AccessControlDomainError> {
        self.events
            .lock()
            .expect("mutex poisoned")
            .push(event.clone());
        Ok(())
    }
}
