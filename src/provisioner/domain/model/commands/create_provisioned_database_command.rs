use crate::provisioner::domain::model::{
    enums::provisioner_domain_error::ProvisionerDomainError,
    value_objects::{
        database_password::DatabasePassword,
        database_username::DatabaseUsername,
        provisioned_database_name::ProvisionedDatabaseName,
    },
};

#[derive(Clone, Debug)]
pub struct CreateProvisionedDatabaseCommand {
    database_name: ProvisionedDatabaseName,
    username: DatabaseUsername,
    password: DatabasePassword,
    apply_seed_data: bool,
}

impl CreateProvisionedDatabaseCommand {
    pub fn new(
        database_name: String,
        username: String,
        password: String,
        apply_seed_data: bool,
    ) -> Result<Self, ProvisionerDomainError> {
        Ok(Self {
            database_name: ProvisionedDatabaseName::new(database_name)?,
            username: DatabaseUsername::new(username)?,
            password: DatabasePassword::new(password)?,
            apply_seed_data,
        })
    }

    pub fn database_name(&self) -> &ProvisionedDatabaseName {
        &self.database_name
    }

    pub fn username(&self) -> &DatabaseUsername {
        &self.username
    }

    pub fn password(&self) -> &DatabasePassword {
        &self.password
    }

    pub fn apply_seed_data(&self) -> bool {
        self.apply_seed_data
    }
}
