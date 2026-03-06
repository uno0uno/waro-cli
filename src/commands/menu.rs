use crate::client::WaroClient;
use crate::output;
use anyhow::Result;
use clap::{Args, Subcommand};
use serde_json::json;

#[derive(Args)]
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
    #[arg(long, default_value = "50")]
    limit: u32,

    #[arg(long, default_value = "0")]
    offset: u32,

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
    #[arg(long, default_value = "50")]
    limit: u32,

    #[arg(long, default_value = "0")]
    offset: u32,

    #[arg(long)]
    is_active: Option<bool>,

    #[arg(long)]
    dry_run: bool,
}

#[derive(Args)]
pub struct ModifiersArgs {
    #[arg(long, default_value = "50")]
    limit: u32,

    #[arg(long, default_value = "0")]
    offset: u32,

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
    let body = json!({
        "limit": a.limit,
        "offset": a.offset,
        "categoryId": a.category_id,
        "isAvailable": a.is_available,
        "includeIngredients": a.include_ingredients,
        "includeRecipeBases": a.include_recipe_bases,
        "includeModifiers": a.include_modifiers,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/menu/products");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/menu/products", body).await?;
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
    let body = json!({
        "limit": a.limit,
        "offset": a.offset,
        "isActive": a.is_active,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/menu/recipes");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/menu/recipes", body).await?;
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
    let body = json!({
        "limit": a.limit,
        "offset": a.offset,
    });

    if a.dry_run {
        println!("DRY RUN — POST /v1/menu/modifiers");
        println!("{}", serde_json::to_string_pretty(&body)?);
        return Ok(());
    }

    let resp = client.post("/v1/menu/modifiers", body).await?;
    let resp = output::apply_fields(resp, fields.as_deref());
    output::print(&resp, format)?;
    Ok(())
}
