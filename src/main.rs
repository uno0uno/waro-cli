mod client;
mod commands;
mod compare;
mod config;
mod contract;
mod output;
mod pagination;
mod spinner;
mod validate;

use anyhow::Result;
use clap::{Parser, Subcommand};
use clap_complete::Shell;

#[derive(Parser)]
#[command(
    name = "waro",
    version,
    about = "WaRo Colombia CLI — Developer tool for the WaRo public API",
    long_about = None,
    arg_required_else_help = true,
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Output format
    #[arg(long, global = true, default_value = "json", value_parser = ["json", "table", "fields", "agent-json"])]
    output: String,

    /// Comma-separated fields to include in response (e.g. id,status,total)
    #[arg(long, global = true)]
    fields: Option<String>,

    /// Disable colored output
    #[arg(long, global = true)]
    no_color: bool,

    /// Profile name from ~/.waro/config.toml
    #[arg(long, global = true)]
    profile: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
    /// Sales / orders commands
    Sales(commands::sales::SalesArgs),
    /// Customer commands
    Customers(commands::customers::CustomersArgs),
    /// Menu commands (products, recipes, modifiers)
    Menu(commands::menu::MenuArgs),
    /// Analytics commands (menu BCG, food cost, alerts, data quality)
    Analytics(commands::analytics::AnalyticsArgs),
    /// Financial analysis commands (product margin, cost, profitability)
    Financial(commands::financial::FinancialArgs),
    /// WaRo loyalty commands (estimate, balances, customer wallet)
    Waros(commands::waros::WarosArgs),
    /// Safe analytical QuerySpec commands
    Queries(commands::queries::QueriesArgs),
    /// Print current config (API URL, key prefix)
    Config,
    /// Generate shell completion script
    Completions {
        /// Shell to generate completions for
        shell: Shell,
    },
    /// Introspect endpoint schema — for AI agents and tooling
    Schema(commands::schema::SchemaArgs),
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

    let error_format = cli.output.clone();
    let error_command = command_label(&cli.command);
    if error_format == "fields" {
        if let Some(contract) = contract::contract_for(&error_command) {
            if let Err(e) = output::print_contract_fields(contract) {
                output::eprint_error(&e.to_string());
                std::process::exit(1);
            }
            return;
        }
    }
    if let Err(e) = run(cli).await {
        if error_format == "agent-json" {
            let message = e.to_string();
            let _ = output::print_agent_error(&error_command, &message, error_kind(&message));
        } else {
            output::eprint_error(&e.to_string());
        }
        std::process::exit(1);
    }
}

fn error_kind(message: &str) -> &'static str {
    if message.contains("WARO_API_KEY")
        || message.contains("config file")
        || message.contains("profile")
        || message.contains("HOME env var")
    {
        "config"
    } else if message.contains("unknown field")
        || message.contains("not allowed")
        || message.contains("must be")
        || message.contains("required")
        || message.contains("Invalid")
        || message.contains("UUID")
        || message.contains("YYYY-MM-DD")
    {
        "validation"
    } else if message.contains("HTTP")
        || message.contains("API")
        || message.contains("Cannot reach")
        || message.contains("Network error")
        || message.contains("request")
        || message.contains("response")
        || message.contains("Resource not found")
        || message.contains("Insufficient scope")
        || message.contains("Rate limit")
        || message.contains("Server error")
    {
        "api"
    } else {
        "unknown"
    }
}

fn command_label(command: &Commands) -> String {
    match command {
        Commands::Sales(args) => args.command_label().to_string(),
        Commands::Customers(args) => args.command_label().to_string(),
        Commands::Menu(args) => args.command_label().to_string(),
        Commands::Analytics(args) => args.command_label().to_string(),
        Commands::Financial(args) => args.command_label().to_string(),
        Commands::Waros(args) => args.command_label().to_string(),
        Commands::Queries(args) => args.command_label().to_string(),
        Commands::Config => "config".to_string(),
        Commands::Completions { .. } => "completions".to_string(),
        Commands::Schema(_) => "schema".to_string(),
    }
}

async fn run(cli: Cli) -> Result<()> {
    // Handle completions and schema before loading config — no API key needed
    if let Commands::Completions { shell } = cli.command {
        use clap::CommandFactory;
        use clap_complete::generate;
        generate(shell, &mut Cli::command(), "waro", &mut std::io::stdout());
        return Ok(());
    }
    if let Commands::Schema(args) = cli.command {
        return commands::schema::run(args);
    }

    let cfg = config::Config::load(cli.profile.as_deref())?;
    let client = client::WaroClient::new(cfg);

    spinner::print_welcome();

    let out_format = cli.output.clone();
    let fields = cli.fields.clone();

    match cli.command {
        Commands::Sales(args) => commands::sales::run(args, &client, &out_format, fields).await?,
        Commands::Customers(args) => {
            commands::customers::run(args, &client, &out_format, fields).await?
        }
        Commands::Menu(args) => commands::menu::run(args, &client, &out_format, fields).await?,
        Commands::Analytics(args) => {
            commands::analytics::run(args, &client, &out_format, fields).await?
        }
        Commands::Financial(args) => {
            commands::financial::run(args, &client, &out_format, fields).await?
        }
        Commands::Waros(args) => commands::waros::run(args, &client, &out_format, fields).await?,
        Commands::Queries(args) => {
            commands::queries::run(args, &client, &out_format, fields).await?
        }
        Commands::Config => {
            client.print_config();
        }
        Commands::Completions { .. } | Commands::Schema(_) => unreachable!(),
    }

    Ok(())
}
