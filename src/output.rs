use anyhow::Result;
use colored::Colorize;
use serde_json::Value;
use tabled::{builder::Builder, settings::Style};

/// Print a red error message to stderr
pub fn eprint_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}

/// Print a yellow warning message to stderr
pub fn eprint_warning(msg: &str) {
    eprintln!("{} {}", "warn:".yellow().bold(), msg);
}

/// Apply field mask to a JSON value (object or array of objects).
///
/// For paginated responses `{data: [...], ...}` the filter is applied to
/// each item inside `data`, not to the top-level wrapper.
pub fn apply_fields(value: Value, fields: Option<&str>) -> Value {
    let Some(fields_str) = fields else {
        return value;
    };
    let keys: Vec<&str> = fields_str.split(',').map(|s| s.trim()).collect();

    match value {
        Value::Array(arr) => Value::Array(
            arr.into_iter()
                .map(|item| filter_object(item, &keys))
                .collect(),
        ),
        Value::Object(ref map) if map.get("data").is_some_and(|v| v.is_array()) => {
            // Paginated response — filter items inside data, keep wrapper intact
            let mut out = map.clone();
            if let Some(Value::Array(arr)) = map.get("data") {
                let filtered = arr
                    .iter()
                    .map(|item| filter_object(item.clone(), &keys))
                    .collect();
                out.insert("data".to_string(), Value::Array(filtered));
            }
            Value::Object(out)
        }
        obj @ Value::Object(_) => filter_object(obj, &keys),
        other => other,
    }
}

fn filter_object(value: Value, keys: &[&str]) -> Value {
    if let Value::Object(map) = value {
        let filtered: serde_json::Map<String, Value> = map
            .into_iter()
            .filter(|(k, _)| keys.contains(&k.as_str()))
            .collect();
        Value::Object(filtered)
    } else {
        value
    }
}

/// Print JSON value — pretty for json mode, table for table mode
pub fn print(value: &Value, format: &str) -> Result<()> {
    match format {
        "table" => print_table(value),
        _ => {
            println!("{}", serde_json::to_string_pretty(value)?);
            Ok(())
        }
    }
}

/// Render a single JSON value as a readable table cell string.
/// - Strings/numbers/bools → as-is
/// - Null → ""
/// - Arrays → "[N]" (or "" if empty)
/// - Objects → prefer "name" or "title" field; otherwise comma-list of scalar fields
fn cell_value(v: &Value) -> String {
    match v {
        Value::String(s) => s.clone(),
        Value::Null => String::new(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Array(arr) => {
            if arr.is_empty() {
                String::new()
            } else {
                format!("[{}]", arr.len())
            }
        }
        Value::Object(map) => {
            // Prefer a human-readable label field
            for label_key in &["name", "title", "label"] {
                if let Some(Value::String(s)) = map.get(*label_key) {
                    return s.clone();
                }
            }
            // Fall back to "key: value" pairs for scalar fields only
            let parts: Vec<String> = map
                .iter()
                .filter_map(|(k, v)| match v {
                    Value::String(s) => Some(format!("{}: {}", k, s)),
                    Value::Number(n) => Some(format!("{}: {}", k, n)),
                    Value::Bool(b) => Some(format!("{}: {}", k, b)),
                    _ => None,
                })
                .collect();
            if parts.is_empty() {
                "{...}".to_string()
            } else {
                parts.join(", ")
            }
        }
    }
}

/// Locate the best rows to display from an API response.
///
/// Resolution order:
/// 1. Value is already an array → use it directly
/// 2. `data` key is an array → use it
/// 3. `data` key is an object → look for a nested array in common keys
///    (`menu_items`, `alerts`, `products`, `orders`, `items`, `rows`)
/// 4. `data` key is a flat scalar-only object → show as a single row
/// 5. Top-level known array keys: `top_customers`, `products`, `items`,
///    `results`, `records`, `rows`
/// 6. `balances` key is a map of id → value → convert to [{profile_id, balance}] rows
/// 7. Fallback: treat the whole object as a single row
fn find_rows(value: &Value) -> Vec<Value> {
    match value {
        Value::Array(arr) => arr.clone(),
        Value::Object(map) => {
            // 1. data → array
            if let Some(Value::Array(arr)) = map.get("data") {
                return arr.clone();
            }

            // 2. data → object
            if let Some(Value::Object(data_obj)) = map.get("data") {
                // 2a. nested array inside data
                for key in &[
                    "menu_items",
                    "alerts",
                    "products",
                    "orders",
                    "items",
                    "rows",
                ] {
                    if let Some(Value::Array(arr)) = data_obj.get(*key) {
                        return arr.clone();
                    }
                }
                // 2b. flat data object → single row
                return vec![Value::Object(data_obj.clone())];
            }

            // 3. top-level known array keys
            for key in &[
                "top_customers",
                "products",
                "items",
                "results",
                "records",
                "rows",
            ] {
                if let Some(Value::Array(arr)) = map.get(*key) {
                    return arr.clone();
                }
            }

            // 4. balances map → [{profile_id, balance}]
            if let Some(Value::Object(balances)) = map.get("balances") {
                return balances
                    .iter()
                    .map(|(k, v)| serde_json::json!({"profile_id": k, "balance": v}))
                    .collect();
            }

            // 5. fallback: whole object as single row
            vec![value.clone()]
        }
        _ => vec![],
    }
}

fn print_table(value: &Value) -> Result<()> {
    let rows = find_rows(value);

    if rows.is_empty() {
        println!("{}", "(no results)".dimmed());
        return Ok(());
    }

    // Collect headers from the union of all row keys (preserving first-row order)
    let headers: Vec<String> = {
        let mut seen = std::collections::HashSet::new();
        let mut order = Vec::new();
        for row in &rows {
            if let Value::Object(map) = row {
                for k in map.keys() {
                    if seen.insert(k.clone()) {
                        order.push(k.clone());
                    }
                }
            }
        }
        order
    };

    if headers.is_empty() {
        // No object rows — fall back to pretty JSON
        println!("{}", serde_json::to_string_pretty(value)?);
        return Ok(());
    }

    let mut builder = Builder::default();
    builder.push_record(headers.clone());

    for row in &rows {
        if let Value::Object(map) = row {
            let cells: Vec<String> = headers
                .iter()
                .map(|h| map.get(h).map(cell_value).unwrap_or_default())
                .collect();
            builder.push_record(cells);
        }
    }

    let table = builder.build().with(Style::modern()).to_string();
    println!("{}", table);

    Ok(())
}
