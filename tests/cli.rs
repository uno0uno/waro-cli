use serde_json::json;
use std::sync::Mutex;
use waro_cli::config::Config;
use waro_cli::output::apply_fields;
use waro_cli::validate::{validate_date, validate_enum, validate_uuid};

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

    let result = waro_cli::config::Config::load(None);

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("WARO_API_KEY"));
}

#[test]
fn config_uses_default_api_url_when_not_set() {
    let _guard = ENV_MUTEX.lock().unwrap();
    std::env::remove_var("WARO_API_URL");
    std::env::set_var("WARO_API_KEY", "waro_sk_test");

    let config = waro_cli::config::Config::load(None).unwrap();

    assert_eq!(config.api_url, "https://api.warocol.com");
    assert_eq!(config.api_key, "waro_sk_test");

    std::env::remove_var("WARO_API_KEY");
}

// ── validate_uuid ─────────────────────────────────────────────────────────────

#[test]
fn validate_uuid_accepts_valid() {
    assert!(validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000").is_ok());
}

#[test]
fn validate_uuid_rejects_path_traversal() {
    let err = validate_uuid("id", "../etc/passwd")
        .unwrap_err()
        .to_string();
    assert!(err.contains("path traversal"));
}

#[test]
fn validate_uuid_rejects_null_byte() {
    let err = validate_uuid("id", "550e8400-e29b-41d4\x00-a716-446655440000")
        .unwrap_err()
        .to_string();
    assert!(err.contains("null bytes"));
}

#[test]
fn validate_uuid_rejects_malformed() {
    let err = validate_uuid("id", "not-a-uuid").unwrap_err().to_string();
    assert!(err.contains("UUID format"));
}

#[test]
fn validate_uuid_rejects_query_param() {
    let err = validate_uuid("id", "550e8400-e29b-41d4-a716-446655440000?admin=1")
        .unwrap_err()
        .to_string();
    assert!(err.contains("query parameters"));
}

// ── validate_date ─────────────────────────────────────────────────────────────

#[test]
fn validate_date_accepts_valid() {
    assert!(validate_date("date-from", "2026-03-01").is_ok());
}

#[test]
fn validate_date_rejects_crlf() {
    let err = validate_date("date-from", "2026-03-01\r\nX-Bad: header")
        .unwrap_err()
        .to_string();
    assert!(err.contains("newline"));
}

#[test]
fn validate_date_rejects_wrong_format() {
    let err = validate_date("date-from", "01/03/2026")
        .unwrap_err()
        .to_string();
    assert!(err.contains("YYYY-MM-DD"));
}

#[test]
fn validate_date_rejects_invalid_month() {
    let err = validate_date("date-from", "2026-13-01")
        .unwrap_err()
        .to_string();
    assert!(err.contains("YYYY-MM-DD"));
}

// ── validate_enum ─────────────────────────────────────────────────────────────

#[test]
fn validate_enum_accepts_valid() {
    assert!(validate_enum(
        "status",
        "completed",
        &["completed", "cancelled", "pending"]
    )
    .is_ok());
}

#[test]
fn validate_enum_rejects_unknown() {
    let err = validate_enum("status", "unknown", &["completed", "cancelled", "pending"])
        .unwrap_err()
        .to_string();
    assert!(err.contains("not allowed"));
    assert!(err.contains("completed"));
}

// ── Config profile loading ────────────────────────────────────────────────────

#[test]
fn config_loads_profile_from_toml() {
    let _guard = ENV_MUTEX.lock().unwrap();

    // Write a temp config.toml under a temp HOME
    let tmp = std::env::temp_dir().join("waro_test_home");
    let waro_dir = tmp.join(".waro");
    std::fs::create_dir_all(&waro_dir).unwrap();
    std::fs::write(
        waro_dir.join("config.toml"),
        r#"
[profiles.staging]
api_url = "https://staging.example.com"
api_key  = "waro_sk_staging_test"

[profiles.prod]
api_key = "waro_sk_prod_test"
"#,
    )
    .unwrap();

    std::env::set_var("HOME", tmp.to_str().unwrap());
    std::env::remove_var("WARO_PROFILE");

    let config = Config::load(Some("staging")).unwrap();
    assert_eq!(config.api_url, "https://staging.example.com");
    assert_eq!(config.api_key, "waro_sk_staging_test");
    assert_eq!(config.profile_name.as_deref(), Some("staging"));

    // prod profile omits api_url — should get default
    let config_prod = Config::load(Some("prod")).unwrap();
    assert_eq!(config_prod.api_url, "https://api.warocol.com");
    assert_eq!(config_prod.api_key, "waro_sk_prod_test");

    std::env::remove_var("HOME");
}

#[test]
fn config_errors_on_missing_profile() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let tmp = std::env::temp_dir().join("waro_test_home2");
    let waro_dir = tmp.join(".waro");
    std::fs::create_dir_all(&waro_dir).unwrap();
    std::fs::write(
        waro_dir.join("config.toml"),
        "[profiles.prod]\napi_key = \"waro_sk_prod\"\n",
    )
    .unwrap();

    std::env::set_var("HOME", tmp.to_str().unwrap());
    std::env::remove_var("WARO_PROFILE");

    let err = Config::load(Some("nonexistent")).unwrap_err().to_string();
    assert!(err.contains("nonexistent"));
    assert!(err.contains("not found"));

    std::env::remove_var("HOME");
}

#[test]
fn config_uses_waro_profile_env_var() {
    let _guard = ENV_MUTEX.lock().unwrap();

    let tmp = std::env::temp_dir().join("waro_test_home3");
    let waro_dir = tmp.join(".waro");
    std::fs::create_dir_all(&waro_dir).unwrap();
    std::fs::write(
        waro_dir.join("config.toml"),
        "[profiles.local]\napi_key = \"waro_sk_local\"\napi_url = \"http://localhost:8000\"\n",
    )
    .unwrap();

    std::env::set_var("HOME", tmp.to_str().unwrap());
    std::env::set_var("WARO_PROFILE", "local");

    let config = Config::load(None).unwrap();
    assert_eq!(config.api_url, "http://localhost:8000");
    assert_eq!(config.api_key, "waro_sk_local");
    assert_eq!(config.profile_name.as_deref(), Some("local"));

    std::env::remove_var("WARO_PROFILE");
    std::env::remove_var("HOME");
}
