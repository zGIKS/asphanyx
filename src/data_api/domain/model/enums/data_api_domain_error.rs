use thiserror::Error;

#[derive(Debug, Error)]
pub enum DataApiDomainError {
    #[error("tenant id is required")]
    InvalidTenantId,

    #[error("schema name is invalid")]
    InvalidSchemaName,

    #[error("table name is invalid")]
    InvalidTableName,

    #[error("column name is invalid")]
    InvalidColumnName,

    #[error("invalid row identifier")]
    InvalidRowIdentifier,

    #[error("unsupported API version")]
    UnsupportedApiVersion,

    #[error("table is not exposed by allowlist")]
    TableNotAllowed,

    #[error("authentication is required (x-api-key or bearer token)")]
    MissingAuthentication,

    #[error("authentication mechanism is invalid")]
    InvalidAuthentication,

    #[error("access denied by ACL")]
    AccessDenied,

    #[error("table not found")]
    TableNotFound,

    #[error("tenant database not found")]
    TenantDatabaseNotFound,

    #[error("table has no primary key")]
    PrimaryKeyNotFound,

    #[error("payload size exceeded")]
    PayloadTooLarge,

    #[error("payload must be an object")]
    InvalidPayload,

    #[error("column is not editable: {0}")]
    NonEditableColumn(String),

    #[error("invalid filter or sort expression")]
    InvalidQueryParameters,

    #[error("record not found")]
    RecordNotFound,

    #[error("infrastructure error: {0}")]
    InfrastructureError(String),
}
