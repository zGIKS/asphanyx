use async_trait::async_trait;

use crate::provisioner::domain::model::{
    enums::provisioner_domain_error::ProvisionerDomainError,
    value_objects::{
        database_password::DatabasePassword, database_username::DatabaseUsername,
        provisioned_database_name::ProvisionedDatabaseName,
    },
};

#[async_trait]
pub trait PostgresDatabaseAdministrationRepository: Send + Sync {
    async fn create_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
        password: &DatabasePassword,
        apply_seed_data: bool,
    ) -> Result<(), ProvisionerDomainError>;

    async fn delete_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError>;

    async fn rollback_database_stack(
        &self,
        database_name: &ProvisionedDatabaseName,
        username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError>;
}
