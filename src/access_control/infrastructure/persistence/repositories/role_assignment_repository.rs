use async_trait::async_trait;

use crate::access_control::domain::model::{
    enums::access_control_domain_error::AccessControlDomainError,
    value_objects::{principal_id::PrincipalId, role_name::RoleName, tenant_id::TenantId},
};

#[async_trait]
pub trait RoleAssignmentRepository: Send + Sync {
    async fn assign_role(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
        role_name: &RoleName,
    ) -> Result<(), AccessControlDomainError>;

    async fn find_roles_by_principal(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
    ) -> Result<Vec<String>, AccessControlDomainError>;
}
