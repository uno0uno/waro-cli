use crate::client::WaroClient;
use crate::output;
use crate::spinner::Spinner;
use anyhow::{bail, Context, Result};
use clap::{Args, Subcommand};
use serde_json::Value;
use std::path::Path;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct QueriesArgs {
    #[command(subcommand)]
    pub command: QueriesCommands,
}

#[derive(Subcommand)]
pub enum QueriesCommands {
    /// Fetch available safe QuerySpec datasets and fields
    Schema,
    /// Run a safe QuerySpec against the API
    Run(RunArgs),
}

impl QueriesArgs {
    pub fn command_label(&self) -> &'static str {
        match self.command {
            QueriesCommands::Schema => "queries schema",
            QueriesCommands::Run(_) => "queries run",
        }
    }
}

#[derive(Args)]
pub struct RunArgs {
    /// QuerySpec JSON string or path to a JSON file
    #[arg(long)]
    spec: String,

    /// Validate and print request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

pub async fn run(
    args: QueriesArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        QueriesCommands::Schema => schema(client, format, fields).await,
        QueriesCommands::Run(a) => run_query(a, client, format, fields).await,
    }
}

async fn schema(client: &WaroClient, format: &str, fields: Option<String>) -> Result<()> {
    let sp = Spinner::start();
    let resp = client.get("/v1/queries/schema").await?;
    sp.stop();
    output::emit("queries schema", resp, format, fields.as_deref())?;
    Ok(())
}

async fn run_query(
    a: RunArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let spec = parse_queryspec(&a.spec)?;

    if a.dry_run {
        println!("DRY RUN - POST /v1/queries/run");
        println!("{}", serde_json::to_string_pretty(&spec)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/queries/run", spec).await?;
    sp.stop();
    output::emit("queries run", resp, format, fields.as_deref())?;
    Ok(())
}

fn parse_queryspec(spec: &str) -> Result<Value> {
    let raw = if Path::new(spec).is_file() {
        std::fs::read_to_string(spec)
            .with_context(|| format!("Invalid QuerySpec file: cannot read {spec}"))?
    } else {
        spec.to_string()
    };

    let value: Value =
        serde_json::from_str(&raw).with_context(|| "Invalid QuerySpec JSON".to_string())?;
    let Some(obj) = value.as_object() else {
        bail!("Invalid QuerySpec: spec must be a JSON object");
    };
    match obj.get("dataset").and_then(Value::as_str) {
        Some(dataset) if !dataset.trim().is_empty() => Ok(value),
        _ => bail!("Invalid QuerySpec: dataset is required"),
    }
}

#[cfg(test)]
mod tests {
    use super::parse_queryspec;
    use serde_json::json;

    #[test]
    fn parse_queryspec_accepts_json_string() {
        let value = parse_queryspec(
            r#"{"dataset":"sales_items","measures":["revenue"],"dimensions":["product"],"limit":5}"#,
        )
        .unwrap();

        assert_eq!(value["dataset"], "sales_items");
    }

    #[test]
    fn parse_queryspec_accepts_file_path() {
        let path = std::env::temp_dir().join("waro_queryspec_test.json");
        std::fs::write(
            &path,
            serde_json::to_string(&json!({
                "dataset": "customers",
                "measures": ["total_spent"],
                "dimensions": ["customer"],
                "limit": 5
            }))
            .unwrap(),
        )
        .unwrap();

        let value = parse_queryspec(path.to_str().unwrap()).unwrap();
        let _ = std::fs::remove_file(path);

        assert_eq!(value["dataset"], "customers");
    }

    #[test]
    fn parse_queryspec_rejects_malformed_json() {
        let err = parse_queryspec("{bad-json").unwrap_err().to_string();

        assert!(err.contains("Invalid QuerySpec JSON"));
    }

    #[test]
    fn parse_queryspec_requires_dataset() {
        let err = parse_queryspec(r#"{"measures":["revenue"]}"#)
            .unwrap_err()
            .to_string();

        assert!(err.contains("dataset is required"));
    }
}
