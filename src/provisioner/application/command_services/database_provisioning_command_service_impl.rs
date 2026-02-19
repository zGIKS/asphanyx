use std::sync::Arc;

use async_trait::async_trait;
use chrono::Utc;

use crate::provisioner::{
    domain::{
        model::{
            commands::{
                create_provisioned_database_command::CreateProvisionedDatabaseCommand,
                delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
            },
            entities::provisioned_database::ProvisionedDatabase,
            enums::{
                provisioned_database_status::ProvisionedDatabaseStatus,
                provisioner_domain_error::ProvisionerDomainError,
            },
            events::{
                provisioned_database_created_event::ProvisionedDatabaseCreatedEvent,
                provisioned_database_deleted_event::ProvisionedDatabaseDeletedEvent,
            },
        },
        services::database_provisioning_command_service::DatabaseProvisioningCommandService,
    },
    infrastructure::persistence::repositories::{
        postgres_database_administration_repository::PostgresDatabaseAdministrationRepository,
        provisioned_database_repository::ProvisionedDatabaseRepository,
        provisioning_audit_event_repository::{
            ProvisioningAuditEventRecord, ProvisioningAuditEventRepository,
        },
    },
};

pub struct DatabaseProvisioningCommandServiceImpl {
    metadata_repository: Arc<dyn ProvisionedDatabaseRepository>,
    postgres_administration_repository: Arc<dyn PostgresDatabaseAdministrationRepository>,
    audit_event_repository: Arc<dyn ProvisioningAuditEventRepository>,
}

impl DatabaseProvisioningCommandServiceImpl {
    pub fn new(
        metadata_repository: Arc<dyn ProvisionedDatabaseRepository>,
        postgres_administration_repository: Arc<dyn PostgresDatabaseAdministrationRepository>,
        audit_event_repository: Arc<dyn ProvisioningAuditEventRepository>,
    ) -> Self {
        Self {
            metadata_repository,
            postgres_administration_repository,
            audit_event_repository,
        }
    }
}

#[async_trait]
impl DatabaseProvisioningCommandService for DatabaseProvisioningCommandServiceImpl {
    async fn handle_create(
        &self,
        command: CreateProvisionedDatabaseCommand,
    ) -> Result<ProvisionedDatabase, ProvisionerDomainError> {
        if self
            .metadata_repository
            .find_by_name(command.database_name())
            .await?
            .is_some()
        {
            return Err(ProvisionerDomainError::DatabaseAlreadyProvisioned);
        }

        let mut database = ProvisionedDatabase::new_provisioning(
            command.database_name().clone(),
            command.username().clone(),
            Utc::now(),
        );

        self.metadata_repository.save(&database).await?;
        let _ = self
            .audit_event_repository
            .save_event(&ProvisioningAuditEventRecord::new(
                "database_provision_started",
                command.database_name().value(),
                Some(command.username().value().to_string()),
                database.status().as_str(),
                None,
                Utc::now(),
            ))
            .await;

        let creation_result = self
            .postgres_administration_repository
            .create_database_stack(
                command.database_name(),
                command.username(),
                command.password(),
                command.apply_seed_data(),
            )
            .await;

        match creation_result {
            Ok(()) => {
                database.mark_active()?;
                self.metadata_repository.save(&database).await?;
                let event = ProvisionedDatabaseCreatedEvent::new(
                    command.database_name().clone(),
                    command.username().clone(),
                    Utc::now(),
                );
                let _ = self
                    .audit_event_repository
                    .save_event(&ProvisioningAuditEventRecord::new(
                        "database_provision_succeeded",
                        event.database_name.value(),
                        Some(event.username.value().to_string()),
                        database.status().as_str(),
                        None,
                        event.occurred_at,
                    ))
                    .await;

                Ok(database)
            }
            Err(error) => {
                database.mark_failed();
                self.metadata_repository.save(&database).await?;
                let _ = self
                    .audit_event_repository
                    .save_event(&ProvisioningAuditEventRecord::new(
                        "database_provision_failed",
                        command.database_name().value(),
                        Some(command.username().value().to_string()),
                        database.status().as_str(),
                        Some(error.to_string()),
                        Utc::now(),
                    ))
                    .await;

                let _ = self
                    .postgres_administration_repository
                    .rollback_database_stack(command.database_name(), command.username())
                    .await;

                Err(error)
            }
        }
    }

    async fn handle_delete(
        &self,
        command: DeleteProvisionedDatabaseCommand,
    ) -> Result<(), ProvisionerDomainError> {
        let mut database = self
            .metadata_repository
            .find_by_name(command.database_name())
            .await?
            .ok_or(ProvisionerDomainError::DatabaseNotFound)?;

        if database.status() == ProvisionedDatabaseStatus::Deleted {
            return Ok(());
        }

        database.mark_deleting()?;
        self.metadata_repository.save(&database).await?;
        let _ = self
            .audit_event_repository
            .save_event(&ProvisioningAuditEventRecord::new(
                "database_delete_started",
                database.database_name().value(),
                Some(database.username().value().to_string()),
                database.status().as_str(),
                None,
                Utc::now(),
            ))
            .await;

        let delete_result = self
            .postgres_administration_repository
            .delete_database_stack(database.database_name(), database.username())
            .await;

        if let Err(error) = delete_result {
            let _ = self
                .audit_event_repository
                .save_event(&ProvisioningAuditEventRecord::new(
                    "database_delete_failed",
                    database.database_name().value(),
                    Some(database.username().value().to_string()),
                    database.status().as_str(),
                    Some(error.to_string()),
                    Utc::now(),
                ))
                .await;

            return Err(error);
        }

        database.mark_deleted()?;
        self.metadata_repository.save(&database).await?;
        let event =
            ProvisionedDatabaseDeletedEvent::new(command.database_name().clone(), Utc::now());
        let _ = self
            .audit_event_repository
            .save_event(&ProvisioningAuditEventRecord::new(
                "database_delete_succeeded",
                event.database_name.value(),
                Some(database.username().value().to_string()),
                database.status().as_str(),
                None,
                event.occurred_at,
            ))
            .await;

        Ok(())
    }
}
