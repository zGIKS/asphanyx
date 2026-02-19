use async_trait::async_trait;
use sqlx::PgPool;

use crate::access_control::{
    domain::model::{
        enums::access_control_domain_error::AccessControlDomainError,
        value_objects::{principal_id::PrincipalId, role_name::RoleName, tenant_id::TenantId},
    },
    infrastructure::persistence::repositories::role_assignment_repository::RoleAssignmentRepository,
};

pub struct SqlxRoleAssignmentRepositoryImpl {
    pool: PgPool,
}

impl SqlxRoleAssignmentRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl RoleAssignmentRepository for SqlxRoleAssignmentRepositoryImpl {
    async fn assign_role(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
        role_name: &RoleName,
    ) -> Result<(), AccessControlDomainError> {
        let statement = r#"
            INSERT INTO access_role_assignments (tenant_id, principal_id, role_name)
            VALUES ($1, $2, $3)
            ON CONFLICT (tenant_id, principal_id, role_name)
            DO NOTHING
        "#;

        sqlx::query(statement)
            .bind(tenant_id.value())
            .bind(principal_id.value())
            .bind(role_name.value())
            .execute(&self.pool)
            .await
            .map_err(|e| AccessControlDomainError::InfrastructureError(e.to_string()))?;

        Ok(())
    }

    async fn find_roles_by_principal(
        &self,
        tenant_id: &TenantId,
        principal_id: &PrincipalId,
    ) -> Result<Vec<String>, AccessControlDomainError> {
        let statement = r#"
            SELECT role_name
            FROM access_role_assignments
            WHERE tenant_id = $1 AND principal_id = $2
        "#;

        let rows = sqlx::query_scalar::<_, String>(statement)
            .bind(tenant_id.value())
            .bind(principal_id.value())
            .fetch_all(&self.pool)
            .await
            .map_err(|e| AccessControlDomainError::InfrastructureError(e.to_string()))?;

        Ok(rows)
    }
}
