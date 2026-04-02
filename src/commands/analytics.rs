use crate::client::WaroClient;
use crate::compare;
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
    /// WaRos loyalty program analytics — totals and optional grouping by day/week/customer
    WarosAnalytics(WarosAnalyticsArgs),
    /// RFM customer segmentation — Champions, Loyal, At Risk, Hibernating, Lost
    Rfm(RfmArgs),
    /// Customers at churn risk — silence exceeds N × their personal avg visit interval
    ChurnRisk(ChurnRiskArgs),
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

    /// Compare current period to: previous-period | previous-year | YYYY-MM-DD:YYYY-MM-DD
    #[arg(long)]
    compare_to: Option<String>,

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

// ── waros-analytics ───────────────────────────────────────────────────────────

#[derive(Args)]
pub struct WarosAnalyticsArgs {
    /// Start date YYYY-MM-DD
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD
    #[arg(long)]
    date_to: Option<String>,

    /// Group by: day | week | customer (omit for summary only)
    #[arg(long)]
    group_by: Option<String>,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── rfm ───────────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct RfmArgs {
    /// Start date YYYY-MM-DD (evaluation window)
    #[arg(long)]
    date_from: Option<String>,

    /// End date YYYY-MM-DD (evaluation window)
    #[arg(long)]
    date_to: Option<String>,

    /// Filter output to one segment: champions | loyal | at-risk | hibernating | lost
    #[arg(long)]
    segment: Option<String>,

    /// Quintile count for scoring (2-10, default 5)
    #[arg(long, default_value = "5")]
    segments: u32,

    /// Expand each segment with individual customer rows (name, scores, spent, last order)
    #[arg(long)]
    show_customers: bool,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── churn-risk ────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ChurnRiskArgs {
    /// Minimum lifetime orders to include a customer (default: 3)
    #[arg(long, default_value = "3")]
    min_orders: u32,

    /// Flag when silence exceeds N × avg visit interval (default: 2.0)
    #[arg(long, default_value = "2.0")]
    multiplier: f64,

    /// Max customers to return (default: 20)
    #[arg(long, default_value = "20")]
    limit: u32,

    /// Filter by risk level: high | medium (omit for all)
    #[arg(long)]
    risk: Option<String>,

    /// Include phone number column (opt-in — contains PII)
    #[arg(long)]
    show_contact: bool,

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
        AnalyticsCommands::WarosAnalytics(a) => waros_analytics(a, client, format, fields).await,
        AnalyticsCommands::Rfm(a) => rfm(a, client, format, fields).await,
        AnalyticsCommands::ChurnRisk(a) => churn_risk(a, client, format, fields).await,
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
        "compareTo": compare_to_val,
        "compareFrom": compare_from_val,
        "compareDateTo": compare_date_to_val,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/food-cost");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/food-cost", body).await?;
    sp.stop();

    if format == "table" && a.compare_to.is_some() {
        print_food_cost_comparison(&resp)?;
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

fn print_food_cost_comparison(value: &serde_json::Value) -> Result<()> {
    let cur = match value.get("current_period") {
        Some(c) => c,
        None => {
            output::print(value, "json")?;
            return Ok(());
        }
    };
    let prev = value.get("previous_period");
    let cmp = value.get("comparison");

    let get_f64 = |obj: &serde_json::Value, key: &str| -> Option<f64> {
        obj.get(key).and_then(|v| v.as_f64())
    };

    let cur_pct = get_f64(cur, "food_cost_pct");
    let cur_revenue = get_f64(cur, "revenue");
    let cur_cost = get_f64(cur, "total_cost");

    let prev_pct = prev.and_then(|p| get_f64(p, "food_cost_pct"));
    let prev_revenue = prev.and_then(|p| get_f64(p, "revenue"));
    let prev_cost = prev.and_then(|p| get_f64(p, "total_cost"));

    // food_cost_pct change_pct from comparison object
    let pct_delta =
        cmp.and_then(|c| get_f64(c, "change_pct"))
            .or_else(|| match (cur_pct, prev_pct) {
                (Some(c), Some(p)) if p != 0.0 => Some((c - p) / p * 100.0),
                _ => None,
            });
    let revenue_delta = match (cur_revenue, prev_revenue) {
        (Some(c), Some(p)) if p != 0.0 => Some((c - p) / p * 100.0),
        _ => None,
    };
    let cost_delta = match (cur_cost, prev_cost) {
        (Some(c), Some(p)) if p != 0.0 => Some((c - p) / p * 100.0),
        _ => None,
    };

    let fmt_cop = |v: Option<f64>| -> String {
        v.map(|f| format!("${}", f as i64))
            .unwrap_or_else(|| "-".to_string())
    };
    let fmt_pct = |v: Option<f64>| -> String {
        v.map(|f| format!("{:.1}%", f))
            .unwrap_or_else(|| "-".to_string())
    };

    let lbl_w = 16usize;
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
            "Revenue",
            fmt_cop(cur_revenue),
            fmt_cop(prev_revenue),
            compare::format_delta(revenue_delta, false, false),
        ),
        (
            "Total Cost",
            fmt_cop(cur_cost),
            fmt_cop(prev_cost),
            compare::format_delta(cost_delta, false, false),
        ),
        (
            "Food Cost %",
            fmt_pct(cur_pct),
            fmt_pct(prev_pct),
            compare::format_delta(pct_delta, true, true),
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
    println!(
        "{}",
        "─".repeat(cohort_w + 2 + size_w + (n_periods * (pct_w + 2)))
    );

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

async fn waros_analytics(
    a: WarosAnalyticsArgs,
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
        validate::validate_enum("group-by", v, &["day", "week", "customer"])?;
    }

    // API requires groupBy — default to "day" when not specified
    let group_by = a.group_by.clone().unwrap_or_else(|| "day".to_string());

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "groupBy": group_by,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/waros");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/waros", body).await?;
    sp.stop();

    if format == "table" {
        let has_groups = resp
            .get("groups")
            .and_then(|v| v.as_array())
            .map(|arr| !arr.is_empty())
            .unwrap_or(false);
        print_waros_summary(&resp)?;
        if has_groups {
            println!();
            print_waros_groups(&resp, &group_by)?;
        }
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

/// Color-code redemption rate: green ≥30%, yellow 10–29%, red <10%.
fn color_rate(pct: f64) -> String {
    let s = format!("{:.1}%", pct);
    if pct >= 30.0 {
        s.green().to_string()
    } else if pct >= 10.0 {
        s.yellow().to_string()
    } else {
        s.red().to_string()
    }
}

fn print_waros_summary(value: &serde_json::Value) -> Result<()> {
    let summary = match value.get("summary") {
        Some(s) => s,
        None => {
            println!("(no summary data)");
            return Ok(());
        }
    };

    let issued = summary
        .get("total_issued")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let redeemed = summary
        .get("total_redeemed")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);
    let rate = summary
        .get("redemption_rate_pct")
        .and_then(|v| v.as_f64())
        .unwrap_or(0.0);
    let members = summary
        .get("active_members")
        .and_then(|v| v.as_i64())
        .unwrap_or(0);

    println!("{}", "── WaRos Summary ─────────────────".bold());
    println!("  {:<20} {:>10}", "Issued".bold(), issued);
    println!("  {:<20} {:>10}", "Redeemed".bold(), redeemed);
    println!(
        "  {:<20} {:>10}",
        "Redemption rate".bold(),
        color_rate(rate)
    );
    println!("  {:<20} {:>10}", "Active members".bold(), members);

    Ok(())
}

fn print_waros_groups(value: &serde_json::Value, group_by: &str) -> Result<()> {
    let groups = match value.get("groups").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => {
            println!("(no group data)");
            return Ok(());
        }
    };

    if group_by == "customer" {
        // NAME | EARNED | REDEEMED | TXS  (no phone — no PII)
        let name_w = 28usize;
        let num_w = 10usize;
        println!(
            "{:<name_w$}  {:>num_w$}  {:>num_w$}  {:>6}",
            "NAME".bold(),
            "EARNED".bold(),
            "REDEEMED".bold(),
            "TXS".bold(),
        );
        println!("{}", "─".repeat(name_w + 2 + num_w + 2 + num_w + 2 + 6));
        for row in groups {
            let name = row.get("name").and_then(|v| v.as_str()).unwrap_or("-");
            let name_trunc = if name.len() > name_w {
                format!("{}…", &name[..name_w - 1])
            } else {
                name.to_string()
            };
            let earned = row
                .get("total_earned")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let redeemed = row
                .get("total_redeemed")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let txs = row
                .get("transaction_count")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            println!(
                "{:<name_w$}  {:>num_w$}  {:>num_w$}  {:>6}",
                name_trunc, earned, redeemed, txs,
            );
        }
    } else {
        // PERIOD | EARNED | REDEEMED | MEMBERS
        let period_w = 14usize;
        let num_w = 10usize;
        println!(
            "{:<period_w$}  {:>num_w$}  {:>num_w$}  {:>8}",
            "PERIOD".bold(),
            "EARNED".bold(),
            "REDEEMED".bold(),
            "MEMBERS".bold(),
        );
        println!("{}", "─".repeat(period_w + 2 + num_w + 2 + num_w + 2 + 8));
        for row in groups {
            let period = row.get("period").and_then(|v| v.as_str()).unwrap_or("-");
            let period_trunc = if period.len() > period_w {
                format!("{}…", &period[..period_w - 1])
            } else {
                period.to_string()
            };
            let earned = row
                .get("total_earned")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let redeemed = row
                .get("total_redeemed")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            let members = row
                .get("active_members")
                .and_then(|v| v.as_i64())
                .unwrap_or(0);
            println!(
                "{:<period_w$}  {:>num_w$}  {:>num_w$}  {:>8}",
                period_trunc, earned, redeemed, members,
            );
        }
    }

    Ok(())
}

// ── Segment name helpers ───────────────────────────────────────────────────────

/// Map CLI kebab-case segment flag to the API's title-case spelling.
fn normalize_segment(s: &str) -> Option<&'static str> {
    match s.to_lowercase().as_str() {
        "champions" => Some("Champions"),
        "loyal" => Some("Loyal"),
        "at-risk" | "at_risk" => Some("At Risk"),
        "hibernating" => Some("Hibernating"),
        "lost" => Some("Lost"),
        _ => None,
    }
}

/// Fixed display order for segment summary rows.
const SEGMENT_ORDER: &[&str] = &["Champions", "Loyal", "At Risk", "Hibernating", "Lost"];

async fn rfm(a: RfmArgs, client: &WaroClient, format: &str, fields: Option<String>) -> Result<()> {
    if let Some(ref v) = a.date_from {
        validate::validate_date("date-from", v)?;
    }
    if let Some(ref v) = a.date_to {
        validate::validate_date("date-to", v)?;
    }

    // Validate and normalize --segment flag
    let segment_filter: Option<&'static str> = if let Some(ref s) = a.segment {
        match normalize_segment(s) {
            Some(canonical) => Some(canonical),
            None => {
                anyhow::bail!(
                    "Invalid segment '{}'. Valid values: champions, loyal, at-risk, hibernating, lost",
                    s
                );
            }
        }
    } else {
        None
    };

    let body = json!({
        "dateFrom": a.date_from,
        "dateTo": a.date_to,
        "segments": a.segments,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/rfm");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/rfm", body).await?;
    sp.stop();

    if format == "table" {
        let data = match resp.get("data") {
            Some(d) => d,
            None => {
                println!("(no data)");
                return Ok(());
            }
        };
        let customers: &[serde_json::Value] = data
            .get("customers")
            .and_then(|v| v.as_array())
            .map(|v| v.as_slice())
            .unwrap_or(&[]);

        // Apply client-side segment filter
        let filtered: Vec<&serde_json::Value> = customers
            .iter()
            .filter(|c| {
                if let Some(seg) = segment_filter {
                    c.get("segment").and_then(|v| v.as_str()) == Some(seg)
                } else {
                    true
                }
            })
            .collect();

        if filtered.is_empty() {
            println!("(no customers found in this period)");
            return Ok(());
        }

        let evaluated_to = data
            .get("evaluated_to")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let evaluated_from = data
            .get("evaluated_from")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let segments_used = data
            .get("segments_used")
            .and_then(|v| v.as_u64())
            .unwrap_or(5);

        if a.show_customers {
            print_rfm_customers(&filtered, evaluated_to)?;
        } else {
            print_rfm_summary(&filtered, evaluated_to)?;
        }

        println!();
        println!(
            "Evaluated: {} → {}  ({} customers, {} quintiles)",
            evaluated_from,
            evaluated_to,
            filtered.len(),
            segments_used
        );
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

fn print_rfm_summary(customers: &[&serde_json::Value], evaluated_to: &str) -> Result<()> {
    // Parse evaluated_to date for recency calculation
    let eval_date = NaiveDate::parse_from_str(evaluated_to, "%Y-%m-%d").ok();

    // Aggregate per segment
    struct SegStats {
        count: usize,
        total_ticket: f64,
        total_orders: f64,
        total_recency_days: f64,
    }

    use std::collections::HashMap;
    let mut stats: HashMap<&str, SegStats> = HashMap::new();

    for c in customers {
        let seg = c.get("segment").and_then(|v| v.as_str()).unwrap_or("Lost");
        let order_count = c
            .get("order_count")
            .and_then(|v| v.as_f64())
            .unwrap_or(1.0)
            .max(1.0);
        let total_spent = c.get("total_spent").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let avg_ticket = total_spent / order_count;

        // Compute recency_days from last_order_date vs evaluated_to
        let recency_days = if let Some(ed) = eval_date {
            let last_str = c
                .get("last_order_date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            // last_order_date is "YYYY-MM-DDTHH:MM:SS"
            let last_date =
                NaiveDate::parse_from_str(&last_str[..last_str.len().min(10)], "%Y-%m-%d").ok();
            last_date
                .map(|ld| ed.signed_duration_since(ld).num_days() as f64)
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let entry = stats.entry(seg).or_insert(SegStats {
            count: 0,
            total_ticket: 0.0,
            total_orders: 0.0,
            total_recency_days: 0.0,
        });
        entry.count += 1;
        entry.total_ticket += avg_ticket;
        entry.total_orders += order_count;
        entry.total_recency_days += recency_days;
    }

    let seg_w = 14usize;
    let cnt_w = 7usize;
    let ticket_w = 12usize;
    let orders_w = 11usize;
    let days_w = 9usize;

    println!(
        "{:<seg_w$}  {:>cnt_w$}  {:>ticket_w$}  {:>orders_w$}  {:>days_w$}",
        "SEGMENT".bold(),
        "COUNT".bold(),
        "AVG TICKET".bold(),
        "AVG ORDERS".bold(),
        "AVG DAYS".bold(),
    );
    println!(
        "{}",
        "─".repeat(seg_w + 2 + cnt_w + 2 + ticket_w + 2 + orders_w + 2 + days_w)
    );

    for &seg in SEGMENT_ORDER {
        if let Some(s) = stats.get(seg) {
            let avg_ticket = s.total_ticket / s.count as f64;
            let avg_orders = s.total_orders / s.count as f64;
            let avg_days = s.total_recency_days / s.count as f64;
            println!(
                "{:<seg_w$}  {:>cnt_w$}  {:>ticket_w$}  {:>orders_w$.1}  {:>days_w$.0}",
                seg,
                s.count,
                format!("${}", avg_ticket as i64),
                avg_orders,
                avg_days,
            );
        }
    }

    Ok(())
}

fn print_rfm_customers(customers: &[&serde_json::Value], evaluated_to: &str) -> Result<()> {
    let eval_date = NaiveDate::parse_from_str(evaluated_to, "%Y-%m-%d").ok();

    let seg_w = 14usize;
    let name_w = 24usize;
    let score_w = 3usize;
    let orders_w = 7usize;
    let spent_w = 10usize;
    let date_w = 11usize;

    println!(
        "{:<seg_w$}  {:<name_w$}  {:>score_w$}  {:>score_w$}  {:>score_w$}  {:>orders_w$}  {:>spent_w$}  {:<date_w$}",
        "SEGMENT".bold(),
        "NAME".bold(),
        "R".bold(),
        "F".bold(),
        "M".bold(),
        "ORDERS".bold(),
        "SPENT".bold(),
        "LAST ORDER".bold(),
    );
    println!(
        "{}",
        "─".repeat(
            seg_w
                + 2
                + name_w
                + 2
                + score_w
                + 2
                + score_w
                + 2
                + score_w
                + 2
                + orders_w
                + 2
                + spent_w
                + 2
                + date_w
        )
    );

    // Group by segment order
    for &seg in SEGMENT_ORDER {
        for c in customers
            .iter()
            .filter(|c| c.get("segment").and_then(|v| v.as_str()) == Some(seg))
        {
            let name = c
                .get("customer_name")
                .and_then(|v| v.as_str())
                .unwrap_or("-");
            let name_trunc = if name.len() > name_w {
                format!("{}…", &name[..name_w - 1])
            } else {
                name.to_string()
            };

            let r = c.get("r_score").and_then(|v| v.as_i64()).unwrap_or(0);
            let f = c.get("f_score").and_then(|v| v.as_i64()).unwrap_or(0);
            let m = c.get("m_score").and_then(|v| v.as_i64()).unwrap_or(0);
            let orders = c.get("order_count").and_then(|v| v.as_i64()).unwrap_or(0);
            let spent = c.get("total_spent").and_then(|v| v.as_i64()).unwrap_or(0);

            let last_str = c
                .get("last_order_date")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let last_display = &last_str[..last_str.len().min(10)];

            // Recency highlighting: green if recent (≤7d), yellow (≤30d), red (>30d)
            let _ = eval_date; // used for summary; in customer view we just show the date

            println!(
                "{:<seg_w$}  {:<name_w$}  {:>score_w$}  {:>score_w$}  {:>score_w$}  {:>orders_w$}  {:>spent_w$}  {:<date_w$}",
                seg,
                name_trunc,
                r, f, m,
                orders,
                format!("${}", spent),
                last_display,
            );
        }
    }

    Ok(())
}

async fn churn_risk(
    a: ChurnRiskArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    if let Some(ref v) = a.risk {
        validate::validate_enum("risk", v, &["high", "medium"])?;
    }

    let body = json!({
        "thresholdMultiplier": a.multiplier,
        "minOrders": a.min_orders,
        "limit": a.limit,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/analytics/churn-risk");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/analytics/churn-risk", body).await?;
    sp.stop();

    if format == "table" {
        print_churn_risk_table(
            &resp,
            a.show_contact,
            a.risk.as_deref(),
            a.multiplier,
            a.min_orders,
        )?;
    } else {
        let resp = output::apply_fields(resp, fields.as_deref());
        output::print(&resp, format)?;
    }
    Ok(())
}

/// Derive HIGH/MEDIUM risk label from risk_score.
/// All API-returned customers have already exceeded the threshold.
/// score >= 0.5 → HIGH (at 2× their avg interval), < 0.5 → MEDIUM
fn risk_label(score: f64) -> &'static str {
    if score >= 0.5 {
        "HIGH"
    } else {
        "MEDIUM"
    }
}

fn color_risk(score: f64) -> String {
    if score >= 0.5 {
        "🔴 HIGH".red().bold().to_string()
    } else {
        "🟡 MEDIUM".yellow().to_string()
    }
}

/// Format a COP value with thousands comma separator: 4052000 → "$4,052,000"
fn format_ltv(v: f64) -> String {
    let n = v as i64;
    let s = n.to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3 + 1);
    for (i, &b) in bytes.iter().enumerate() {
        if i > 0 && (bytes.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(b as char);
    }
    format!("${}", out)
}

fn print_churn_risk_table(
    value: &serde_json::Value,
    show_contact: bool,
    risk_filter: Option<&str>,
    multiplier: f64,
    min_orders: u32,
) -> Result<()> {
    let customers = match value.get("customers").and_then(|v| v.as_array()) {
        Some(arr) if !arr.is_empty() => arr,
        _ => {
            println!("(no customers at churn risk)");
            return Ok(());
        }
    };

    // Apply client-side risk filter
    let filtered: Vec<&serde_json::Value> = customers
        .iter()
        .filter(|c| {
            if let Some(f) = risk_filter {
                let score = c.get("risk_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
                risk_label(score).to_lowercase() == f
            } else {
                true
            }
        })
        .collect();

    if filtered.is_empty() {
        println!("(no customers match the risk filter)");
        return Ok(());
    }

    let name_w = 20usize;
    let phone_w = 14usize;
    let orders_w = 6usize;
    let ltv_w = 12usize;
    let interval_w = 13usize;
    let silent_w = 11usize;
    let risk_w = 10usize;

    if show_contact {
        println!(
            "{:<name_w$}  {:<phone_w$}  {:>orders_w$}  {:>ltv_w$}  {:>interval_w$}  {:>silent_w$}  {}",
            "CUSTOMER".bold(),
            "PHONE".bold(),
            "ORDERS".bold(),
            "LTV".bold(),
            "AVG INTERVAL".bold(),
            "DAYS SILENT".bold(),
            "RISK".bold(),
        );
        println!(
            "{}",
            "─".repeat(
                name_w
                    + 2
                    + phone_w
                    + 2
                    + orders_w
                    + 2
                    + ltv_w
                    + 2
                    + interval_w
                    + 2
                    + silent_w
                    + 2
                    + risk_w
            )
        );
    } else {
        println!(
            "{:<name_w$}  {:>orders_w$}  {:>ltv_w$}  {:>interval_w$}  {:>silent_w$}  {}",
            "CUSTOMER".bold(),
            "ORDERS".bold(),
            "LTV".bold(),
            "AVG INTERVAL".bold(),
            "DAYS SILENT".bold(),
            "RISK".bold(),
        );
        println!(
            "{}",
            "─".repeat(
                name_w + 2 + orders_w + 2 + ltv_w + 2 + interval_w + 2 + silent_w + 2 + risk_w
            )
        );
    }

    for c in &filtered {
        let name = c.get("name").and_then(|v| v.as_str()).unwrap_or("-");
        let name_trunc = if name.chars().count() > name_w {
            let truncated: String = name.chars().take(name_w - 1).collect();
            format!("{}…", truncated)
        } else {
            name.to_string()
        };

        let phone = c.get("phone").and_then(|v| v.as_str()).unwrap_or("-");

        let orders = c
            .get("order_count")
            .or_else(|| c.get("orders"))
            .and_then(|v| v.as_i64())
            .unwrap_or(0);

        let ltv = c
            .get("lifetime_value")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

        let avg_interval = c
            .get("avg_visit_interval_days")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);
        let interval_str = format!("{:.1} days", avg_interval);

        let days_silent = c
            .get("days_since_last_order")
            .and_then(|v| v.as_i64())
            .unwrap_or(0);
        let silent_str = format!("{} days", days_silent);

        let score = c.get("risk_score").and_then(|v| v.as_f64()).unwrap_or(0.0);
        let risk_col = color_risk(score);

        if show_contact {
            println!(
                "{:<name_w$}  {:<phone_w$}  {:>orders_w$}  {:>ltv_w$}  {:>interval_w$}  {:>silent_w$}  {}",
                name_trunc,
                phone,
                orders,
                format_ltv(ltv),
                interval_str,
                silent_str,
                risk_col,
            );
        } else {
            println!(
                "{:<name_w$}  {:>orders_w$}  {:>ltv_w$}  {:>interval_w$}  {:>silent_w$}  {}",
                name_trunc,
                orders,
                format_ltv(ltv),
                interval_str,
                silent_str,
                risk_col,
            );
        }
    }

    let total = value
        .get("total_count")
        .and_then(|v| v.as_u64())
        .unwrap_or(filtered.len() as u64);
    println!();
    println!(
        "Showing {} of {} at-risk customers  (multiplier: {:.1}×, min orders: {})",
        filtered.len(),
        total,
        multiplier,
        min_orders,
    );

    Ok(())
}
