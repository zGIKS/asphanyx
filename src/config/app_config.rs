#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_admin_database: String,
    pub iam_grpc_endpoint: String,
    pub iam_grpc_timeout_ms: u64,
    pub iam_token_cache_ttl_seconds: u64,
    pub iam_grpc_circuit_breaker_failure_threshold: u32,
    pub iam_grpc_circuit_breaker_open_seconds: u64,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            port: std::env::var("PORT")
                .unwrap_or_else(|_| "8081".to_string())
                .parse()
                .unwrap_or(8081),
            postgres_host: std::env::var("POSTGRES_HOST")
                .unwrap_or_else(|_| "127.0.0.1".to_string()),
            postgres_port: std::env::var("POSTGRES_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .unwrap_or(5432),
            postgres_user: std::env::var("POSTGRES_USER")
                .unwrap_or_else(|_| "postgres".to_string()),
            postgres_password: std::env::var("POSTGRES_PASSWORD")
                .unwrap_or_else(|_| "admin".to_string()),
            postgres_admin_database: std::env::var("POSTGRES_ADMIN_DATABASE")
                .unwrap_or_else(|_| "postgres".to_string()),
            iam_grpc_endpoint: std::env::var("IAM_GRPC_ENDPOINT")
                .unwrap_or_else(|_| "http://127.0.0.1:50051".to_string()),
            iam_grpc_timeout_ms: std::env::var("IAM_GRPC_TIMEOUT_MS")
                .unwrap_or_else(|_| "400".to_string())
                .parse()
                .unwrap_or(400),
            iam_token_cache_ttl_seconds: std::env::var("IAM_TOKEN_CACHE_TTL_SECONDS")
                .unwrap_or_else(|_| "45".to_string())
                .parse()
                .unwrap_or(45),
            iam_grpc_circuit_breaker_failure_threshold: std::env::var(
                "IAM_GRPC_CIRCUIT_BREAKER_FAILURE_THRESHOLD",
            )
            .unwrap_or_else(|_| "5".to_string())
            .parse()
            .unwrap_or(5),
            iam_grpc_circuit_breaker_open_seconds: std::env::var(
                "IAM_GRPC_CIRCUIT_BREAKER_OPEN_SECONDS",
            )
            .unwrap_or_else(|_| "30".to_string())
            .parse()
            .unwrap_or(30),
        }
    }

    pub fn admin_database_url(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            self.postgres_admin_database
        )
    }

    pub fn database_url_for(&self, database_name: &str) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.postgres_user,
            self.postgres_password,
            self.postgres_host,
            self.postgres_port,
            database_name
        )
    }
}
