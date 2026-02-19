use swagger_axum_api::provisioner::domain::{
    model::enums::{
        provisioned_database_status::ProvisionedDatabaseStatus,
        provisioner_domain_error::ProvisionerDomainError,
    },
    services::database_provisioning_command_service::DatabaseProvisioningCommandService,
};

use crate::support::{change_password_command, create_harness, database_with_status};

#[tokio::test]
async fn handle_change_password_succeeds_for_active_database() {
    let harness = create_harness(
        vec![database_with_status(ProvisionedDatabaseStatus::Active)],
        false,
        false,
    );

    let result = harness
        .service
        .handle_change_password(change_password_command())
        .await;

    assert!(result.is_ok());
    assert_eq!(harness.postgres_repository.stats(), (0, 0, 0, 1));
    assert_eq!(
        harness.audit_repository.saved_event_names(),
        vec![
            "database_password_change_started".to_string(),
            "database_password_change_succeeded".to_string(),
        ]
    );
}

#[tokio::test]
async fn handle_change_password_returns_not_found_when_database_is_missing() {
    let harness = create_harness(vec![], false, false);

    let result = harness
        .service
        .handle_change_password(change_password_command())
        .await;

    assert!(matches!(
        result,
        Err(ProvisionerDomainError::DatabaseNotFound)
    ));
    assert_eq!(harness.postgres_repository.stats(), (0, 0, 0, 0));
    assert!(harness.audit_repository.saved_event_names().is_empty());
}
