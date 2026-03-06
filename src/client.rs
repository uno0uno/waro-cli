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
            .await?;

        let status = resp.status();
        let json: Value = resp.json().await?;

        if !status.is_success() {
            let msg = json
                .get("detail")
                .or_else(|| json.get("message"))
                .and_then(|v| v.as_str())
                .unwrap_or("Unknown error");
            anyhow::bail!("API error {}: {}", status, msg);
        }

        Ok(json)
    }

    pub fn print_config(&self) {
        let key_preview = if self.config.api_key.len() > 12 {
            format!(
                "{}...{}",
                &self.config.api_key[..8],
                &self.config.api_key[self.config.api_key.len() - 4..]
            )
        } else {
            "***".to_string()
        };

        println!("WARO_API_URL : {}", self.config.api_url);
        println!("WARO_API_KEY : {}", key_preview);
    }
}
