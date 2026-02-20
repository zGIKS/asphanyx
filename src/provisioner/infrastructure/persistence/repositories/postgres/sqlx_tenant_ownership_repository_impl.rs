use async_trait::async_trait;
use sqlx::{PgPool, Row};
use uuid::Uuid;

use crate::provisioner::infrastructure::persistence::repositories::tenant_ownership_repository::TenantOwnershipRepository;

pub struct SqlxTenantOwnershipRepositoryImpl {
    pool: PgPool,
}

impl SqlxTenantOwnershipRepositoryImpl {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl TenantOwnershipRepository for SqlxTenantOwnershipRepositoryImpl {
    async fn save_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), String> {
        sqlx::query(
            r#"
            INSERT INTO tenant_ownerships (tenant_id, user_id)
            VALUES ($1, $2)
            ON CONFLICT (tenant_id)
            DO UPDATE SET user_id = EXCLUDED.user_id
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .execute(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        Ok(())
    }

    async fn exists_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<bool, String> {
        let row = sqlx::query(
            r#"
            SELECT EXISTS(
                SELECT 1
                FROM tenant_ownerships
                WHERE tenant_id = $1 AND user_id = $2
            ) AS ownership_exists
            "#,
        )
        .bind(tenant_id)
        .bind(user_id)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        row.try_get::<bool, _>("ownership_exists")
            .map_err(|e| e.to_string())
    }

    async fn list_tenant_ids_by_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, String> {
        let rows = sqlx::query(
            r#"
            SELECT tenant_id
            FROM tenant_ownerships
            WHERE user_id = $1
            ORDER BY created_at DESC
            "#,
        )
        .bind(user_id)
        .fetch_all(&self.pool)
        .await
        .map_err(|e| e.to_string())?;

        rows.into_iter()
            .map(|row| {
                row.try_get::<Uuid, _>("tenant_id")
                    .map_err(|e| e.to_string())
            })
            .collect()
    }
}
