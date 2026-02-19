use swagger_axum_api::access_control::{
    domain::services::access_control_command_service::AccessControlCommandService,
    infrastructure::persistence::repositories::policy_rule_repository::PolicyRuleRecord,
};

use crate::support::{
    assign_role_command, create_command_harness, upsert_policy_allow_all_command,
    upsert_policy_deny_all_command,
};

#[tokio::test]
async fn handle_assign_role_persists_assignment() {
    let harness = create_command_harness();

    let result = harness
        .service
        .handle_assign_role(assign_role_command())
        .await;

    assert!(result.is_ok());
    assert_eq!(harness.role_repository.assign_calls(), 1);
}

#[tokio::test]
async fn handle_upsert_policy_persists_rule_with_denied_columns() {
    let harness = create_command_harness();

    let command = upsert_policy_allow_all_command();
    let result = harness.service.handle_upsert_policy(command).await;

    assert!(result.is_ok());
    assert_eq!(harness.policy_repository.upsert_calls(), 1);
    let last_upsert = harness
        .policy_repository
        .last_upsert()
        .expect("upsert should be captured");
    assert_eq!(last_upsert.resource_name, "productos");
    assert!(last_upsert.denied_columns.is_none());
}

#[tokio::test]
async fn handle_upsert_policy_accepts_deny_effect() {
    let harness = create_command_harness();

    let result = harness
        .service
        .handle_upsert_policy(upsert_policy_deny_all_command())
        .await;

    assert!(result.is_ok());
    assert_eq!(harness.policy_repository.upsert_calls(), 1);
    let last: PolicyRuleRecord = harness
        .policy_repository
        .last_upsert()
        .expect("deny upsert should be captured");
    assert_eq!(last.effect.as_str(), "deny");
}
