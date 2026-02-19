use swagger_axum_api::provisioner::domain::{
    model::enums::{
        provisioned_database_status::ProvisionedDatabaseStatus,
        provisioner_domain_error::ProvisionerDomainError,
    },
    services::database_provisioning_command_service::DatabaseProvisioningCommandService,
};

use crate::support::{create_command, create_harness, database_with_status};

#[tokio::test]
async fn handle_create_succeeds_and_persists_active_status() {
    let harness = create_harness(vec![], false, false);

    let result = harness.service.handle_create(create_command()).await;

    let provisioned = result.expect("provisioning should succeed");
    assert_eq!(provisioned.status(), ProvisionedDatabaseStatus::Active);
    assert_eq!(
        harness.metadata_repository.saved_statuses(),
        vec![
            ProvisionedDatabaseStatus::Provisioning,
            ProvisionedDatabaseStatus::Active,
        ]
    );
    assert_eq!(harness.postgres_repository.stats(), (1, 0, 0, 0));
    assert_eq!(
        harness.audit_repository.saved_event_names(),
        vec![
            "database_provision_started".to_string(),
            "database_provision_succeeded".to_string(),
        ]
    );
}

#[tokio::test]
async fn handle_create_returns_error_when_database_already_exists() {
    let harness = create_harness(
        vec![database_with_status(ProvisionedDatabaseStatus::Active)],
        false,
        false,
    );

    let result = harness.service.handle_create(create_command()).await;

    assert!(matches!(
        result,
        Err(ProvisionerDomainError::DatabaseAlreadyProvisioned)
    ));
    assert!(harness.metadata_repository.saved_statuses().is_empty());
    assert_eq!(harness.postgres_repository.stats(), (0, 0, 0, 0));
    assert!(harness.audit_repository.saved_event_names().is_empty());
}

#[tokio::test]
async fn handle_create_marks_failed_and_rolls_back_when_infra_fails() {
    let harness = create_harness(vec![], true, false);

    let result = harness.service.handle_create(create_command()).await;

    assert!(matches!(
        result,
        Err(ProvisionerDomainError::InfrastructureError(message)) if message == "create failed"
    ));
    assert_eq!(
        harness.metadata_repository.saved_statuses(),
        vec![
            ProvisionedDatabaseStatus::Provisioning,
            ProvisionedDatabaseStatus::Failed,
        ]
    );
    assert_eq!(harness.postgres_repository.stats(), (1, 0, 1, 0));
    assert_eq!(
        harness.audit_repository.saved_event_names(),
        vec![
            "database_provision_started".to_string(),
            "database_provision_failed".to_string(),
        ]
    );
}
