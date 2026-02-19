#[derive(Clone, Debug)]
pub struct AppConfig {
    pub port: u16,
    pub postgres_host: String,
    pub postgres_port: u16,
    pub postgres_user: String,
    pub postgres_password: String,
    pub postgres_admin_database: String,
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
}
