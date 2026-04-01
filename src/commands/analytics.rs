use crate::client::WaroClient;
use crate::output;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
pub struct AnalyticsArgs {
    #[command(subcommand)]
    pub command: AnalyticsCommands,
}

#[derive(Subcommand)]
pub enum AnalyticsCommands {
    /// BCG menu analysis — classify products as Stars, Plowhorses, Puzzles, Dogs
    Menu(MenuArgs),
    /// Food cost analysis per product with margins
    FoodCost(FoodCostArgs),
    /// Inventory and operational alerts (zero stock, slow movers, etc.)
    Alerts(AlertsArgs),
    /// Data quality report — price spikes, drops and anomalies in purchase history
    DataQuality(DataQualityArgs),
}

// ── menu ──────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct MenuArgs {
    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Max number of products to return (1-100)
    #[arg(long, default_value = "10")]
    limit: u32,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── food-cost ─────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct FoodCostArgs {
    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── alerts ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct AlertsArgs {
    /// Max number of alerts to return (1-100)
    #[arg(long, default_value = "10")]
    limit: u32,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── data-quality ──────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DataQualityArgs {
    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── handlers ──────────────────────────────────────────────────────────────────

pub async fn run(
    args: AnalyticsArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        AnalyticsCommands::Menu(a) => menu(a, client, format, fields).await,
        AnalyticsCommands::FoodCost(a) => food_cost(a, client, format, fields).await,
        AnalyticsCommands::Alerts(a) => alerts(a, client, format, fields).await,
        AnalyticsCommands::DataQuality(a) => data_quality(a, client, format, fields).await,
    }
}

async fn menu(
    a: MenuArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "limit": a.limit,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/menu-analysis");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/menu-analysis", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn food_cost(
    a: FoodCostArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/food-cost");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/food-cost", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn alerts(
    a: AlertsArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let body = json!({
        "limit": a.limit,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/alerts");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/alerts", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn data_quality(
    a: DataQualityArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let body = json!({});

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/data-quality");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/data-quality", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
