use async_trait::async_trait;

use crate::provisioner::domain::model::{
    commands::{
        change_provisioned_database_password_command::ChangeProvisionedDatabasePasswordCommand,
        create_provisioned_database_command::CreateProvisionedDatabaseCommand,
        delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
    },
    entities::provisioned_database::ProvisionedDatabase,
    enums::provisioner_domain_error::ProvisionerDomainError,
};

#[async_trait]
pub trait DatabaseProvisioningCommandService: Send + Sync {
    async fn handle_create(
        &self,
        command: CreateProvisionedDatabaseCommand,
    ) -> Result<ProvisionedDatabase, ProvisionerDomainError>;

    async fn handle_delete(
        &self,
        command: DeleteProvisionedDatabaseCommand,
    ) -> Result<(), ProvisionerDomainError>;

    async fn handle_change_password(
        &self,
        command: ChangeProvisionedDatabasePasswordCommand,
    ) -> Result<(), ProvisionerDomainError>;
}
