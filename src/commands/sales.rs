use crate::client::WaroClient;
use crate::output;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
pub struct SalesArgs {
    #[command(subcommand)]
    pub command: SalesCommands,
}

#[derive(Subcommand)]
pub enum SalesCommands {
    /// List sales with optional filters
    List(ListArgs),
    /// Get metrics / analytics for sales
    Metrics(MetricsArgs),
    /// Get full detail of a single sale
    Detail(DetailArgs),
}

// ── list ──────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ListArgs {
    /// Max results (1-250)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Filter by payment method: cash | card | digital
    #[arg(long)]
    payment_method: Option<String>,

    /// Filter by status: completed | cancelled | pending
    #[arg(long)]
    status: Option<String>,

    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Timezone (default: America/Bogota)
    #[arg(long, default_value = "America/Bogota")]
    timezone: String,

    /// Sort field
    #[arg(long, default_value = "order_date")]
    sort_field: String,

    /// Sort direction: asc | desc
    #[arg(long, default_value = "desc")]
    sort_direction: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── metrics ───────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct MetricsArgs {
    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Group by: date | weekday | hour | product | payment | ticket
    #[arg(long)]
    group_by: Option<String>,

    /// Timezone (default: America/Bogota)
    #[arg(long, default_value = "America/Bogota")]
    timezone: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── detail ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DetailArgs {
    /// Order UUID
    #[arg(long)]
    order_id: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── handlers ──────────────────────────────────────────────────────────────────

pub async fn run(
    args: SalesArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        SalesCommands::List(a) => list(a, client, format, fields).await,
        SalesCommands::Metrics(a) => metrics(a, client, format, fields).await,
        SalesCommands::Detail(a) => detail(a, client, format, fields).await,
    }
}

async fn list(
    a: ListArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let body = json!({
        "limit": a.limit,
        "offset": a.offset,
        "paymentMethod": a.payment_method,
        "status": a.status,
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "timezone": a.timezone,
        "sortField": a.sort_field,
        "sortDirection": a.sort_direction,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/sales", body).await?;
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn metrics(
    a: MetricsArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "groupBy": a.group_by,
        "timezone": a.timezone,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales/metrics");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/sales/metrics", body).await?;
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn detail(
    a: DetailArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let body = json!({ "orderId": a.order_id });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales/detail");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/sales/detail", body).await?;
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
