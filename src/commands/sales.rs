use crate::client::WaroClient;
use crate::compare;
use crate::output;
use crate::pagination;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use colored::Colorize;
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

    /// Compare current period to: previous-period | previous-year | YYYY-MM-DD:YYYY-MM-DD
    #[arg(long)]
    compare_to: Option<String>,

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

    // Parse optional --compare-to
    let (compare_to_val, compare_from_val, compare_date_to_val) = if let Some(ref ct) = a.compare_to
    {
        let (mode, from, to) = compare::parse_compare_to(ct)?;
        (
            serde_json::Value::String(mode),
            from.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
            to.map(serde_json::Value::String)
                .unwrap_or(serde_json::Value::Null),
        )
    } else {
        (
            serde_json::Value::Null,
            serde_json::Value::Null,
            serde_json::Value::Null,
        )
    };

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "groupBy": a.group_by,
        "timezone": a.timezone,
        "limit": a.limit,
        "sortBy": a.sort_by,
        "ranges": ranges_value,
        "compareTo": compare_to_val,
        "compareFrom": compare_from_val,
        "compareDateTo": compare_date_to_val,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/sales/metrics");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/sales/metrics", body).await?;
    sp.stop();

    if format == "table" && a.compare_to.is_some() {
        print_sales_comparison(&resp)?;
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

fn print_sales_comparison(value: &serde_json::Value) -> Result<()> {
    let data = match value.get("data") {
        Some(d) => d,
        None => {
            output::print(value, "json")?;
            return Ok(());
        }
    };

    let prev = data
        .get("comparison")
        .and_then(|c| c.as_object())
        .and_then(|m| {
            // API returns the period key dynamically; grab the first value
            m.values().next()
        });

    let get_f64 = |obj: &serde_json::Value, key: &str| -> Option<f64> {
        obj.get(key).and_then(|v| v.as_f64())
    };

    let cur_sales = get_f64(data, "totalSales");
    let cur_orders = get_f64(data, "totalOrders");
    let cur_ticket = get_f64(data, "avgTicket");

    let prev_sales = prev.and_then(|p| get_f64(p, "totalSales"));
    let prev_orders = prev.and_then(|p| get_f64(p, "totalOrders"));
    let prev_ticket = prev.and_then(|p| get_f64(p, "avgTicket"));

    let sales_pct = get_f64(data, "totalSales_change_pct");
    let ticket_pct = get_f64(data, "avgTicket_change_pct");
    // Compute orders delta client-side if not in response
    let orders_pct =
        get_f64(data, "totalOrders_change_pct").or_else(|| match (cur_orders, prev_orders) {
            (Some(c), Some(p)) if p != 0.0 => Some((c - p) / p * 100.0),
            _ => None,
        });

    let fmt_cop = |v: Option<f64>| -> String {
        v.map(|f| format!("${}", f as i64))
            .unwrap_or_else(|| "-".to_string())
    };
    let fmt_int = |v: Option<f64>| -> String {
        v.map(|f| format!("{}", f as i64))
            .unwrap_or_else(|| "-".to_string())
    };

    let lbl_w = 20usize;
    let col_w = 14usize;

    println!(
        "{:<lbl_w$}  {:>col_w$}  {:>col_w$}  {}",
        "METRIC".bold(),
        "CURRENT".bold(),
        "PREVIOUS".bold(),
        "CHANGE".bold(),
    );
    println!("{}", "─".repeat(lbl_w + 2 + col_w + 2 + col_w + 2 + 12));

    let rows: &[(&str, String, String, String)] = &[
        (
            "Total Sales",
            fmt_cop(cur_sales),
            fmt_cop(prev_sales),
            compare::format_delta(sales_pct, false, false),
        ),
        (
            "Total Orders",
            fmt_int(cur_orders),
            fmt_int(prev_orders),
            compare::format_delta(orders_pct, false, false),
        ),
        (
            "Avg Ticket",
            fmt_cop(cur_ticket),
            fmt_cop(prev_ticket),
            compare::format_delta(ticket_pct, false, false),
        ),
    ];

    for (label, cur, prev, delta) in rows {
        println!(
            "{:<lbl_w$}  {:>col_w$}  {:>col_w$}  {}",
            label, cur, prev, delta
        );
    }

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
