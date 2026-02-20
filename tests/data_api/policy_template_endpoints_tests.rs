use std::{collections::HashSet, sync::Mutex};

use async_trait::async_trait;
use axum::{
    Json,
    extract::{Path, State},
    http::{HeaderMap, HeaderValue, StatusCode},
};
use swagger_axum_api::{
    data_api::{
        application::{
            command_services::{
                data_api_command_service_impl::DataApiCommandServiceImpl,
                data_api_policy_template_command_service_impl::DataApiPolicyTemplateCommandServiceImpl,
            },
            query_services::{
                data_api_policy_template_query_service_impl::DataApiPolicyTemplateQueryServiceImpl,
                data_api_query_service_impl::DataApiQueryServiceImpl,
            },
        },
        domain::services::{
            data_api_command_service::DataApiCommandService,
            data_api_policy_template_command_service::DataApiPolicyTemplateCommandService,
            data_api_policy_template_query_service::DataApiPolicyTemplateQueryService,
            data_api_query_service::DataApiQueryService,
        },
        interfaces::rest::{
            controllers::data_api_rest_controller::{
                DataApiRestControllerState, apply_table_policy_template, create_row, delete_row,
                get_row, patch_row,
            },
            resources::{
                apply_table_policy_template_request_resource::ApplyTablePolicyTemplateRequestResource,
                data_api_payload_resource::DataApiPayloadResource,
            },
        },
    },
    iam_integration::{
        domain::model::value_objects::authenticated_user_id::AuthenticatedUserId,
        interfaces::acl::iam_authentication_facade::{
            IamAuthenticationFacade, IamIntegrationError, VerifiedUserContext,
        },
    },
    provisioner::infrastructure::persistence::repositories::tenant_ownership_repository::TenantOwnershipRepository,
};
use uuid::Uuid;

use crate::support::{
    fakes::{
        FakeAccessControlFacade, FakeDataApiAuditLogRepository, FakeDataApiRepository,
        FakeTenantSchemaResolverRepository,
    },
    fixtures::{TENANT_1_ID, sample_payload},
};

const USER_A_ID: &str = "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa";
const USER_B_ID: &str = "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb";

struct EndpointHarness {
    state: DataApiRestControllerState,
    access_control: std::sync::Arc<FakeAccessControlFacade>,
}

fn build_harness(owner_user_id: &str) -> EndpointHarness {
    let repository = std::sync::Arc::new(FakeDataApiRepository::new());
    let tenant_schema_resolver =
        std::sync::Arc::new(FakeTenantSchemaResolverRepository::new("public"));
    let access_control = std::sync::Arc::new(FakeAccessControlFacade::new());
    let audit = std::sync::Arc::new(FakeDataApiAuditLogRepository::new());
    let iam = std::sync::Arc::new(FakeIamAuthenticationFacade);
    let ownership = std::sync::Arc::new(FakeTenantOwnershipRepository::new(
        TENANT_1_ID,
        owner_user_id,
    ));

    let command_service: std::sync::Arc<dyn DataApiCommandService> =
        std::sync::Arc::new(DataApiCommandServiceImpl::new(
            repository.clone(),
            tenant_schema_resolver.clone(),
            access_control.clone(),
            audit.clone(),
        ));
    let query_service: std::sync::Arc<dyn DataApiQueryService> =
        std::sync::Arc::new(DataApiQueryServiceImpl::new(
            repository.clone(),
            tenant_schema_resolver.clone(),
            access_control.clone(),
            audit,
        ));
    let policy_template_command_service: std::sync::Arc<dyn DataApiPolicyTemplateCommandService> =
        std::sync::Arc::new(DataApiPolicyTemplateCommandServiceImpl::new(
            repository.clone(),
            tenant_schema_resolver,
            access_control.clone(),
        ));
    let policy_template_query_service: std::sync::Arc<dyn DataApiPolicyTemplateQueryService> =
        std::sync::Arc::new(DataApiPolicyTemplateQueryServiceImpl::new());

    EndpointHarness {
        state: DataApiRestControllerState {
            command_service,
            query_service,
            policy_template_command_service,
            policy_template_query_service,
            repository,
            iam_authentication_facade: iam,
            tenant_ownership_repository: ownership,
        },
        access_control,
    }
}

fn headers_for(user_id: &str) -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("x-tenant-id", HeaderValue::from_static(TENANT_1_ID));
    headers.insert(
        "authorization",
        HeaderValue::from_str(&format!("Bearer {user_id}")).expect("valid auth header"),
    );
    headers
}

async fn apply_template(
    state: DataApiRestControllerState,
    user_id: &str,
    template_name: &str,
) -> Result<StatusCode, (StatusCode, Json<swagger_axum_api::data_api::interfaces::rest::resources::data_api_error_response_resource::DataApiErrorResponseResource>)>
{
    apply_table_policy_template(
        State(state),
        Path("productos".to_string()),
        headers_for(user_id),
        Json(ApplyTablePolicyTemplateRequestResource {
            template_name: template_name.to_string(),
        }),
    )
    .await
}

#[tokio::test]
async fn apply_acl_read_only_blocks_writes_and_allows_get() {
    let harness = build_harness(USER_A_ID);

    let apply_result = apply_template(harness.state.clone(), USER_A_ID, "acl_read_only").await;
    assert_eq!(
        apply_result.expect("template apply should succeed"),
        StatusCode::NO_CONTENT
    );

    let create_result = create_row(
        State(harness.state.clone()),
        Path("productos".to_string()),
        headers_for(USER_A_ID),
        Json(DataApiPayloadResource {
            payload: sample_payload(),
        }),
    )
    .await;
    assert!(matches!(create_result, Err((StatusCode::FORBIDDEN, _))));

    let patch_result = patch_row(
        State(harness.state.clone()),
        Path(("productos".to_string(), "1".to_string())),
        headers_for(USER_A_ID),
        Json(DataApiPayloadResource {
            payload: sample_payload(),
        }),
    )
    .await;
    assert!(matches!(patch_result, Err((StatusCode::FORBIDDEN, _))));

    let delete_result = delete_row(
        State(harness.state.clone()),
        Path(("productos".to_string(), "1".to_string())),
        headers_for(USER_A_ID),
    )
    .await;
    assert!(matches!(delete_result, Err((StatusCode::FORBIDDEN, _))));

    let get_result = get_row(
        State(harness.state),
        Path(("productos".to_string(), "1".to_string())),
        headers_for(USER_A_ID),
    )
    .await;
    assert!(get_result.is_ok());
}

#[tokio::test]
async fn apply_acl_crud_allows_crud_after_read_only() {
    let harness = build_harness(USER_A_ID);

    let _ = apply_template(harness.state.clone(), USER_A_ID, "acl_read_only").await;
    let apply_result = apply_template(harness.state.clone(), USER_A_ID, "acl_crud").await;
    assert_eq!(
        apply_result.expect("template apply should succeed"),
        StatusCode::NO_CONTENT
    );

    let create_result = create_row(
        State(harness.state.clone()),
        Path("productos".to_string()),
        headers_for(USER_A_ID),
        Json(DataApiPayloadResource {
            payload: sample_payload(),
        }),
    )
    .await;
    assert!(create_result.is_ok());

    let patch_result = patch_row(
        State(harness.state.clone()),
        Path(("productos".to_string(), "1".to_string())),
        headers_for(USER_A_ID),
        Json(DataApiPayloadResource {
            payload: sample_payload(),
        }),
    )
    .await;
    assert!(patch_result.is_ok());

    let delete_result = delete_row(
        State(harness.state.clone()),
        Path(("productos".to_string(), "1".to_string())),
        headers_for(USER_A_ID),
    )
    .await;
    assert_eq!(
        delete_result.expect("delete should succeed"),
        StatusCode::NO_CONTENT
    );
}

#[tokio::test]
async fn apply_authenticated_crud_works_without_acl_rules() {
    let harness = build_harness(USER_A_ID);
    harness.access_control.set_deny(true);

    let apply_result = apply_template(harness.state.clone(), USER_A_ID, "authenticated_crud").await;
    assert_eq!(
        apply_result.expect("template apply should succeed"),
        StatusCode::NO_CONTENT
    );
    assert!(harness.access_control.template_calls().is_empty());

    let create_result = create_row(
        State(harness.state.clone()),
        Path("productos".to_string()),
        headers_for(USER_A_ID),
        Json(DataApiPayloadResource {
            payload: sample_payload(),
        }),
    )
    .await;
    assert!(create_result.is_ok());
    assert!(harness.access_control.calls().is_empty());
}

#[tokio::test]
async fn user_without_tenant_ownership_cannot_apply_template() {
    let harness = build_harness(USER_A_ID);

    let result = apply_template(harness.state, USER_B_ID, "acl_crud").await;
    assert!(matches!(result, Err((StatusCode::FORBIDDEN, _))));
}

struct FakeIamAuthenticationFacade;

#[async_trait]
impl IamAuthenticationFacade for FakeIamAuthenticationFacade {
    async fn verify_access_token(
        &self,
        access_token: &str,
    ) -> Result<VerifiedUserContext, IamIntegrationError> {
        let subject_id =
            AuthenticatedUserId::new(access_token).map_err(IamIntegrationError::InvalidToken)?;

        Ok(VerifiedUserContext {
            subject_id,
            jti: None,
            exp_epoch_seconds: u64::MAX,
        })
    }
}

struct FakeTenantOwnershipRepository {
    ownership_pairs: Mutex<HashSet<(Uuid, Uuid)>>,
}

impl FakeTenantOwnershipRepository {
    fn new(tenant_id: &str, user_id: &str) -> Self {
        let tenant_uuid = Uuid::parse_str(tenant_id).expect("valid tenant id");
        let user_uuid = Uuid::parse_str(user_id).expect("valid user id");
        let mut ownership_pairs = HashSet::new();
        ownership_pairs.insert((tenant_uuid, user_uuid));
        Self {
            ownership_pairs: Mutex::new(ownership_pairs),
        }
    }
}

#[async_trait]
impl TenantOwnershipRepository for FakeTenantOwnershipRepository {
    async fn save_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), String> {
        self.ownership_pairs
            .lock()
            .expect("mutex poisoned")
            .insert((tenant_id, user_id));
        Ok(())
    }

    async fn exists_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<bool, String> {
        Ok(self
            .ownership_pairs
            .lock()
            .expect("mutex poisoned")
            .contains(&(tenant_id, user_id)))
    }

    async fn list_tenant_ids_by_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, String> {
        Ok(self
            .ownership_pairs
            .lock()
            .expect("mutex poisoned")
            .iter()
            .filter_map(|(tenant_id, owner_id)| {
                if owner_id == &user_id {
                    Some(*tenant_id)
                } else {
                    None
                }
            })
            .collect())
    }
}
