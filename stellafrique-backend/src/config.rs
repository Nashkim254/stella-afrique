use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub app_env: String,
    pub host: String,
    pub port: u16,
    pub database_url: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            app_env: env::var("APP_ENV").unwrap_or_else(|_| "development".to_string()),
            host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8080".to_string())
                .parse()
                .context("APP_PORT must be a valid port number")?,
            database_url: env::var("DATABASE_URL")
                .context("DATABASE_URL is required to start the backend")?,
        })
    }

    pub fn bind_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}
