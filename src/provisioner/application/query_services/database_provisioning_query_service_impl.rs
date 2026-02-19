use std::sync::Arc;

use async_trait::async_trait;

use crate::provisioner::{
    domain::{
        model::{
            entities::provisioned_database::ProvisionedDatabase,
            enums::provisioner_domain_error::ProvisionerDomainError,
            queries::list_provisioned_databases_query::ListProvisionedDatabasesQuery,
        },
        services::database_provisioning_query_service::DatabaseProvisioningQueryService,
    },
    infrastructure::persistence::repositories::provisioned_database_repository::ProvisionedDatabaseRepository,
};

pub struct DatabaseProvisioningQueryServiceImpl {
    metadata_repository: Arc<dyn ProvisionedDatabaseRepository>,
}

impl DatabaseProvisioningQueryServiceImpl {
    pub fn new(metadata_repository: Arc<dyn ProvisionedDatabaseRepository>) -> Self {
        Self {
            metadata_repository,
        }
    }
}

#[async_trait]
impl DatabaseProvisioningQueryService for DatabaseProvisioningQueryServiceImpl {
    async fn handle_list(
        &self,
        query: ListProvisionedDatabasesQuery,
    ) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError> {
        if query.include_deleted() {
            self.metadata_repository.list_all().await
        } else {
            self.metadata_repository.list_active_and_failed().await
        }
    }
}
