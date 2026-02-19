use axum::Router;
use dotenvy::dotenv;
use swagger_axum_api::{
    access_control::{
        build_access_control_router,
        interfaces::rest::resources::{
            access_control_error_response_resource::AccessControlErrorResponseResource,
            assign_role_request_resource::AssignRoleRequestResource,
            evaluate_permission_request_resource::{
                EvaluatePermissionRequestResource, EvaluatePermissionResponseResource,
            },
            upsert_policy_rule_request_resource::UpsertPolicyRuleRequestResource,
        },
    },
    config::app_config::AppConfig,
    data_api::{
        build_data_api_router,
        interfaces::rest::resources::{
            data_api_auth_headers_resource::DataApiAuthHeadersResource,
            data_api_error_response_resource::DataApiErrorResponseResource,
            data_api_list_rows_query_resource::DataApiListRowsQueryResource,
            data_api_payload_resource::DataApiPayloadResource,
        },
    },
    provisioner::{
        build_provisioner_router,
        interfaces::rest::resources::{
            create_provisioned_database_request_resource::{
                CreateProvisionedDatabaseRequestResource, ListProvisionedDatabasesQueryResource,
            },
            error_response_resource::ErrorResponseResource,
            provisioned_database_resource::ProvisionedDatabaseResource,
        },
    },
};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[derive(OpenApi)]
#[openapi(
    paths(
        swagger_axum_api::provisioner::interfaces::rest::controllers::provisioner_rest_controller::create_provisioned_database,
        swagger_axum_api::provisioner::interfaces::rest::controllers::provisioner_rest_controller::delete_provisioned_database,
        swagger_axum_api::provisioner::interfaces::rest::controllers::provisioner_rest_controller::list_provisioned_databases,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::list_rows,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::get_row,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::create_row,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::patch_row,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::delete_row,
        swagger_axum_api::data_api::interfaces::rest::controllers::data_api_rest_controller::introspect_table_schema,
        swagger_axum_api::access_control::interfaces::rest::controllers::access_control_rest_controller::assign_role_to_principal,
        swagger_axum_api::access_control::interfaces::rest::controllers::access_control_rest_controller::upsert_policy_rule,
        swagger_axum_api::access_control::interfaces::rest::controllers::access_control_rest_controller::evaluate_permission
    ),
    components(
        schemas(
            CreateProvisionedDatabaseRequestResource,
            ListProvisionedDatabasesQueryResource,
            ProvisionedDatabaseResource,
            ErrorResponseResource,
            DataApiAuthHeadersResource,
            DataApiErrorResponseResource,
            DataApiListRowsQueryResource,
            DataApiPayloadResource,
            AssignRoleRequestResource,
            UpsertPolicyRuleRequestResource,
            EvaluatePermissionRequestResource,
            EvaluatePermissionResponseResource,
            AccessControlErrorResponseResource
        )
    ),
    tags(
        (name = "provisioner", description = "PostgreSQL database provisioning bounded context"),
        (name = "data-api", description = "Dynamic and versioned CRUD data API bounded context"),
        (name = "access-control", description = "Authorization policy engine bounded context")
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config = AppConfig::from_env();

    let provisioner_router = build_provisioner_router(&config)
        .await
        .expect("failed to build provisioner router");
    let data_api_router = build_data_api_router(&config)
        .await
        .expect("failed to build data api router");
    let access_control_router = build_access_control_router(&config)
        .await
        .expect("failed to build access control router");

    let app = Router::new()
        .merge(provisioner_router)
        .merge(data_api_router)
        .merge(access_control_router)
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()));

    let addr = format!("0.0.0.0:{}", config.port);
    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("failed to bind server address");

    println!("Servidor corriendo en http://localhost:{}", config.port);
    println!(
        "Swagger UI disponible en http://localhost:{}/swagger-ui",
        config.port
    );

    axum::serve(listener, app)
        .await
        .expect("failed to start axum server");
}
