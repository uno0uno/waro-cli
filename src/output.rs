use anyhow::Result;
use serde_json::Value;

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
    // Flatten to array
    let rows = match value {
        Value::Array(arr) => arr.clone(),
        obj @ Value::Object(_) => vec![obj.clone()],
        _ => {
            println!("{}", serde_json::to_string_pretty(value)?);
            return Ok(());
        }
    };

    if rows.is_empty() {
        println!("(no results)");
        return Ok(());
    }

    // Collect headers from first row
    let headers: Vec<String> = if let Value::Object(map) = &rows[0] {
        map.keys().cloned().collect()
    } else {
        println!("{}", serde_json::to_string_pretty(value)?);
        return Ok(());
    };

    // Print header
    println!("{}", headers.join("\t|\t"));
    println!("{}", "-".repeat(headers.len() * 20));

    // Print rows
    for row in &rows {
        if let Value::Object(map) = row {
            let cells: Vec<String> = headers
                .iter()
                .map(|h| {
                    map.get(h)
                        .map(|v| match v {
                            Value::String(s) => s.clone(),
                            other => other.to_string(),
                        })
                        .unwrap_or_default()
                })
                .collect();
            println!("{}", cells.join("\t|\t"));
        }
    }

    Ok(())
}
