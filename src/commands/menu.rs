use crate::client::WaroClient;
use crate::output;
use crate::pagination;
use crate::spinner::Spinner;
use crate::validate;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
#[command(arg_required_else_help = true)]
pub struct MenuArgs {
    #[command(subcommand)]
    pub command: MenuCommands,
}

#[derive(Subcommand)]
pub enum MenuCommands {
    /// List menu products
    Products(ProductsArgs),
    /// List recipe bases
    Recipes(RecipesArgs),
    /// List modifier groups
    Modifiers(ModifiersArgs),
}

#[derive(Args)]
pub struct ProductsArgs {
    /// Max results per page (1-250)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset (ignored when --all is set)
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Fetch all pages automatically and output NDJSON
    #[arg(long)]
    all: bool,

    /// Filter by category UUID
    #[arg(long)]
    category_id: Option<String>,

    /// Filter by availability
    #[arg(long)]
    is_available: Option<bool>,

    /// Include ingredients (default: true)
    #[arg(long, default_value = "true")]
    include_ingredients: bool,

    /// Include recipe bases (default: true)
    #[arg(long, default_value = "true")]
    include_recipe_bases: bool,

    /// Include modifiers (default: true)
    #[arg(long, default_value = "true")]
    include_modifiers: bool,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Args)]
pub struct RecipesArgs {
    /// Max results per page (1-250)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset (ignored when --all is set)
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Fetch all pages automatically and output NDJSON
    #[arg(long)]
    all: bool,

    #[arg(long)]
    is_active: Option<bool>,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Args)]
pub struct ModifiersArgs {
    /// Max results per page (1-250)
    #[arg(long, default_value = "50")]
    limit: u32,

    /// Pagination offset (ignored when --all is set)
    #[arg(long, default_value = "0")]
    offset: u32,

    /// Fetch all pages automatically and output NDJSON
    #[arg(long)]
    all: bool,

    #[arg(long)]
    dry_run: bool,
}

pub async fn run(
    args: MenuArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    match args.command {
        MenuCommands::Products(a) => products(a, client, format, fields).await,
        MenuCommands::Recipes(a) => recipes(a, client, format, fields).await,
        MenuCommands::Modifiers(a) => modifiers(a, client, format, fields).await,
    }
}

async fn products(
    a: ProductsArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    // Validate inputs before any API call
    if let Some(ref v) = a.category_id {
        validate::validate_uuid("category-id", v)?;
    }

    let filters = json!({
        "categoryId": a.category_id,
        "isAvailable": a.is_available,
        "includeIngredients": a.include_ingredients,
        "includeRecipeBases": a.include_recipe_bases,
        "includeModifiers": a.include_modifiers,
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
        println!("DRY RUN — POST /v1/menu/products{}", suffix);
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(
            client,
            "/v1/menu/products",
            filters,
            a.limit,
            fields.as_deref(),
            format,
        )
        .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);
    let sp = Spinner::start();
    let resp = client.post("/v1/menu/products", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn recipes(
    a: RecipesArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let filters = json!({
        "isActive": a.is_active,
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
        println!("DRY RUN — POST /v1/menu/recipes{}", suffix);
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(
            client,
            "/v1/menu/recipes",
            filters,
            a.limit,
            fields.as_deref(),
            format,
        )
        .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);
    let sp = Spinner::start();
    let resp = client.post("/v1/menu/recipes", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}

async fn modifiers(
    a: ModifiersArgs,
    client: &WaroClient,
    format: &str,
    fields: Option<String>,
) -> Result<()> {
    let filters = json!({});

    if a.dry_run {
        let suffix = if a.all {
            " (--all mode, showing first page)"
        } else {
            ""
        };
        let mut body = filters.clone();
        body["limit"] = json!(a.limit);
        body["offset"] = json!(if a.all { 0 } else { a.offset });
        println!("DRY RUN — POST /v1/menu/modifiers{}", suffix);
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    if a.all {
        return pagination::fetch_all(
            client,
            "/v1/menu/modifiers",
            filters,
            a.limit,
            fields.as_deref(),
            format,
        )
        .await;
    }

    let mut body = filters;
    body["limit"] = json!(a.limit);
    body["offset"] = json!(a.offset);
    let sp = Spinner::start();
    let resp = client.post("/v1/menu/modifiers", body).await?;
    sp.stop();
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
