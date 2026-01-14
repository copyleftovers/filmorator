use std::num::NonZeroU16;

#[derive(Debug, Clone)]
pub struct Config {
    pub port: NonZeroU16,
    pub database_url: String,
    pub bucket: String,
    pub s3_endpoint: Option<String>,
    pub s3_public_url: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("PORT invalid: {0}")]
    InvalidPort(String),
    #[error("DATABASE_URL required")]
    MissingDatabaseUrl,
    #[error("FILMORATOR_BUCKET required")]
    MissingBucket,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let port_str = std::env::var("PORT").unwrap_or_else(|_| "3000".into());
        let port = port_str
            .parse::<u16>()
            .ok()
            .and_then(NonZeroU16::new)
            .ok_or(ConfigError::InvalidPort(port_str))?;

        Ok(Self {
            port,
            database_url: std::env::var("DATABASE_URL")
                .map_err(|_| ConfigError::MissingDatabaseUrl)?,
            bucket: std::env::var("FILMORATOR_BUCKET").map_err(|_| ConfigError::MissingBucket)?,
            s3_endpoint: std::env::var("AWS_ENDPOINT_URL").ok(),
            s3_public_url: std::env::var("S3_PUBLIC_URL").ok(),
        })
    }
}
