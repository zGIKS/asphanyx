use std::time::Duration;

use swagger_axum_api::access_control::{
    domain::{
        model::enums::permission_effect::PermissionEffect,
        services::access_control_query_service::AccessControlQueryService,
    },
    infrastructure::persistence::repositories::policy_rule_repository::PolicyRuleRecord,
};

use crate::support::{
    create_query_harness, evaluate_query, evaluate_query_with_columns,
    evaluate_query_with_request_id, TENANT_A_ID,
};

#[tokio::test]
async fn evaluate_permission_default_deny_when_no_roles() {
    let harness = create_query_harness(Duration::from_secs(30));

    let result = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await;

    let decision = result.expect("decision expected");
    assert!(!decision.allowed);
    assert_eq!(decision.reason, "no roles assigned");
}

#[tokio::test]
async fn evaluate_permission_allow_when_matching_allow_rule() {
    let harness = create_query_harness(Duration::from_secs(30));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![PolicyRuleRecord {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    }]);

    let decision = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("decision expected");

    assert!(decision.allowed);
    assert_eq!(decision.reason, "allow rule matched");
}

#[tokio::test]
async fn evaluate_permission_deny_wins_when_same_specificity_conflict() {
    let harness = create_query_harness(Duration::from_secs(30));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![
        PolicyRuleRecord {
            tenant_id: TENANT_A_ID.to_string(),
            role_name: "admin".to_string(),
            resource_name: "productos".to_string(),
            action_name: "read".to_string(),
            effect: PermissionEffect::Allow,
            allowed_columns: None,
            denied_columns: None,
            owner_scope: false,
        },
        PolicyRuleRecord {
            tenant_id: TENANT_A_ID.to_string(),
            role_name: "admin".to_string(),
            resource_name: "productos".to_string(),
            action_name: "read".to_string(),
            effect: PermissionEffect::Deny,
            allowed_columns: None,
            denied_columns: None,
            owner_scope: false,
        },
    ]);

    let decision = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("decision expected");

    assert!(!decision.allowed);
    assert_eq!(decision.reason, "deny rule won by precedence");
}

#[tokio::test]
async fn evaluate_permission_specific_allow_beats_wildcard_deny() {
    let harness = create_query_harness(Duration::from_secs(30));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![
        PolicyRuleRecord {
            tenant_id: TENANT_A_ID.to_string(),
            role_name: "admin".to_string(),
            resource_name: "*".to_string(),
            action_name: "*".to_string(),
            effect: PermissionEffect::Deny,
            allowed_columns: None,
            denied_columns: None,
            owner_scope: false,
        },
        PolicyRuleRecord {
            tenant_id: TENANT_A_ID.to_string(),
            role_name: "admin".to_string(),
            resource_name: "productos".to_string(),
            action_name: "read".to_string(),
            effect: PermissionEffect::Allow,
            allowed_columns: None,
            denied_columns: None,
            owner_scope: false,
        },
    ]);

    let decision = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("decision expected");

    assert!(decision.allowed);
    assert_eq!(decision.reason, "allow rule won by specificity");
}

#[tokio::test]
async fn evaluate_permission_denies_when_requested_column_is_denied() {
    let harness = create_query_harness(Duration::from_secs(30));
    harness
        .role_repository
        .set_roles(vec!["editor".to_string()]);
    harness.policy_repository.set_rules(vec![PolicyRuleRecord {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "editor".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: Some(vec!["precio".to_string()]),
        owner_scope: false,
    }]);

    let decision = harness
        .service
        .handle_evaluate_permission(evaluate_query_with_columns(vec!["precio"]))
        .await
        .expect("decision expected");

    assert!(!decision.allowed);
    assert_eq!(decision.reason, "no rule matched context/columns");
}

#[tokio::test]
async fn evaluate_permission_caches_decision_within_ttl() {
    let harness = create_query_harness(Duration::from_secs(2));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![PolicyRuleRecord {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    }]);

    let first = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("decision expected");
    let second = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("decision expected");

    assert!(first.allowed && second.allowed);
    assert_eq!(harness.role_repository.find_calls(), 1);
    assert_eq!(harness.policy_repository.find_calls(), 1);
}

#[tokio::test]
async fn evaluate_permission_cache_expires_after_ttl() {
    let harness = create_query_harness(Duration::from_millis(20));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![PolicyRuleRecord {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    }]);

    let _ = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("first decision expected");

    tokio::time::sleep(Duration::from_millis(30)).await;

    let _ = harness
        .service
        .handle_evaluate_permission(evaluate_query())
        .await
        .expect("second decision expected");

    assert_eq!(harness.role_repository.find_calls(), 2);
    assert_eq!(harness.policy_repository.find_calls(), 2);
}

#[tokio::test]
async fn evaluate_permission_audits_request_id() {
    let harness = create_query_harness(Duration::from_secs(30));
    harness.role_repository.set_roles(vec!["admin".to_string()]);
    harness.policy_repository.set_rules(vec![PolicyRuleRecord {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    }]);

    let _ = harness
        .service
        .handle_evaluate_permission(evaluate_query_with_request_id("req-123"))
        .await
        .expect("decision expected");

    let events = harness.audit_repository.events();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].request_id.as_deref(), Some("req-123"));
}
