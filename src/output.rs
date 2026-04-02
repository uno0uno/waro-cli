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

/// Apply field mask to a JSON value (object or array of objects)
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

fn print_table(value: &Value) -> Result<()> {
    // Attempt to "unwrap" paginated or list-wrapped responses
    let rows = match value {
        Value::Array(arr) => arr.clone(),
        Value::Object(map) => {
            let mut found_data = None;
            // Common keys for wrapped arrays
            for key in &["data", "items", "results", "records"] {
                if let Some(Value::Array(arr)) = map.get(*key) {
                    found_data = Some(arr.clone());
                    break;
                }
            }
            // If we found a wrapped array, use it; otherwise use the object itself as a row
            found_data.unwrap_or_else(|| vec![Value::Object(map.clone())])
        }
        _ => {
            // Not a list or object, fallback to pretty JSON
            println!("{}", serde_json::to_string_pretty(value)?);
            return Ok(());
        }
    };

    if rows.is_empty() {
        println!("{}", "(no results)".dimmed());
        return Ok(());
    }

    // Build the table using tabled crate
    let mut builder = Builder::default();

    // Collect headers from the first object row (if it is one)
    let headers: Vec<String> = if let Some(Value::Object(map)) = rows.first() {
        map.keys().cloned().collect()
    } else {
        // Fallback for non-object arrays
        println!("{}", serde_json::to_string_pretty(value)?);
        return Ok(());
    };

    // Push headers to the table builder
    builder.push_record(headers.clone());

    // Push rows to the table builder
    for row in rows {
        if let Value::Object(map) = row {
            let cells: Vec<String> = headers
                .iter()
                .map(|h| {
                    map.get(h)
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            Value::Null => "".to_string(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default()
                })
                .collect();
            builder.push_record(cells);
        }
    }

    // Render the table with a clean style
    let table = builder.build().with(Style::modern()).to_string();
    println!("{}", table);

    Ok(())
}
