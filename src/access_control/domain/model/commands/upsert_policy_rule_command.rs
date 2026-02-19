use crate::access_control::domain::model::{
    enums::{
        access_control_domain_error::AccessControlDomainError, permission_effect::PermissionEffect,
    },
    value_objects::{
        action_name::ActionName, resource_name::ResourceName, role_name::RoleName,
        tenant_id::TenantId,
    },
};

#[derive(Clone, Debug)]
pub struct UpsertPolicyRuleCommand {
    tenant_id: TenantId,
    role_name: RoleName,
    resource_name: ResourceName,
    action_name: ActionName,
    effect: PermissionEffect,
    allowed_columns: Option<Vec<String>>,
    denied_columns: Option<Vec<String>>,
    owner_scope: bool,
}

pub struct UpsertPolicyRuleCommandParts {
    pub tenant_id: String,
    pub role_name: String,
    pub resource_name: String,
    pub action_name: String,
    pub effect: PermissionEffect,
    pub allowed_columns: Option<Vec<String>>,
    pub denied_columns: Option<Vec<String>>,
    pub owner_scope: bool,
}

impl UpsertPolicyRuleCommand {
    pub fn new(parts: UpsertPolicyRuleCommandParts) -> Result<Self, AccessControlDomainError> {
        if let Some(columns) = &parts.allowed_columns {
            for column in columns {
                if column.trim().is_empty() {
                    return Err(AccessControlDomainError::InvalidResourceName);
                }
            }
        }
        if let Some(columns) = &parts.denied_columns {
            for column in columns {
                if column.trim().is_empty() {
                    return Err(AccessControlDomainError::InvalidResourceName);
                }
            }
        }

        Ok(Self {
            tenant_id: TenantId::new(parts.tenant_id)?,
            role_name: RoleName::new(parts.role_name)?,
            resource_name: ResourceName::new(parts.resource_name)?,
            action_name: ActionName::new(parts.action_name)?,
            effect: parts.effect,
            allowed_columns: parts.allowed_columns,
            denied_columns: parts.denied_columns,
            owner_scope: parts.owner_scope,
        })
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    pub fn role_name(&self) -> &RoleName {
        &self.role_name
    }
    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }
    pub fn action_name(&self) -> &ActionName {
        &self.action_name
    }
    pub fn effect(&self) -> PermissionEffect {
        self.effect
    }
    pub fn allowed_columns(&self) -> Option<&[String]> {
        self.allowed_columns.as_deref()
    }
    pub fn denied_columns(&self) -> Option<&[String]> {
        self.denied_columns.as_deref()
    }
    pub fn owner_scope(&self) -> bool {
        self.owner_scope
    }
}
