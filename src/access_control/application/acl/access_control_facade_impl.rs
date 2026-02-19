use std::sync::Arc;

use async_trait::async_trait;

use crate::access_control::{
    domain::model::{
        commands::{
            assign_role_to_principal_command::AssignRoleToPrincipalCommand,
            upsert_policy_rule_command::{UpsertPolicyRuleCommand, UpsertPolicyRuleCommandParts},
        },
        enums::permission_effect::PermissionEffect,
    },
    domain::{
        model::queries::evaluate_permission_query::{
            EvaluatePermissionQuery, EvaluatePermissionQueryParts,
        },
        services::access_control_command_service::AccessControlCommandService,
        services::access_control_query_service::AccessControlQueryService,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, AccessControlPermissionDecision, AccessControlPermissionRequest,
        DataApiAccessBootstrapRequest,
    },
};

pub struct AccessControlFacadeImpl {
    command_service: Arc<dyn AccessControlCommandService>,
    query_service: Arc<dyn AccessControlQueryService>,
}

impl AccessControlFacadeImpl {
    pub fn new(
        command_service: Arc<dyn AccessControlCommandService>,
        query_service: Arc<dyn AccessControlQueryService>,
    ) -> Self {
        Self {
            command_service,
            query_service,
        }
    }
}

#[async_trait]
impl AccessControlFacade for AccessControlFacadeImpl {
    async fn check_permission(
        &self,
        request: AccessControlPermissionRequest,
    ) -> Result<AccessControlPermissionDecision, crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError>{
        let query = EvaluatePermissionQuery::new(EvaluatePermissionQueryParts {
            tenant_id: request.tenant_id,
            principal_id: request.principal_id,
            resource_name: request.resource_name,
            action_name: request.action_name,
            requested_columns: request.requested_columns,
            subject_owner_id: request.subject_owner_id,
            row_owner_id: request.row_owner_id,
            request_id: request.request_id,
        })?;

        let result = self.query_service.handle_evaluate_permission(query).await?;

        Ok(AccessControlPermissionDecision {
            allowed: result.allowed,
            reason: result.reason,
        })
    }

    async fn bootstrap_data_api_access(
        &self,
        request: DataApiAccessBootstrapRequest,
    ) -> Result<(), crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError>{
        let role_name = "data_api_authenticated".to_string();

        self.command_service
            .handle_assign_role(AssignRoleToPrincipalCommand::new(
                request.tenant_id.clone(),
                request.principal_id,
                role_name.clone(),
            )?)
            .await?;

        let unique_readable = request
            .readable_columns
            .into_iter()
            .filter(|column| !column.trim().is_empty())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        let unique_writable = request
            .writable_columns
            .into_iter()
            .filter(|column| !column.trim().is_empty())
            .collect::<std::collections::BTreeSet<_>>()
            .into_iter()
            .collect::<Vec<_>>();

        self.command_service
            .handle_upsert_policy(UpsertPolicyRuleCommand::new(
                UpsertPolicyRuleCommandParts {
                    tenant_id: request.tenant_id.clone(),
                    role_name: role_name.clone(),
                    resource_name: request.resource_name.clone(),
                    action_name: "read".to_string(),
                    effect: PermissionEffect::Allow,
                    allowed_columns: if unique_readable.is_empty() {
                        None
                    } else {
                        Some(unique_readable)
                    },
                    denied_columns: None,
                    owner_scope: false,
                },
            )?)
            .await?;

        self.command_service
            .handle_upsert_policy(UpsertPolicyRuleCommand::new(
                UpsertPolicyRuleCommandParts {
                    tenant_id: request.tenant_id.clone(),
                    role_name: role_name.clone(),
                    resource_name: request.resource_name.clone(),
                    action_name: "create".to_string(),
                    effect: PermissionEffect::Allow,
                    allowed_columns: if unique_writable.is_empty() {
                        None
                    } else {
                        Some(unique_writable.clone())
                    },
                    denied_columns: None,
                    owner_scope: false,
                },
            )?)
            .await?;

        self.command_service
            .handle_upsert_policy(UpsertPolicyRuleCommand::new(
                UpsertPolicyRuleCommandParts {
                    tenant_id: request.tenant_id.clone(),
                    role_name: role_name.clone(),
                    resource_name: request.resource_name.clone(),
                    action_name: "update".to_string(),
                    effect: PermissionEffect::Allow,
                    allowed_columns: if unique_writable.is_empty() {
                        None
                    } else {
                        Some(unique_writable)
                    },
                    denied_columns: None,
                    owner_scope: false,
                },
            )?)
            .await?;

        self.command_service
            .handle_upsert_policy(UpsertPolicyRuleCommand::new(
                UpsertPolicyRuleCommandParts {
                    tenant_id: request.tenant_id,
                    role_name,
                    resource_name: request.resource_name,
                    action_name: "delete".to_string(),
                    effect: PermissionEffect::Allow,
                    allowed_columns: None,
                    denied_columns: None,
                    owner_scope: false,
                },
            )?)
            .await
    }
}
