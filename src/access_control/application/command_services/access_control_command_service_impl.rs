use std::sync::Arc;

use async_trait::async_trait;

use crate::access_control::{
    domain::{
        model::{
            commands::{
                assign_role_to_principal_command::AssignRoleToPrincipalCommand,
                upsert_policy_rule_command::UpsertPolicyRuleCommand,
            },
            enums::access_control_domain_error::AccessControlDomainError,
        },
        services::access_control_command_service::AccessControlCommandService,
    },
    infrastructure::persistence::repositories::{
        policy_rule_repository::{PolicyRuleRecord, PolicyRuleRepository},
        role_assignment_repository::RoleAssignmentRepository,
    },
};

pub struct AccessControlCommandServiceImpl {
    role_assignment_repository: Arc<dyn RoleAssignmentRepository>,
    policy_rule_repository: Arc<dyn PolicyRuleRepository>,
}

impl AccessControlCommandServiceImpl {
    pub fn new(
        role_assignment_repository: Arc<dyn RoleAssignmentRepository>,
        policy_rule_repository: Arc<dyn PolicyRuleRepository>,
    ) -> Self {
        Self {
            role_assignment_repository,
            policy_rule_repository,
        }
    }
}

#[async_trait]
impl AccessControlCommandService for AccessControlCommandServiceImpl {
    async fn handle_assign_role(
        &self,
        command: AssignRoleToPrincipalCommand,
    ) -> Result<(), AccessControlDomainError> {
        self.role_assignment_repository
            .assign_role(
                command.tenant_id(),
                command.principal_id(),
                command.role_name(),
            )
            .await
    }

    async fn handle_upsert_policy(
        &self,
        command: UpsertPolicyRuleCommand,
    ) -> Result<(), AccessControlDomainError> {
        self.policy_rule_repository
            .upsert_rule(PolicyRuleRecord {
                tenant_id: command.tenant_id().value().to_string(),
                role_name: command.role_name().value().to_string(),
                resource_name: command.resource_name().value().to_string(),
                action_name: command.action_name().value().to_string(),
                effect: command.effect(),
                allowed_columns: command.allowed_columns().map(|c| c.to_vec()),
                denied_columns: command.denied_columns().map(|c| c.to_vec()),
                owner_scope: command.owner_scope(),
            })
            .await
    }
}
