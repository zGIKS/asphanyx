use std::collections::HashSet;
use swagger_axum_api::data_api::domain::{
    model::enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
    services::data_api_command_service::DataApiCommandService,
};

use crate::support::{
    create_command_harness, create_row_command, patch_row_command, sample_payload,
};

#[tokio::test]
async fn handle_create_routes_to_tenant_and_audits_success() {
    let harness = create_command_harness(&["productos"]);

    let result = harness
        .service
        .handle_create(create_row_command(sample_payload()))
        .await;

    let payload = result.expect("create should succeed");
    assert_eq!(payload["table"], "productos");
    assert_eq!(harness.repository.create_calls(), 1);
    assert_eq!(
        harness.repository.last_tenant_for_create().as_deref(),
        Some("tienda1")
    );
    assert_eq!(harness.tenant_schema_resolver.calls(), 1);

    let acl_calls = harness.access_control.calls();
    assert_eq!(acl_calls.len(), 1);
    assert_eq!(acl_calls[0].action_name, "create");
    assert_eq!(acl_calls[0].table_name, "productos");
    assert_eq!(acl_calls[0].tenant_id, "tienda1");
    assert_eq!(acl_calls[0].principal, "api-key-test");
    assert_eq!(acl_calls[0].request_id.as_deref(), Some("req-1"));
    assert_eq!(acl_calls[0].subject_owner_id.as_deref(), Some("owner-1"));
    assert_eq!(acl_calls[0].row_owner_id.as_deref(), Some("owner-1"));
    let column_set = acl_calls[0].columns.iter().cloned().collect::<HashSet<_>>();
    assert_eq!(
        column_set,
        HashSet::from([
            "nombre".to_string(),
            "precio".to_string(),
            "image_url".to_string()
        ])
    );

    let audit_events = harness.audit.saved_events();
    assert_eq!(audit_events.len(), 1);
    assert!(matches!(audit_events[0].action, DataApiAction::Create));
    assert!(audit_events[0].success);
    assert_eq!(audit_events[0].status_code, 201);
}

#[tokio::test]
async fn handle_create_fails_when_table_not_in_allowlist() {
    let harness = create_command_harness(&["ordenes"]);

    let result = harness
        .service
        .handle_create(create_row_command(sample_payload()))
        .await;

    assert!(matches!(result, Err(DataApiDomainError::TableNotAllowed)));
    assert_eq!(harness.repository.create_calls(), 0);
    assert!(harness.audit.saved_events().is_empty());
}

#[tokio::test]
async fn handle_patch_returns_not_found_when_row_does_not_exist() {
    let harness = create_command_harness(&["productos"]);
    harness.repository.set_patch_should_return_none(true);

    let result = harness
        .service
        .handle_patch(patch_row_command(sample_payload()))
        .await;

    assert!(matches!(result, Err(DataApiDomainError::RecordNotFound)));
    assert_eq!(harness.repository.patch_calls(), 1);

    let audit_events = harness.audit.saved_events();
    assert_eq!(audit_events.len(), 1);
    assert!(!audit_events[0].success);
    assert_eq!(audit_events[0].status_code, 404);
}

#[tokio::test]
async fn handle_create_returns_infra_error_and_audits_failure() {
    let harness = create_command_harness(&["productos"]);
    harness.repository.set_create_should_fail(true);

    let result = harness
        .service
        .handle_create(create_row_command(sample_payload()))
        .await;

    assert!(matches!(
        result,
        Err(DataApiDomainError::InfrastructureError(message)) if message == "create failed"
    ));
    assert_eq!(harness.repository.create_calls(), 1);

    let audit_events = harness.audit.saved_events();
    assert_eq!(audit_events.len(), 1);
    assert!(!audit_events[0].success);
    assert_eq!(audit_events[0].status_code, 500);
}

#[tokio::test]
async fn handle_create_fails_when_acl_denies_access() {
    let harness = create_command_harness(&["productos"]);
    harness.access_control.set_deny(true);

    let result = harness
        .service
        .handle_create(create_row_command(sample_payload()))
        .await;

    assert!(matches!(result, Err(DataApiDomainError::AccessDenied)));
}
