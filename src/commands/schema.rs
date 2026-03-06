use anyhow::Result;
use clap::Args;
use serde_json::{json, Value};

#[derive(Args)]
pub struct SchemaArgs {
    /// Command group: sales | menu (omit to list all schemas)
    group: Option<String>,

    /// Subcommand: list | metrics | detail | products | recipes | modifiers
    subcommand: Option<String>,
}

pub fn run(args: SchemaArgs) -> Result<()> {
    match (args.group.as_deref(), args.subcommand.as_deref()) {
        (None, None) => {
            // No args → list all schemas
            let all = all_schemas();
            println!("{}", serde_json::to_string_pretty(&all)?);
        }
        (Some(g), Some(s)) => match schema_for(g, s) {
            Some(schema) => println!("{}", serde_json::to_string_pretty(&schema)?),
            None => {
                anyhow::bail!(
                    "Unknown command: {} {}.\nValid commands: {}",
                    g,
                    s,
                    valid_commands()
                );
            }
        },
        _ => {
            anyhow::bail!(
                "Usage: waro schema <group> <subcommand>\nValid commands: {}",
                valid_commands()
            );
        }
    }
    Ok(())
}

fn valid_commands() -> &'static str {
    "sales list, sales metrics, sales detail, menu products, menu recipes, menu modifiers"
}

fn all_schemas() -> Value {
    json!([
        schema_for("sales", "list").unwrap(),
        schema_for("sales", "metrics").unwrap(),
        schema_for("sales", "detail").unwrap(),
        schema_for("menu", "products").unwrap(),
        schema_for("menu", "recipes").unwrap(),
        schema_for("menu", "modifiers").unwrap(),
    ])
}

fn schema_for(group: &str, subcommand: &str) -> Option<Value> {
    match (group, subcommand) {
        ("sales", "list") => Some(json!({
            "command": "sales list",
            "method": "POST",
            "path": "/v1/sales",
            "scope": "orders:read",
            "paginates": true,
            "params": [
                { "name": "limit",          "type": "integer", "default": 50,                "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset",         "type": "integer", "default": 0,                 "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",            "type": "boolean", "default": false,             "required": false, "description": "Fetch all pages automatically, output NDJSON" },
                { "name": "payment_method", "type": "string",  "default": null,              "required": false, "description": "Filter by payment method: cash | card | digital" },
                { "name": "status",         "type": "string",  "default": null,              "required": false, "description": "Filter by status: completed | cancelled | pending" },
                { "name": "date_from",      "type": "string",  "default": null,              "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date_to",        "type": "string",  "default": null,              "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "timezone",       "type": "string",  "default": "America/Bogota",  "required": false, "description": "IANA timezone" },
                { "name": "sort_field",     "type": "string",  "default": "order_date",      "required": false, "description": "Field to sort by" },
                { "name": "sort_direction", "type": "string",  "default": "desc",            "required": false, "description": "Sort direction: asc | desc" }
            ]
        })),
        ("sales", "metrics") => Some(json!({
            "command": "sales metrics",
            "method": "POST",
            "path": "/v1/sales/metrics",
            "scope": "orders:read",
            "paginates": false,
            "params": [
                { "name": "date_from", "type": "string", "default": null,             "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date_to",   "type": "string", "default": null,             "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "group_by",  "type": "string", "default": null,             "required": false, "description": "Aggregation: date | weekday | hour | product | payment | ticket" },
                { "name": "timezone",  "type": "string", "default": "America/Bogota", "required": false, "description": "IANA timezone" }
            ]
        })),
        ("sales", "detail") => Some(json!({
            "command": "sales detail",
            "method": "POST",
            "path": "/v1/sales/detail",
            "scope": "orders:read",
            "paginates": false,
            "params": [
                { "name": "order_id", "type": "string", "default": null, "required": true, "description": "Order UUID" }
            ]
        })),
        ("menu", "products") => Some(json!({
            "command": "menu products",
            "method": "POST",
            "path": "/v1/menu/products",
            "scope": "menu:read",
            "paginates": true,
            "params": [
                { "name": "limit",                "type": "integer", "default": 50,    "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset",               "type": "integer", "default": 0,     "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",                  "type": "boolean", "default": false, "required": false, "description": "Fetch all pages automatically, output NDJSON" },
                { "name": "category_id",          "type": "string",  "default": null,  "required": false, "description": "Filter by category UUID" },
                { "name": "is_available",         "type": "boolean", "default": null,  "required": false, "description": "Filter by availability" },
                { "name": "include_ingredients",  "type": "boolean", "default": true,  "required": false, "description": "Include ingredient details" },
                { "name": "include_recipe_bases", "type": "boolean", "default": true,  "required": false, "description": "Include recipe base details" },
                { "name": "include_modifiers",    "type": "boolean", "default": true,  "required": false, "description": "Include modifier group details" }
            ]
        })),
        ("menu", "recipes") => Some(json!({
            "command": "menu recipes",
            "method": "POST",
            "path": "/v1/menu/recipes",
            "scope": "menu:read",
            "paginates": true,
            "params": [
                { "name": "limit",     "type": "integer", "default": 50,    "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset",    "type": "integer", "default": 0,     "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",       "type": "boolean", "default": false, "required": false, "description": "Fetch all pages automatically, output NDJSON" },
                { "name": "is_active", "type": "boolean", "default": null,  "required": false, "description": "Filter by active status" }
            ]
        })),
        ("menu", "modifiers") => Some(json!({
            "command": "menu modifiers",
            "method": "POST",
            "path": "/v1/menu/modifiers",
            "scope": "menu:read",
            "paginates": true,
            "params": [
                { "name": "limit",  "type": "integer", "default": 50,    "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset", "type": "integer", "default": 0,     "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",    "type": "boolean", "default": false, "required": false, "description": "Fetch all pages automatically, output NDJSON" }
            ]
        })),
        _ => None,
    }
}
