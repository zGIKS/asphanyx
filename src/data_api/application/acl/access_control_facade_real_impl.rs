use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    access_control::interfaces::acl::access_control_facade::{
        AccessControlFacade as AccessControlBcFacade, AccessControlPermissionRequest,
    },
    data_api::{
        domain::model::{
            enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
            value_objects::{table_name::TableName, tenant_id::TenantId},
        },
        interfaces::acl::access_control_facade::AccessControlFacade,
    },
};

pub struct AccessControlFacadeRealImpl {
    facade: Arc<dyn AccessControlBcFacade>,
}

impl AccessControlFacadeRealImpl {
    pub fn new(facade: Arc<dyn AccessControlBcFacade>) -> Self {
        Self { facade }
    }
}

#[async_trait]
impl AccessControlFacade for AccessControlFacadeRealImpl {
    async fn check_table_permission(
        &self,
        tenant_id: &TenantId,
        principal: &str,
        table_name: &TableName,
        action: DataApiAction,
        columns: &[String],
    ) -> Result<(), DataApiDomainError> {
        let decision = self
            .facade
            .check_permission(AccessControlPermissionRequest {
                tenant_id: tenant_id.value().to_string(),
                principal_id: principal.to_string(),
                resource_name: table_name.value().to_string(),
                action_name: action.as_str().to_string(),
                requested_columns: columns.to_vec(),
                subject_owner_id: None,
                row_owner_id: None,
                request_id: None,
            })
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        if decision.allowed {
            Ok(())
        } else {
            Err(DataApiDomainError::AccessDenied)
        }
    }
}
