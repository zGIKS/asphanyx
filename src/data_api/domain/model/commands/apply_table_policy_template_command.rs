use crate::data_api::domain::model::{
    enums::{
        data_api_domain_error::DataApiDomainError,
        data_api_policy_template_name::DataApiPolicyTemplateName,
    },
    value_objects::{schema_name::SchemaName, table_name::TableName, tenant_id::TenantId},
};

#[derive(Clone, Debug)]
pub struct ApplyTablePolicyTemplateCommand {
    tenant_id: TenantId,
    schema_name: SchemaName,
    table_name: TableName,
    principal_id: String,
    template_name: DataApiPolicyTemplateName,
}

pub struct ApplyTablePolicyTemplateCommandParts {
    pub tenant_id: String,
    pub schema_name: String,
    pub table_name: String,
    pub principal_id: String,
    pub template_name: String,
}

impl ApplyTablePolicyTemplateCommand {
    pub fn new(parts: ApplyTablePolicyTemplateCommandParts) -> Result<Self, DataApiDomainError> {
        Ok(Self {
            tenant_id: TenantId::new(parts.tenant_id)?,
            schema_name: SchemaName::new(parts.schema_name)?,
            table_name: TableName::new(parts.table_name)?,
            principal_id: parts.principal_id,
            template_name: DataApiPolicyTemplateName::parse(&parts.template_name)?,
        })
    }

    pub fn tenant_id(&self) -> &TenantId {
        &self.tenant_id
    }

    pub fn schema_name(&self) -> &SchemaName {
        &self.schema_name
    }

    pub fn table_name(&self) -> &TableName {
        &self.table_name
    }

    pub fn principal_id(&self) -> &str {
        &self.principal_id
    }

    pub fn template_name(&self) -> DataApiPolicyTemplateName {
        self.template_name
    }
}
