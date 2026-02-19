use chrono::{DateTime, Utc};

use crate::provisioner::domain::model::{
    enums::{
        provisioned_database_status::ProvisionedDatabaseStatus,
        provisioner_domain_error::ProvisionerDomainError,
    },
    value_objects::{
        database_username::DatabaseUsername,
        provisioned_database_name::ProvisionedDatabaseName,
    },
};

#[derive(Clone, Debug)]
pub struct ProvisionedDatabase {
    database_name: ProvisionedDatabaseName,
    username: DatabaseUsername,
    status: ProvisionedDatabaseStatus,
    created_at: DateTime<Utc>,
}

impl ProvisionedDatabase {
    pub fn new_provisioning(
        database_name: ProvisionedDatabaseName,
        username: DatabaseUsername,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            database_name,
            username,
            status: ProvisionedDatabaseStatus::Provisioning,
            created_at,
        }
    }

    pub fn restore(
        database_name: ProvisionedDatabaseName,
        username: DatabaseUsername,
        status: ProvisionedDatabaseStatus,
        created_at: DateTime<Utc>,
    ) -> Self {
        Self {
            database_name,
            username,
            status,
            created_at,
        }
    }

    pub fn mark_active(&mut self) -> Result<(), ProvisionerDomainError> {
        if self.status != ProvisionedDatabaseStatus::Provisioning {
            return Err(ProvisionerDomainError::InvalidStatusTransition);
        }
        self.status = ProvisionedDatabaseStatus::Active;
        Ok(())
    }

    pub fn mark_failed(&mut self) {
        self.status = ProvisionedDatabaseStatus::Failed;
    }

    pub fn mark_deleting(&mut self) -> Result<(), ProvisionerDomainError> {
        if self.status != ProvisionedDatabaseStatus::Active
            && self.status != ProvisionedDatabaseStatus::Failed
        {
            return Err(ProvisionerDomainError::InvalidStatusTransition);
        }
        self.status = ProvisionedDatabaseStatus::Deleting;
        Ok(())
    }

    pub fn mark_deleted(&mut self) -> Result<(), ProvisionerDomainError> {
        if self.status != ProvisionedDatabaseStatus::Deleting {
            return Err(ProvisionerDomainError::InvalidStatusTransition);
        }
        self.status = ProvisionedDatabaseStatus::Deleted;
        Ok(())
    }

    pub fn database_name(&self) -> &ProvisionedDatabaseName {
        &self.database_name
    }

    pub fn username(&self) -> &DatabaseUsername {
        &self.username
    }

    pub fn status(&self) -> ProvisionedDatabaseStatus {
        self.status
    }

    pub fn created_at(&self) -> DateTime<Utc> {
        self.created_at
    }
}
