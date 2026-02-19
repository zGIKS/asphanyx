use std::collections::BTreeMap;

use serde_json::{Value, json};
use swagger_axum_api::data_api::domain::model::{
    commands::{
        create_row_command::{CreateRowCommand, CreateRowCommandParts},
        patch_row_command::{PatchRowCommand, PatchRowCommandParts},
    },
    enums::data_api_principal_type::DataApiPrincipalType,
    queries::{
        get_row_query::{GetRowQuery, GetRowQueryParts},
        list_rows_query::{ListRowsQuery, ListRowsQueryParts},
    },
};

pub fn create_row_command(payload: Value) -> CreateRowCommand {
    CreateRowCommand::new(CreateRowCommandParts {
        api_version: "v1".to_string(),
        tenant_id: "tienda1".to_string(),
        schema_name: "public".to_string(),
        table_name: "productos".to_string(),
        principal: "api-key-test".to_string(),
        principal_type: DataApiPrincipalType::ApiKey,
        request_id: Some("req-1".to_string()),
        subject_owner_id: Some("owner-1".to_string()),
        row_owner_id: Some("owner-1".to_string()),
        payload,
    })
    .expect("valid command")
}

pub fn patch_row_command(payload: Value) -> PatchRowCommand {
    PatchRowCommand::new(PatchRowCommandParts {
        api_version: "v1".to_string(),
        tenant_id: "tienda1".to_string(),
        schema_name: "public".to_string(),
        table_name: "productos".to_string(),
        row_identifier: "1".to_string(),
        principal: "api-key-test".to_string(),
        principal_type: DataApiPrincipalType::ApiKey,
        request_id: Some("req-2".to_string()),
        subject_owner_id: Some("owner-1".to_string()),
        row_owner_id: Some("owner-2".to_string()),
        payload,
    })
    .expect("valid command")
}

pub fn list_rows_query() -> ListRowsQuery {
    let filters = BTreeMap::from([
        ("nombre".to_string(), "Mouse".to_string()),
        ("campo_inexistente".to_string(), "x".to_string()),
    ]);

    ListRowsQuery::new(ListRowsQueryParts {
        api_version: "v1".to_string(),
        tenant_id: "tienda1".to_string(),
        schema_name: "public".to_string(),
        table_name: "productos".to_string(),
        principal: "api-key-test".to_string(),
        principal_type: DataApiPrincipalType::ApiKey,
        request_id: Some("req-3".to_string()),
        subject_owner_id: Some("owner-1".to_string()),
        row_owner_id: Some("owner-1".to_string()),
        select_fields: vec!["nombre".to_string(), "campo_inexistente".to_string()],
        filters,
        limit: 20,
        offset: 0,
        order_by: Some("campo_inexistente".to_string()),
        order_desc: false,
    })
    .expect("valid query")
}

pub fn get_row_query() -> GetRowQuery {
    GetRowQuery::new(GetRowQueryParts {
        api_version: "v1".to_string(),
        tenant_id: "tienda1".to_string(),
        schema_name: "public".to_string(),
        table_name: "productos".to_string(),
        row_identifier: "1".to_string(),
        principal: "api-key-test".to_string(),
        principal_type: DataApiPrincipalType::ApiKey,
        request_id: Some("req-4".to_string()),
        subject_owner_id: Some("owner-1".to_string()),
        row_owner_id: Some("owner-1".to_string()),
    })
    .expect("valid query")
}

pub fn sample_payload() -> Value {
    json!({
        "nombre": "Mouse",
        "precio": 49.99,
        "image_url": "https://image.example/test.png"
    })
}
