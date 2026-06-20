use crate::contract::{self, CommandContract, ResponseShape};
use anyhow::Result;
use colored::Colorize;
use serde_json::{json, Value};
use tabled::{builder::Builder, settings::Style};

/// Print a red error message to stderr
pub fn eprint_error(msg: &str) {
    eprintln!("{} {}", "error:".red().bold(), msg);
}

/// Print a yellow warning message to stderr
pub fn eprint_warning(msg: &str) {
    eprintln!("{} {}", "warn:".yellow().bold(), msg);
}

pub fn print_agent_error(command: &str, message: &str, kind: &str) -> Result<()> {
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema_version": "waro.agent.v1",
            "ok": false,
            "command": command,
            "error": {
                "message": message,
                "kind": kind,
            },
        }))?
    );
    Ok(())
}

pub fn print_contract_fields(contract: CommandContract) -> Result<()> {
    println!("Available fields:");
    for field in contract.fields {
        println!("  {}", field);
    }
    if !contract.top_level_keys.is_empty() {
        println!("Top-level fields:");
        for field in contract.top_level_keys {
            println!("  {}", field);
        }
    }
    Ok(())
}

pub fn emit(command: &str, value: Value, format: &str, fields: Option<&str>) -> Result<()> {
    let contract = contract::contract_for(command);
    if let Some(contract) = contract {
        contract::validate_fields(contract, fields)?;
    }

    let filtered = if let Some(contract) = contract {
        apply_fields_with_contract(value, fields, contract)
    } else {
        apply_fields(value, fields)
    };

    match format {
        "agent-json" => {
            let Some(contract) = contract else {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&json!({
                        "schema_version": "waro.agent.v1",
                        "ok": false,
                        "command": command,
                        "error": {
                            "message": format!("missing response contract for {command}"),
                            "kind": "unknown",
                        },
                    }))?
                );
                return Ok(());
            };
            print_agent_json(contract, &filtered, fields)
        }
        "table" => print_table(&filtered),
        "fields" => print_fields_for_contract(&filtered, contract),
        _ => {
            println!("{}", serde_json::to_string_pretty(&filtered)?);
            Ok(())
        }
    }
}

pub fn emit_with_contract(
    contract: CommandContract,
    value: Value,
    format: &str,
    fields: Option<&str>,
) -> Result<()> {
    contract::validate_fields(contract, fields)?;
    let filtered = apply_fields_with_contract(value, fields, contract);

    match format {
        "agent-json" => print_agent_json(contract, &filtered, fields),
        "table" => print_table(&filtered),
        "fields" => print_fields_for_contract(&filtered, Some(contract)),
        _ => {
            println!("{}", serde_json::to_string_pretty(&filtered)?);
            Ok(())
        }
    }
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
            // Paginated response: {data: [...]} — filter items inside data array
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
        Value::Object(ref map) if map.get("data").is_some_and(|v| v.is_object()) => {
            // Nested response: {data: {alerts/menu_items/products/...: [...]}}
            // Filter items inside the first known nested array key
            if let Some(Value::Object(data_obj)) = map.get("data") {
                for key in &[
                    "menu_items",
                    "alerts",
                    "products",
                    "orders",
                    "items",
                    "rows",
                ] {
                    if let Some(Value::Array(arr)) = data_obj.get(*key) {
                        let filtered: Vec<Value> = arr
                            .iter()
                            .map(|item| filter_object(item.clone(), &keys))
                            .collect();
                        let mut new_data = data_obj.clone();
                        new_data.insert(key.to_string(), Value::Array(filtered));
                        let mut out = map.clone();
                        out.insert("data".to_string(), Value::Object(new_data));
                        return Value::Object(out);
                    }
                }
            }
            // data is a flat object — filter the whole response top-level
            filter_object(value, &keys)
        }
        obj @ Value::Object(_) => filter_object(obj, &keys),
        other => other,
    }
}

pub fn apply_fields_with_contract(
    value: Value,
    fields: Option<&str>,
    contract: CommandContract,
) -> Value {
    let Some(fields_str) = fields else {
        return value;
    };
    let keys: Vec<&str> = fields_str
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .collect();
    if keys.is_empty() {
        return value;
    }

    match contract.shape {
        ResponseShape::DataRows => filter_data_rows(value, &keys),
        ResponseShape::DataObject => filter_data_object_or_top(value, &keys, contract),
        ResponseShape::NestedRows => filter_nested_rows(value, contract.row_path, &keys),
        ResponseShape::TopLevelRows => filter_top_level_rows(value, contract, &keys),
        ResponseShape::TopLevelObject => filter_object(value, &keys),
        ResponseShape::BalancesMap => value,
    }
}

fn filter_data_rows(value: Value, keys: &[&str]) -> Value {
    if let Value::Object(map) = value {
        let mut out = map.clone();
        if let Some(Value::Array(arr)) = map.get("data") {
            let filtered = arr
                .iter()
                .map(|item| filter_object(item.clone(), keys))
                .collect();
            out.insert("data".to_string(), Value::Array(filtered));
        }
        Value::Object(out)
    } else {
        apply_fields(value, Some(&keys.join(",")))
    }
}

fn filter_data_object_or_top(value: Value, keys: &[&str], contract: CommandContract) -> Value {
    if keys.iter().any(|key| contract.top_level_keys.contains(key)) {
        return filter_object(value, keys);
    }
    if let Value::Object(map) = value {
        let mut out = map.clone();
        if let Some(Value::Object(data)) = map.get("data") {
            out.insert(
                "data".to_string(),
                filter_object(Value::Object(data.clone()), keys),
            );
        }
        Value::Object(out)
    } else {
        value
    }
}

fn filter_nested_rows(value: Value, row_path: &str, keys: &[&str]) -> Value {
    let path: Vec<&str> = row_path.split('.').collect();
    if path.len() != 2 {
        return value;
    }
    let [outer, inner] = [path[0], path[1]];
    if let Value::Object(map) = value {
        let mut out = map.clone();
        if let Some(Value::Object(data_obj)) = map.get(outer) {
            let mut new_data = data_obj.clone();
            if let Some(Value::Array(arr)) = data_obj.get(inner) {
                let filtered = arr
                    .iter()
                    .map(|item| filter_object(item.clone(), keys))
                    .collect();
                new_data.insert(inner.to_string(), Value::Array(filtered));
            }
            out.insert(outer.to_string(), Value::Object(new_data));
        }
        Value::Object(out)
    } else {
        value
    }
}

fn filter_top_level_rows(value: Value, contract: CommandContract, keys: &[&str]) -> Value {
    if let Value::Object(map) = value {
        let top_keys: Vec<&str> = keys
            .iter()
            .copied()
            .filter(|key| contract.top_level_keys.contains(key))
            .collect();
        let row_keys: Vec<&str> = keys
            .iter()
            .copied()
            .filter(|key| contract.fields.contains(key))
            .collect();

        let mut out = if top_keys.is_empty() {
            map.clone()
        } else {
            map.iter()
                .filter(|(key, _)| top_keys.contains(&key.as_str()))
                .map(|(key, value)| (key.clone(), value.clone()))
                .collect()
        };

        if !row_keys.is_empty() {
            let source = map
                .get(contract.row_path)
                .or_else(|| out.get(contract.row_path));
            if let Some(Value::Array(arr)) = source {
                let filtered = arr
                    .iter()
                    .map(|item| filter_object(item.clone(), &row_keys))
                    .collect();
                out.insert(contract.row_path.to_string(), Value::Array(filtered));
            }
        } else if top_keys.is_empty() {
            return Value::Object(map);
        }

        if let Some(Value::Array(arr)) = map.get(contract.row_path) {
            if top_keys.contains(&contract.row_path) && row_keys.is_empty() {
                out.insert(contract.row_path.to_string(), Value::Array(arr.clone()));
            }
        }

        Value::Object(out)
    } else {
        value
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

/// Print JSON value — pretty for json mode, table for table mode, fields to list available fields
pub fn print(value: &Value, format: &str) -> Result<()> {
    match format {
        "table" => print_table(value),
        "fields" => print_fields(value),
        _ => {
            println!("{}", serde_json::to_string_pretty(value)?);
            Ok(())
        }
    }
}

/// Print the field names available in the first row of the response
fn print_fields(value: &Value) -> Result<()> {
    let rows = find_rows(value);
    let Some(Value::Object(map)) = rows.first() else {
        println!("(no fields found)");
        return Ok(());
    };
    println!("Available fields:");
    for key in map.keys() {
        println!("  {}", key);
    }
    Ok(())
}

fn print_fields_for_contract(value: &Value, contract: Option<CommandContract>) -> Result<()> {
    if let Some(contract) = contract {
        return print_contract_fields(contract);
    }
    print_fields(value)
}

/// Truncate an ISO 8601 datetime string to "YYYY-MM-DD HH:MM".
/// Returns None if the string doesn't look like a datetime.
fn truncate_datetime(s: &str) -> Option<String> {
    // Must be at least "YYYY-MM-DDTHH:MM" = 16 chars and contain 'T'
    if s.len() >= 16 && s.chars().nth(10) == Some('T') {
        let date = &s[..10];
        let time = &s[11..16];
        Some(format!("{} {}", date, time))
    } else {
        None
    }
}

/// Render a single JSON value as a readable table cell string.
/// - Strings/numbers/bools → as-is (datetime strings are truncated to YYYY-MM-DD HH:MM)
/// - Null → ""
/// - Arrays → "[N]" (or "" if empty)
/// - Objects → prefer "name" or "title" field; otherwise comma-list of scalar fields
fn cell_value(v: &Value) -> String {
    match v {
        Value::String(s) => truncate_datetime(s).unwrap_or_else(|| s.clone()),
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
            // Prefer a human-readable label field (skip empty strings)
            for label_key in &["name", "title", "label"] {
                if let Some(Value::String(s)) = map.get(*label_key) {
                    if !s.is_empty() {
                        return s.clone();
                    }
                }
            }
            // Secondary: phone as a short identifier
            if let Some(Value::String(s)) = map.get("phone") {
                if !s.is_empty() {
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
/// 5. Top-level known array keys: `series`, `top_customers`, `products`, `items`,
///    `results`, `records`, `rows`  (`series` checked first so group-by time series
///    wins over `top_customers` when both are present)
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
            // `series` before `top_customers` so time-series group-by results
            // take priority over the top_customers summary list
            for key in &[
                "series",
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

pub fn rows_for_contract(value: &Value, contract: CommandContract) -> Vec<Value> {
    match contract.shape {
        ResponseShape::DataRows => value
            .get("data")
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        ResponseShape::DataObject => value
            .get("data")
            .and_then(Value::as_object)
            .map(|map| vec![Value::Object(map.clone())])
            .unwrap_or_default(),
        ResponseShape::NestedRows => {
            let mut current = value;
            for part in contract.row_path.split('.') {
                let Some(next) = current.get(part) else {
                    return Vec::new();
                };
                current = next;
            }
            current.as_array().cloned().unwrap_or_default()
        }
        ResponseShape::TopLevelRows => value
            .get(contract.row_path)
            .and_then(Value::as_array)
            .cloned()
            .unwrap_or_default(),
        ResponseShape::TopLevelObject => vec![value.clone()],
        ResponseShape::BalancesMap => value
            .get("balances")
            .and_then(Value::as_object)
            .map(|balances| {
                balances
                    .iter()
                    .map(|(key, value)| json!({"profile_id": key, "balance": value}))
                    .collect()
            })
            .unwrap_or_default(),
    }
}

fn pagination_for_contract(value: &Value, contract: CommandContract) -> Value {
    if !contract.paginates {
        return Value::Null;
    }
    if let Some(pagination) = value.get("pagination") {
        return pagination.clone();
    }
    let mut pagination = serde_json::Map::new();
    for key in ["limit", "offset", "total", "hasMore"] {
        if let Some(value) = value.get(key) {
            pagination.insert(key.to_string(), value.clone());
        }
    }
    if pagination.is_empty() {
        Value::Null
    } else {
        Value::Object(pagination)
    }
}

fn data_for_contract(value: &Value, contract: CommandContract) -> Value {
    match contract.shape {
        ResponseShape::DataRows => Value::Null,
        ResponseShape::DataObject => value.get("data").cloned().unwrap_or(Value::Null),
        ResponseShape::NestedRows => value.get("data").cloned().unwrap_or(Value::Null),
        ResponseShape::TopLevelRows => {
            let mut out = serde_json::Map::new();
            for key in contract.top_level_keys {
                if *key != contract.row_path {
                    if let Some(value) = value.get(*key) {
                        out.insert((*key).to_string(), value.clone());
                    }
                }
            }
            Value::Object(out)
        }
        ResponseShape::TopLevelObject | ResponseShape::BalancesMap => value.clone(),
    }
}

fn print_agent_json(contract: CommandContract, value: &Value, fields: Option<&str>) -> Result<()> {
    let applied_fields = fields.map(|fields| {
        fields
            .split(',')
            .map(str::trim)
            .filter(|field| !field.is_empty())
            .collect::<Vec<&str>>()
    });
    println!(
        "{}",
        serde_json::to_string_pretty(&json!({
            "schema_version": "waro.agent.v1",
            "ok": true,
            "command": contract.command,
            "method": contract.method,
            "path": contract.path,
            "scope": contract.scope,
            "paginates": contract.paginates,
            "row_path": contract.row_path,
            "rows": rows_for_contract(value, contract),
            "data": data_for_contract(value, contract),
            "pagination": pagination_for_contract(value, contract),
            "available_fields": contract.fields,
            "applied_fields": applied_fields,
        }))?
    );
    Ok(())
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
        // All rows are empty objects — field filter matched nothing
        eprint_warning("no fields matched. Use --output fields to see available field names.");
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
