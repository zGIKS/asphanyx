use chrono::Utc;
use swagger_axum_api::provisioner::domain::model::{
    commands::{
        change_provisioned_database_password_command::ChangeProvisionedDatabasePasswordCommand,
        create_provisioned_database_command::CreateProvisionedDatabaseCommand,
        delete_provisioned_database_command::DeleteProvisionedDatabaseCommand,
    },
    entities::provisioned_database::ProvisionedDatabase,
    enums::provisioned_database_status::ProvisionedDatabaseStatus,
    value_objects::{
        database_password_hash::DatabasePasswordHash, database_username::DatabaseUsername,
        provisioned_database_id::ProvisionedDatabaseId,
        provisioned_database_name::ProvisionedDatabaseName,
    },
};

pub fn create_command() -> CreateProvisionedDatabaseCommand {
    CreateProvisionedDatabaseCommand::new(
        "tenant_alpha".to_string(),
        "tenant_alpha_user".to_string(),
        "supersecret".to_string(),
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$somehashvalue".to_string(),
        true,
    )
    .expect("valid create command")
}

pub fn delete_command() -> DeleteProvisionedDatabaseCommand {
    DeleteProvisionedDatabaseCommand::new("tenant_alpha".to_string()).expect("valid command")
}

pub fn change_password_command() -> ChangeProvisionedDatabasePasswordCommand {
    ChangeProvisionedDatabasePasswordCommand::new(
        "tenant_alpha".to_string(),
        "new_supersecret".to_string(),
        "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$newhashvalue".to_string(),
    )
    .expect("valid command")
}

pub fn database_with_status(status: ProvisionedDatabaseStatus) -> ProvisionedDatabase {
    ProvisionedDatabase::restore(
        ProvisionedDatabaseId::new_random(),
        ProvisionedDatabaseName::new("tenant_alpha".to_string()).expect("valid name"),
        DatabaseUsername::new("tenant_alpha_user".to_string()).expect("valid username"),
        DatabasePasswordHash::new(
            "$argon2id$v=19$m=19456,t=2,p=1$c29tZXNhbHQ$somehashvalue".to_string(),
        )
        .expect("valid password hash"),
        status,
        Utc::now(),
    )
}
