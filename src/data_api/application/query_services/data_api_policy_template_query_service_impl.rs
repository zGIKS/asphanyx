use async_trait::async_trait;

use crate::data_api::domain::{
    model::{
        enums::{
            data_api_domain_error::DataApiDomainError,
            data_api_policy_template_name::DataApiPolicyTemplateName,
        },
        queries::list_policy_templates_query::ListPolicyTemplatesQuery,
    },
    services::data_api_policy_template_query_service::DataApiPolicyTemplateQueryService,
};

pub struct DataApiPolicyTemplateQueryServiceImpl;

impl DataApiPolicyTemplateQueryServiceImpl {
    pub fn new() -> Self {
        Self
    }
}

impl Default for DataApiPolicyTemplateQueryServiceImpl {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl DataApiPolicyTemplateQueryService for DataApiPolicyTemplateQueryServiceImpl {
    async fn handle_list_policy_templates(
        &self,
        _query: ListPolicyTemplatesQuery,
    ) -> Result<Vec<DataApiPolicyTemplateName>, DataApiDomainError> {
        Ok(DataApiPolicyTemplateName::all().to_vec())
    }
}
