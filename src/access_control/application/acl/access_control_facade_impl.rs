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
        DataApiAccessBootstrapRequest, DataApiPolicyBatchUpsertRequest,
        DataApiPolicyRuleUpsertRequest,
    },
};

const DATA_API_AUTHENTICATED_ROLE: &str = "data_api_authenticated";

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

    async fn assign_data_api_authenticated_role(
        &self,
        tenant_id: String,
        principal_id: String,
    ) -> Result<(), crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError>
    {
        self.command_service
            .handle_assign_role(AssignRoleToPrincipalCommand::new(
                tenant_id,
                principal_id,
                DATA_API_AUTHENTICATED_ROLE.to_string(),
            )?)
            .await
    }

    async fn upsert_data_api_policy(
        &self,
        tenant_id: String,
        resource_name: String,
        policy: DataApiPolicyRuleUpsertRequest,
    ) -> Result<(), crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError>
    {
        let unique_allowed = policy
            .allowed_columns
            .map(|columns| {
                columns
                    .into_iter()
                    .filter(|column| !column.trim().is_empty())
                    .collect::<std::collections::BTreeSet<_>>()
                    .into_iter()
                    .collect::<Vec<_>>()
            })
            .filter(|columns| !columns.is_empty());

        self.command_service
            .handle_upsert_policy(UpsertPolicyRuleCommand::new(
                UpsertPolicyRuleCommandParts {
                    tenant_id,
                    role_name: DATA_API_AUTHENTICATED_ROLE.to_string(),
                    resource_name,
                    action_name: policy.action_name,
                    effect: PermissionEffect::Allow,
                    allowed_columns: unique_allowed,
                    denied_columns: None,
                    owner_scope: false,
                },
            )?)
            .await
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
        self.upsert_data_api_policies(DataApiPolicyBatchUpsertRequest {
            tenant_id: request.tenant_id,
            principal_id: request.principal_id,
            resource_name: request.resource_name,
            policies: vec![
                DataApiPolicyRuleUpsertRequest {
                    action_name: "read".to_string(),
                    allowed_columns: Some(request.readable_columns),
                },
                DataApiPolicyRuleUpsertRequest {
                    action_name: "create".to_string(),
                    allowed_columns: Some(request.writable_columns.clone()),
                },
                DataApiPolicyRuleUpsertRequest {
                    action_name: "update".to_string(),
                    allowed_columns: Some(request.writable_columns),
                },
                DataApiPolicyRuleUpsertRequest {
                    action_name: "delete".to_string(),
                    allowed_columns: None,
                },
            ],
        })
        .await
    }

    async fn upsert_data_api_policies(
        &self,
        request: DataApiPolicyBatchUpsertRequest,
    ) -> Result<(), crate::access_control::domain::model::enums::access_control_domain_error::AccessControlDomainError>
    {
        self.assign_data_api_authenticated_role(request.tenant_id.clone(), request.principal_id)
            .await?;

        for policy in request.policies {
            self.upsert_data_api_policy(
                request.tenant_id.clone(),
                request.resource_name.clone(),
                policy,
            )
            .await?;
        }

        Ok(())
    }
}
