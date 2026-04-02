use anyhow::Result;
use clap::Args;
use serde_json::{json, Value};

#[derive(Args)]
pub struct SchemaArgs {
    /// Command group: sales | customers | menu | analytics | financial | waros (omit to list all)
    group: Option<String>,

    /// Subcommand: list | metrics | detail | products | recipes | modifiers |
    ///             menu | food-cost | alerts | data-quality | estimate | balances | customer
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
    "sales list, sales metrics, sales detail, \
     customers list, customers detail, customers metrics, \
     menu products, menu recipes, menu modifiers, \
     analytics menu, analytics food-cost, analytics alerts, analytics data-quality, \
     financial products, \
     waros estimate, waros balances, waros customer"
}

fn all_schemas() -> Value {
    json!([
        schema_for("sales", "list").unwrap(),
        schema_for("sales", "metrics").unwrap(),
        schema_for("sales", "detail").unwrap(),
        schema_for("customers", "list").unwrap(),
        schema_for("customers", "detail").unwrap(),
        schema_for("customers", "metrics").unwrap(),
        schema_for("menu", "products").unwrap(),
        schema_for("menu", "recipes").unwrap(),
        schema_for("menu", "modifiers").unwrap(),
        schema_for("analytics", "menu").unwrap(),
        schema_for("analytics", "food-cost").unwrap(),
        schema_for("analytics", "alerts").unwrap(),
        schema_for("analytics", "data-quality").unwrap(),
        schema_for("financial", "products").unwrap(),
        schema_for("waros", "estimate").unwrap(),
        schema_for("waros", "balances").unwrap(),
        schema_for("waros", "customer").unwrap(),
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
                { "name": "limit",          "type": "integer", "default": 50,               "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset",         "type": "integer", "default": 0,                "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",            "type": "boolean", "default": false,            "required": false, "description": "Fetch all pages automatically, output NDJSON" },
                { "name": "payment-method", "type": "string",  "default": null,             "required": false, "description": "Filter by payment method: cash | card | digital" },
                { "name": "status",         "type": "string",  "default": null,             "required": false, "description": "Filter by status: completed | cancelled | pending" },
                { "name": "date-from",      "type": "string",  "default": null,             "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",        "type": "string",  "default": null,             "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "timezone",       "type": "string",  "default": "America/Bogota", "required": false, "description": "IANA timezone" },
                { "name": "sort-field",     "type": "string",  "default": "order_date",     "required": false, "description": "Field to sort by" },
                { "name": "sort-direction", "type": "string",  "default": "desc",           "required": false, "description": "Sort direction: asc | desc" }
            ]
        })),
        ("sales", "metrics") => Some(json!({
            "command": "sales metrics",
            "method": "POST",
            "path": "/v1/sales/metrics",
            "scope": "orders:read",
            "paginates": false,
            "params": [
                { "name": "date-from",   "type": "string",  "default": null,             "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",     "type": "string",  "default": null,             "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "group-by",    "type": "string",  "default": null,             "required": false, "description": "Aggregation: date | weekday | hour | product | payment | ticket" },
                { "name": "timezone",    "type": "string",  "default": "America/Bogota", "required": false, "description": "IANA timezone" },
                { "name": "limit",       "type": "integer", "default": 20,               "required": false, "description": "Top N products (1-100, used with group-by=product)" },
                { "name": "sort-by",     "type": "string",  "default": "quantity",       "required": false, "description": "Sort for product grouping: quantity | revenue" },
                { "name": "ranges",      "type": "string",  "default": null,             "required": false, "description": "Comma-separated integers for ticket bins (used with group-by=ticket)" },
                { "name": "compare-to",  "type": "string",  "default": null,             "required": false, "description": "Compare to: previous-period | previous-year | YYYY-MM-DD:YYYY-MM-DD" }
            ]
        })),
        ("sales", "detail") => Some(json!({
            "command": "sales detail",
            "method": "POST",
            "path": "/v1/sales/detail",
            "scope": "orders:read",
            "paginates": false,
            "params": [
                { "name": "order-id", "type": "string", "default": null, "required": true, "description": "Order UUID" }
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
                { "name": "category-id",          "type": "string",  "default": null, "required": false, "description": "Filter by category UUID" },
                { "name": "is-available",         "type": "boolean", "default": null, "required": false, "description": "Filter by availability" },
                { "name": "include-ingredients",  "type": "boolean", "default": true, "required": false, "description": "Include ingredient details" },
                { "name": "include-recipe-bases", "type": "boolean", "default": true, "required": false, "description": "Include recipe base details" },
                { "name": "include-modifiers",    "type": "boolean", "default": true, "required": false, "description": "Include modifier group details" }
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
                { "name": "is-active", "type": "boolean", "default": null, "required": false, "description": "Filter by active status" }
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
        ("customers", "list") => Some(json!({
            "command": "customers list",
            "method": "POST",
            "path": "/v1/customers",
            "scope": "customers:read",
            "paginates": true,
            "params": [
                { "name": "limit",          "type": "integer", "default": 50,               "required": false, "description": "Max results per page (1-250)" },
                { "name": "offset",         "type": "integer", "default": 0,                "required": false, "description": "Pagination offset (ignored with --all)" },
                { "name": "all",            "type": "boolean", "default": false,            "required": false, "description": "Fetch all pages automatically, output NDJSON" },
                { "name": "search",         "type": "string",  "default": null,             "required": false, "description": "Partial match on name or phone" },
                { "name": "date-from",      "type": "string",  "default": null,             "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",        "type": "string",  "default": null,             "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "timezone",       "type": "string",  "default": "America/Bogota", "required": false, "description": "IANA timezone" },
                { "name": "sort-field",     "type": "string",  "default": "total_spent",    "required": false, "description": "total_spent|order_count|last_order_date|avg_ticket|waros_balance" },
                { "name": "sort-direction", "type": "string",  "default": "desc",           "required": false, "description": "asc|desc" }
            ]
        })),
        ("customers", "detail") => Some(json!({
            "command": "customers detail",
            "method": "POST",
            "path": "/v1/customers/detail",
            "scope": "customers:read",
            "paginates": false,
            "params": [
                { "name": "customer-id", "type": "string",  "default": null,             "required": true,  "description": "Customer UUID" },
                { "name": "limit",       "type": "integer", "default": 20,               "required": false, "description": "Max orders to return (1-100)" },
                { "name": "offset",      "type": "integer", "default": 0,                "required": false, "description": "Orders pagination offset" },
                { "name": "date-from",   "type": "string",  "default": null,             "required": false, "description": "Filter order history start date YYYY-MM-DD" },
                { "name": "date-to",     "type": "string",  "default": null,             "required": false, "description": "Filter order history end date YYYY-MM-DD" },
                { "name": "timezone",    "type": "string",  "default": "America/Bogota", "required": false, "description": "IANA timezone" }
            ]
        })),
        ("customers", "metrics") => Some(json!({
            "command": "customers metrics",
            "method": "POST",
            "path": "/v1/customers/metrics",
            "scope": "customers:read",
            "paginates": false,
            "params": [
                { "name": "date-from", "type": "string", "default": null,             "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",   "type": "string", "default": null,             "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "group-by",  "type": "string", "default": null,             "required": false, "description": "Time series: date|weekday|month" },
                { "name": "timezone",  "type": "string", "default": "America/Bogota", "required": false, "description": "IANA timezone" }
            ]
        })),
        ("analytics", "menu") => Some(json!({
            "command": "analytics menu",
            "method": "POST",
            "path": "/v1/analytics/menu-analysis",
            "scope": "analytics:read",
            "paginates": false,
            "params": [
                { "name": "date-from", "type": "string",  "default": null, "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",   "type": "string",  "default": null, "required": false, "description": "End date YYYY-MM-DD" },
                { "name": "limit",     "type": "integer", "default": 10,   "required": false, "description": "Max products to return (1-100)" }
            ]
        })),
        ("analytics", "food-cost") => Some(json!({
            "command": "analytics food-cost",
            "method": "POST",
            "path": "/v1/analytics/food-cost",
            "scope": "analytics:read",
            "paginates": false,
            "params": [
                { "name": "date-from", "type": "string", "default": null, "required": false, "description": "Start date YYYY-MM-DD" },
                { "name": "date-to",   "type": "string", "default": null, "required": false, "description": "End date YYYY-MM-DD" }
            ]
        })),
        ("analytics", "alerts") => Some(json!({
            "command": "analytics alerts",
            "method": "POST",
            "path": "/v1/analytics/alerts",
            "scope": "analytics:read",
            "paginates": false,
            "params": [
                { "name": "limit", "type": "integer", "default": 10, "required": false, "description": "Max alerts to return (1-100)" }
            ]
        })),
        ("analytics", "data-quality") => Some(json!({
            "command": "analytics data-quality",
            "method": "POST",
            "path": "/v1/analytics/data-quality",
            "scope": "analytics:read",
            "paginates": false,
            "params": []
        })),
        ("financial", "products") => Some(json!({
            "command": "financial products",
            "method": "POST",
            "path": "/v1/financial/products",
            "scope": "financial:read",
            "paginates": false,
            "params": [
                { "name": "period",     "type": "integer", "default": 365,      "required": false, "description": "Analysis period in days (1-730)" },
                { "name": "sort-by",    "type": "string",  "default": "margin", "required": false, "description": "Sort field: margin | revenue | cost | quantity" },
                { "name": "min-margin", "type": "integer", "default": null,     "required": false, "description": "Minimum margin percentage filter" },
                { "name": "category",   "type": "string",  "default": null,     "required": false, "description": "Filter by category name" }
            ]
        })),
        ("waros", "estimate") => Some(json!({
            "command": "waros estimate",
            "method": "POST",
            "path": "/v1/waros/estimate",
            "scope": "waros:read",
            "paginates": false,
            "params": [
                { "name": "total",       "type": "number", "default": null, "required": true,  "description": "Purchase total amount (>= 0)" },
                { "name": "customer-id", "type": "string", "default": null, "required": false, "description": "Customer UUID (optional, for personalized estimate)" }
            ]
        })),
        ("waros", "balances") => Some(json!({
            "command": "waros balances",
            "method": "POST",
            "path": "/v1/waros/balances",
            "scope": "waros:read",
            "paginates": false,
            "params": [
                { "name": "profile-ids", "type": "string", "default": null, "required": true, "description": "Comma-separated customer profile UUIDs" }
            ]
        })),
        ("waros", "customer") => Some(json!({
            "command": "waros customer",
            "method": "POST",
            "path": "/v1/waros/customer-summary",
            "scope": "waros:read",
            "paginates": false,
            "params": [
                { "name": "profile-id", "type": "string", "default": null, "required": true, "description": "Customer profile UUID" }
            ]
        })),
        _ => None,
    }
}
