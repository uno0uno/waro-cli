use crate::client::WaroClient;
use crate::output;
use anyhow::Result;
use serde_json::{json, Value};

/// Fetch all pages from a paginated endpoint and print results as NDJSON.
///
/// Iterates offset = 0, limit, 2×limit, … until a page returns fewer items
/// than `limit` (last partial page) or an empty page.
pub async fn fetch_all(
    client: &WaroClient,
    endpoint: &str,
    mut base_body: Value,
    limit: u32,
    fields: Option<&str>,
) -> Result<()> {
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
            let item = output::apply_fields(item, fields);
            println!("{}", serde_json::to_string(&item)?);
        }

        if page_len < limit as usize {
            break;
        }

        offset += limit;
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
