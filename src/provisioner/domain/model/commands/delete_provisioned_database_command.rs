use crate::provisioner::domain::model::{
    enums::provisioner_domain_error::ProvisionerDomainError,
    value_objects::provisioned_database_name::ProvisionedDatabaseName,
};

#[derive(Clone, Debug)]
pub struct DeleteProvisionedDatabaseCommand {
    database_name: ProvisionedDatabaseName,
}

impl DeleteProvisionedDatabaseCommand {
    pub fn new(database_name: String) -> Result<Self, ProvisionerDomainError> {
        Ok(Self {
            database_name: ProvisionedDatabaseName::new(database_name)?,
        })
    }

    pub fn database_name(&self) -> &ProvisionedDatabaseName {
        &self.database_name
    }
}
