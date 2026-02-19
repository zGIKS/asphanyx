use async_trait::async_trait;

use crate::data_api::{
    domain::model::enums::data_api_domain_error::DataApiDomainError,
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiAuthorizationCheckRequest,
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
}
