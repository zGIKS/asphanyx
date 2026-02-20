use std::{collections::BTreeMap, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    routing::{delete, get, patch, post, put},
};
use serde_json::Value;
use uuid::Uuid;
use validator::Validate;

use crate::data_api::{
    domain::{
        model::{
            commands::{
                apply_table_policy_template_command::{
                    ApplyTablePolicyTemplateCommand, ApplyTablePolicyTemplateCommandParts,
                },
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
                list_policy_templates_query::ListPolicyTemplatesQuery,
                list_rows_query::{ListRowsQuery, ListRowsQueryParts},
                table_schema_introspection_query::{
                    TableSchemaIntrospectionQuery, TableSchemaIntrospectionQueryParts,
                },
            },
        },
        services::{
            data_api_command_service::DataApiCommandService,
            data_api_policy_template_command_service::DataApiPolicyTemplateCommandService,
            data_api_policy_template_query_service::DataApiPolicyTemplateQueryService,
            data_api_query_service::DataApiQueryService,
        },
    },
    infrastructure::persistence::repositories::data_api_repository::{
        ColumnMetadataUpdateCriteria, DataApiRepository, TableMetadataUpdateCriteria,
    },
    interfaces::rest::resources::{
        apply_table_policy_template_request_resource::ApplyTablePolicyTemplateRequestResource,
        data_api_column_access_metadata_update_request_resource::DataApiColumnAccessMetadataUpdateRequestResource,
        data_api_error_response_resource::DataApiErrorResponseResource,
        data_api_payload_resource::DataApiPayloadResource,
        data_api_table_access_catalog_resource::DataApiTableAccessCatalogEntryResource,
        data_api_table_access_metadata_update_request_resource::DataApiTableAccessMetadataUpdateRequestResource,
        policy_template_catalog_resource::PolicyTemplateCatalogResource,
    },
};
use crate::{
    iam_integration::interfaces::acl::iam_authentication_facade::{
        IamAuthenticationFacade, IamIntegrationError,
    },
    provisioner::infrastructure::persistence::repositories::tenant_ownership_repository::TenantOwnershipRepository,
};

#[derive(Clone)]
pub struct DataApiRestControllerState {
    pub command_service: Arc<dyn DataApiCommandService>,
    pub query_service: Arc<dyn DataApiQueryService>,
    pub policy_template_command_service: Arc<dyn DataApiPolicyTemplateCommandService>,
    pub policy_template_query_service: Arc<dyn DataApiPolicyTemplateQueryService>,
    pub repository: Arc<dyn DataApiRepository>,
    pub iam_authentication_facade: Arc<dyn IamAuthenticationFacade>,
    pub tenant_ownership_repository: Arc<dyn TenantOwnershipRepository>,
}

pub fn router(state: DataApiRestControllerState) -> Router {
    Router::new()
        .route("/api/v1/_metadata", get(list_access_catalog))
        .route(
            "/api/v1/_metadata/policy-templates",
            get(list_policy_templates),
        )
        .route(
            "/api/v1/_metadata/:table_name",
            put(upsert_table_access_metadata),
        )
        .route(
            "/api/v1/_metadata/:table_name/policy-templates",
            post(apply_table_policy_template),
        )
        .route(
            "/api/v1/_metadata/:table_name/columns/:column_name",
            put(upsert_column_access_metadata),
        )
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
    path = "/api/v1/_metadata",
    tag = "data-api",
    params(
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional")
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Catálogo de metadatos", body = [DataApiTableAccessCatalogEntryResource]),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn list_access_catalog(
    State(state): State<DataApiRestControllerState>,
    headers: HeaderMap,
) -> Result<
    Json<Vec<DataApiTableAccessCatalogEntryResource>>,
    (StatusCode, Json<DataApiErrorResponseResource>),
> {
    let auth = parse_auth_context(&state, &headers).await?;
    let tenant_id = parse_tenant_id(&auth.tenant_id)?;

    state
        .repository
        .synchronize_metadata(&tenant_id, &auth.schema_name)
        .await
        .map_err(map_domain_error)?;

    let catalog = state
        .repository
        .list_access_catalog(&tenant_id, &auth.schema_name)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(
        catalog
            .into_iter()
            .map(|entry| DataApiTableAccessCatalogEntryResource {
                table_name: entry.table_name,
                exposed: entry.exposed,
                read_enabled: entry.read_enabled,
                create_enabled: entry.create_enabled,
                update_enabled: entry.update_enabled,
                delete_enabled: entry.delete_enabled,
                introspect_enabled: entry.introspect_enabled,
                authorization_mode: entry.authorization_mode,
                writable_columns: entry.writable_columns,
            })
            .collect(),
    ))
}

#[utoipa::path(
    get,
    path = "/api/v1/_metadata/policy-templates",
    tag = "data-api",
    params(
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
    ),
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Catálogo de templates de políticas", body = [PolicyTemplateCatalogResource]),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource),
        (status = 503, description = "IAM no disponible", body = DataApiErrorResponseResource)
    )
)]
pub async fn list_policy_templates(
    State(state): State<DataApiRestControllerState>,
    headers: HeaderMap,
) -> Result<
    Json<Vec<PolicyTemplateCatalogResource>>,
    (StatusCode, Json<DataApiErrorResponseResource>),
> {
    let _ = parse_auth_context(&state, &headers).await?;

    let templates = state
        .policy_template_query_service
        .handle_list_policy_templates(ListPolicyTemplatesQuery::new())
        .await
        .map_err(map_domain_error)?;

    Ok(Json(
        templates
            .into_iter()
            .map(|template| {
                let (
                    _,
                    read_enabled,
                    create_enabled,
                    update_enabled,
                    delete_enabled,
                    introspect_enabled,
                ) = template.metadata_flags();
                PolicyTemplateCatalogResource {
                    template_name: template.as_str().to_string(),
                    authorization_mode: template.authorization_mode().to_string(),
                    read_enabled,
                    create_enabled,
                    update_enabled,
                    delete_enabled,
                    introspect_enabled,
                }
            })
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/_metadata/{table_name}/policy-templates",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
    ),
    request_body = ApplyTablePolicyTemplateRequestResource,
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 204, description = "Template aplicado"),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 403, description = "Sin permisos", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource),
        (status = 503, description = "IAM no disponible", body = DataApiErrorResponseResource)
    )
)]
pub async fn apply_table_policy_template(
    State(state): State<DataApiRestControllerState>,
    Path(table_name): Path<String>,
    headers: HeaderMap,
    Json(resource): Json<ApplyTablePolicyTemplateRequestResource>,
) -> Result<StatusCode, (StatusCode, Json<DataApiErrorResponseResource>)> {
    if let Err(validation_error) = resource.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(DataApiErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let auth = parse_auth_context(&state, &headers).await?;

    let command = ApplyTablePolicyTemplateCommand::new(ApplyTablePolicyTemplateCommandParts {
        tenant_id: auth.tenant_id,
        schema_name: auth.schema_name,
        table_name,
        principal_id: auth.principal,
        template_name: resource.template_name,
    })
    .map_err(map_domain_error)?;

    state
        .policy_template_command_service
        .handle_apply_table_policy_template(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    put,
    path = "/api/v1/_metadata/{table_name}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
    ),
    request_body = DataApiTableAccessMetadataUpdateRequestResource,
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 200, description = "Metadatos de tabla actualizados", body = DataApiTableAccessCatalogEntryResource),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn upsert_table_access_metadata(
    State(state): State<DataApiRestControllerState>,
    Path(table_name): Path<String>,
    headers: HeaderMap,
    Json(resource): Json<DataApiTableAccessMetadataUpdateRequestResource>,
) -> Result<
    Json<DataApiTableAccessCatalogEntryResource>,
    (StatusCode, Json<DataApiErrorResponseResource>),
> {
    let auth = parse_auth_context(&state, &headers).await?;
    if !matches!(
        resource.authorization_mode.as_str(),
        "acl" | "authenticated"
    ) {
        return Err(map_domain_error(DataApiDomainError::InvalidQueryParameters));
    }

    let tenant_id = parse_tenant_id(&auth.tenant_id)?;

    state
        .repository
        .synchronize_metadata(&tenant_id, &auth.schema_name)
        .await
        .map_err(map_domain_error)?;

    let metadata = state
        .repository
        .upsert_table_access_metadata(
            &tenant_id,
            &auth.schema_name,
            &table_name,
            TableMetadataUpdateCriteria {
                exposed: resource.exposed,
                read_enabled: resource.read_enabled,
                create_enabled: resource.create_enabled,
                update_enabled: resource.update_enabled,
                delete_enabled: resource.delete_enabled,
                introspect_enabled: resource.introspect_enabled,
                authorization_mode: resource.authorization_mode,
            },
        )
        .await
        .map_err(map_domain_error)?;

    let writable_columns = state
        .repository
        .list_writable_columns(&tenant_id, &auth.schema_name, &table_name)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(DataApiTableAccessCatalogEntryResource {
        table_name,
        exposed: metadata.exposed,
        read_enabled: metadata.read_enabled,
        create_enabled: metadata.create_enabled,
        update_enabled: metadata.update_enabled,
        delete_enabled: metadata.delete_enabled,
        introspect_enabled: metadata.introspect_enabled,
        authorization_mode: metadata.authorization_mode,
        writable_columns,
    }))
}

#[utoipa::path(
    put,
    path = "/api/v1/_metadata/{table_name}/columns/{column_name}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla"),
        ("column_name" = String, Path, description = "Nombre de columna"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
    ),
    request_body = DataApiColumnAccessMetadataUpdateRequestResource,
    security(
        ("bearerAuth" = [])
    ),
    responses(
        (status = 204, description = "Metadatos de columna actualizados"),
        (status = 400, description = "Request inválido", body = DataApiErrorResponseResource),
        (status = 401, description = "Auth faltante o inválida", body = DataApiErrorResponseResource),
        (status = 500, description = "Error interno", body = DataApiErrorResponseResource)
    )
)]
pub async fn upsert_column_access_metadata(
    State(state): State<DataApiRestControllerState>,
    Path((table_name, column_name)): Path<(String, String)>,
    headers: HeaderMap,
    Json(resource): Json<DataApiColumnAccessMetadataUpdateRequestResource>,
) -> Result<StatusCode, (StatusCode, Json<DataApiErrorResponseResource>)> {
    let auth = parse_auth_context(&state, &headers).await?;
    let tenant_id = parse_tenant_id(&auth.tenant_id)?;

    state
        .repository
        .synchronize_metadata(&tenant_id, &auth.schema_name)
        .await
        .map_err(map_domain_error)?;

    state
        .repository
        .upsert_column_access_metadata(
            &tenant_id,
            &auth.schema_name,
            &table_name,
            &column_name,
            ColumnMetadataUpdateCriteria {
                readable: resource.readable,
                writable: resource.writable,
            },
        )
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/api/v1/{table_name}",
    tag = "data-api",
    params(
        ("table_name" = String, Path, description = "Nombre de tabla expuesta"),
        ("x-tenant-id" = String, Header, description = "Tenant id"),
        ("x-tenant-schema" = Option<String>, Header, description = "Schema opcional por tenant"),
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso"),
        ("fields" = Option<String>, Query, description = "Campos separados por coma"),
        ("limit" = Option<i64>, Query, description = "Límite (1..500)"),
        ("offset" = Option<i64>, Query, description = "Offset >= 0"),
        ("order_by" = Option<String>, Query, description = "Campo de orden"),
        ("order_dir" = Option<String>, Query, description = "asc|desc"),
    ),
    security(
        ("bearerAuth" = [])
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
    let auth = parse_auth_context(&state, &headers).await?;

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
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    security(
        ("bearerAuth" = [])
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
    let auth = parse_auth_context(&state, &headers).await?;

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
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    request_body = DataApiPayloadResource,
    security(
        ("bearerAuth" = [])
    ),
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

    let auth = parse_auth_context(&state, &headers).await?;

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
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    request_body = DataApiPayloadResource,
    security(
        ("bearerAuth" = [])
    ),
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

    let auth = parse_auth_context(&state, &headers).await?;

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
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    security(
        ("bearerAuth" = [])
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
    let auth = parse_auth_context(&state, &headers).await?;

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
        ("x-request-id" = Option<String>, Header, description = "Correlation id opcional"),
        ("x-subject-owner-id" = Option<String>, Header, description = "Owner id del sujeto"),
        ("x-row-owner-id" = Option<String>, Header, description = "Owner id del recurso")
    ),
    security(
        ("bearerAuth" = [])
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
    let auth = parse_auth_context(&state, &headers).await?;

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

async fn parse_auth_context(
    state: &DataApiRestControllerState,
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

    let token = if let Some(token) = authorization.strip_prefix("Bearer ") {
        let token = token.trim();
        if token.is_empty() {
            return Err(map_domain_error(DataApiDomainError::InvalidAuthentication));
        }
        token.to_string()
    } else {
        return Err(map_domain_error(DataApiDomainError::InvalidAuthentication));
    };

    let verification = state
        .iam_authentication_facade
        .verify_access_token(&token)
        .await
        .map_err(map_iam_error)?;
    let principal = verification.subject_id.as_string();

    let tenant_uuid = Uuid::parse_str(&tenant_id)
        .map_err(|_| map_domain_error(DataApiDomainError::InvalidTenantId))?;
    let user_uuid = verification.subject_id.value();
    let has_ownership = state
        .tenant_ownership_repository
        .exists_ownership(tenant_uuid, user_uuid)
        .await
        .map_err(map_infra_error)?;

    if !has_ownership {
        return Err(map_domain_error(DataApiDomainError::AccessDenied));
    }

    let request_id = headers
        .get("x-request-id")
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .filter(|v| !v.is_empty())
        .map(str::to_string);

    let subject_owner_id = Some(principal.clone());

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
        principal_type: DataApiPrincipalType::Jwt,
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

fn parse_tenant_id(
    tenant_id: &str,
) -> Result<
    crate::data_api::domain::model::value_objects::tenant_id::TenantId,
    (StatusCode, Json<DataApiErrorResponseResource>),
> {
    crate::data_api::domain::model::value_objects::tenant_id::TenantId::new(tenant_id.to_string())
        .map_err(map_domain_error)
}

fn map_iam_error(error: IamIntegrationError) -> (StatusCode, Json<DataApiErrorResponseResource>) {
    match error {
        IamIntegrationError::InvalidToken(message) => (
            StatusCode::UNAUTHORIZED,
            Json(DataApiErrorResponseResource { message }),
        ),
        IamIntegrationError::Unavailable(message) => (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(DataApiErrorResponseResource { message }),
        ),
    }
}

fn map_infra_error(message: String) -> (StatusCode, Json<DataApiErrorResponseResource>) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(DataApiErrorResponseResource { message }),
    )
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
        | DataApiDomainError::InvalidPolicyTemplateName
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
