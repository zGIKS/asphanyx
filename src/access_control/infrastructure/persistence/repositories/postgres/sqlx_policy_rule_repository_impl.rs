use async_trait::async_trait;
use sqlx::PgPool;

use crate::access_control::{
    domain::model::{
        enums::access_control_domain_error::AccessControlDomainError,
        value_objects::{
            action_name::ActionName, resource_name::ResourceName, tenant_id::TenantId,
        },
    },
    infrastructure::persistence::repositories::policy_rule_repository::{
        PolicyRuleRecord, PolicyRuleRepository,
    },
};

pub struct SqlxPolicyRuleRepositoryImpl {
    pool: PgPool,
}

impl SqlxPolicyRuleRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl PolicyRuleRepository for SqlxPolicyRuleRepositoryImpl {
    async fn upsert_rule(&self, rule: PolicyRuleRecord) -> Result<(), AccessControlDomainError> {
        let statement = r#"
            INSERT INTO access_policy_rules (
                tenant_id,
                role_name,
                resource_name,
                action_name,
                effect,
                allowed_columns,
                denied_columns,
                owner_scope
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (tenant_id, role_name, resource_name, action_name)
            DO UPDATE SET
                effect = EXCLUDED.effect,
                allowed_columns = EXCLUDED.allowed_columns,
                denied_columns = EXCLUDED.denied_columns,
                owner_scope = EXCLUDED.owner_scope
        "#;

        sqlx::query(statement)
            .bind(rule.tenant_id)
            .bind(rule.role_name)
            .bind(rule.resource_name)
            .bind(rule.action_name)
            .bind(rule.effect.as_str())
            .bind(rule.allowed_columns)
            .bind(rule.denied_columns)
            .bind(rule.owner_scope)
            .execute(&self.pool)
            .await
            .map_err(|e| AccessControlDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_rules_for_roles(
        &self,
        tenant_id: &TenantId,
        resource_name: &ResourceName,
        action_name: &ActionName,
        role_names: &[String],
    ) -> Result<Vec<PolicyRuleRecord>, AccessControlDomainError> {
        if role_names.is_empty() {
            return Ok(Vec::new());
        }

        let statement = r#"
            SELECT role_name, effect, allowed_columns, denied_columns, owner_scope
            FROM access_policy_rules
            WHERE tenant_id = $1
              AND resource_name = ANY($2)
              AND action_name = ANY($3)
              AND role_name = ANY($4)
        "#;

        let resource_candidates = [resource_name.value().to_string(), "*".to_string()];
        let action_candidates = [action_name.value().to_string(), "*".to_string()];

        let rows = sqlx::query_as::<
            _,
            (
                String,
                String,
                Option<Vec<String>>,
                Option<Vec<String>>,
                bool,
            ),
        >(statement)
        .bind(tenant_id.value())
        .bind(resource_candidates.as_slice())
        .bind(action_candidates.as_slice())
        .bind(role_names)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| AccessControlDomainError::InfrastructureError(e.to_string()))?;

        rows.into_iter()
            .map(
                |(role_name, effect, allowed_columns, denied_columns, owner_scope)| {
                    Ok(PolicyRuleRecord {
                        tenant_id: tenant_id.value().to_string(),
                        role_name,
                        resource_name: resource_name.value().to_string(),
                        action_name: action_name.value().to_string(),
                        effect: effect.parse()?,
                        allowed_columns,
                        denied_columns,
                        owner_scope,
                    })
                },
            )
            .collect()
    }
}
