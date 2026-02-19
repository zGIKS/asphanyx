use async_trait::async_trait;

use crate::data_api::domain::model::{
    enums::{data_api_action::DataApiAction, data_api_domain_error::DataApiDomainError},
    value_objects::table_name::TableName,
};

#[async_trait]
pub trait AccessControlFacade: Send + Sync {
    async fn check_table_permission(
        &self,
        principal: &str,
        table_name: &TableName,
        action: DataApiAction,
        columns: &[String],
    ) -> Result<(), DataApiDomainError>;
}
