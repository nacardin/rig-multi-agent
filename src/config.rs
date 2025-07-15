use std::env;

#[derive(Debug, Clone)]
pub struct Config {
    pub xai_api_key: String,
    pub surreal_config: SurrealConfig,
}

#[derive(Debug, Clone)]
pub struct SurrealConfig {
    pub host: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub database: String,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenv::dotenv().ok(); // Load .env file if it exists, ignore if not found

        let xai_api_key =
            env::var("XAI_API_KEY").map_err(|_| ConfigError::MissingEnvVar("XAI_API_KEY"))?;

        // Validate API key format
        if !xai_api_key.starts_with("xai-") {
            return Err(ConfigError::InvalidValue(
                "XAI_API_KEY must start with 'xai-'",
            ));
        }

        let surreal_config = SurrealConfig {
            host: env::var("SURREAL_HOST")
                .map_err(|_| ConfigError::MissingEnvVar("SURREAL_HOST"))?,
            username: env::var("SURREAL_USERNAME")
                .map_err(|_| ConfigError::MissingEnvVar("SURREAL_USERNAME"))?,
            password: env::var("SURREAL_PASSWORD")
                .map_err(|_| ConfigError::MissingEnvVar("SURREAL_PASSWORD"))?,
            namespace: env::var("SURREAL_NAMESPACE")
                .map_err(|_| ConfigError::MissingEnvVar("SURREAL_NAMESPACE"))?,
            database: env::var("SURREAL_DATABASE")
                .map_err(|_| ConfigError::MissingEnvVar("SURREAL_DATABASE"))?,
        };

        // Validate that required fields are not empty
        if surreal_config.host.trim().is_empty() {
            return Err(ConfigError::InvalidValue("SURREAL_HOST cannot be empty"));
        }
        if surreal_config.username.trim().is_empty() {
            return Err(ConfigError::InvalidValue(
                "SURREAL_USERNAME cannot be empty",
            ));
        }
        if surreal_config.password.trim().is_empty() {
            return Err(ConfigError::InvalidValue(
                "SURREAL_PASSWORD cannot be empty",
            ));
        }

        Ok(Config {
            xai_api_key,
            surreal_config,
        })
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnvVar(&'static str),
    InvalidValue(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ConfigError::MissingEnvVar(var) => write!(f, "Missing environment variable: {}", var),
            ConfigError::InvalidValue(msg) => write!(f, "Invalid configuration value: {}", msg),
        }
    }
}

impl std::error::Error for ConfigError {}
