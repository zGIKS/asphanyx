use chrono::Utc;
use swagger_axum_api::provisioner::domain::model::{
    commands::{
        create_provisioned_database_command::CreateProvisionedDatabaseCommand,
        delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
    },
    entities::provisioned_database::ProvisionedDatabase,
    enums::provisioned_database_status::ProvisionedDatabaseStatus,
    value_objects::{
        database_username::DatabaseUsername, provisioned_database_name::ProvisionedDatabaseName,
    },
};

pub fn create_command() -> CreateProvisionedDatabaseCommand {
    CreateProvisionedDatabaseCommand::new(
        "tenant_alpha".to_string(),
        "tenant_alpha_user".to_string(),
        "supersecret".to_string(),
        true,
    )
    .expect("valid create command")
}

pub fn delete_command() -> DeleteProvisionedDatabaseCommand {
    DeleteProvisionedDatabaseCommand::new("tenant_alpha".to_string()).expect("valid command")
}

pub fn database_with_status(status: ProvisionedDatabaseStatus) -> ProvisionedDatabase {
    ProvisionedDatabase::restore(
        ProvisionedDatabaseName::new("tenant_alpha".to_string()).expect("valid name"),
        DatabaseUsername::new("tenant_alpha_user".to_string()).expect("valid username"),
        status,
        Utc::now(),
    )
}
