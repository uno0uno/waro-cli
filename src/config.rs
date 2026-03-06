use anyhow::{bail, Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub api_url: String,
    pub api_key: String,
    pub profile_name: Option<String>,
}

// ── TOML structs ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct TomlConfig {
    profiles: HashMap<String, Profile>,
}

#[derive(Deserialize)]
struct Profile {
    api_url: Option<String>,
    api_key: String,
}

// ── Config impl ───────────────────────────────────────────────────────────────

impl Config {
    /// Load config from a named profile in `~/.waro/config.toml`,
    /// or fall back to `WARO_API_KEY` / `WARO_API_URL` env vars.
    ///
    /// Profile resolution order:
    /// 1. `profile` argument (from `--profile` flag)
    /// 2. `WARO_PROFILE` env var
    /// 3. Env vars only (current behaviour)
    pub fn load(profile: Option<&str>) -> Result<Self> {
        let profile_name = profile
            .map(str::to_owned)
            .or_else(|| env::var("WARO_PROFILE").ok());

        if let Some(ref name) = profile_name {
            return Self::from_profile(name);
        }

        Self::from_env_vars(None)
    }

    fn from_profile(name: &str) -> Result<Self> {
        let home = env::var("HOME").context("HOME env var not set")?;
        let path = format!("{}/.waro/config.toml", home);

        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Cannot read config file: {}", path))?;

        let toml: TomlConfig =
            toml::from_str(&content).with_context(|| format!("Invalid TOML in {}", path))?;

        let profile = toml.profiles.get(name).ok_or_else(|| {
            let available = toml.profiles.keys().cloned().collect::<Vec<_>>().join(", ");
            anyhow::anyhow!(
                "profile '{}' not found in {}. Available profiles: {}",
                name,
                path,
                if available.is_empty() {
                    "(none)".to_string()
                } else {
                    available
                }
            )
        })?;

        let api_key = profile.api_key.clone();
        if api_key.is_empty() {
            bail!("api_key is empty for profile '{}' in {}", name, path);
        }

        Ok(Self {
            api_url: profile
                .api_url
                .clone()
                .unwrap_or_else(|| "https://api.warocol.com".to_string()),
            api_key,
            profile_name: Some(name.to_owned()),
        })
    }

    fn from_env_vars(profile_name: Option<String>) -> Result<Self> {
        let api_url =
            env::var("WARO_API_URL").unwrap_or_else(|_| "https://api.warocol.com".to_string());

        let api_key = env::var("WARO_API_KEY")
            .context("WARO_API_KEY env var is required. Set it in .env or export it.")?;

        Ok(Self {
            api_url,
            api_key,
            profile_name,
        })
    }
}
