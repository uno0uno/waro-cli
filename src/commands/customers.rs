use crate::client::WaroClient;
use crate::output;
use crate::pagination;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
pub struct CustomersArgs {
    #[command(subcommand)]
    pub command: CustomersCommands,
}

#[derive(Subcommand)]
pub enum CustomersCommands {
    /// List customers with optional filters and pagination
    List(ListArgs),
    /// Get full profile, order history and waros summary for a customer
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

    /// Partial match on name or phone number
    #[arg(long)]
    search: Option<String>,

    /// Start date YYYY-MM-DD (scopes order aggregation)
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Timezone (default: America/Bogota)
    #[arg(long, default_value = "America/Bogota")]
    timezone: String,

    /// Sort field: total_spent | order_count | last_order_date | avg_ticket
    #[arg(long, default_value = "total_spent")]
    sort_field: String,

    /// Sort direction: asc | desc
    #[arg(long, default_value = "desc")]
    sort_direction: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── detail ─────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct DetailArgs {
    /// Customer UUID
    #[arg(long)]
    customer_id: String,

    /// Max orders to return (1-100)
    #[arg(long, default_value = "20")]
    limit: u32,

    /// Orders pagination offset
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Filter order history start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// Filter order history end date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Timezone (default: America/Bogota)
    #[arg(long, default_value = "America/Bogota")]
    timezone: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── handlers ──────────────────────────────────────────────────────────────────

pub async fn run(
    args: CustomersArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        CustomersCommands::List(a) => list(a, client, format, fields).await,
        CustomersCommands::Detail(a) => detail(a, client, format, fields).await,
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
    validate::validate_enum(
        "sort-field",
        &a.sort_field,
        &["total_spent", "order_count", "last_order_date", "avg_ticket"],
    )?;
    validate::validate_enum("sort-direction", &a.sort_direction, &["asc", "desc"])?;

    // Filters shared by single-page and --all modes
    let filters = json!({
        "search": a.search,
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
        println!("DRY RUN — POST /v1/customers{}", suffix);
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(client, "/v1/customers", filters, a.limit, fields.as_deref())
            .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);
    let sp = Spinner::start();
    let resp = client.post("/v1/customers", body).await?;
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
    validate::validate_uuid("customer-id", &a.customer_id)?;
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }

    let body = json!({
        "customerId": a.customer_id,
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "timezone": a.timezone,
        "limit": a.limit,
        "offset": a.offset,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/customers/detail");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/customers/detail", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
