use async_trait::async_trait;

use crate::data_api::{
    domain::model::{
        enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
        value_objects::{table_name::TableName, tenant_id::TenantId},
    },
    interfaces::acl::access_control_facade::AccessControlFacade,
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
        _tenant_id: &TenantId,
        _principal: &str,
        _table_name: &TableName,
        _action: DataApiAction,
        _columns: &[String],
    ) -> Result<(), DataApiDomainError> {
        Ok(())
    }
}
