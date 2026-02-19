use chrono::{DateTime, Utc};

use crate::provisioner::domain::model::value_objects::{
    database_username::DatabaseUsername, provisioned_database_name::ProvisionedDatabaseName,
};

#[derive(Clone, Debug)]
pub struct ProvisionedDatabaseCreatedEvent {
    pub database_name: ProvisionedDatabaseName,
    pub username: DatabaseUsername,
    pub occurred_at: DateTime<Utc>,
}

impl ProvisionedDatabaseCreatedEvent {
    pub fn new(
        database_name: ProvisionedDatabaseName,
        username: DatabaseUsername,
        occurred_at: DateTime<Utc>,
    ) -> Self {
        Self {
            database_name,
            username,
            occurred_at,
        }
    }
}
