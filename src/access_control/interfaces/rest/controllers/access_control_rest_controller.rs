use std::sync::Arc;

use axum::{Json, Router, extract::State, http::StatusCode, routing::post};
use validator::Validate;

use crate::access_control::{
    domain::{
        model::{
            commands::{
                assign_role_to_principal_command::AssignRoleToPrincipalCommand,
                upsert_policy_rule_command::{
                    UpsertPolicyRuleCommand, UpsertPolicyRuleCommandParts,
                },
            },
            enums::{
                access_control_domain_error::AccessControlDomainError,
                permission_effect::PermissionEffect,
            },
            queries::evaluate_permission_query::{
                EvaluatePermissionQuery, EvaluatePermissionQueryParts,
            },
        },
        services::{
            access_control_command_service::AccessControlCommandService,
            access_control_query_service::AccessControlQueryService,
        },
    },
    interfaces::rest::resources::{
        access_control_error_response_resource::AccessControlErrorResponseResource,
        assign_role_request_resource::AssignRoleRequestResource,
        evaluate_permission_request_resource::{
            EvaluatePermissionRequestResource, EvaluatePermissionResponseResource,
        },
        upsert_policy_rule_request_resource::UpsertPolicyRuleRequestResource,
    },
};

#[derive(Clone)]
pub struct AccessControlRestControllerState {
    pub command_service: Arc<dyn AccessControlCommandService>,
    pub query_service: Arc<dyn AccessControlQueryService>,
}

pub fn router(state: AccessControlRestControllerState) -> Router {
    Router::new()
        .route(
            "/access-control/roles/assign",
            post(assign_role_to_principal),
        )
        .route("/access-control/policies/upsert", post(upsert_policy_rule))
        .route(
            "/access-control/permissions/evaluate",
            post(evaluate_permission),
        )
        .with_state(state)
}

#[utoipa::path(
    post,
    path = "/access-control/roles/assign",
    tag = "access-control",
    request_body = AssignRoleRequestResource,
    responses(
        (status = 204, description = "Role assigned"),
        (status = 400, description = "Invalid request", body = AccessControlErrorResponseResource),
        (status = 500, description = "Infrastructure error", body = AccessControlErrorResponseResource)
    )
)]
pub async fn assign_role_to_principal(
    State(state): State<AccessControlRestControllerState>,
    Json(request): Json<AssignRoleRequestResource>,
) -> Result<StatusCode, (StatusCode, Json<AccessControlErrorResponseResource>)> {
    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AccessControlErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let command = AssignRoleToPrincipalCommand::new(
        request.tenant_id,
        request.principal_id,
        request.role_name,
    )
    .map_err(map_domain_error)?;

    state
        .command_service
        .handle_assign_role(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/access-control/policies/upsert",
    tag = "access-control",
    request_body = UpsertPolicyRuleRequestResource,
    responses(
        (status = 204, description = "Policy upserted"),
        (status = 400, description = "Invalid request", body = AccessControlErrorResponseResource),
        (status = 500, description = "Infrastructure error", body = AccessControlErrorResponseResource)
    )
)]
pub async fn upsert_policy_rule(
    State(state): State<AccessControlRestControllerState>,
    Json(request): Json<UpsertPolicyRuleRequestResource>,
) -> Result<StatusCode, (StatusCode, Json<AccessControlErrorResponseResource>)> {
    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AccessControlErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let effect = match request.effect.as_str() {
        "allow" => PermissionEffect::Allow,
        "deny" => PermissionEffect::Deny,
        _ => {
            return Err(map_domain_error(
                AccessControlDomainError::InfrastructureError("invalid effect".to_string()),
            ));
        }
    };

    let command = UpsertPolicyRuleCommand::new(UpsertPolicyRuleCommandParts {
        tenant_id: request.tenant_id,
        role_name: request.role_name,
        resource_name: request.resource_name,
        action_name: request.action_name,
        effect,
        allowed_columns: request.allowed_columns,
        denied_columns: request.denied_columns,
        owner_scope: request.owner_scope,
    })
    .map_err(map_domain_error)?;

    state
        .command_service
        .handle_upsert_policy(command)
        .await
        .map_err(map_domain_error)?;

    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/access-control/permissions/evaluate",
    tag = "access-control",
    request_body = EvaluatePermissionRequestResource,
    responses(
        (status = 200, description = "Authorization decision", body = EvaluatePermissionResponseResource),
        (status = 400, description = "Invalid request", body = AccessControlErrorResponseResource),
        (status = 500, description = "Infrastructure error", body = AccessControlErrorResponseResource)
    )
)]
pub async fn evaluate_permission(
    State(state): State<AccessControlRestControllerState>,
    Json(request): Json<EvaluatePermissionRequestResource>,
) -> Result<
    Json<EvaluatePermissionResponseResource>,
    (StatusCode, Json<AccessControlErrorResponseResource>),
> {
    if let Err(validation_error) = request.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(AccessControlErrorResponseResource {
                message: validation_error.to_string(),
            }),
        ));
    }

    let query = EvaluatePermissionQuery::new(EvaluatePermissionQueryParts {
        tenant_id: request.tenant_id,
        principal_id: request.principal_id,
        resource_name: request.resource_name,
        action_name: request.action_name,
        requested_columns: request.requested_columns,
        subject_owner_id: request.subject_owner_id,
        row_owner_id: request.row_owner_id,
        request_id: request.request_id,
    })
    .map_err(map_domain_error)?;

    let decision = state
        .query_service
        .handle_evaluate_permission(query)
        .await
        .map_err(map_domain_error)?;

    Ok(Json(EvaluatePermissionResponseResource {
        allowed: decision.allowed,
        reason: decision.reason,
    }))
}

fn map_domain_error(
    error: AccessControlDomainError,
) -> (StatusCode, Json<AccessControlErrorResponseResource>) {
    let status = match error {
        AccessControlDomainError::InvalidTenantId
        | AccessControlDomainError::InvalidPrincipalId
        | AccessControlDomainError::InvalidRoleName
        | AccessControlDomainError::InvalidResourceName
        | AccessControlDomainError::InvalidActionName
        | AccessControlDomainError::PolicyNotFound => StatusCode::BAD_REQUEST,
        AccessControlDomainError::AccessDenied => StatusCode::FORBIDDEN,
        AccessControlDomainError::InfrastructureError(_) => StatusCode::INTERNAL_SERVER_ERROR,
    };

    (
        status,
        Json(AccessControlErrorResponseResource {
            message: error.to_string(),
        }),
    )
}
