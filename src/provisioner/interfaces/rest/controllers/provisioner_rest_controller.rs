use std::{collections::HashSet, sync::Arc};

use argon2::{
    Argon2,
    password_hash::{PasswordHasher, SaltString, rand_core::OsRng},
};
use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
};
use rand::{Rng, distributions::Alphanumeric, thread_rng};
use uuid::Uuid;
use validator::Validate;

use crate::{
    iam_integration::interfaces::acl::iam_authentication_facade::{
        IamAuthenticationFacade, IamIntegrationError,
    },
    provisioner::{
        domain::{
            model::{
                commands::{
                    change_provisioned_database_password_command::ChangeProvisionedDatabasePasswordCommand,
                    create_provisioned_database_command::CreateProvisionedDatabaseCommand,
                    delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
                },
                enums::provisioner_domain_error::ProvisionerDomainError,
                queries::list_provisioned_databases_query::ListProvisionedDatabasesQuery,
                value_objects::provisioned_database_name::ProvisionedDatabaseName,
            },
            services::{
                database_provisioning_command_service::DatabaseProvisioningCommandService,
                database_provisioning_query_service::DatabaseProvisioningQueryService,
            },
        },
        infrastructure::persistence::repositories::{
            provisioned_database_repository::ProvisionedDatabaseRepository,
            tenant_ownership_repository::TenantOwnershipRepository,
        },
        interfaces::rest::resources::{
            change_provisioned_database_password_request_resource::ChangeProvisionedDatabasePasswordRequestResource,
            create_provisioned_database_request_resource::{
                CreateProvisionedDatabaseRequestResource, ListProvisionedDatabasesQueryResource,
            },
            error_response_resource::ErrorResponseResource,
            provisioned_database_resource::ProvisionedDatabaseResource,
        },
    },
};

#[derive(Clone)]
pub struct ProvisionerRestControllerState {
    pub command_service: Arc<dyn DatabaseProvisioningCommandService>,
    pub query_service: Arc<dyn DatabaseProvisioningQueryService>,
    pub metadata_repository: Arc<dyn ProvisionedDatabaseRepository>,
    pub tenant_ownership_repository: Arc<dyn TenantOwnershipRepository>,
    pub iam_authentication_facade: Arc<dyn IamAuthenticationFacade>,
}

pub fn router(state: ProvisionerRestControllerState) -> Router {
    Router::new()
        .route("/provisioner/databases", post(create_provisioned_database))
        .route("/provisioner/databases", get(list_provisioned_databases))
        .route(
            "/provisioner/databases/:database_name",
            delete(delete_provisioned_database),
        )
        .route(
            "/provisioner/databases/:database_name/password",
            patch(change_provisioned_database_password),
        )
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/provisioner/databases",
    tag = "provisioner",
    security(
        ("bearerAuth" = [])
    ),
    request_body = CreateProvisionedDatabaseRequestResource,
    responses(
        (status = 201, description = "Provisioned database created", body = ProvisionedDatabaseResource),
        (status = 400, description = "Invalid payload", body = ErrorResponseResource),
        (status = 401, description = "Invalid or missing bearer token", body = ErrorResponseResource),
        (status = 409, description = "Database already exists", body = ErrorResponseResource),
        (status = 503, description = "IAM unavailable", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn create_provisioned_database(
    State(state): State<ProvisionerRestControllerState>,
    headers: HeaderMap,
    Json(request): Json<CreateProvisionedDatabaseRequestResource>,
) -> Result<
    (StatusCode, Json<ProvisionedDatabaseResource>),
    (StatusCode, Json<ErrorResponseResource>),
> {
    let authenticated_user_id = authenticate_bearer_subject(&state, &headers).await?;

    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let generated_username = generate_database_username();
    let password_hash = hash_database_password(&request.password)?;

    let command = CreateProvisionedDatabaseCommand::new(
        request.database_name,
        generated_username,
        request.password,
        password_hash,
        request.apply_seed_data,
    )
    .map_err(map_domain_error)?;

    let created = state
        .command_service
        .handle_create(command)
        .await
        .map_err(map_domain_error)?;

    let tenant_id = created.id().value();

    state
        .tenant_ownership_repository
        .save_ownership(tenant_id, authenticated_user_id)
        .await
        .map_err(map_infra_error)?;

    Ok((
        StatusCode::CREATED,
        Json(ProvisionedDatabaseResource {
            id: created.id().value().to_string(),
            database_name: created.database_name().value().to_string(),
            username: created.username().value().to_string(),
            status: created.status().as_str().to_string(),
            created_at: created.created_at().to_rfc3339(),
        }),
    ))
}

fn generate_database_username() -> String {
    format!("dbu_{}", random_alphanumeric_lowercase(16))
}

fn random_alphanumeric_lowercase(len: usize) -> String {
    let mut rng = thread_rng();
    let mut value = String::with_capacity(len);

    for _ in 0..len {
        let candidate = rng.sample(Alphanumeric) as char;
        value.push(candidate.to_ascii_lowercase());
    }

    value
}

fn hash_database_password(
    password: &str,
) -> Result<String, (StatusCode, Json<ErrorResponseResource>)> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map_err(|error| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorResponseResource {
                    message: format!("failed to hash provided database password: {error}"),
                }),
            )
        })?
        .to_string();

    Ok(hash)
}

#[utoipa::path(
    delete,
    path = "/provisioner/databases/{database_name}",
    tag = "provisioner",
    params(
        ("database_name" = String, Path, description = "Database identifier")
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 204, description = "Provisioned database deleted"),
        (status = 401, description = "Invalid or missing bearer token", body = ErrorResponseResource),
        (status = 403, description = "Tenant does not belong to user", body = ErrorResponseResource),
        (status = 404, description = "Database not found", body = ErrorResponseResource),
        (status = 400, description = "Invalid database name", body = ErrorResponseResource),
        (status = 503, description = "IAM unavailable", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn delete_provisioned_database(
    State(state): State<ProvisionerRestControllerState>,
    Path(database_name): Path<String>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponseResource>)> {
    let authenticated_user_id = authenticate_bearer_subject(&state, &headers).await?;

    let command = DeleteProvisionedDatabaseCommand::new(database_name).map_err(map_domain_error)?;
    enforce_database_ownership(
        &state,
        command.database_name().value(),
        authenticated_user_id,
    )
    .await?;

    state
        .command_service
        .handle_delete(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    patch,
    path = "/provisioner/databases/{database_name}/password",
    tag = "provisioner",
    params(
        ("database_name" = String, Path, description = "Database identifier")
    ),
    security(
        ("bearerAuth" = [])
    ),
    request_body = ChangeProvisionedDatabasePasswordRequestResource,
    responses(
        (status = 204, description = "Database password changed"),
        (status = 401, description = "Invalid or missing bearer token", body = ErrorResponseResource),
        (status = 403, description = "Tenant does not belong to user", body = ErrorResponseResource),
        (status = 400, description = "Invalid payload", body = ErrorResponseResource),
        (status = 404, description = "Database not found", body = ErrorResponseResource),
        (status = 503, description = "IAM unavailable", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn change_provisioned_database_password(
    State(state): State<ProvisionerRestControllerState>,
    Path(database_name): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ChangeProvisionedDatabasePasswordRequestResource>,
) -> Result<StatusCode, (StatusCode, Json<ErrorResponseResource>)> {
    let authenticated_user_id = authenticate_bearer_subject(&state, &headers).await?;

    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    enforce_database_ownership(&state, &database_name, authenticated_user_id).await?;

    let password_hash = hash_database_password(&request.password)?;
    let command = ChangeProvisionedDatabasePasswordCommand::new(
        database_name,
        request.password,
        password_hash,
    )
    .map_err(map_domain_error)?;

    state
        .command_service
        .handle_change_password(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/provisioner/databases",
    tag = "provisioner",
    params(
        ("include_deleted" = Option<bool>, Query, description = "Include deleted entries")
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Provisioned database metadata", body = [ProvisionedDatabaseResource]),
        (status = 401, description = "Invalid or missing bearer token", body = ErrorResponseResource),
        (status = 503, description = "IAM unavailable", body = ErrorResponseResource),
        (status = 500, description = "Infrastructure failure", body = ErrorResponseResource)
    )
)]
pub async fn list_provisioned_databases(
    State(state): State<ProvisionerRestControllerState>,
    headers: HeaderMap,
    Query(query): Query<ListProvisionedDatabasesQueryResource>,
) -> Result<Json<Vec<ProvisionedDatabaseResource>>, (StatusCode, Json<ErrorResponseResource>)> {
    let authenticated_user_id = authenticate_bearer_subject(&state, &headers).await?;

    let query = ListProvisionedDatabasesQuery::new(query.include_deleted.unwrap_or(false));
    let tenant_ids = state
        .tenant_ownership_repository
        .list_tenant_ids_by_user(authenticated_user_id)
        .await
        .map_err(map_infra_error)?;
    let tenant_ids = tenant_ids.into_iter().collect::<HashSet<Uuid>>();

    let databases = state
        .query_service
        .handle_list(query)
        .await
        .map_err(map_domain_error)?;

    let payload = databases
        .into_iter()
        .filter(|database| tenant_ids.contains(&database.id().value()))
        .map(|database| ProvisionedDatabaseResource {
            id: database.id().value().to_string(),
            database_name: database.database_name().value().to_string(),
            username: database.username().value().to_string(),
            status: database.status().as_str().to_string(),
            created_at: database.created_at().to_rfc3339(),
        })
        .collect();

    Ok(Json(payload))
}

async fn enforce_database_ownership(
    state: &ProvisionerRestControllerState,
    database_name: &str,
    user_id: Uuid,
) -> Result<(), (StatusCode, Json<ErrorResponseResource>)> {
    let database_name_vo =
        ProvisionedDatabaseName::new(database_name.to_string()).map_err(map_domain_error)?;

    let database = state
        .metadata_repository
        .find_by_name(&database_name_vo)
        .await
        .map_err(map_domain_error)?
        .ok_or_else(|| map_domain_error(ProvisionerDomainError::DatabaseNotFound))?;

    let tenant_id = database.id().value();

    let is_owner = state
        .tenant_ownership_repository
        .exists_ownership(tenant_id, user_id)
        .await
        .map_err(map_infra_error)?;

    if !is_owner {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ErrorResponseResource {
                message: "tenant does not belong to authenticated user".to_string(),
            }),
        ));
    }

    Ok(())
}

async fn authenticate_bearer_subject(
    state: &ProvisionerRestControllerState,
    headers: &HeaderMap,
) -> Result<Uuid, (StatusCode, Json<ErrorResponseResource>)> {
    let token = extract_bearer_token(headers)?;

    let verification = state
        .iam_authentication_facade
        .verify_access_token(&token)
        .await
        .map_err(map_iam_error)?;

    Ok(verification.subject_id.value())
}

fn extract_bearer_token(
    headers: &HeaderMap,
) -> Result<String, (StatusCode, Json<ErrorResponseResource>)> {
    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponseResource {
                    message: "missing authorization header".to_string(),
                }),
            )
        })?;

    let token = authorization
        .strip_prefix("Bearer ")
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            (
                StatusCode::UNAUTHORIZED,
                Json(ErrorResponseResource {
                    message: "authorization header must be Bearer token".to_string(),
                }),
            )
        })?;

    Ok(token.to_string())
}

fn map_iam_error(error: IamIntegrationError) -> (StatusCode, Json<ErrorResponseResource>) {
    match error {
        IamIntegrationError::InvalidToken(message) => (
            StatusCode::UNAUTHORIZED,
            Json(ErrorResponseResource { message }),
        ),
        IamIntegrationError::Unavailable(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ErrorResponseResource { message }),
        ),
    }
}

fn map_infra_error(message: String) -> (StatusCode, Json<ErrorResponseResource>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorResponseResource { message }),
    )
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
