use async_trait::async_trait;
use uuid::Uuid;

#[async_trait]
pub trait TenantOwnershipRepository: Send + Sync {
    async fn save_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<(), String>;

    async fn exists_ownership(&self, tenant_id: Uuid, user_id: Uuid) -> Result<bool, String>;

    async fn list_tenant_ids_by_user(&self, user_id: Uuid) -> Result<Vec<Uuid>, String>;
}
