use swagger_axum_api::provisioner::domain::{
    model::enums::{
        provisioned_database_status::ProvisionedDatabaseStatus,
        provisioner_domain_error::ProvisionerDomainError,
    },
    services::database_provisioning_command_service::DatabaseProvisioningCommandService,
};

use crate::support::{create_harness, database_with_status, delete_command};

#[tokio::test]
async fn handle_delete_succeeds_and_marks_deleted() {
    let harness = create_harness(
        vec![database_with_status(ProvisionedDatabaseStatus::Active)],
        false,
        false,
    );

    let result = harness.service.handle_delete(delete_command()).await;

    assert!(result.is_ok());
    assert_eq!(
        harness.metadata_repository.saved_statuses(),
        vec![
            ProvisionedDatabaseStatus::Deleting,
            ProvisionedDatabaseStatus::Deleted,
        ]
    );
    assert_eq!(harness.postgres_repository.stats(), (0, 1, 0));
    assert_eq!(
        harness.audit_repository.saved_event_names(),
        vec![
            "database_delete_started".to_string(),
            "database_delete_succeeded".to_string(),
        ]
    );
}

#[tokio::test]
async fn handle_delete_returns_not_found_when_database_is_missing() {
    let harness = create_harness(vec![], false, false);

    let result = harness.service.handle_delete(delete_command()).await;

    assert!(matches!(
        result,
        Err(ProvisionerDomainError::DatabaseNotFound)
    ));
    assert!(harness.metadata_repository.saved_statuses().is_empty());
    assert_eq!(harness.postgres_repository.stats(), (0, 0, 0));
    assert!(harness.audit_repository.saved_event_names().is_empty());
}

#[tokio::test]
async fn handle_delete_is_idempotent_when_database_is_already_deleted() {
    let harness = create_harness(
        vec![database_with_status(ProvisionedDatabaseStatus::Deleted)],
        false,
        false,
    );

    let result = harness.service.handle_delete(delete_command()).await;

    assert!(result.is_ok());
    assert!(harness.metadata_repository.saved_statuses().is_empty());
    assert_eq!(harness.postgres_repository.stats(), (0, 0, 0));
    assert!(harness.audit_repository.saved_event_names().is_empty());
}
