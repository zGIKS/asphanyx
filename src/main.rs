use axum::Router;
use dotenvy::dotenv;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::{
    config::app_config::AppConfig,
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

mod config;
mod provisioner;

#[derive(OpenApi)]
#[openapi(
    paths(
        provisioner::interfaces::rest::controllers::provisioner_rest_controller::create_provisioned_database,
        provisioner::interfaces::rest::controllers::provisioner_rest_controller::delete_provisioned_database,
        provisioner::interfaces::rest::controllers::provisioner_rest_controller::list_provisioned_databases
    ),
    components(
        schemas(
            CreateProvisionedDatabaseRequestResource,
            ListProvisionedDatabasesQueryResource,
            ProvisionedDatabaseResource,
            ErrorResponseResource
        )
    ),
    tags(
        (name = "provisioner", description = "PostgreSQL database provisioning bounded context")
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

    let app = Router::new()
        .merge(provisioner_router)
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
