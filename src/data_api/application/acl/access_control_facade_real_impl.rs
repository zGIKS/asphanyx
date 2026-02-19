use std::sync::Arc;

use async_trait::async_trait;

use crate::{
    access_control::interfaces::acl::access_control_facade::{
        AccessControlFacade as AccessControlBcFacade, AccessControlPermissionRequest,
        DataApiAccessBootstrapRequest,
    },
    data_api::{
        domain::model::enums::data_api_domain_error::DataApiDomainError,
        interfaces::acl::access_control_facade::{
            AccessControlFacade, DataApiAuthorizationBootstrapRequest,
            DataApiAuthorizationCheckRequest,
        },
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
        request: DataApiAuthorizationCheckRequest,
    ) -> Result<(), DataApiDomainError> {
        let decision = self
            .facade
            .check_permission(AccessControlPermissionRequest {
                tenant_id: request.tenant_id,
                principal_id: request.principal_id,
                resource_name: request.resource_name,
                action_name: request.action_name,
                requested_columns: request.requested_columns,
                subject_owner_id: request.subject_owner_id,
                row_owner_id: request.row_owner_id,
                request_id: request.request_id,
            })
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))?;

        if decision.allowed {
            Ok(())
        } else {
            Err(DataApiDomainError::AccessDenied)
        }
    }

    async fn bootstrap_table_access(
        &self,
        request: DataApiAuthorizationBootstrapRequest,
    ) -> Result<(), DataApiDomainError> {
        self.facade
            .bootstrap_data_api_access(DataApiAccessBootstrapRequest {
                tenant_id: request.tenant_id,
                principal_id: request.principal_id,
                resource_name: request.resource_name,
                readable_columns: request.readable_columns,
                writable_columns: request.writable_columns,
            })
            .await
            .map_err(|e| DataApiDomainError::InfrastructureError(e.to_string()))
    }
}
