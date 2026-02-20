use async_trait::async_trait;

use crate::data_api::domain::model::{
    enums::{
        data_api_domain_error::DataApiDomainError,
        data_api_policy_template_name::DataApiPolicyTemplateName,
    },
    queries::list_policy_templates_query::ListPolicyTemplatesQuery,
};

#[async_trait]
pub trait DataApiPolicyTemplateQueryService: Send + Sync {
    async fn handle_list_policy_templates(
        &self,
        query: ListPolicyTemplatesQuery,
    ) -> Result<Vec<DataApiPolicyTemplateName>, DataApiDomainError>;
}
