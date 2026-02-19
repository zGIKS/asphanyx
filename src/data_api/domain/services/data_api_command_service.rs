use async_trait::async_trait;
use serde_json::Value;

use crate::data_api::domain::model::{
    commands::{
        create_row_command::CreateRowCommand, delete_row_command::DeleteRowCommand,
        patch_row_command::PatchRowCommand,
    },
    enums::data_api_domain_error::DataApiDomainError,
};

#[async_trait]
pub trait DataApiCommandService: Send + Sync {
    async fn handle_create(&self, command: CreateRowCommand) -> Result<Value, DataApiDomainError>;
    async fn handle_patch(&self, command: PatchRowCommand) -> Result<Value, DataApiDomainError>;
    async fn handle_delete(&self, command: DeleteRowCommand) -> Result<(), DataApiDomainError>;
}
