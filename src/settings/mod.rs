#[derive(serde::Deserialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(serde::Deserialize, Clone)]
pub struct SslConfig {
    pub enabled: bool,
    pub cert_path: Option<String>,
    pub key_path: Option<String>,
}

#[derive(serde::Deserialize, Clone)]
pub struct GoogleConfig {
    pub redirect_url: String,
    pub auth_url: String,
    pub token_url: String,
}

#[derive(serde::Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub ssl: SslConfig,
    pub google: GoogleConfig,
    pub database_url: String,
    pub google_client_id: String,
    pub google_client_secret: String,
}

#[derive(PartialEq, Clone)]
pub enum Environment {
    Development,
    Production,
}

impl std::fmt::Display for Environment {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Environment::Production => write!(f, "prod"),
            Environment::Development => write!(f, "dev"),
        }
    }
}
