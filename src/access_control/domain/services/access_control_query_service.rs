use async_trait::async_trait;

use crate::access_control::domain::model::{
    enums::access_control_domain_error::AccessControlDomainError,
    queries::evaluate_permission_query::EvaluatePermissionQuery,
};

#[derive(Clone, Debug)]
pub struct AuthorizationDecisionResult {
    pub allowed: bool,
    pub reason: String,
}

#[async_trait]
pub trait AccessControlQueryService: Send + Sync {
    async fn handle_evaluate_permission(
        &self,
        query: EvaluatePermissionQuery,
    ) -> Result<AuthorizationDecisionResult, AccessControlDomainError>;
}
