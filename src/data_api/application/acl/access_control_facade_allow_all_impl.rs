use async_trait::async_trait;

use crate::data_api::{
    domain::model::enums::data_api_domain_error::DataApiDomainError,
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiAuthorizationBootstrapRequest, DataApiAuthorizationCheckRequest,
    },
};

pub struct AccessControlFacadeAllowAllImpl;

impl AccessControlFacadeAllowAllImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for AccessControlFacadeAllowAllImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl AccessControlFacade for AccessControlFacadeAllowAllImpl {
    async fn check_table_permission(
        &self,
        _request: DataApiAuthorizationCheckRequest,
    ) -> Result<(), DataApiDomainError> {
        Ok(())
    }

    async fn bootstrap_table_access(
        &self,
        _request: DataApiAuthorizationBootstrapRequest,
    ) -> Result<(), DataApiDomainError> {
        Ok(())
    }
}
