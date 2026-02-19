use std::sync::Arc;

use swagger_axum_api::provisioner::{
    application::command_services::database_provisioning_command_service_impl::DatabaseProvisioningCommandServiceImpl,
    domain::model::entities::provisioned_database::ProvisionedDatabase,
};

use super::fakes::{
    FakeAuditEventRepository, FakeMetadataRepository, FakePostgresAdministrationRepository,
};

pub struct ProvisioningTestHarness {
    pub metadata_repository: Arc<FakeMetadataRepository>,
    pub postgres_repository: Arc<FakePostgresAdministrationRepository>,
    pub audit_repository: Arc<FakeAuditEventRepository>,
    pub service: DatabaseProvisioningCommandServiceImpl,
}

pub fn create_harness(
    entries: Vec<ProvisionedDatabase>,
    create_should_fail: bool,
    delete_should_fail: bool,
) -> ProvisioningTestHarness {
    let metadata_repository = Arc::new(FakeMetadataRepository::with_entries(entries));
    let postgres_repository = Arc::new(FakePostgresAdministrationRepository::new(
        create_should_fail,
        delete_should_fail,
    ));
    let audit_repository = Arc::new(FakeAuditEventRepository::new());

    let service = DatabaseProvisioningCommandServiceImpl::new(
        metadata_repository.clone(),
        postgres_repository.clone(),
        audit_repository.clone(),
    );

    ProvisioningTestHarness {
        metadata_repository,
        postgres_repository,
        audit_repository,
        service,
    }
}
