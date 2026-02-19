use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post},
};
use serde_json::Value;
use validator::Validate;

use crate::data_api::{
    domain::{
        model::{
            commands::{
                create_row_command::{CreateRowCommand, CreateRowCommandParts},
                delete_row_command::{DeleteRowCommand, DeleteRowCommandParts},
                patch_row_command::{PatchRowCommand, PatchRowCommandParts},
            },
            enums::{
                data_api_domain_error::DataApiDomainError,
                data_api_principal_type::DataApiPrincipalType,
            },
            queries::{
                get_row_query::{GetRowQuery, GetRowQueryParts},
                list_rows_query::{ListRowsQuery, ListRowsQueryParts},
                table_schema_introspection_query::{
                    TableSchemaIntrospectionQuery, TableSchemaIntrospectionQueryParts,
                },
            },
        },
        services::{
            data_api_command_service::DataApiCommandService,
            data_api_query_service::DataApiQueryService,
        },
    },
    interfaces::rest::resources::{
        data_api_error_response_resource::DataApiErrorResponseResource,
        data_api_payload_resource::DataApiPayloadResource,
    },
};

#[derive(Clone)]
pub struct DataApiRestControllerState {
    pub command_service: Arc<dyn DataApiCommandService>,
    pub query_service: Arc<dyn DataApiQueryService>,
}

pub fn router(state: DataApiRestControllerState) -> Router {
    Router::new()
        .route("/api/v1/:table_name", get(list_rows))
        .route("/api/v1/:table_name", post(create_row))
        .route("/api/v1/:table_name/_schema", get(introspect_table_schema))
        .route("/api/v1/:table_name/:row_id", get(get_row))
        .route("/api/v1/:table_name/:row_id", patch(patch_row))
        .route("/api/v1/:table_name/:row_id", delete(delete_row))
        .with_state(state)
}

#[utoipa::path(
    get,
    path = "/api/v1/{table_name}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla expuesta"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso"),
        ("fields" = Option<String>, Query, description = "Campos separados por coma"),
        ("limit" = Option<i64>, Query, description = "Límite (1..500)"),
        ("offset" = Option<i64>, Query, description = "Offset >= 0"),
        ("order_by" = Option<String>, Query, description = "Campo de orden"),
        ("order_dir" = Option<String>, Query, description = "asc|desc"),
    ),
    responses(
        (status = 200, description = "Listado dinámico", body = Value),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 404, description = "Tabla o registro no encontrado", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn list_rows(
    State(state): State<DataApiRestControllerState>,
    Path(table_name): Path<String>,
    Query(params): Query<BTreeMap<String, String>>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let auth = parse_auth_headers(&headers)?;

    let fields = params
        .get("fields")
        .map(|raw| {
            raw.split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let limit = params
        .get("limit")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(50);
    let offset = params
        .get("offset")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(0);

    let order_by = params.get("order_by").cloned();
    let order_desc = params
        .get("order_dir")
        .map(|v| v.eq_ignore_ascii_case("desc"))
        .unwrap_or(false);

    let filters = params
        .iter()
        .filter(|(key, _)| key.starts_with("filter_"))
        .map(|(key, value)| (key.trim_start_matches("filter_").to_string(), value.clone()))
        .collect::<BTreeMap<_, _>>();

    let query = ListRowsQuery::new(ListRowsQueryParts {
        api_version: "v1".to_string(),
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
        select_fields: fields,
        filters,
        limit,
        offset,
        order_by,
        order_desc,
    })
    .map_err(map_domain_error)?;

    let rows = state
        .query_service
        .handle_list(query)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(rows))
}

#[utoipa::path(
    get,
    path = "/api/v1/{table_name}/{row_id}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("row_id" = String, Path, description = "ID lógico (columna PK)"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    responses(
        (status = 200, description = "Registro encontrado", body = Value),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 404, description = "No encontrado", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn get_row(
    State(state): State<DataApiRestControllerState>,
    Path((table_name, row_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let auth = parse_auth_headers(&headers)?;

    let query = GetRowQuery::new(GetRowQueryParts {
        api_version: "v1".to_string(),
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        row_identifier: row_id,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
    })
    .map_err(map_domain_error)?;

    let row = state
        .query_service
        .handle_get(query)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(row))
}

#[utoipa::path(
    post,
    path = "/api/v1/{table_name}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    request_body = DataApiPayloadResource,
    responses(
        (status = 201, description = "Registro creado", body = Value),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn create_row(
    State(state): State<DataApiRestControllerState>,
    Path(table_name): Path<String>,
    headers: HeaderMap,
    Json(resource): Json<DataApiPayloadResource>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<DataApiErrorResponseResource>)> {
    if let Err(validation_error) = resource.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(DataApiErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let auth = parse_auth_headers(&headers)?;

    let command = CreateRowCommand::new(CreateRowCommandParts {
        api_version: "v1".to_string(),
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
        payload: resource.payload,
    })
    .map_err(map_domain_error)?;

    let created = state
        .command_service
        .handle_create(command)
        .await
        .map_err(map_domain_error)?;

    Ok((StatusCode::CREATED, Json(created)))
}

#[utoipa::path(
    patch,
    path = "/api/v1/{table_name}/{row_id}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("row_id" = String, Path, description = "ID lógico (columna PK)"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    request_body = DataApiPayloadResource,
    responses(
        (status = 200, description = "Registro actualizado", body = Value),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 404, description = "No encontrado", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn patch_row(
    State(state): State<DataApiRestControllerState>,
    Path((table_name, row_id)): Path<(String, String)>,
    headers: HeaderMap,
    Json(resource): Json<DataApiPayloadResource>,
) -> Result<Json<Value>, (StatusCode, Json<DataApiErrorResponseResource>)> {
    if let Err(validation_error) = resource.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(DataApiErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let auth = parse_auth_headers(&headers)?;

    let command = PatchRowCommand::new(PatchRowCommandParts {
        api_version: "v1".to_string(),
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        row_identifier: row_id,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
        payload: resource.payload,
    })
    .map_err(map_domain_error)?;

    let updated = state
        .command_service
        .handle_patch(command)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(updated))
}

#[utoipa::path(
    delete,
    path = "/api/v1/{table_name}/{row_id}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("row_id" = String, Path, description = "ID lógico (columna PK)"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    responses(
        (status = 204, description = "Registro eliminado"),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 404, description = "No encontrado", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn delete_row(
    State(state): State<DataApiRestControllerState>,
    Path((table_name, row_id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<StatusCode, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let auth = parse_auth_headers(&headers)?;

    let command = DeleteRowCommand::new(DeleteRowCommandParts {
        api_version: "v1".to_string(),
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        row_identifier: row_id,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
    })
    .map_err(map_domain_error)?;

    state
        .command_service
        .handle_delete(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/{table_name}/_schema",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("authorization" = String, Header, description = "JWT o API key del principal"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    responses(
        (status = 200, description = "Metadatos de schema", body = Value),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 404, description = "Tabla no encontrada", body = DataApiErrorResponseResource)
    )
)]
pub async fn introspect_table_schema(
    State(state): State<DataApiRestControllerState>,
    Path(table_name): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Value>, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let auth = parse_auth_headers(&headers)?;

    let query = TableSchemaIntrospectionQuery::new(TableSchemaIntrospectionQueryParts {
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        principal: auth.principal,
        principal_type: auth.principal_type,
        request_id: auth.request_id,
        subject_owner_id: auth.subject_owner_id,
        row_owner_id: auth.row_owner_id,
    })
    .map_err(map_domain_error)?;

    let metadata = state
        .query_service
        .handle_schema_introspection(query)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(metadata))
}

struct AuthContext {
    tenant_id: String,
    schema_name: String,
    principal: String,
    principal_type: DataApiPrincipalType,
    request_id: Option<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
}

fn parse_auth_headers(
    headers: &HeaderMap,
) -> Result<AuthContext, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let tenant_id = header_string(headers, "x-tenant-id")?;
    let schema_name = headers
        .get("x-tenant-schema")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("public")
        .to_string();

    let authorization = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .ok_or_else(|| map_domain_error(DataApiDomainError::MissingAuthentication))?;

    let (principal, principal_type) = if let Some(token) = authorization.strip_prefix("Bearer ") {
        let token = token.trim();
        if token.is_empty() {
            return Err(map_domain_error(DataApiDomainError::InvalidAuthentication));
        }
        (token.to_string(), DataApiPrincipalType::Jwt)
    } else {
        (authorization.to_string(), DataApiPrincipalType::ApiKey)
    };

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let subject_owner_id = headers
        .get("x-subject-owner-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let row_owner_id = headers
        .get("x-row-owner-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    Ok(AuthContext {
        tenant_id,
        schema_name,
        principal,
        principal_type,
        request_id,
        subject_owner_id,
        row_owner_id,
    })
}

fn header_string(
    headers: &HeaderMap,
    name: &str,
) -> Result<String, (StatusCode, Json<DataApiErrorResponseResource>)> {
    headers
        .get(name)
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .ok_or_else(|| map_domain_error(DataApiDomainError::InvalidTenantId))
}

fn map_domain_error(error: DataApiDomainError) -> (StatusCode, Json<DataApiErrorResponseResource>) {
    let status = match error {
        DataApiDomainError::InvalidTenantId
        | DataApiDomainError::InvalidSchemaName
        | DataApiDomainError::InvalidTableName
        | DataApiDomainError::InvalidColumnName
        | DataApiDomainError::InvalidRowIdentifier
        | DataApiDomainError::UnsupportedApiVersion
        | DataApiDomainError::PayloadTooLarge
        | DataApiDomainError::InvalidPayload
        | DataApiDomainError::InvalidQueryParameters
        | DataApiDomainError::NonEditableColumn(_) => StatusCode::BAD_REQUEST,
        DataApiDomainError::MissingAuthentication | DataApiDomainError::InvalidAuthentication => {
            StatusCode::UNAUTHORIZED
        }
        DataApiDomainError::AccessDenied | DataApiDomainError::TableNotAllowed => {
            StatusCode::FORBIDDEN
        }
        DataApiDomainError::TableNotFound
        | DataApiDomainError::TenantDatabaseNotFound
        | DataApiDomainError::PrimaryKeyNotFound
        | DataApiDomainError::RecordNotFound => StatusCode::NOT_FOUND,
        DataApiDomainError::InfrastructureError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(DataApiErrorResponseResource {
            message: error.to_string(),
        }),
    )
}
