use crate::client::WaroClient;
use crate::output;
use crate::pagination;
use crate::spinner::Spinner;
use crate::validate;
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
    /// Max results per page (1-250)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset (ignored when --all is set)
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Fetch all pages automatically and output NDJSON
    #[arg(long)]
    all: bool,

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

    /// Top N products to return (1-100, only used with --group-by product)
    #[arg(long, default_value = "20")]
    limit: u32,

    /// Sort for product grouping: quantity | revenue (only used with --group-by product)
    #[arg(long, default_value = "quantity")]
    sort_by: String,

    /// Custom price bins for ticket distribution, comma-separated integers
    /// e.g. --ranges 0,5000,15000,30000 (only used with --group-by ticket)
    #[arg(long)]
    ranges: Option<String>,

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
    // Validate inputs before any API call
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }
    if let Some(ref v) = a.status {
        validate::validate_enum("status", v, &["completed", "cancelled", "pending"])?;
    }
    if let Some(ref v) = a.payment_method {
        validate::validate_enum("payment-method", v, &["cash", "card", "digital"])?;
    }
    validate::validate_enum("sort-direction", &a.sort_direction, &["asc", "desc"])?;

    // Filters shared by single-page and --all modes
    let filters = json!({
        "paymentMethod": a.payment_method,
        "status": a.status,
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "timezone": a.timezone,
        "sortField": a.sort_field,
        "sortDirection": a.sort_direction,
    });

    if a.dry_run {
        let suffix = if a.all {
            " (--all mode, showing first page)"
        } else {
            ""
        };
        let mut body = filters.clone();
        body["limit"] = json!(a.limit);
        body["offset"] = json!(if a.all { 0 } else { a.offset });
        println!("DRY RUN — POST /v1/sales{}", suffix);
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(client, "/v1/sales", filters, a.limit, fields.as_deref())
            .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);
    let sp = Spinner::start();
    let resp = client.post("/v1/sales", body).await?;
    sp.stop();
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
    // Validate inputs before any API call
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }
    if let Some(ref v) = a.group_by {
        validate::validate_enum(
            "group-by",
            v,
            &["date", "weekday", "hour", "product", "payment", "ticket"],
        )?;
    }
    validate::validate_enum("sort-by", &a.sort_by, &["quantity", "revenue"])?;

    // Parse --ranges into a JSON array of integers (only meaningful for group-by ticket)
    let ranges_value: serde_json::Value = match &a.ranges {
        None => serde_json::Value::Null,
        Some(s) => {
            let parts: Result<Vec<u64>, _> =
                s.split(',').map(|p| p.trim().parse::<u64>()).collect();
            match parts {
                Ok(v) => serde_json::json!(v),
                Err(_) => anyhow::bail!(
                    "--ranges must be comma-separated integers (e.g. 0,5000,15000), got: '{}'",
                    s
                ),
            }
        }
    };

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "groupBy": a.group_by,
        "timezone": a.timezone,
        "limit": a.limit,
        "sortBy": a.sort_by,
        "ranges": ranges_value,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales/metrics");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/sales/metrics", body).await?;
    sp.stop();
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
    // Validate inputs before any API call
    validate::validate_uuid("order-id", &a.order_id)?;

    let body = json!({ "orderId": a.order_id });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales/detail");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/sales/detail", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
