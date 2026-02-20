use std::{collections::BTreeSet, sync::Arc};

use async_trait::async_trait;

use crate::data_api::{
    domain::{
        model::{
            commands::apply_table_policy_template_command::ApplyTablePolicyTemplateCommand,
            enums::{
                data_api_domain_error::DataApiDomainError,
                data_api_policy_template_name::DataApiPolicyTemplateName,
            },
        },
        services::data_api_policy_template_command_service::DataApiPolicyTemplateCommandService,
    },
    infrastructure::persistence::repositories::{
        data_api_repository::{DataApiRepository, TableMetadataUpdateCriteria},
        tenant_schema_resolver_repository::TenantSchemaResolverRepository,
    },
    interfaces::acl::access_control_facade::{
        AccessControlFacade, DataApiPolicyTemplateApplyRequest,
        DataApiPolicyTemplateRuleUpsertRequest,
    },
};

pub struct DataApiPolicyTemplateCommandServiceImpl {
    repository: Arc<dyn DataApiRepository>,
    tenant_schema_resolver: Arc<dyn TenantSchemaResolverRepository>,
    access_control_facade: Arc<dyn AccessControlFacade>,
}

impl DataApiPolicyTemplateCommandServiceImpl {
    pub fn new(
        repository: Arc<dyn DataApiRepository>,
        tenant_schema_resolver: Arc<dyn TenantSchemaResolverRepository>,
        access_control_facade: Arc<dyn AccessControlFacade>,
    ) -> Self {
        Self {
            repository,
            tenant_schema_resolver,
            access_control_facade,
        }
    }

    fn unique_non_empty(values: Vec<String>) -> Vec<String> {
        values
            .into_iter()
            .filter(|value| !value.trim().is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
}

#[async_trait]
impl DataApiPolicyTemplateCommandService for DataApiPolicyTemplateCommandServiceImpl {
    async fn handle_apply_table_policy_template(
        &self,
        command: ApplyTablePolicyTemplateCommand,
    ) -> Result<(), DataApiDomainError> {
        let schema_name = self
            .tenant_schema_resolver
            .resolve_schema(command.tenant_id(), Some(command.schema_name().value()))
            .await?;

        self.repository
            .synchronize_metadata(command.tenant_id(), schema_name.value())
            .await?;

        let _ = self
            .repository
            .introspect_table(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
            )
            .await?;

        let readable_columns = Self::unique_non_empty(
            self.repository
                .list_readable_columns(
                    command.tenant_id(),
                    schema_name.value(),
                    command.table_name().value(),
                )
                .await?,
        );
        let writable_columns = Self::unique_non_empty(
            self.repository
                .list_writable_columns(
                    command.tenant_id(),
                    schema_name.value(),
                    command.table_name().value(),
                )
                .await?,
        );

        let (
            exposed,
            read_enabled,
            create_enabled,
            update_enabled,
            delete_enabled,
            introspect_enabled,
        ) = command.template_name().metadata_flags();
        self.repository
            .upsert_table_access_metadata(
                command.tenant_id(),
                schema_name.value(),
                command.table_name().value(),
                TableMetadataUpdateCriteria {
                    exposed,
                    read_enabled,
                    create_enabled,
                    update_enabled,
                    delete_enabled,
                    introspect_enabled,
                    authorization_mode: command.template_name().authorization_mode().to_string(),
                },
            )
            .await?;

        match command.template_name() {
            DataApiPolicyTemplateName::AclCrud => {
                self.access_control_facade
                    .apply_table_policy_template(DataApiPolicyTemplateApplyRequest {
                        tenant_id: command.tenant_id().value().to_string(),
                        principal_id: command.principal_id().to_string(),
                        resource_name: command.table_name().value().to_string(),
                        policies: vec![
                            DataApiPolicyTemplateRuleUpsertRequest {
                                action_name: "read".to_string(),
                                allowed_columns: Some(readable_columns),
                            },
                            DataApiPolicyTemplateRuleUpsertRequest {
                                action_name: "create".to_string(),
                                allowed_columns: Some(writable_columns.clone()),
                            },
                            DataApiPolicyTemplateRuleUpsertRequest {
                                action_name: "update".to_string(),
                                allowed_columns: Some(writable_columns),
                            },
                            DataApiPolicyTemplateRuleUpsertRequest {
                                action_name: "delete".to_string(),
                                allowed_columns: None,
                            },
                        ],
                    })
                    .await?;
            }
            DataApiPolicyTemplateName::AclReadOnly => {
                self.access_control_facade
                    .apply_table_policy_template(DataApiPolicyTemplateApplyRequest {
                        tenant_id: command.tenant_id().value().to_string(),
                        principal_id: command.principal_id().to_string(),
                        resource_name: command.table_name().value().to_string(),
                        policies: vec![DataApiPolicyTemplateRuleUpsertRequest {
                            action_name: "read".to_string(),
                            allowed_columns: Some(readable_columns),
                        }],
                    })
                    .await?;
            }
            DataApiPolicyTemplateName::AuthenticatedCrud => {}
        }

        Ok(())
    }
}
