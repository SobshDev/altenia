use std::env;

/// Application configuration loaded from environment variables
#[derive(Debug, Clone)]
pub struct Config {
    pub database_url: String,
    pub jwt_access_secret: String,
    pub jwt_refresh_secret: String,
    pub refresh_token_duration_days: i64,
    pub host: String,
    pub port: u16,
}

impl Config {
    /// Load configuration from environment variables
    pub fn from_env() -> Result<Self, ConfigError> {
        dotenvy::dotenv().ok(); // Load .env file if present

        Ok(Self {
            database_url: env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingEnv("DATABASE_URL"))?,
            jwt_access_secret: env::var("JWT_ACCESS_SECRET")
                .map_err(|_| ConfigError::MissingEnv("JWT_ACCESS_SECRET"))?,
            jwt_refresh_secret: env::var("JWT_REFRESH_SECRET")
                .map_err(|_| ConfigError::MissingEnv("JWT_REFRESH_SECRET"))?,
            refresh_token_duration_days: env::var("REFRESH_TOKEN_DURATION_DAYS")
                .unwrap_or_else(|_| "7".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("REFRESH_TOKEN_DURATION_DAYS"))?,
            host: env::var("HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .map_err(|_| ConfigError::InvalidValue("PORT"))?,
        })
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Debug)]
pub enum ConfigError {
    MissingEnv(&'static str),
    InvalidValue(&'static str),
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingEnv(var) => write!(f, "Missing environment variable: {}", var),
            Self::InvalidValue(var) => write!(f, "Invalid value for environment variable: {}", var),
        }
    }
}

impl std::error::Error for ConfigError {}
