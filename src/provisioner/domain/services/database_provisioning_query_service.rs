use async_trait::async_trait;

use crate::provisioner::domain::model::{
    entities::provisioned_database::ProvisionedDatabase,
    enums::provisioner_domain_error::ProvisionerDomainError,
    queries::list_provisioned_databases_query::ListProvisionedDatabasesQuery,
};

#[async_trait]
pub trait DatabaseProvisioningQueryService: Send + Sync {
    async fn handle_list(
        &self,
        query: ListProvisionedDatabasesQuery,
    ) -> Result<Vec<ProvisionedDatabase>, ProvisionerDomainError>;
}
