use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProvisionerDomainError {
    #[error("database name is invalid; use [a-z][a-z0-9_] and length 3..63")]
    InvalidDatabaseName,

    #[error("database username is invalid; use [a-z][a-z0-9_] and length 3..63")]
    InvalidDatabaseUsername,

    #[error("database password is invalid; minimum length is 8")]
    InvalidDatabasePassword,

    #[error("database already provisioned")]
    DatabaseAlreadyProvisioned,

    #[error("database not found")]
    DatabaseNotFound,

    #[error("invalid status transition")]
    InvalidStatusTransition,

    #[error("infrastructure error: {0}")]
    InfrastructureError(String),
}
