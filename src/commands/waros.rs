use crate::client::WaroClient;
use crate::output;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct WarosArgs {
    #[command(subcommand)]
    pub command: WarosCommands,
}

#[derive(Subcommand)]
pub enum WarosCommands {
    /// Estimate WaRos earned for a given purchase amount
    Estimate(EstimateArgs),
    /// Batch WaRos balance lookup for multiple customer profiles
    Balances(BalancesArgs),
    /// WaRos wallet summary and recent transactions for a customer
    Customer(CustomerArgs),
}

// ── estimate ──────────────────────────────────────────────────────────────────

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct EstimateArgs {
    /// Purchase total amount (required)
    #[arg(long)]
    total: f64,

    /// Customer UUID (optional — for personalized estimate)
    #[arg(long)]
    customer_id: Option<String>,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── balances ──────────────────────────────────────────────────────────────────

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct BalancesArgs {
    /// Comma-separated customer profile UUIDs (e.g. uuid1,uuid2,uuid3)
    #[arg(long)]
    profile_ids: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── customer ──────────────────────────────────────────────────────────────────

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct CustomerArgs {
    /// Customer profile UUID
    #[arg(long)]
    profile_id: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── handlers ──────────────────────────────────────────────────────────────────

pub async fn run(
    args: WarosArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        WarosCommands::Estimate(a) => estimate(a, client, format, fields).await,
        WarosCommands::Balances(a) => balances(a, client, format, fields).await,
        WarosCommands::Customer(a) => customer(a, client, format, fields).await,
    }
}

async fn estimate(
    a: EstimateArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    if a.total < 0.0 {
        anyhow::bail!("--total must be >= 0, got: {}", a.total);
    }
    if let Some(ref v) = a.customer_id {
        validate::validate_uuid("customer-id", v)?;
    }

    let body = json!({
        "totalAmount": a.total,
        "customerId": a.customer_id,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/waros/estimate");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/waros/estimate", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn balances(
    a: BalancesArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let ids: Vec<&str> = a
        .profile_ids
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();

    if ids.is_empty() {
        anyhow::bail!("--profile-ids must contain at least one UUID");
    }
    for id in &ids {
        validate::validate_uuid("profile-ids", id)?;
    }

    let body = json!({
        "profileIds": ids,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/waros/balances");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/waros/balances", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn customer(
    a: CustomerArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    validate::validate_uuid("profile-id", &a.profile_id)?;

    let body = json!({
        "profileId": a.profile_id,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/waros/customer-summary");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/waros/customer-summary", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
