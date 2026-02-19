use async_trait::async_trait;

use crate::provisioner::domain::model::{
    entities::provisioned_database::ProvisionedDatabase,
    enums::provisioner_domain_error::ProvisionerDomainError,
    value_objects::provisioned_database_name::ProvisionedDatabaseName,
};

#[async_trait]
pub trait ProvisionedDatabaseRepository: Send + Sync {
    async fn save(&self, database: &ProvisionedDatabase) -> Result<(), ProvisionerDomainError>;

    async fn find_by_name(
        &self,
        database_name: &ProvisionedDatabaseName,
    ) -> Result<Option<ProvisionedDatabase>, ProvisionerDomainError>;

    async fn list_all(&self) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError>;

    async fn list_active_and_failed(
        &self,
    ) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError>;
}
