use serde_json::json;
use std::sync::Mutex;
use waro_cli::output::apply_fields;

/// Mutex to serialize tests that mutate environment variables.
static ENV_MUTEX: Mutex<()> = Mutex::new(());

#[test]
fn apply_fields_filters_object_keys() {
    let value = json!({
        "id": "abc-123",
        "status": "completed",
        "total": 50000,
        "customer_email": "hidden@example.com"
    });

    let result = apply_fields(value, Some("id,status,total"));

    assert_eq!(result["id"], "abc-123");
    assert_eq!(result["status"], "completed");
    assert_eq!(result["total"], 50000);
    assert!(result.get("customer_email").is_none());
}

#[test]
fn apply_fields_filters_array_of_objects() {
    let value = json!([
        { "id": "1", "name": "Burger", "cost": 1000 },
        { "id": "2", "name": "Pizza",  "cost": 2000 },
    ]);

    let result = apply_fields(value, Some("id,name"));

    let arr = result.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    assert_eq!(arr[0]["id"], "1");
    assert_eq!(arr[0]["name"], "Burger");
    assert!(arr[0].get("cost").is_none());
}

#[test]
fn apply_fields_returns_value_unchanged_when_no_fields() {
    let value = json!({ "id": "1", "name": "Burger" });
    let cloned = value.clone();

    let result = apply_fields(value, None);

    assert_eq!(result, cloned);
}

#[test]
fn config_errors_without_api_key() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("WARO_API_KEY");

    let result = waro_cli::config::Config::from_env();

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("WARO_API_KEY"));
}

#[test]
fn config_uses_default_api_url_when_not_set() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("WARO_API_URL");
    std::env::set_var("WARO_API_KEY", "waro_sk_test");

    let config = waro_cli::config::Config::from_env().unwrap();

    assert_eq!(config.api_url, "https://api.warocol.com");
    assert_eq!(config.api_key, "waro_sk_test");

    std::env::remove_var("WARO_API_KEY");
}
