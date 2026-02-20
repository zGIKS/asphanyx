use async_trait::async_trait;

use crate::data_api::domain::model::{
    commands::apply_table_policy_template_command::ApplyTablePolicyTemplateCommand,
    enums::data_api_domain_error::DataApiDomainError,
};

#[async_trait]
pub trait DataApiPolicyTemplateCommandService: Send + Sync {
    async fn handle_apply_table_policy_template(
        &self,
        command: ApplyTablePolicyTemplateCommand,
    ) -> Result<(), DataApiDomainError>;
}
