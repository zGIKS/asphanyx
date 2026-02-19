use thiserror::Error;

#[derive(Debug, Error)]
pub enum AccessControlDomainError {
    #[error("tenant id is invalid")]
    InvalidTenantId,

    #[error("principal id is invalid")]
    InvalidPrincipalId,

    #[error("role name is invalid")]
    InvalidRoleName,

    #[error("resource name is invalid")]
    InvalidResourceName,

    #[error("action name is invalid")]
    InvalidActionName,

    #[error("access denied")]
    AccessDenied,

    #[error("policy not found")]
    PolicyNotFound,

    #[error("infrastructure error: {0}")]
    InfrastructureError(String),
}
