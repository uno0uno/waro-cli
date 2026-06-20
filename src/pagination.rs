use crate::client::WaroClient;
use crate::contract;
use crate::output;
use crate::spinner::Spinner;
use anyhow::Result;
use serde_json::{json, Value};

/// Fetch all pages from a paginated endpoint.
///
/// - `format == "table"`: collects every row across all pages, then renders a
///   single table (spinner stays up during collection).
/// - Any other format (default NDJSON): streams one JSON object per line.
///   No spinner is shown — the streaming output itself is the progress signal,
///   and running the spinner in parallel caused visual interleaving.
///
/// Iterates offset = 0, limit, 2×limit, … until a page returns fewer items
/// than `limit` (last partial page) or an empty page.
pub async fn fetch_all(
    command: &str,
    client: &WaroClient,
    endpoint: &str,
    mut base_body: Value,
    limit: u32,
    fields: Option<&str>,
    format: &str,
) -> Result<()> {
    let contract = contract::contract_for(command);
    if let Some(contract) = contract {
        contract::validate_fields(contract, fields)?;
    }

    if format == "table" || format == "agent-json" {
        // Table and agent-json modes: collect all rows, render once
        let sp = Spinner::start();
        let mut all_items: Vec<Value> = Vec::new();
        let mut offset: u32 = 0;
        loop {
            base_body["limit"] = json!(limit);
            base_body["offset"] = json!(offset);

            let resp = client.post(endpoint, base_body.clone()).await?;
            let items = extract_items(&resp);
            let page_len = items.len();

            for item in items {
                all_items.push(item);
            }

            if page_len == 0 || page_len < limit as usize {
                break;
            }
            offset += limit;
        }
        sp.stop();

        let value = json!({
            "data": all_items,
            "pagination": {
                "limit": limit,
                "offset": 0,
                "total": null,
                "hasMore": false,
            },
            "success": true,
        });
        if let Some(contract) = contract {
            output::emit_with_contract(contract, value, format, fields)?;
        } else {
            output::emit(command, value, format, fields)?;
        }
    } else {
        // Streaming NDJSON mode: no spinner (it would interleave with stdout output)
        let mut offset: u32 = 0;
        loop {
            base_body["limit"] = json!(limit);
            base_body["offset"] = json!(offset);

            let resp = client.post(endpoint, base_body.clone()).await?;
            let items = extract_items(&resp);

            if items.is_empty() {
                break;
            }

            let page_len = items.len();
            for item in items {
                let item = if let Some(contract) = contract {
                    output::apply_fields_with_contract(json!({"data": [item]}), fields, contract)
                        .get("data")
                        .and_then(Value::as_array)
                        .and_then(|rows| rows.first())
                        .cloned()
                        .unwrap_or(Value::Null)
                } else {
                    output::apply_fields(item, fields)
                };
                println!("{}", serde_json::to_string(&item)?);
            }

            if page_len < limit as usize {
                break;
            }
            offset += limit;
        }
    }

    Ok(())
}

/// Extract items array from a response value.
/// Handles direct arrays and common wrapped shapes: { data, items, results, records }.
fn extract_items(resp: &Value) -> Vec<Value> {
    match resp {
        Value::Array(arr) => arr.clone(),
        Value::Object(map) => {
            for key in &["data", "items", "results", "records"] {
                if let Some(Value::Array(arr)) = map.get(*key) {
                    return arr.clone();
                }
            }
            output::eprint_warning(
                "unexpected response shape — could not find items array in response",
            );
            vec![]
        }
        _ => vec![],
    }
}
