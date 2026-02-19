use crate::access_control::domain::model::{
    enums::access_control_domain_error::AccessControlDomainError,
    value_objects::{principal_id::PrincipalId, role_name::RoleName, tenant_id::TenantId},
};

#[derive(Clone, Debug)]
pub struct AssignRoleToPrincipalCommand {
    tenant_id: TenantId,
    principal_id: PrincipalId,
    role_name: RoleName,
}

impl AssignRoleToPrincipalCommand {
    pub fn new(
        tenant_id: String,
        principal_id: String,
        role_name: String,
    ) -> Result<Self, AccessControlDomainError> {
        Ok(Self {
            tenant_id: TenantId::new(tenant_id)?,
            principal_id: PrincipalId::new(principal_id)?,
            role_name: RoleName::new(role_name)?,
        })
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    pub fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }
    pub fn role_name(&self) -> &RoleName {
        &self.role_name
    }
}
