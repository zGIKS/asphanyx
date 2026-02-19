use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    routing::{delete, get, post},
};
use validator::Validate;

use crate::provisioner::{
    domain::{
        model::{
            commands::{
                create_provisioned_database_command::CreateProvisionedDatabaseCommand,
                delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
            },
            enums::provisioner_domain_error::ProvisionerDomainError,
            queries::list_provisioned_databases_query::ListProvisionedDatabasesQuery,
        },
        services::{
            database_provisioning_command_service::DatabaseProvisioningCommandService,
            database_provisioning_query_service::DatabaseProvisioningQueryService,
        },
    },
    interfaces::rest::resources::{
        create_provisioned_database_request_resource::{
            CreateProvisionedDatabaseRequestResource, ListProvisionedDatabasesQueryResource,
        },
        error_response_resource::ErrorResponseResource,
        provisioned_database_resource::ProvisionedDatabaseResource,
    },
};

#[derive(Clone)]
pub struct ProvisionerRestControllerState {
    pub command_service: Arc<dyn DatabaseProvisioningCommandService>,
    pub query_service: Arc<dyn DatabaseProvisioningQueryService>,
}

pub fn router(state: ProvisionerRestControllerState) -> Router {
    Router::new()
        .route("/provisioner/databases", post(create_provisioned_database))
        .route("/provisioner/databases", get(list_provisioned_databases))
        .route(
            "/provisioner/databases/:database_name",
            delete(delete_provisioned_database),
        )
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/provisioner/databases",
    tag = "provisioner",
    request_body = CreateProvisionedDatabaseRequestResource,
    responses(
        (status = 201, description = "Provisioned database created", body = ProvisionedDatabaseResource),
        (status = 400, description = "Invalid payload", body = ErrorResponseResource),
        (status = 409, description = "Database already exists", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn create_provisioned_database(
    State(state): State<ProvisionerRestControllerState>,
    Json(request): Json<CreateProvisionedDatabaseRequestResource>,
) -> Result<
    (StatusCode, Json<ProvisionedDatabaseResource>),
    (StatusCode, Json<ErrorResponseResource>),
> {
    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let command = CreateProvisionedDatabaseCommand::new(
        request.database_name,
        request.username,
        request.password,
        request.apply_seed_data,
    )
    .map_err(map_domain_error)?;

    let created = state
        .command_service
        .handle_create(command)
        .await
        .map_err(map_domain_error)?;

    Ok((
        StatusCode::CREATED,
        Json(ProvisionedDatabaseResource {
            database_name: created.database_name().value().to_string(),
            username: created.username().value().to_string(),
            status: created.status().as_str().to_string(),
            created_at: created.created_at().to_rfc3339(),
        }),
    ))
}

#[utoipa::path(
    delete,
    path = "/provisioner/databases/{database_name}",
    tag = "provisioner",
    params(("database_name" = String, Path, description = "Database identifier")),
    responses(
        (status = 204, description = "Provisioned database deleted"),
        (status = 404, description = "Database not found", body = ErrorResponseResource),
        (status = 400, description = "Invalid database name", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn delete_provisioned_database(
    State(state): State<ProvisionerRestControllerState>,
    Path(database_name): Path<String>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponseResource>)> {
    let command = DeleteProvisionedDatabaseCommand::new(database_name).map_err(map_domain_error)?;

    state
        .command_service
        .handle_delete(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/provisioner/databases",
    tag = "provisioner",
    params(("include_deleted" = Option<bool>, Query, description = "Include deleted entries")),
    responses(
        (status = 200, description = "Provisioned database metadata", body = [ProvisionedDatabaseResource]),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn list_provisioned_databases(
    State(state): State<ProvisionerRestControllerState>,
    Query(query): Query<ListProvisionedDatabasesQueryResource>,
) -> Result<Json<Vec<ProvisionedDatabaseResource>>, (StatusCode, Json<ErrorResponseResource>)> {
    let query = ListProvisionedDatabasesQuery::new(query.include_deleted.unwrap_or(false));
    let databases = state
        .query_service
        .handle_list(query)
        .await
        .map_err(map_domain_error)?;

    let payload = databases
        .into_iter()
        .map(|database| ProvisionedDatabaseResource {
            database_name: database.database_name().value().to_string(),
            username: database.username().value().to_string(),
            status: database.status().as_str().to_string(),
            created_at: database.created_at().to_rfc3339(),
        })
        .collect();

    Ok(Json(payload))
}

fn map_domain_error(error: ProvisionerDomainError) -> (StatusCode, Json<ErrorResponseResource>) {
    let status = match error {
        ProvisionerDomainError::InvalidDatabaseName
        | ProvisionerDomainError::InvalidDatabaseUsername
        | ProvisionerDomainError::InvalidDatabasePassword
        | ProvisionerDomainError::InvalidStatusTransition => StatusCode::BAD_REQUEST,
        ProvisionerDomainError::DatabaseAlreadyProvisioned => StatusCode::CONFLICT,
        ProvisionerDomainError::DatabaseNotFound => StatusCode::NOT_FOUND,
        ProvisionerDomainError::InfrastructureError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(ErrorResponseResource {
            message: error.to_string(),
        }),
    )
}
