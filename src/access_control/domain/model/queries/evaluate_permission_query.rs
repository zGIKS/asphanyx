use crate::access_control::domain::model::{
    enums::access_control_domain_error::AccessControlDomainError,
    value_objects::{
        action_name::ActionName, principal_id::PrincipalId, resource_name::ResourceName,
        tenant_id::TenantId,
    },
};

#[derive(Clone, Debug)]
pub struct EvaluatePermissionQuery {
    tenant_id: TenantId,
    principal_id: PrincipalId,
    resource_name: ResourceName,
    action_name: ActionName,
    requested_columns: Vec<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
    request_id: Option<String>,
}

pub struct EvaluatePermissionQueryParts {
    pub tenant_id: String,
    pub principal_id: String,
    pub resource_name: String,
    pub action_name: String,
    pub requested_columns: Vec<String>,
    pub subject_owner_id: Option<String>,
    pub row_owner_id: Option<String>,
    pub request_id: Option<String>,
}

impl EvaluatePermissionQuery {
    pub fn new(parts: EvaluatePermissionQueryParts) -> Result<Self, AccessControlDomainError> {
        Ok(Self {
            tenant_id: TenantId::new(parts.tenant_id)?,
            principal_id: PrincipalId::new(parts.principal_id)?,
            resource_name: ResourceName::new(parts.resource_name)?,
            action_name: ActionName::new(parts.action_name)?,
            requested_columns: parts.requested_columns,
            subject_owner_id: parts.subject_owner_id,
            row_owner_id: parts.row_owner_id,
            request_id: parts.request_id,
        })
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }
    pub fn principal_id(&self) -> &PrincipalId {
        &self.principal_id
    }
    pub fn resource_name(&self) -> &ResourceName {
        &self.resource_name
    }
    pub fn action_name(&self) -> &ActionName {
        &self.action_name
    }
    pub fn requested_columns(&self) -> &[String] {
        &self.requested_columns
    }
    pub fn subject_owner_id(&self) -> Option<&str> {
        self.subject_owner_id.as_deref()
    }
    pub fn row_owner_id(&self) -> Option<&str> {
        self.row_owner_id.as_deref()
    }
    pub fn request_id(&self) -> Option<&str> {
        self.request_id.as_deref()
    }
}
