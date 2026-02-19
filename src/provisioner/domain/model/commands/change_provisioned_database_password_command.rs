use crate::provisioner::domain::model::{
    enums::provisioner_domain_error::ProvisionerDomainError,
    value_objects::{
        database_password::DatabasePassword, database_password_hash::DatabasePasswordHash,
        provisioned_database_name::ProvisionedDatabaseName,
    },
};

#[derive(Clone, Debug)]
pub struct ChangeProvisionedDatabasePasswordCommand {
    database_name: ProvisionedDatabaseName,
    password: DatabasePassword,
    password_hash: DatabasePasswordHash,
}

impl ChangeProvisionedDatabasePasswordCommand {
    pub fn new(
        database_name: String,
        password: String,
        password_hash: String,
    ) -> Result<Self, ProvisionerDomainError> {
        Ok(Self {
            database_name: ProvisionedDatabaseName::new(database_name)?,
            password: DatabasePassword::new(password)?,
            password_hash: DatabasePasswordHash::new(password_hash)?,
        })
    }

    pub fn database_name(&self) -> &ProvisionedDatabaseName {
        &self.database_name
    }

    pub fn password(&self) -> &DatabasePassword {
        &self.password
    }

    pub fn password_hash(&self) -> &DatabasePasswordHash {
        &self.password_hash
    }
}
