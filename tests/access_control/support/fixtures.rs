use swagger_axum_api::access_control::domain::model::{
    commands::{
        assign_role_to_principal_command::AssignRoleToPrincipalCommand,
        upsert_policy_rule_command::{UpsertPolicyRuleCommand, UpsertPolicyRuleCommandParts},
    },
    enums::permission_effect::PermissionEffect,
    queries::evaluate_permission_query::{EvaluatePermissionQuery, EvaluatePermissionQueryParts},
};

// UUIDs de prueba consistentes
pub const TENANT_A_ID: &str = "01234567-89ab-7def-0123-456789abcdef";
pub const PRINCIPAL_1_ID: &str = "fedcba98-7654-7321-fedc-ba9876543210";

pub fn assign_role_command() -> AssignRoleToPrincipalCommand {
    AssignRoleToPrincipalCommand::new(
        TENANT_A_ID.to_string(),
        PRINCIPAL_1_ID.to_string(),
        "admin".to_string(),
    )
    .expect("valid assign role command")
}

pub fn upsert_policy_allow_all_command() -> UpsertPolicyRuleCommand {
    UpsertPolicyRuleCommand::new(UpsertPolicyRuleCommandParts {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Allow,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    })
    .expect("valid allow command")
}

pub fn upsert_policy_deny_all_command() -> UpsertPolicyRuleCommand {
    UpsertPolicyRuleCommand::new(UpsertPolicyRuleCommandParts {
        tenant_id: TENANT_A_ID.to_string(),
        role_name: "admin".to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        effect: PermissionEffect::Deny,
        allowed_columns: None,
        denied_columns: None,
        owner_scope: false,
    })
    .expect("valid deny command")
}

pub fn evaluate_query() -> EvaluatePermissionQuery {
    EvaluatePermissionQuery::new(EvaluatePermissionQueryParts {
        tenant_id: TENANT_A_ID.to_string(),
        principal_id: PRINCIPAL_1_ID.to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        requested_columns: vec![],
        subject_owner_id: None,
        row_owner_id: None,
        request_id: None,
    })
    .expect("valid evaluate query")
}

pub fn evaluate_query_with_columns(columns: Vec<&str>) -> EvaluatePermissionQuery {
    EvaluatePermissionQuery::new(EvaluatePermissionQueryParts {
        tenant_id: TENANT_A_ID.to_string(),
        principal_id: PRINCIPAL_1_ID.to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        requested_columns: columns.into_iter().map(str::to_string).collect(),
        subject_owner_id: None,
        row_owner_id: None,
        request_id: None,
    })
    .expect("valid evaluate query with columns")
}

pub fn evaluate_query_with_request_id(request_id: &str) -> EvaluatePermissionQuery {
    EvaluatePermissionQuery::new(EvaluatePermissionQueryParts {
        tenant_id: TENANT_A_ID.to_string(),
        principal_id: PRINCIPAL_1_ID.to_string(),
        resource_name: "productos".to_string(),
        action_name: "read".to_string(),
        requested_columns: vec![],
        subject_owner_id: None,
        row_owner_id: None,
        request_id: Some(request_id.to_string()),
    })
    .expect("valid evaluate query with request_id")
}
