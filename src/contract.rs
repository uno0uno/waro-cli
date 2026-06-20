use anyhow::{bail, Result};
use serde_json::{json, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ResponseShape {
    DataRows,
    DataObject,
    NestedRows,
    TopLevelRows,
    TopLevelObject,
    BalancesMap,
}

impl ResponseShape {
    pub fn as_str(self) -> &'static str {
        match self {
            ResponseShape::DataRows => "data_rows",
            ResponseShape::DataObject => "data_object",
            ResponseShape::NestedRows => "nested_rows",
            ResponseShape::TopLevelRows => "top_level_rows",
            ResponseShape::TopLevelObject => "top_level_object",
            ResponseShape::BalancesMap => "balances_map",
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CommandContract {
    pub command: &'static str,
    pub method: &'static str,
    pub path: &'static str,
    pub scope: &'static str,
    pub paginates: bool,
    pub shape: ResponseShape,
    pub row_path: &'static str,
    pub fields: &'static [&'static str],
    pub default_fields: &'static [&'static str],
    pub top_level_keys: &'static [&'static str],
}

impl CommandContract {
    pub fn response_json(self) -> Value {
        json!({
            "shape": self.shape.as_str(),
            "row_path": self.row_path,
            "fields": self.fields,
            "default_fields": self.default_fields,
            "top_level_keys": self.top_level_keys,
        })
    }
}

const SALES_LIST_FIELDS: &[&str] = &[
    "customer",
    "id",
    "items",
    "itemsCount",
    "orderDate",
    "orderNumber",
    "paymentMethod",
    "status",
    "totalAmount",
];
const SALES_METRICS_FIELDS: &[&str] = &[
    "avgTicket",
    "cancelledOrders",
    "completedOrders",
    "pendingOrders",
    "totalOrders",
    "totalSales",
];
const SALES_GROUPED_METRICS_FIELDS: &[&str] = &[
    "avgPrice",
    "categoryName",
    "ordersCount",
    "productId",
    "productName",
    "rank",
    "totalQuantity",
    "totalRevenue",
];
const SALES_DETAIL_FIELDS: &[&str] = &[
    "customer",
    "id",
    "items",
    "orderDate",
    "orderNumber",
    "paymentMethod",
    "status",
    "totalAmount",
];
const MENU_PRODUCTS_FIELDS: &[&str] = &[
    "allowModifiers",
    "calculatedCost",
    "category",
    "description",
    "id",
    "ingredients",
    "isAvailable",
    "modifierGroups",
    "name",
    "perceivedCost",
    "preparationTime",
    "price",
    "recipeBases",
];
const MENU_RECIPES_FIELDS: &[&str] = &[
    "createdAt",
    "description",
    "id",
    "ingredients",
    "isActive",
    "name",
    "updatedAt",
];
const MENU_MODIFIERS_FIELDS: &[&str] = &[
    "associatedProducts",
    "createdAt",
    "id",
    "isRequired",
    "maxQty",
    "minQty",
    "modifiers",
    "name",
    "updatedAt",
];
const CUSTOMERS_LIST_FIELDS: &[&str] = &[
    "avg_ticket",
    "customer_id",
    "last_order_date",
    "name",
    "order_count",
    "phone",
    "total_spent",
    "waros_balance",
];
const CUSTOMERS_ORDERS_FIELDS: &[&str] = &[
    "id",
    "items",
    "itemsCount",
    "orderDate",
    "orderNumber",
    "paymentMethod",
    "status",
    "totalAmount",
];
const CUSTOMERS_METRICS_FIELDS: &[&str] =
    &["customer_id", "name", "order_count", "phone", "total_spent"];
const CUSTOMERS_SUMMARY_FIELDS: &[&str] = &[
    "avg_orders_per_customer",
    "avg_ticket",
    "new_customers",
    "returning_customers",
    "total_customers",
    "total_revenue",
];
const ANALYTICS_MENU_FIELDS: &[&str] = &[
    "avg_price",
    "category",
    "classification",
    "cost_used_for_classification",
    "costo_percibido",
    "estimated_cost",
    "id",
    "name",
    "order_count",
    "price",
    "profit_margin_pct",
    "profit_margin_real_pct",
    "profit_per_unit",
    "total_profit",
    "total_revenue",
    "total_units_sold",
];
const ANALYTICS_ALERT_FIELDS: &[&str] = &["action", "description", "id", "title", "type"];
const ANALYTICS_DATA_QUALITY_FIELDS: &[&str] = &[
    "actual_value",
    "alert_type",
    "context",
    "corrected_value",
    "created_at",
    "deviation_pct",
    "expected_value",
    "id",
    "ingredient_id",
    "ingredient_name",
    "original_value",
    "purchase_date",
    "purchase_id",
    "purchase_item_id",
    "purchase_number",
    "resolution_note",
    "resolved",
    "resolved_at",
    "resolved_by",
    "rolling_avg",
    "severity",
    "supplier_name",
    "tenant_id",
];
const FINANCIAL_PRODUCTS_FIELDS: &[&str] = &[
    "category",
    "classification",
    "cost",
    "id",
    "last_order_date",
    "margin",
    "name",
    "order_count",
    "price",
    "profit",
    "sales",
    "tirImpact",
];
const WAROS_ESTIMATE_FIELDS: &[&str] = &["earned", "total", "tier"];
const WAROS_BALANCES_FIELDS: &[&str] = &["profile_id", "balance"];
const WAROS_CUSTOMER_FIELDS: &[&str] = &["balance", "customer", "profile_id", "tier"];

const DATA_WRAPPER_KEYS: &[&str] = &["data", "meta", "pagination", "success"];
const DATA_META_SUCCESS_KEYS: &[&str] = &["data", "meta", "success"];
const CUSTOMER_LIST_KEYS: &[&str] = &["data", "limit", "offset", "total"];
const CUSTOMER_ORDERS_KEYS: &[&str] = &["items", "limit", "offset", "total"];
const CUSTOMER_METRICS_KEYS: &[&str] = &["summary", "top_customers"];
const CUSTOMER_SERIES_KEYS: &[&str] = &["summary", "series"];
const FINANCIAL_PRODUCTS_KEYS: &[&str] =
    &["categories", "filters", "insights", "metrics", "products"];
const WAROS_BALANCES_KEYS: &[&str] = &["balances"];

pub const CONTRACTS: &[CommandContract] = &[
    CommandContract {
        command: "sales list",
        method: "POST",
        path: "/v1/sales",
        scope: "orders:read",
        paginates: true,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: SALES_LIST_FIELDS,
        default_fields: &["id", "status", "totalAmount", "orderDate"],
        top_level_keys: DATA_WRAPPER_KEYS,
    },
    CommandContract {
        command: "sales metrics",
        method: "POST",
        path: "/v1/sales/metrics",
        scope: "orders:read",
        paginates: false,
        shape: ResponseShape::DataObject,
        row_path: "data",
        fields: SALES_METRICS_FIELDS,
        default_fields: DATA_META_SUCCESS_KEYS,
        top_level_keys: DATA_META_SUCCESS_KEYS,
    },
    CommandContract {
        command: "sales detail",
        method: "POST",
        path: "/v1/sales/detail",
        scope: "orders:read",
        paginates: false,
        shape: ResponseShape::DataObject,
        row_path: "data",
        fields: SALES_DETAIL_FIELDS,
        default_fields: DATA_META_SUCCESS_KEYS,
        top_level_keys: DATA_META_SUCCESS_KEYS,
    },
    CommandContract {
        command: "customers list",
        method: "POST",
        path: "/v1/customers",
        scope: "customers:read",
        paginates: true,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: CUSTOMERS_LIST_FIELDS,
        default_fields: &["customer_id", "name", "order_count", "total_spent"],
        top_level_keys: CUSTOMER_LIST_KEYS,
    },
    CommandContract {
        command: "customers detail",
        method: "POST",
        path: "/v1/customers/detail",
        scope: "customers:read",
        paginates: false,
        shape: ResponseShape::DataObject,
        row_path: "data",
        fields: &[],
        default_fields: DATA_META_SUCCESS_KEYS,
        top_level_keys: DATA_META_SUCCESS_KEYS,
    },
    CommandContract {
        command: "customers orders",
        method: "POST",
        path: "/v1/customers/orders",
        scope: "customers:read",
        paginates: true,
        shape: ResponseShape::TopLevelRows,
        row_path: "items",
        fields: CUSTOMERS_ORDERS_FIELDS,
        default_fields: &["id", "orderNumber", "totalAmount", "orderDate"],
        top_level_keys: CUSTOMER_ORDERS_KEYS,
    },
    CommandContract {
        command: "customers metrics",
        method: "POST",
        path: "/v1/customers/metrics",
        scope: "customers:read",
        paginates: false,
        shape: ResponseShape::TopLevelRows,
        row_path: "top_customers",
        fields: CUSTOMERS_METRICS_FIELDS,
        default_fields: &["summary", "top_customers"],
        top_level_keys: CUSTOMER_METRICS_KEYS,
    },
    CommandContract {
        command: "menu products",
        method: "POST",
        path: "/v1/menu/products",
        scope: "menu:read",
        paginates: true,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: MENU_PRODUCTS_FIELDS,
        default_fields: &["id", "name", "price", "isAvailable"],
        top_level_keys: DATA_WRAPPER_KEYS,
    },
    CommandContract {
        command: "menu recipes",
        method: "POST",
        path: "/v1/menu/recipes",
        scope: "menu:read",
        paginates: true,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: MENU_RECIPES_FIELDS,
        default_fields: &["id", "name", "isActive", "ingredients"],
        top_level_keys: DATA_WRAPPER_KEYS,
    },
    CommandContract {
        command: "menu modifiers",
        method: "POST",
        path: "/v1/menu/modifiers",
        scope: "menu:read",
        paginates: true,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: MENU_MODIFIERS_FIELDS,
        default_fields: &["id", "name", "isRequired", "modifiers"],
        top_level_keys: DATA_WRAPPER_KEYS,
    },
    CommandContract {
        command: "analytics menu",
        method: "POST",
        path: "/v1/analytics/menu-analysis",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.menu_items",
        fields: ANALYTICS_MENU_FIELDS,
        default_fields: &["data", "success"],
        top_level_keys: &["data", "success"],
    },
    CommandContract {
        command: "analytics food-cost",
        method: "POST",
        path: "/v1/analytics/food-cost",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::DataRows,
        row_path: "data",
        fields: ANALYTICS_MENU_FIELDS,
        default_fields: DATA_META_SUCCESS_KEYS,
        top_level_keys: DATA_META_SUCCESS_KEYS,
    },
    CommandContract {
        command: "analytics alerts",
        method: "POST",
        path: "/v1/analytics/alerts",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.alerts",
        fields: ANALYTICS_ALERT_FIELDS,
        default_fields: &["data", "success"],
        top_level_keys: &["data", "success"],
    },
    CommandContract {
        command: "analytics data-quality",
        method: "POST",
        path: "/v1/analytics/data-quality",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.alerts",
        fields: ANALYTICS_DATA_QUALITY_FIELDS,
        default_fields: &["data", "success"],
        top_level_keys: &["data", "success"],
    },
    CommandContract {
        command: "financial products",
        method: "POST",
        path: "/v1/financial/products",
        scope: "financial:read",
        paginates: false,
        shape: ResponseShape::TopLevelRows,
        row_path: "products",
        fields: FINANCIAL_PRODUCTS_FIELDS,
        default_fields: &["products", "metrics", "insights"],
        top_level_keys: FINANCIAL_PRODUCTS_KEYS,
    },
    CommandContract {
        command: "waros estimate",
        method: "POST",
        path: "/v1/waros/estimate",
        scope: "waros:read",
        paginates: false,
        shape: ResponseShape::TopLevelObject,
        row_path: "$",
        fields: WAROS_ESTIMATE_FIELDS,
        default_fields: WAROS_ESTIMATE_FIELDS,
        top_level_keys: WAROS_ESTIMATE_FIELDS,
    },
    CommandContract {
        command: "waros balances",
        method: "POST",
        path: "/v1/waros/balances",
        scope: "waros:read",
        paginates: false,
        shape: ResponseShape::BalancesMap,
        row_path: "balances",
        fields: WAROS_BALANCES_FIELDS,
        default_fields: WAROS_BALANCES_FIELDS,
        top_level_keys: WAROS_BALANCES_KEYS,
    },
    CommandContract {
        command: "waros customer",
        method: "POST",
        path: "/v1/waros/customer-summary",
        scope: "waros:read",
        paginates: false,
        shape: ResponseShape::TopLevelObject,
        row_path: "$",
        fields: WAROS_CUSTOMER_FIELDS,
        default_fields: WAROS_CUSTOMER_FIELDS,
        top_level_keys: WAROS_CUSTOMER_FIELDS,
    },
];

#[allow(dead_code)]
pub fn all_contracts() -> &'static [CommandContract] {
    CONTRACTS
}

pub fn contract_for(command: &str) -> Option<CommandContract> {
    CONTRACTS
        .iter()
        .copied()
        .find(|contract| contract.command == command)
}

pub fn validate_fields(
    contract: CommandContract,
    fields: Option<&str>,
) -> Result<Option<Vec<String>>> {
    let Some(fields_str) = fields else {
        return Ok(None);
    };
    let requested: Vec<String> = fields_str
        .split(',')
        .map(str::trim)
        .filter(|field| !field.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if requested.is_empty() {
        return Ok(Some(Vec::new()));
    }

    let allowed: Vec<&str> = contract
        .fields
        .iter()
        .chain(contract.default_fields.iter())
        .copied()
        .fold(Vec::new(), |mut acc, field| {
            if !acc.contains(&field) {
                acc.push(field);
            }
            acc
        });
    let unknown: Vec<&str> = requested
        .iter()
        .map(String::as_str)
        .filter(|field| !allowed.contains(field))
        .collect();

    if !unknown.is_empty() {
        let suggestions: Vec<String> = unknown
            .iter()
            .filter_map(|field| {
                closest_field(field, &allowed).map(|candidate| format!("{field}->{candidate}"))
            })
            .collect();
        let hint = if suggestions.is_empty() {
            format!("Available fields: {}", allowed.join(", "))
        } else {
            format!(
                "Suggestions: {}. Available fields: {}",
                suggestions.join(", "),
                allowed.join(", ")
            )
        };
        bail!(
            "unknown field(s) for {}: {}. {}",
            contract.command,
            unknown.join(", "),
            hint
        );
    }

    Ok(Some(requested))
}

fn closest_field<'a>(field: &str, candidates: &'a [&str]) -> Option<&'a str> {
    candidates
        .iter()
        .copied()
        .min_by_key(|candidate| levenshtein(field, candidate))
        .filter(|candidate| levenshtein(field, candidate) <= 2)
}

fn levenshtein(a: &str, b: &str) -> usize {
    let mut costs: Vec<usize> = (0..=b.chars().count()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut last = i;
        costs[0] = i + 1;
        for (j, cb) in b.chars().enumerate() {
            let old = costs[j + 1];
            let cost = if ca == cb { last } else { last + 1 };
            costs[j + 1] = (costs[j] + 1).min(old + 1).min(cost);
            last = old;
        }
    }
    *costs.last().unwrap_or(&0)
}

pub fn dynamic_contract_for_metrics(
    command: &str,
    group_by: Option<&str>,
) -> Option<CommandContract> {
    let mut contract = contract_for(command)?;
    if command == "sales metrics" && group_by == Some("product") {
        contract.shape = ResponseShape::DataRows;
        contract.row_path = "data";
        contract.fields = SALES_GROUPED_METRICS_FIELDS;
        contract.default_fields = DATA_META_SUCCESS_KEYS;
    }
    if command == "customers metrics" && group_by.is_some() {
        contract.shape = ResponseShape::TopLevelRows;
        contract.row_path = "series";
        contract.fields = CUSTOMERS_SUMMARY_FIELDS;
        contract.default_fields = CUSTOMER_SERIES_KEYS;
        contract.top_level_keys = CUSTOMER_SERIES_KEYS;
    }
    Some(contract)
}
