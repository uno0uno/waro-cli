use serde_json::json;
use std::sync::Mutex;
use waro_cli::config::Config;
use waro_cli::contract::{self, ResponseShape};
use waro_cli::output::{apply_fields, apply_fields_with_contract, rows_for_contract};
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
fn contract_registry_covers_schema_commands() {
    let commands = [
        "sales list",
        "sales metrics",
        "sales detail",
        "customers list",
        "customers detail",
        "customers metrics",
        "customers orders",
        "menu products",
        "menu recipes",
        "menu modifiers",
        "analytics menu",
        "analytics food-cost",
        "analytics alerts",
        "analytics data-quality",
        "analytics cohort",
        "analytics waros",
        "analytics rfm",
        "analytics churn-risk",
        "financial products",
        "waros estimate",
        "waros balances",
        "waros customer",
        "queries schema",
        "queries run",
    ];

    for command in commands {
        let contract = contract::contract_for(command)
            .unwrap_or_else(|| panic!("missing contract for {command}"));
        let response = contract.response_json();
        assert!(response.get("shape").is_some());
        assert!(response.get("row_path").is_some());
        assert!(response.get("fields").is_some());
        assert!(response.get("default_fields").is_some());
        assert!(response.get("top_level_keys").is_some());

        let metadata = contract.metadata_json();
        assert!(metadata.get("domain").is_some());
        assert!(metadata.get("description").is_some());
        assert!(metadata.get("tags").is_some());
        assert!(metadata.get("examples").is_some());
        assert!(metadata.get("capabilities").is_some());

        let capabilities = &metadata["capabilities"];
        assert!(capabilities.get("entity").is_some());
        assert!(capabilities.get("grain").is_some());
        assert!(capabilities.get("measures").is_some());
        assert!(capabilities.get("dimensions").is_some());
        assert!(capabilities.get("supported_operations").is_some());
        assert!(capabilities.get("semantic_aliases").is_some());
        assert!(capabilities.get("answer_patterns").is_some());
    }
}

#[test]
fn semantic_capabilities_describe_customer_rankings() {
    let contract = contract::contract_for("customers list").unwrap();
    let metadata = contract.metadata_json();

    assert_eq!(metadata["domain"], "customers");
    assert_eq!(metadata["capabilities"]["entity"], "customer");
    assert_eq!(metadata["capabilities"]["grain"], "customer_period");

    let total_spent_aliases = metadata["capabilities"]["semantic_aliases"]["total_spent"]
        .as_array()
        .unwrap();
    assert!(total_spent_aliases.contains(&json!("valor comprado")));
    assert!(total_spent_aliases.contains(&json!("mejores clientes")));

    let order_count_aliases = metadata["capabilities"]["semantic_aliases"]["order_count"]
        .as_array()
        .unwrap();
    assert!(order_count_aliases.contains(&json!("clientes frecuentes")));
}

#[test]
fn semantic_capabilities_separate_order_rows_from_product_margin_analysis() {
    let sales_list = contract::contract_for("sales list")
        .unwrap()
        .metadata_json();
    assert_eq!(sales_list["capabilities"]["entity"], "order");
    assert!(sales_list["capabilities"]["cannot_answer"]
        .as_array()
        .unwrap()
        .contains(&json!("product_margin_analysis")));

    let food_cost = contract::contract_for("analytics food-cost")
        .unwrap()
        .metadata_json();
    assert_eq!(food_cost["capabilities"]["entity"], "product");
    assert_eq!(food_cost["capabilities"]["grain"], "product_period");
    assert!(food_cost["capabilities"]["measures"]
        .as_array()
        .unwrap()
        .contains(&json!("profit_margin_pct")));
    assert!(food_cost["capabilities"]["measures"]
        .as_array()
        .unwrap()
        .contains(&json!("total_units_sold")));
}

#[test]
fn contract_rejects_unknown_row_fields_with_suggestion() {
    let contract = contract::contract_for("customers list").unwrap();

    let err = contract::validate_fields(contract, Some("customer_id,data,nmae"))
        .unwrap_err()
        .to_string();

    assert!(err.contains("data"));
    assert!(err.contains("nmae->name"));
}

#[test]
fn data_rows_contract_filters_rows_and_rejects_wrapper_field() {
    let contract = contract::contract_for("customers list").unwrap();
    let value = json!({
        "data": [
            { "customer_id": "c1", "name": "Ana", "phone": "555", "total_spent": 1000 }
        ],
        "total": 1,
        "limit": 50,
        "offset": 0
    });

    assert!(contract::validate_fields(contract, Some("data")).is_err());
    let filtered = apply_fields_with_contract(value, Some("customer_id,name"), contract);
    assert_eq!(filtered["data"][0]["customer_id"], "c1");
    assert_eq!(filtered["data"][0]["name"], "Ana");
    assert!(filtered["data"][0].get("phone").is_none());
}

#[test]
fn rows_for_contract_extracts_supported_shapes() {
    let data_rows = contract::contract_for("sales list").unwrap();
    assert_eq!(data_rows.shape, ResponseShape::DataRows);
    let rows = rows_for_contract(
        &json!({ "data": [{ "id": "o1" }, { "id": "o2" }] }),
        data_rows,
    );
    assert_eq!(rows.len(), 2);

    let nested_rows = contract::contract_for("analytics menu").unwrap();
    assert_eq!(nested_rows.shape, ResponseShape::NestedRows);
    let rows = rows_for_contract(
        &json!({ "data": { "menu_items": [{ "id": "p1" }] }, "success": true }),
        nested_rows,
    );
    assert_eq!(rows[0]["id"], "p1");

    let top_rows = contract::contract_for("customers metrics").unwrap();
    assert_eq!(top_rows.shape, ResponseShape::TopLevelRows);
    let rows = rows_for_contract(
        &json!({ "summary": {}, "top_customers": [{ "customer_id": "c1" }] }),
        top_rows,
    );
    assert_eq!(rows[0]["customer_id"], "c1");

    let products = contract::contract_for("financial products").unwrap();
    let rows = rows_for_contract(
        &json!({ "products": [{ "id": "p1", "margin": 42 }], "metrics": {} }),
        products,
    );
    assert_eq!(rows[0]["margin"], 42);

    let balances = contract::contract_for("waros balances").unwrap();
    let rows = rows_for_contract(
        &json!({ "balances": { "profile-a": 120, "profile-b": 80 } }),
        balances,
    );
    assert_eq!(rows.len(), 2);

    let cohort = contract::contract_for("analytics cohort").unwrap();
    let rows = rows_for_contract(
        &json!({ "cohorts": [{ "cohort_label": "2026-W24", "cohort_size": 12 }] }),
        cohort,
    );
    assert_eq!(rows[0]["cohort_size"], 12);

    let rfm = contract::contract_for("analytics rfm").unwrap();
    let rows = rows_for_contract(
        &json!({ "data": { "customers": [{ "customer_id": "c1", "segment": "Champions" }] } }),
        rfm,
    );
    assert_eq!(rows[0]["segment"], "Champions");

    let churn = contract::contract_for("analytics churn-risk").unwrap();
    let rows = rows_for_contract(
        &json!({ "customers": [{ "customer_id": "c1", "risk_score": 0.8 }] }),
        churn,
    );
    assert_eq!(rows[0]["risk_score"], 0.8);

    let queries_schema = contract::contract_for("queries schema").unwrap();
    assert_eq!(queries_schema.shape, ResponseShape::NestedRows);
    let rows = rows_for_contract(
        &json!({ "success": true, "data": { "datasets": [{ "name": "sales_items" }] } }),
        queries_schema,
    );
    assert_eq!(rows[0]["name"], "sales_items");

    let queries_run = contract::contract_for("queries run").unwrap();
    assert_eq!(queries_run.shape, ResponseShape::NestedRows);
    let rows = rows_for_contract(
        &json!({ "success": true, "data": { "rows": [{ "product": "Burger", "revenue": 120000 }], "columns": ["product", "revenue"] } }),
        queries_run,
    );
    assert_eq!(rows[0]["product"], "Burger");
}

#[test]
fn config_errors_without_api_key() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let tmp = std::env::temp_dir().join("waro_test_no_config_home");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::env::set_var("HOME", tmp.to_str().unwrap());
    std::env::remove_var("WARO_API_KEY");
    std::env::remove_var("WARO_API_URL");
    std::env::remove_var("WARO_PROFILE");

    let result = waro_cli::config::Config::load(None);

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("WARO_API_KEY"));
    std::env::remove_var("HOME");
}

#[test]
fn config_uses_default_api_url_when_not_set() {
    let _guard = ENV_MUTEX.lock().unwrap();
    let tmp = std::env::temp_dir().join("waro_test_env_home");
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    std::env::set_var("HOME", tmp.to_str().unwrap());
    std::env::remove_var("WARO_API_URL");
    std::env::remove_var("WARO_PROFILE");
    std::env::set_var("WARO_API_KEY", "waro_sk_test");

    let config = waro_cli::config::Config::load(None).unwrap();

    assert_eq!(config.api_url, "https://api.warolabs.com");
    assert_eq!(config.api_key, "waro_sk_test");

    std::env::remove_var("WARO_API_KEY");
    std::env::remove_var("HOME");
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
    assert_eq!(config_prod.api_url, "https://api.warolabs.com");
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
