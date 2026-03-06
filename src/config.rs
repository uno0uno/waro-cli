use anyhow::{Context, Result};
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let api_url =
            env::var("WARO_API_URL").unwrap_or_else(|_| "https://api.warocol.com".to_string());

        let api_key = env::var("WARO_API_KEY")
            .context("WARO_API_KEY env var is required. Set it in .env or export it.")?;

        Ok(Self { api_url, api_key })
    }
}
