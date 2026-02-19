use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    sync::Arc,
    time::{Duration, Instant},
};

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::access_control::{
    domain::{
        model::{
            enums::{
                access_control_domain_error::AccessControlDomainError,
                permission_effect::PermissionEffect,
            },
            events::authorization_decision_audited_event::AuthorizationDecisionAuditedEvent,
            queries::evaluate_permission_query::EvaluatePermissionQuery,
        },
        services::access_control_query_service::{
            AccessControlQueryService, AuthorizationDecisionResult,
        },
    },
    infrastructure::persistence::repositories::{
        authorization_decision_audit_repository::AuthorizationDecisionAuditRepository,
        policy_rule_repository::{PolicyRuleRecord, PolicyRuleRepository},
        role_assignment_repository::RoleAssignmentRepository,
    },
};

#[derive(Clone, Debug, Eq)]
struct DecisionCacheKey {
    tenant_id: String,
    principal_id: String,
    resource_name: String,
    action_name: String,
    requested_columns: Vec<String>,
    subject_owner_id: Option<String>,
    row_owner_id: Option<String>,
}

impl PartialEq for DecisionCacheKey {
    fn eq(&self, other: &Self) -> bool {
        self.tenant_id == other.tenant_id
            && self.principal_id == other.principal_id
            && self.resource_name == other.resource_name
            && self.action_name == other.action_name
            && self.requested_columns == other.requested_columns
            && self.subject_owner_id == other.subject_owner_id
            && self.row_owner_id == other.row_owner_id
    }
}

impl Hash for DecisionCacheKey {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.tenant_id.hash(state);
        self.principal_id.hash(state);
        self.resource_name.hash(state);
        self.action_name.hash(state);
        self.requested_columns.hash(state);
        self.subject_owner_id.hash(state);
        self.row_owner_id.hash(state);
    }
}

#[derive(Clone, Debug)]
struct DecisionCacheEntry {
    decision: AuthorizationDecisionResult,
    expires_at: Instant,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
struct RuleSpecificity {
    resource_specificity: u8,
    action_specificity: u8,
    column_specificity: u8,
    owner_specificity: u8,
}

pub struct AccessControlQueryServiceImpl {
    policy_rule_repository: Arc<dyn PolicyRuleRepository>,
    role_assignment_repository: Arc<dyn RoleAssignmentRepository>,
    decision_audit_repository: Arc<dyn AuthorizationDecisionAuditRepository>,
    decision_cache: RwLock<HashMap<DecisionCacheKey, DecisionCacheEntry>>,
    cache_ttl: Duration,
}

impl AccessControlQueryServiceImpl {
    pub fn new(
        policy_rule_repository: Arc<dyn PolicyRuleRepository>,
        role_assignment_repository: Arc<dyn RoleAssignmentRepository>,
        decision_audit_repository: Arc<dyn AuthorizationDecisionAuditRepository>,
    ) -> Self {
        Self::new_with_cache_ttl(
            policy_rule_repository,
            role_assignment_repository,
            decision_audit_repository,
            Duration::from_secs(30),
        )
    }

    pub fn new_with_cache_ttl(
        policy_rule_repository: Arc<dyn PolicyRuleRepository>,
        role_assignment_repository: Arc<dyn RoleAssignmentRepository>,
        decision_audit_repository: Arc<dyn AuthorizationDecisionAuditRepository>,
        cache_ttl: Duration,
    ) -> Self {
        Self {
            policy_rule_repository,
            role_assignment_repository,
            decision_audit_repository,
            decision_cache: RwLock::new(HashMap::new()),
            cache_ttl,
        }
    }

    fn build_cache_key(query: &EvaluatePermissionQuery) -> DecisionCacheKey {
        let mut requested_columns = query.requested_columns().to_vec();
        requested_columns.sort();

        DecisionCacheKey {
            tenant_id: query.tenant_id().value().to_string(),
            principal_id: query.principal_id().value().to_string(),
            resource_name: query.resource_name().value().to_string(),
            action_name: query.action_name().value().to_string(),
            requested_columns,
            subject_owner_id: query.subject_owner_id().map(str::to_string),
            row_owner_id: query.row_owner_id().map(str::to_string),
        }
    }

    async fn load_cached_decision(
        &self,
        key: &DecisionCacheKey,
    ) -> Option<AuthorizationDecisionResult> {
        let read_guard = self.decision_cache.read().await;
        read_guard.get(key).and_then(|entry| {
            if entry.expires_at > Instant::now() {
                Some(entry.decision.clone())
            } else {
                None
            }
        })
    }

    async fn cache_decision(&self, key: DecisionCacheKey, decision: AuthorizationDecisionResult) {
        let mut write_guard = self.decision_cache.write().await;
        write_guard.insert(
            key,
            DecisionCacheEntry {
                decision,
                expires_at: Instant::now() + self.cache_ttl,
            },
        );
    }

    fn rule_matches_columns(
        requested_columns: &[String],
        allowed_columns: Option<&[String]>,
        denied_columns: Option<&[String]>,
    ) -> bool {
        let allowed_match = match allowed_columns {
            None => true,
            Some(allowed) => requested_columns
                .iter()
                .all(|c| allowed.iter().any(|a| a == c)),
        };

        if !allowed_match {
            return false;
        }

        match denied_columns {
            None => true,
            Some(denied) => requested_columns
                .iter()
                .all(|c| denied.iter().all(|d| d != c)),
        }
    }

    fn rule_matches_owner_scope(
        owner_scope: bool,
        subject_owner_id: Option<&str>,
        row_owner_id: Option<&str>,
    ) -> bool {
        if !owner_scope {
            return true;
        }

        match (subject_owner_id, row_owner_id) {
            (Some(subject), Some(row_owner)) => subject == row_owner,
            _ => false,
        }
    }

    fn rule_specificity(
        rule: &PolicyRuleRecord,
        query: &EvaluatePermissionQuery,
    ) -> RuleSpecificity {
        RuleSpecificity {
            resource_specificity: if rule.resource_name == query.resource_name().value() {
                2
            } else {
                1
            },
            action_specificity: if rule.action_name == query.action_name().value() {
                2
            } else {
                1
            },
            column_specificity: if rule.allowed_columns.is_some() || rule.denied_columns.is_some() {
                1
            } else {
                0
            },
            owner_specificity: if rule.owner_scope { 1 } else { 0 },
        }
    }

    fn evaluate_rules(
        query: &EvaluatePermissionQuery,
        rules: &[PolicyRuleRecord],
    ) -> AuthorizationDecisionResult {
        let applicable = rules
            .iter()
            .filter(|rule| {
                Self::rule_matches_columns(
                    query.requested_columns(),
                    rule.allowed_columns.as_deref(),
                    rule.denied_columns.as_deref(),
                )
            })
            .filter(|rule| {
                Self::rule_matches_owner_scope(
                    rule.owner_scope,
                    query.subject_owner_id(),
                    query.row_owner_id(),
                )
            })
            .collect::<Vec<_>>();

        if applicable.is_empty() {
            return AuthorizationDecisionResult {
                allowed: false,
                reason: "no rule matched context/columns".to_string(),
            };
        }

        let mut best_allow: Option<RuleSpecificity> = None;
        let mut best_deny: Option<RuleSpecificity> = None;

        for rule in &applicable {
            let specificity = Self::rule_specificity(rule, query);
            match rule.effect {
                PermissionEffect::Allow => {
                    best_allow = Some(best_allow.map_or(specificity, |s| s.max(specificity)));
                }
                PermissionEffect::Deny => {
                    best_deny = Some(best_deny.map_or(specificity, |s| s.max(specificity)));
                }
            }
        }

        match (best_allow, best_deny) {
            (None, Some(_)) => AuthorizationDecisionResult {
                allowed: false,
                reason: "explicit deny rule".to_string(),
            },
            (Some(_), None) => AuthorizationDecisionResult {
                allowed: true,
                reason: "allow rule matched".to_string(),
            },
            (Some(allow_spec), Some(deny_spec)) => {
                if deny_spec >= allow_spec {
                    AuthorizationDecisionResult {
                        allowed: false,
                        reason: "deny rule won by precedence".to_string(),
                    }
                } else {
                    AuthorizationDecisionResult {
                        allowed: true,
                        reason: "allow rule won by specificity".to_string(),
                    }
                }
            }
            (None, None) => AuthorizationDecisionResult {
                allowed: false,
                reason: "no matching policy rule".to_string(),
            },
        }
    }

    async fn audit(&self, query: &EvaluatePermissionQuery, decision: &AuthorizationDecisionResult) {
        let _ = self
            .decision_audit_repository
            .save_decision(&AuthorizationDecisionAuditedEvent {
                tenant_id: query.tenant_id().value().to_string(),
                principal_id: query.principal_id().value().to_string(),
                request_id: query.request_id().map(str::to_string),
                resource_name: query.resource_name().value().to_string(),
                action_name: query.action_name().value().to_string(),
                allowed: decision.allowed,
                reason: decision.reason.clone(),
                occurred_at: Utc::now(),
            })
            .await;
    }
}

#[async_trait]
impl AccessControlQueryService for AccessControlQueryServiceImpl {
    async fn handle_evaluate_permission(
        &self,
        query: EvaluatePermissionQuery,
    ) -> Result<AuthorizationDecisionResult, AccessControlDomainError> {
        let cache_key = Self::build_cache_key(&query);
        if let Some(cached) = self.load_cached_decision(&cache_key).await {
            self.audit(
                &query,
                &AuthorizationDecisionResult {
                    allowed: cached.allowed,
                    reason: format!("cached: {}", cached.reason),
                },
            )
            .await;
            return Ok(cached);
        }

        let roles = self
            .role_assignment_repository
            .find_roles_by_principal(query.tenant_id(), query.principal_id())
            .await?;

        if roles.is_empty() {
            let decision = AuthorizationDecisionResult {
                allowed: false,
                reason: "no roles assigned".to_string(),
            };
            self.audit(&query, &decision).await;
            self.cache_decision(cache_key, decision.clone()).await;
            return Ok(decision);
        }

        let rules = self
            .policy_rule_repository
            .find_rules_for_roles(
                query.tenant_id(),
                query.resource_name(),
                query.action_name(),
                &roles,
            )
            .await?;

        if rules.is_empty() {
            let decision = AuthorizationDecisionResult {
                allowed: false,
                reason: "no matching policy rule".to_string(),
            };
            self.audit(&query, &decision).await;
            self.cache_decision(cache_key, decision.clone()).await;
            return Ok(decision);
        }

        let decision = Self::evaluate_rules(&query, &rules);
        self.audit(&query, &decision).await;
        self.cache_decision(cache_key, decision.clone()).await;
        Ok(decision)
    }
}
