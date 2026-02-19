use async_trait::async_trait;

use crate::access_control::domain::model::{
    commands::{
        assign_role_to_principal_command::AssignRoleToPrincipalCommand,
        upsert_policy_rule_command::UpsertPolicyRuleCommand,
    },
    enums::access_control_domain_error::AccessControlDomainError,
};

#[async_trait]
pub trait AccessControlCommandService: Send + Sync {
    async fn handle_assign_role(
        &self,
        command: AssignRoleToPrincipalCommand,
    ) -> Result<(), AccessControlDomainError>;

    async fn handle_upsert_policy(
        &self,
        command: UpsertPolicyRuleCommand,
    ) -> Result<(), AccessControlDomainError>;
}
