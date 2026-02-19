use chrono::{DateTime, Utc};

use crate::provisioner::domain::model::value_objects::provisioned_database_name::ProvisionedDatabaseName;

#[derive(Clone, Debug)]
pub struct ProvisionedDatabaseDeletedEvent {
    pub database_name: ProvisionedDatabaseName,
    pub occurred_at: DateTime<Utc>,
}

impl ProvisionedDatabaseDeletedEvent {
    pub fn new(database_name: ProvisionedDatabaseName, occurred_at: DateTime<Utc>) -> Self {
        Self {
            database_name,
            occurred_at,
        }
    }
}
