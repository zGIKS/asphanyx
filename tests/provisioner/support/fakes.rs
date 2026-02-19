use std::{collections::HashMap, sync::Mutex};

use async_trait::async_trait;
use swagger_axum_api::provisioner::{
    domain::model::{
        entities::provisioned_database::ProvisionedDatabase,
        enums::{
            provisioned_database_status::ProvisionedDatabaseStatus,
            provisioner_domain_error::ProvisionerDomainError,
        },
        value_objects::{
            database_password::DatabasePassword, database_username::DatabaseUsername,
            provisioned_database_name::ProvisionedDatabaseName,
        },
    },
    infrastructure::persistence::repositories::{
        postgres_database_administration_repository::PostgresDatabaseAdministrationRepository,
        provisioned_database_repository::ProvisionedDatabaseRepository,
        provisioning_audit_event_repository::{
            ProvisioningAuditEventRecord, ProvisioningAuditEventRepository,
        },
    },
};

#[derive(Default)]
struct FakeMetadataRepositoryState {
    entries: HashMap<String, ProvisionedDatabase>,
    saved_statuses: Vec<ProvisionedDatabaseStatus>,
}

pub struct FakeMetadataRepository {
    state: Mutex<FakeMetadataRepositoryState>,
}

impl FakeMetadataRepository {
    pub fn with_entries(entries: Vec<ProvisionedDatabase>) -> Self {
        let mut map = HashMap::new();
        for database in entries {
            map.insert(database.database_name().value().to_string(), database);
        }

        Self {
            state: Mutex::new(FakeMetadataRepositoryState {
                entries: map,
                saved_statuses: Vec::new(),
            }),
        }
    }

    pub fn saved_statuses(&self) -> Vec<ProvisionedDatabaseStatus> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .saved_statuses
            .clone()
    }
}

#[async_trait]
impl ProvisionedDatabaseRepository for FakeMetadataRepository {
    async fn save(&self, database: &ProvisionedDatabase) -> Result<(), ProvisionerDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.saved_statuses.push(database.status());
        state.entries.insert(
            database.database_name().value().to_string(),
            database.clone(),
        );
        Ok(())
    }

    async fn find_by_name(
        &self,
        database_name: &ProvisionedDatabaseName,
    ) -> Result<Option<ProvisionedDatabase>, ProvisionerDomainError> {
        let state = self.state.lock().expect("mutex poisoned");
        Ok(state.entries.get(database_name.value()).cloned())
    }

    async fn list_all(&self) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError> {
        let state = self.state.lock().expect("mutex poisoned");
        Ok(state.entries.values().cloned().collect())
    }

    async fn list_active_and_failed(
        &self,
    ) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError> {
        let state = self.state.lock().expect("mutex poisoned");
        Ok(state
            .entries
            .values()
            .filter(|database| {
                database.status() == ProvisionedDatabaseStatus::Active
                    || database.status() == ProvisionedDatabaseStatus::Failed
            })
            .cloned()
            .collect())
    }
}

#[derive(Default)]
struct FakePostgresAdministrationState {
    create_calls: usize,
    delete_calls: usize,
    rollback_calls: usize,
    create_should_fail: bool,
    delete_should_fail: bool,
}

pub struct FakePostgresAdministrationRepository {
    state: Mutex<FakePostgresAdministrationState>,
}

impl FakePostgresAdministrationRepository {
    pub fn new(create_should_fail: bool, delete_should_fail: bool) -> Self {
        Self {
            state: Mutex::new(FakePostgresAdministrationState {
                create_calls: 0,
                delete_calls: 0,
                rollback_calls: 0,
                create_should_fail,
                delete_should_fail,
            }),
        }
    }

    pub fn stats(&self) -> (usize, usize, usize) {
        let state = self.state.lock().expect("mutex poisoned");
        (state.create_calls, state.delete_calls, state.rollback_calls)
    }
}

#[async_trait]
impl PostgresDatabaseAdministrationRepository for FakePostgresAdministrationRepository {
    async fn create_database_stack(
        &self,
        _database_name: &ProvisionedDatabaseName,
        _username: &DatabaseUsername,
        _password: &DatabasePassword,
        _apply_seed_data: bool,
    ) -> Result<(), ProvisionerDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.create_calls += 1;
        if state.create_should_fail {
            return Err(ProvisionerDomainError::InfrastructureError(
                "create failed".to_string(),
            ));
        }
        Ok(())
    }

    async fn delete_database_stack(
        &self,
        _database_name: &ProvisionedDatabaseName,
        _username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError> {
        let mut state = self.state.lock().expect("mutex poisoned");
        state.delete_calls += 1;
        if state.delete_should_fail {
            return Err(ProvisionerDomainError::InfrastructureError(
                "delete failed".to_string(),
            ));
        }
        Ok(())
    }

    async fn rollback_database_stack(
        &self,
        _database_name: &ProvisionedDatabaseName,
        _username: &DatabaseUsername,
    ) -> Result<(), ProvisionerDomainError> {
        self.state.lock().expect("mutex poisoned").rollback_calls += 1;
        Ok(())
    }
}

#[derive(Default)]
struct FakeAuditEventRepositoryState {
    saved_event_names: Vec<String>,
}

pub struct FakeAuditEventRepository {
    state: Mutex<FakeAuditEventRepositoryState>,
}

impl FakeAuditEventRepository {
    pub fn new() -> Self {
        Self {
            state: Mutex::new(FakeAuditEventRepositoryState::default()),
        }
    }

    pub fn saved_event_names(&self) -> Vec<String> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .saved_event_names
            .clone()
    }
}

#[async_trait]
impl ProvisioningAuditEventRepository for FakeAuditEventRepository {
    async fn save_event(
        &self,
        event: &ProvisioningAuditEventRecord,
    ) -> Result<(), ProvisionerDomainError> {
        self.state
            .lock()
            .expect("mutex poisoned")
            .saved_event_names
            .push(event.event_name().to_string());
        Ok(())
    }
}
