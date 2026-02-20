use async_trait::async_trait;

use crate::iam_integration::domain::model::value_objects::authenticated_user_id::AuthenticatedUserId;

#[derive(Clone, Debug)]
pub struct VerifiedUserContext {
    pub subject_id: AuthenticatedUserId,
    pub jti: Option<String>,
    pub exp_epoch_seconds: u64,
}

#[derive(Debug, thiserror::Error)]
pub enum IamIntegrationError {
    #[error("invalid token: {0}")]
    InvalidToken(String),

    #[error("iam unavailable: {0}")]
    Unavailable(String),
}

#[async_trait]
pub trait IamAuthenticationFacade: Send + Sync {
    async fn verify_access_token(
        &self,
        access_token: &str,
    ) -> Result<VerifiedUserContext, IamIntegrationError>;
}
