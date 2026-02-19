use async_trait::async_trait;

use crate::access_control::domain::model::{
    enums::{
        access_control_domain_error::AccessControlDomainError, permission_effect::PermissionEffect,
    },
    value_objects::{action_name::ActionName, resource_name::ResourceName, tenant_id::TenantId},
};

#[derive(Clone, Debug)]
pub struct PolicyRuleRecord {
    pub tenant_id: String,
    pub role_name: String,
    pub resource_name: String,
    pub action_name: String,
    pub effect: PermissionEffect,
    pub allowed_columns: Option<Vec<String>>,
    pub denied_columns: Option<Vec<String>>,
    pub owner_scope: bool,
}

#[async_trait]
pub trait PolicyRuleRepository: Send + Sync {
    async fn upsert_rule(&self, rule: PolicyRuleRecord) -> Result<(), AccessControlDomainError>;

    async fn find_rules_for_roles(
        &self,
        tenant_id: &TenantId,
        resource_name: &ResourceName,
        action_name: &ActionName,
        role_names: &[String],
    ) -> Result<Vec<PolicyRuleRecord>, AccessControlDomainError>;
}
