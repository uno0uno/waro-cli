mod client;
mod commands;
mod config;
mod output;

use anyhow::Result;
use clap::{Parser, Subcommand};

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

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,
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
async fn main() {
    dotenvy::dotenv().ok();

    let cli = Cli::parse();

    // Disable colors if --no-color, NO_COLOR env var, or stdout is not a TTY
    if cli.no_color || std::env::var_os("NO_COLOR").is_some() {
        colored::control::set_override(false);
    } else {
        use std::io::IsTerminal;
        if !std::io::stdout().is_terminal() {
            colored::control::set_override(false);
        }
    }

    if let Err(e) = run(cli).await {
        output::eprint_error(&e.to_string());
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
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
