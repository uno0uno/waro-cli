mod commands;
mod config;
mod client;
mod output;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(
    name = "waro",
    version,
    about = "WaRo Colombia CLI — Developer tool for the WaRo public API",
    long_about = None,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format: json (default) | table
    #[arg(long, global = true, default_value = "json")]
    output: String,

    /// Comma-separated fields to include in response (e.g. id,status,total)
    #[arg(long, global = true)]
    fields: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sales / orders commands
    Sales(commands::sales::SalesArgs),
    /// Menu commands (products, recipes, modifiers)
    Menu(commands::menu::MenuArgs),
    /// Print current config (API URL, key prefix)
    Config,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();
    let cfg = config::Config::from_env()?;
    let client = client::WaroClient::new(cfg);

    let out_format = cli.output.clone();
    let fields = cli.fields.clone();

    match cli.command {
        Commands::Sales(args) => commands::sales::run(args, &client, &out_format, fields).await?,
        Commands::Menu(args) => commands::menu::run(args, &client, &out_format, fields).await?,
        Commands::Config => {
            client.print_config();
        }
    }

    Ok(())
}
