use std::sync::Arc;

use swagger_axum_api::data_api::application::{
    command_services::data_api_command_service_impl::DataApiCommandServiceImpl,
    query_services::data_api_query_service_impl::DataApiQueryServiceImpl,
};

use super::fakes::{
    FakeAccessControlFacade, FakeDataApiAuditLogRepository, FakeDataApiRepository,
    FakeTenantSchemaResolverRepository,
};

pub struct DataApiCommandHarness {
    pub repository: Arc<FakeDataApiRepository>,
    pub tenant_schema_resolver: Arc<FakeTenantSchemaResolverRepository>,
    pub access_control: Arc<FakeAccessControlFacade>,
    pub audit: Arc<FakeDataApiAuditLogRepository>,
    pub service: DataApiCommandServiceImpl,
}

pub struct DataApiQueryHarness {
    pub repository: Arc<FakeDataApiRepository>,
    pub tenant_schema_resolver: Arc<FakeTenantSchemaResolverRepository>,
    pub access_control: Arc<FakeAccessControlFacade>,
    pub audit: Arc<FakeDataApiAuditLogRepository>,
    pub service: DataApiQueryServiceImpl,
}

pub fn create_command_harness(allowed_tables: &[&str]) -> DataApiCommandHarness {
    let repository = Arc::new(FakeDataApiRepository::new());
    let tenant_schema_resolver = Arc::new(FakeTenantSchemaResolverRepository::new("public"));
    let access_control = Arc::new(FakeAccessControlFacade::new());
    let audit = Arc::new(FakeDataApiAuditLogRepository::new());

    repository.set_table_exposed(allowed_tables.contains(&"productos"));

    let service = DataApiCommandServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver.clone(),
        access_control.clone(),
        audit.clone(),
    );

    DataApiCommandHarness {
        repository,
        tenant_schema_resolver,
        access_control,
        audit,
        service,
    }
}

pub fn create_query_harness(allowed_tables: &[&str]) -> DataApiQueryHarness {
    let repository = Arc::new(FakeDataApiRepository::new());
    let tenant_schema_resolver = Arc::new(FakeTenantSchemaResolverRepository::new("public"));
    let access_control = Arc::new(FakeAccessControlFacade::new());
    let audit = Arc::new(FakeDataApiAuditLogRepository::new());

    repository.set_table_exposed(allowed_tables.contains(&"productos"));

    let service = DataApiQueryServiceImpl::new(
        repository.clone(),
        tenant_schema_resolver.clone(),
        access_control.clone(),
        audit.clone(),
    );

    DataApiQueryHarness {
        repository,
        tenant_schema_resolver,
        access_control,
        audit,
        service,
    }
}
