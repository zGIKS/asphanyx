use swagger_axum_api::data_api::domain::{
    model::enums::data_api_domain_error::DataApiDomainError,
    services::data_api_query_service::DataApiQueryService,
};

use crate::support::{create_query_harness, get_row_query, list_rows_query, fixtures};

#[tokio::test]
async fn handle_list_filters_unknown_columns_before_repository_call() {
    let harness = create_query_harness(&["productos"]);

    let result = harness.service.handle_list(list_rows_query()).await;

    assert!(result.is_ok());
    let criteria = harness
        .repository
        .last_list_criteria()
        .expect("list criteria should be captured");
    assert_eq!(criteria.fields, vec!["nombre".to_string()]);
    assert_eq!(
        criteria.filters,
        vec![("nombre".to_string(), "Mouse".to_string())]
    );
    assert_eq!(criteria.order_by, None);
    assert_eq!(
        harness.repository.last_tenant_for_list().as_deref(),
        Some(fixtures::TENANT_1_ID)
    );
    assert_eq!(harness.tenant_schema_resolver.calls(), 1);

    let acl_calls = harness.access_control.calls();
    assert_eq!(acl_calls.len(), 1);
    assert_eq!(acl_calls[0].action_name, "read");

    let audit_events = harness.audit.saved_events();
    assert_eq!(audit_events.len(), 1);
    assert!(audit_events[0].success);
    assert_eq!(audit_events[0].status_code, 200);
}

#[tokio::test]
async fn handle_get_returns_not_found_when_repository_returns_none() {
    let harness = create_query_harness(&["productos"]);
    harness.repository.set_get_should_return_none(true);

    let result = harness.service.handle_get(get_row_query()).await;

    assert!(matches!(result, Err(DataApiDomainError::RecordNotFound)));
    let audit_events = harness.audit.saved_events();
    assert_eq!(audit_events.len(), 1);
    assert!(!audit_events[0].success);
    assert_eq!(audit_events[0].status_code, 404);
}

#[tokio::test]
async fn handle_get_fails_when_primary_key_is_missing() {
    let harness = create_query_harness(&["productos"]);
    harness.repository.set_without_primary_key();

    let result = harness.service.handle_get(get_row_query()).await;

    assert!(matches!(
        result,
        Err(DataApiDomainError::PrimaryKeyNotFound)
    ));
}
