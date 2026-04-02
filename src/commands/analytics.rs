use crate::client::WaroClient;
use crate::output;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use chrono::{Datelike, NaiveDate};
use clap::{Args, Subcommand};
use colored::Colorize;
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
    /// Customer retention cohort matrix
    Cohort(CohortArgs),
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

// ── cohort ────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct CohortArgs {
    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Cohort period: weekly (default) | monthly
    #[arg(long, default_value = "weekly")]
    period: String,

    /// Number of retention periods to show (1-52)
    #[arg(long, default_value = "8")]
    periods: u32,

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
        AnalyticsCommands::Cohort(a) => cohort(a, client, format, fields).await,
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

async fn cohort(
    a: CohortArgs,
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
    validate::validate_enum("period", &a.period, &["weekly", "monthly"])?;

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "period": a.period,
        "periods": a.periods,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/cohort");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/cohort", body).await?;
    sp.stop();

    if format == "table" {
        print_cohort_table(&resp, &a.period)?;
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

/// Color-code a retention percentage cell.
/// red < 20%, yellow 20–35%, green > 35%. Dash for not-yet-elapsed periods.
fn color_pct(pct: f64) -> String {
    let s = format!("{:.1}%", pct);
    if pct > 35.0 {
        s.green().to_string()
    } else if pct >= 20.0 {
        s.yellow().to_string()
    } else {
        s.red().to_string()
    }
}

/// Returns true if the Nth period has fully elapsed for a cohort starting on cohort_date.
/// For weekly: period N elapses when today >= cohort_date + N*7 days.
/// For monthly: period N elapses when today >= cohort_date + N months.
fn period_elapsed(cohort_date: &NaiveDate, n: u32, period: &str) -> bool {
    let today = chrono::Local::now().date_naive();
    let threshold = if period == "monthly" {
        // Add N months: month() is 1-indexed; convert to 0-indexed for arithmetic.
        let total_months = (cohort_date.month() - 1) + n;
        let years = cohort_date.year() + (total_months / 12) as i32;
        let month = (total_months % 12) + 1;
        let day = cohort_date.day().min(days_in_month(years, month));
        NaiveDate::from_ymd_opt(years, month, day).unwrap_or(*cohort_date)
    } else {
        // weekly: add N*7 days
        *cohort_date + chrono::Duration::days((n * 7) as i64)
    };
    today >= threshold
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let next = if month == 12 {
        NaiveDate::from_ymd_opt(year + 1, 1, 1)
    } else {
        NaiveDate::from_ymd_opt(year, month + 1, 1)
    };
    next.map(|d| (d - chrono::Duration::days(1)).day())
        .unwrap_or(28)
}

fn print_cohort_table(value: &serde_json::Value, period: &str) -> Result<()> {
    let cohorts = match value.get("cohorts").and_then(|v| v.as_array()) {
        Some(c) if !c.is_empty() => c,
        _ => {
            println!("(no cohort data)");
            return Ok(());
        }
    };

    // Determine number of period columns from the first cohort's retention array.
    let n_periods = cohorts[0]
        .get("retention")
        .and_then(|r| r.as_array())
        .map(|r| r.len())
        .unwrap_or(0);

    // Column widths: COHORT=12, SIZE=6, each +N=8
    let cohort_w = 12usize;
    let size_w = 6usize;
    let pct_w = 8usize;

    // Header
    let mut header = format!("{:<cohort_w$}  {:>size_w$}", "COHORT", "SIZE");
    for i in 1..=n_periods {
        header.push_str(&format!("  {:>pct_w$}", format!("+{}", i)));
    }
    println!("{}", header.bold());
    println!("{}", "─".repeat(cohort_w + 2 + size_w + (n_periods * (pct_w + 2))));

    for cohort in cohorts {
        let label = cohort
            .get("cohort_label")
            .and_then(|v| v.as_str())
            .unwrap_or("?");
        let cohort_date_str = cohort
            .get("cohort_date")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let size = cohort
            .get("cohort_size")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        let retention = cohort
            .get("retention")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        let cohort_date = NaiveDate::parse_from_str(cohort_date_str, "%Y-%m-%d").ok();

        let mut row = format!("{:<cohort_w$}  {:>size_w$}", label, size);
        for i in 1..=n_periods {
            let cell = if let Some(ref cd) = cohort_date {
                if !period_elapsed(cd, i as u32, period) {
                    // Period hasn't elapsed yet
                    "  —".dimmed().to_string()
                } else if let Some(entry) = retention.get(i - 1) {
                    let pct = entry.get("pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    color_pct(pct)
                } else {
                    "  —".dimmed().to_string()
                }
            } else {
                // Can't parse date, show raw value
                if let Some(entry) = retention.get(i - 1) {
                    let pct = entry.get("pct").and_then(|v| v.as_f64()).unwrap_or(0.0);
                    color_pct(pct)
                } else {
                    "  —".dimmed().to_string()
                }
            };
            // Right-align within pct_w (accounting for ANSI escape codes not counting toward width)
            row.push_str(&format!("  {:>pct_w$}", cell));
        }
        println!("{}", row);
    }

    Ok(())
}
