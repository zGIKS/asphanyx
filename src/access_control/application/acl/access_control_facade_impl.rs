use std::sync::Arc;

use async_trait::async_trait;

use crate::access_control::{
    domain::{
        model::queries::evaluate_permission_query::{
            EvaluatePermissionQuery, EvaluatePermissionQueryParts,
        },
        services::access_control_query_service::AccessControlQueryService,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, AccessControlPermissionDecision, AccessControlPermissionRequest,
    },
};

pub struct AccessControlFacadeImpl {
    query_service: Arc<dyn AccessControlQueryService>,
}

impl AccessControlFacadeImpl {
    pub fn new(query_service: Arc<dyn AccessControlQueryService>) -> Self {
        Self { query_service }
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
}
