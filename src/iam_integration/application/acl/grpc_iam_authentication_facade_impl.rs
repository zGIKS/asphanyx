use std::{
    collections::HashMap,
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use sha2::{Digest, Sha256};
use tokio::sync::{Mutex, RwLock};
use tonic::transport::{Channel, Endpoint};

use crate::{
    iam_grpc::{
        VerifyAccessTokenRequest,
        authentication_verification_service_client::AuthenticationVerificationServiceClient,
    },
    iam_integration::{
        domain::model::value_objects::authenticated_user_id::AuthenticatedUserId,
        interfaces::acl::iam_authentication_facade::{
            IamAuthenticationFacade, IamIntegrationError, VerifiedUserContext,
        },
    },
};

#[derive(Clone)]
struct CachedVerification {
    context: VerifiedUserContext,
    expires_at: Instant,
}

#[derive(Default)]
struct CircuitState {
    consecutive_failures: u32,
    opened_until: Option<Instant>,
}

pub struct GrpcIamAuthenticationFacadeImpl {
    endpoint: String,
    timeout: Duration,
    cache_ttl: Duration,
    failure_threshold: u32,
    open_duration: Duration,
    cache: Arc<RwLock<HashMap<String, CachedVerification>>>,
    circuit: Arc<Mutex<CircuitState>>,
}

impl GrpcIamAuthenticationFacadeImpl {
    pub fn new(
        endpoint: String,
        timeout: Duration,
        cache_ttl: Duration,
        failure_threshold: u32,
        open_duration: Duration,
    ) -> Self {
        Self {
            endpoint,
            timeout,
            cache_ttl,
            failure_threshold,
            open_duration,
            cache: Arc::new(RwLock::new(HashMap::new())),
            circuit: Arc::new(Mutex::new(CircuitState::default())),
        }
    }

    fn token_hash(token: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(token.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    async fn get_cached(&self, token_hash: &str) -> Option<VerifiedUserContext> {
        let guard = self.cache.read().await;
        guard.get(token_hash).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.context.clone())
            } else {
                None
            }
        })
    }

    async fn set_cache(&self, token_hash: String, context: VerifiedUserContext) {
        let mut guard = self.cache.write().await;
        guard.insert(
            token_hash,
            CachedVerification {
                context,
                expires_at: Instant::now() + self.cache_ttl,
            },
        );
    }

    async fn can_attempt_call(&self) -> bool {
        let mut guard = self.circuit.lock().await;
        match guard.opened_until {
            Some(until) if until > Instant::now() => false,
            Some(_) => {
                guard.opened_until = None;
                true
            }
            None => true,
        }
    }

    async fn register_success(&self) {
        let mut guard = self.circuit.lock().await;
        guard.consecutive_failures = 0;
        guard.opened_until = None;
    }

    async fn register_failure(&self) {
        let mut guard = self.circuit.lock().await;
        guard.consecutive_failures = guard.consecutive_failures.saturating_add(1);

        if guard.consecutive_failures >= self.failure_threshold {
            guard.opened_until = Some(Instant::now() + self.open_duration);
            guard.consecutive_failures = 0;
        }
    }

    async fn grpc_client(
        &self,
    ) -> Result<AuthenticationVerificationServiceClient<Channel>, IamIntegrationError> {
        let endpoint = Endpoint::from_shared(self.endpoint.clone())
            .map_err(|e| IamIntegrationError::Unavailable(e.to_string()))?
            .connect_timeout(self.timeout)
            .timeout(self.timeout);

        let channel = endpoint
            .connect()
            .await
            .map_err(|e| IamIntegrationError::Unavailable(e.to_string()))?;

        Ok(AuthenticationVerificationServiceClient::new(channel))
    }
}

#[async_trait]
impl IamAuthenticationFacade for GrpcIamAuthenticationFacadeImpl {
    async fn verify_access_token(
        &self,
        access_token: &str,
    ) -> Result<VerifiedUserContext, IamIntegrationError> {
        if access_token.trim().is_empty() {
            return Err(IamIntegrationError::InvalidToken(
                "access token is empty".to_string(),
            ));
        }

        if !self.can_attempt_call().await {
            return Err(IamIntegrationError::Unavailable(
                "circuit breaker is open".to_string(),
            ));
        }

        let token_hash = Self::token_hash(access_token);

        if let Some(cached) = self.get_cached(&token_hash).await {
            return Ok(cached);
        }

        let mut client = self.grpc_client().await?;

        let response = client
            .verify_access_token(VerifyAccessTokenRequest {
                access_token: access_token.to_string(),
            })
            .await;

        let response = match response {
            Ok(value) => {
                self.register_success().await;
                value.into_inner()
            }
            Err(error) => {
                self.register_failure().await;
                return Err(IamIntegrationError::Unavailable(error.to_string()));
            }
        };

        if !response.is_valid {
            return Err(IamIntegrationError::InvalidToken(
                response.error_message.clone(),
            ));
        }

        let context = VerifiedUserContext {
            subject_id: AuthenticatedUserId::new(&response.subject_id)
                .map_err(IamIntegrationError::InvalidToken)?,
            jti: if response.jti.is_empty() {
                None
            } else {
                Some(response.jti)
            },
            exp_epoch_seconds: response.exp_epoch_seconds,
        };

        self.set_cache(token_hash, context.clone()).await;

        Ok(context)
    }
}
