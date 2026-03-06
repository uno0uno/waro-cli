use crate::config::Config;
use anyhow::Result;
use reqwest::Client;
use serde_json::Value;

pub struct WaroClient {
    http: Client,
    config: Config,
}

impl WaroClient {
    pub fn new(config: Config) -> Self {
        let http = Client::builder()
            .user_agent("waro-cli/0.1.0")
            .build()
            .expect("Failed to build HTTP client");

        Self { http, config }
    }

    pub async fn post(&self, path: &str, body: Value) -> Result<Value> {
        let url = format!("{}{}", self.config.api_url, path);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.config.api_key))
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                if e.is_connect() || e.is_timeout() {
                    anyhow::anyhow!("Cannot reach {} — check WARO_API_URL", self.config.api_url)
                } else {
                    anyhow::anyhow!("Network error: {}", e)
                }
            })?;

        let status = resp.status();
        let json: Value = resp.json().await?;

        if !status.is_success() {
            let api_detail = json
                .get("detail")
                .or_else(|| json.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("");

            return Err(match status.as_u16() {
                401 => anyhow::anyhow!(
                    "Invalid API key. Check WARO_API_KEY in your environment or .env file."
                ),
                403 => {
                    if api_detail.is_empty() {
                        anyhow::anyhow!(
                            "Insufficient scope. This command requires a higher-privilege API key."
                        )
                    } else {
                        anyhow::anyhow!("Insufficient scope: {}", api_detail)
                    }
                }
                404 => anyhow::anyhow!("Resource not found."),
                422 => {
                    if api_detail.is_empty() {
                        anyhow::anyhow!("Invalid request (422). Check the parameters you provided.")
                    } else {
                        anyhow::anyhow!("Invalid request: {}", api_detail)
                    }
                }
                429 => anyhow::anyhow!("Rate limit exceeded. Try again later."),
                500..=599 => anyhow::anyhow!(
                    "Server error ({}). The WaRo API is having issues — try again later.",
                    status.as_u16()
                ),
                _ => anyhow::anyhow!("API error {}: {}", status, api_detail),
            });
        }

        Ok(json)
    }

    pub fn print_config(&self) {
        use colored::Colorize;
        let key_preview = if self.config.api_key.len() > 12 {
            format!(
                "{}...{}",
                &self.config.api_key[..8],
                &self.config.api_key[self.config.api_key.len() - 4..]
            )
        } else {
            "***".to_string()
        };

        println!("{} {}", "WARO_API_URL :".bold(), self.config.api_url);
        println!("{} {}", "WARO_API_KEY :".bold(), key_preview.dimmed());
    }
}
