use crate::client::WaroClient;
use crate::output;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct FinancialArgs {
    #[command(subcommand)]
    pub command: FinancialCommands,
}

#[derive(Subcommand)]
pub enum FinancialCommands {
    /// Financial product analysis — margin, cost, revenue, and profitability
    Products(ProductsArgs),
}

// ── products ──────────────────────────────────────────────────────────────────

#[derive(Args)]
pub struct ProductsArgs {
    /// Analysis period in days (1-730)
    #[arg(long, default_value = "365")]
    period: u32,

    /// Sort field: margin | revenue | cost | quantity
    #[arg(long, default_value = "margin")]
    sort_by: String,

    /// Minimum margin percentage filter (optional)
    #[arg(long)]
    min_margin: Option<i32>,

    /// Filter by category name (optional)
    #[arg(long)]
    category: Option<String>,

    /// Validate request locally without calling the API
    #[arg(long)]
    dry_run: bool,
}

// ── handlers ──────────────────────────────────────────────────────────────────

pub async fn run(
    args: FinancialArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        FinancialCommands::Products(a) => products(a, client, format, fields).await,
    }
}

async fn products(
    a: ProductsArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    validate::validate_enum(
        "sort-by",
        &a.sort_by,
        &["margin", "revenue", "cost", "quantity"],
    )?;

    let body = json!({
        "period": a.period,
        "sortBy": a.sort_by,
        "minMargin": a.min_margin,
        "category": a.category,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/financial/products");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let sp = Spinner::start();
    let resp = client.post("/v1/financial/products", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
