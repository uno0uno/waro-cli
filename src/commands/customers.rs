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
    /// Get aggregate customer analytics with optional time-series grouping
    Metrics(MetricsArgs),
    /// Paginated order history for a specific customer
    Orders(OrdersArgs),
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

    /// Sort field: total_spent | order_count | last_order_date | avg_ticket | waros_balance
    #[arg(long, default_value = "total_spent")]
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

    /// Group by: date | weekday | month — enables time series in response
    #[arg(long)]
    group_by: Option<String>,

    /// Timezone (default: America/Bogota)
    #[arg(long, default_value = "America/Bogota")]
    timezone: String,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── orders ─────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct OrdersArgs {
    /// Customer UUID (required)
    #[arg(long)]
    customer_id: String,

    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Max orders per page (1-200)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset (ignored when --all is set)
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Fetch all pages automatically and output NDJSON
    #[arg(long)]
    all: bool,

    /// Include line items per order in output
    #[arg(long)]
    include_items: bool,

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
        CustomersCommands::Metrics(a) => metrics(a, client, format, fields).await,
        CustomersCommands::Orders(a) => orders(a, client, format, fields).await,
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
        &[
            "total_spent",
            "order_count",
            "last_order_date",
            "avg_ticket",
            "waros_balance",
        ],
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

async fn metrics(
    a: MetricsArgs,
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
    if let Some(ref v) = a.group_by {
        validate::validate_enum("group-by", v, &["date", "weekday", "month"])?;
    }

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "groupBy": a.group_by,
        "timezone": a.timezone,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/customers/metrics");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/customers/metrics", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn orders(
    a: OrdersArgs,
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

    let filters = json!({
        "customerId": a.customer_id,
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "includeItems": a.include_items,
    });

    if a.dry_run {
        let mut body = filters.clone();
        body["limit"] = json!(a.limit);
        body["offset"] = json!(if a.all { 0 } else { a.offset });
        println!("DRY RUN — POST /v1/customers/orders");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(
            client,
            "/v1/customers/orders",
            filters,
            a.limit,
            fields.as_deref(),
        )
        .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);

    let sp = Spinner::start();
    let resp = client.post("/v1/customers/orders", body).await?;
    sp.stop();

    if format == "table" {
        print_orders_table(&resp, a.include_items)?;
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

fn print_orders_table(value: &serde_json::Value, include_items: bool) -> Result<()> {
    let orders = match value.get("items").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => {
            println!("(no orders found)");
            return Ok(());
        }
    };

    // Column widths
    let date_w = 16usize;
    let num_w = 7usize;
    let total_w = 11usize;
    let pay_w = 9usize;

    use colored::Colorize;

    // Header
    println!(
        "{:<date_w$}  {:>num_w$}  {:>total_w$}  {:<pay_w$}  {}",
        "DATE".bold(),
        "#".bold(),
        "TOTAL".bold(),
        "PAYMENT".bold(),
        "ITEMS".bold(),
    );
    println!(
        "{}",
        "─".repeat(date_w + 2 + num_w + 2 + total_w + 2 + pay_w + 2 + 30)
    );

    for order in orders {
        // DATE: "2026-04-02T03:23:..." → "2026-04-02 03:23"
        let date = order
            .get("date")
            .and_then(|v| v.as_str())
            .map(|s| {
                let s = if s.len() >= 16 { &s[..16] } else { s };
                s.replace('T', " ")
            })
            .unwrap_or_default();

        let order_num = order
            .get("order_number")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);

        let total = order
            .get("total")
            .and_then(|v| v.as_f64())
            .map(|f| format!("${}", f as u64))
            .unwrap_or_default();

        let payment = order
            .get("payment_method")
            .and_then(|v| v.as_str())
            .unwrap_or("-");

        // ITEMS column: join product names if available, otherwise show count
        let items_cell = if include_items {
            if let Some(items) = order.get("items").and_then(|v| v.as_array()) {
                let joined: String = items
                    .iter()
                    .filter_map(|item| {
                        let name = item.get("product_name").and_then(|v| v.as_str())?;
                        let qty = item.get("quantity").and_then(|v| v.as_f64()).unwrap_or(1.0);
                        Some(format!("{} x{}", name, qty as u32))
                    })
                    .collect::<Vec<_>>()
                    .join(", ");
                // Truncate at 60 chars
                if joined.len() > 60 {
                    format!("{}…", &joined[..59])
                } else {
                    joined
                }
            } else {
                "-".to_string()
            }
        } else {
            let count = order
                .get("items_count")
                .and_then(|v| v.as_u64())
                .unwrap_or(0);
            format!("{} items", count)
        };

        println!(
            "{:<date_w$}  {:>num_w$}  {:>total_w$}  {:<pay_w$}  {}",
            date, order_num, total, payment, items_cell,
        );
    }

    // Footer: show total count
    if let Some(total) = value.get("total").and_then(|v| v.as_u64()) {
        let shown = orders.len();
        let offset = value.get("offset").and_then(|v| v.as_u64()).unwrap_or(0);
        println!();
        println!(
            "Showing {}-{} of {} orders",
            offset + 1,
            offset + shown as u64,
            total
        );
    }

    Ok(())
}
