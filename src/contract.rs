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

    pub fn metadata_json(self) -> Value {
        semantic_metadata_for(self.command).unwrap_or_else(|| {
            let domain = self.command.split_whitespace().next().unwrap_or("unknown");
            json!({
                "domain": domain,
                "description": format!("WARO {} command.", self.command),
                "tags": [domain],
                "examples": [],
                "capabilities": {
                    "entity": domain,
                    "grain": "unknown",
                    "measures": [],
                    "dimensions": self.fields,
                    "supported_operations": ["filter", "limit"],
                    "default_rank": [],
                    "active_condition": [],
                    "supports_period": self.fields.iter().any(|field| *field == "orderDate" || *field == "last_order_date"),
                    "semantic_aliases": {},
                    "answer_patterns": [],
                    "join_keys": [],
                    "cannot_answer": []
                }
            })
        })
    }
}

fn semantic_metadata_for(command: &str) -> Option<Value> {
    let metadata = match command {
        "sales list" => json!({
            "domain": "sales",
            "description": "Lista ordenes individuales con estado, fecha, metodo de pago, cliente, items y total.",
            "tags": ["sales", "orders", "transactions", "payments"],
            "examples": ["ordenes de ayer", "ventas canceladas", "ultimas ventas", "pedidos por metodo de pago"],
            "capabilities": {
                "entity": "order",
                "grain": "order",
                "measures": ["totalAmount", "itemsCount"],
                "dimensions": ["id", "orderNumber", "status", "paymentMethod", "orderDate", "customer", "items"],
                "supported_operations": ["filter", "sort", "limit", "list"],
                "default_rank": ["orderDate"],
                "active_condition": ["status=completed"],
                "supports_period": true,
                "semantic_aliases": {
                    "totalAmount": ["valor", "total", "monto", "venta individual"],
                    "itemsCount": ["items", "productos por orden", "cantidad de items"],
                    "orderDate": ["fecha", "ayer", "hoy", "este mes", "periodo"],
                    "status": ["estado", "canceladas", "pendientes", "completadas"]
                },
                "answer_patterns": ["listar ordenes", "ver pedidos", "ventas individuales", "pedidos cancelados"],
                "join_keys": ["id", "customer.id"],
                "cannot_answer": ["aggregate_sales_metrics", "product_margin_analysis", "customer_period_ranking"]
            }
        }),
        "sales metrics" => json!({
            "domain": "sales",
            "description": "Metricas agregadas de ventas por periodo y agrupaciones de fecha, hora, producto, pago o ticket.",
            "tags": ["sales", "metrics", "revenue", "avg_ticket", "product_sales"],
            "examples": ["cuanto vendi ayer", "ticket promedio", "ventas por hora", "productos mas vendidos"],
            "capabilities": {
                "entity": "sale",
                "grain": "period_or_group",
                "measures": ["totalSales", "totalOrders", "avgTicket", "totalQuantity", "totalRevenue", "ordersCount"],
                "dimensions": ["date", "weekday", "hour", "product", "payment", "ticket"],
                "supported_operations": ["aggregate", "group", "rank", "sort", "limit", "compare"],
                "default_rank": ["totalSales", "totalRevenue", "totalQuantity"],
                "active_condition": ["completedOrders", "totalSales"],
                "supports_period": true,
                "semantic_aliases": {
                    "totalSales": ["ventas", "vendi", "ingresos", "facturacion"],
                    "avgTicket": ["ticket promedio", "promedio por orden", "valor promedio"],
                    "totalOrders": ["ordenes", "pedidos", "transacciones"],
                    "totalQuantity": ["cantidad vendida", "unidades vendidas", "volumen"],
                    "totalRevenue": ["revenue", "ingresos por producto", "valor vendido"]
                },
                "answer_patterns": ["cuanto vendi", "ticket promedio", "ventas agrupadas", "productos mas vendidos"],
                "join_keys": ["productId"],
                "cannot_answer": ["product_cost_or_margin_without_financial_data", "customer_profile_details"]
            }
        }),
        "sales detail" => json!({
            "domain": "sales",
            "description": "Detalle de una orden especifica por UUID.",
            "tags": ["sales", "orders", "detail"],
            "examples": ["detalle de la orden", "items de una venta"],
            "capabilities": {
                "entity": "order",
                "grain": "order_detail",
                "measures": ["totalAmount"],
                "dimensions": ["id", "orderNumber", "status", "paymentMethod", "orderDate", "customer", "items"],
                "supported_operations": ["lookup"],
                "default_rank": [],
                "active_condition": [],
                "supports_period": false,
                "semantic_aliases": {
                    "order-id": ["orden", "pedido", "venta especifica"]
                },
                "answer_patterns": ["detalle de orden", "ver una venta"],
                "join_keys": ["id", "customer.id"],
                "cannot_answer": ["aggregate_metrics", "rankings"]
            }
        }),
        "customers list" => json!({
            "domain": "customers",
            "description": "Lista clientes con frecuencia, valor comprado, ticket promedio, ultima compra y saldo WAROS.",
            "tags": ["customers", "ranking", "frequency", "spend", "loyalty"],
            "examples": ["clientes que mas compraron", "clientes frecuentes", "mejores clientes", "clientes con mayor ticket"],
            "capabilities": {
                "entity": "customer",
                "grain": "customer_period",
                "measures": ["total_spent", "order_count", "avg_ticket", "waros_balance"],
                "dimensions": ["customer_id", "name", "phone", "last_order_date"],
                "supported_operations": ["filter", "rank", "sort", "limit", "compare", "list"],
                "default_rank": ["total_spent", "order_count"],
                "active_condition": ["order_count", "total_spent"],
                "supports_period": true,
                "semantic_aliases": {
                    "total_spent": ["valor comprado", "mayor compra", "compraron", "gasto", "dinero comprado", "mejores clientes"],
                    "order_count": ["frecuencia", "clientes frecuentes", "ordenes", "pedidos", "recompra"],
                    "avg_ticket": ["ticket promedio", "promedio por cliente"],
                    "last_order_date": ["ultima compra", "recientes", "inactivos"]
                },
                "answer_patterns": ["clientes mas frecuentes", "clientes que mas compraron", "mejores clientes", "ranking de clientes"],
                "join_keys": ["customer_id"],
                "cannot_answer": ["order_item_margin", "product_cost_analysis"]
            }
        }),
        "customers detail" => json!({
            "domain": "customers",
            "description": "Detalle e historial de un cliente especifico.",
            "tags": ["customers", "detail", "orders"],
            "examples": ["historial de cliente", "detalle de cliente"],
            "capabilities": {
                "entity": "customer",
                "grain": "customer_detail",
                "measures": ["total_spent", "order_count", "avg_ticket"],
                "dimensions": ["customer_id", "orders"],
                "supported_operations": ["lookup", "summarize"],
                "default_rank": [],
                "active_condition": [],
                "supports_period": true,
                "semantic_aliases": {
                    "customer-id": ["cliente", "perfil", "historial"]
                },
                "answer_patterns": ["detalle de cliente", "historial de cliente"],
                "join_keys": ["customer_id"],
                "cannot_answer": ["global_customer_ranking_without_customer_id"]
            }
        }),
        "customers orders" => json!({
            "domain": "customers",
            "description": "Ordenes de un cliente especifico por periodo.",
            "tags": ["customers", "orders", "history"],
            "examples": ["ordenes de este cliente", "compras de un cliente"],
            "capabilities": {
                "entity": "customer_order",
                "grain": "order",
                "measures": ["totalAmount", "itemsCount"],
                "dimensions": ["customer_id", "id", "orderNumber", "status", "paymentMethod", "orderDate"],
                "supported_operations": ["lookup", "filter", "list"],
                "default_rank": ["orderDate"],
                "active_condition": ["status=completed"],
                "supports_period": true,
                "semantic_aliases": {
                    "totalAmount": ["monto comprado", "valor de compra"],
                    "orderDate": ["fecha de compra", "historial"]
                },
                "answer_patterns": ["ordenes del cliente", "historial de compras"],
                "join_keys": ["customer_id", "id"],
                "cannot_answer": ["all_customer_ranking"]
            }
        }),
        "customers metrics" => json!({
            "domain": "customers",
            "description": "Resumen de clientes, nuevos vs recurrentes, series temporales y top customers.",
            "tags": ["customers", "metrics", "retention", "ranking"],
            "examples": ["clientes nuevos este mes", "clientes recurrentes", "top clientes", "retencion de clientes"],
            "capabilities": {
                "entity": "customer",
                "grain": "customer_period_summary",
                "measures": ["total_customers", "new_customers", "returning_customers", "total_revenue", "avg_ticket", "order_count", "total_spent"],
                "dimensions": ["date", "weekday", "month", "customer_id", "name", "phone"],
                "supported_operations": ["aggregate", "group", "rank", "compare", "summarize"],
                "default_rank": ["total_spent", "order_count"],
                "active_condition": ["returning_customers", "order_count", "total_spent"],
                "supports_period": true,
                "semantic_aliases": {
                    "new_customers": ["clientes nuevos", "nuevos"],
                    "returning_customers": ["clientes recurrentes", "clientes que vuelven"],
                    "order_count": ["frecuencia", "ordenes", "pedidos"],
                    "total_spent": ["valor comprado", "mayor compra", "gasto"]
                },
                "answer_patterns": ["metricas de clientes", "retencion", "clientes nuevos vs recurrentes", "top clientes"],
                "join_keys": ["customer_id"],
                "cannot_answer": ["product_margin_analysis"]
            }
        }),
        "menu products" => json!({
            "domain": "menu",
            "description": "Catalogo actual de productos, precios, costos configurados, recetas, ingredientes, modificadores y disponibilidad.",
            "tags": ["menu", "products", "catalog", "recipes", "availability"],
            "examples": ["productos disponibles", "precios del menu", "productos sin ingredientes", "costos configurados"],
            "capabilities": {
                "entity": "product",
                "grain": "product",
                "measures": ["price", "calculatedCost", "perceivedCost", "preparationTime"],
                "dimensions": ["id", "name", "category", "isAvailable", "ingredients", "recipeBases", "modifierGroups"],
                "supported_operations": ["filter", "lookup", "list", "summarize"],
                "default_rank": ["name"],
                "active_condition": ["isAvailable=true"],
                "supports_period": false,
                "semantic_aliases": {
                    "isAvailable": ["disponible", "activo", "vendible"],
                    "price": ["precio", "valor del producto"],
                    "calculatedCost": ["costo calculado", "costo receta"],
                    "perceivedCost": ["costo percibido"]
                },
                "answer_patterns": ["catalogo de productos", "menu disponible", "precios del menu"],
                "join_keys": ["id"],
                "cannot_answer": ["period_sales_without_sales_or_analytics", "margin_from_live_sales"]
            }
        }),
        "menu recipes" => json!({
            "domain": "menu",
            "description": "Recetas activas e ingredientes configurados.",
            "tags": ["menu", "recipes", "ingredients"],
            "examples": ["recetas activas", "ingredientes por receta"],
            "capabilities": {
                "entity": "recipe",
                "grain": "recipe",
                "measures": [],
                "dimensions": ["id", "name", "description", "ingredients", "isActive"],
                "supported_operations": ["filter", "lookup", "list"],
                "default_rank": ["name"],
                "active_condition": ["isActive=true"],
                "supports_period": false,
                "semantic_aliases": {
                    "ingredients": ["ingredientes", "insumos"],
                    "isActive": ["activa", "inactiva"]
                },
                "answer_patterns": ["recetas", "ingredientes de recetas"],
                "join_keys": ["id"],
                "cannot_answer": ["sales_metrics", "customer_metrics"]
            }
        }),
        "menu modifiers" => json!({
            "domain": "menu",
            "description": "Grupos de modificadores del menu y productos asociados.",
            "tags": ["menu", "modifiers", "addons"],
            "examples": ["modificadores", "adiciones", "opciones del producto"],
            "capabilities": {
                "entity": "modifier_group",
                "grain": "modifier_group",
                "measures": ["minQty", "maxQty"],
                "dimensions": ["id", "name", "isRequired", "modifiers", "associatedProducts"],
                "supported_operations": ["filter", "lookup", "list"],
                "default_rank": ["name"],
                "active_condition": [],
                "supports_period": false,
                "semantic_aliases": {
                    "modifiers": ["modificadores", "adiciones", "opciones"],
                    "isRequired": ["obligatorio", "requerido"]
                },
                "answer_patterns": ["modificadores del menu", "adiciones"],
                "join_keys": ["id"],
                "cannot_answer": ["sales_metrics", "customer_metrics"]
            }
        }),
        "analytics menu" => json!({
            "domain": "analytics",
            "description": "Analisis de performance del menu por producto: unidades vendidas, ingresos, costos estimados, ganancia y margen.",
            "tags": ["analytics", "menu", "products", "margin", "sales"],
            "examples": ["productos vendidos mucho con bajo margen", "productos estrella", "rentabilidad del menu", "productos con baja ganancia"],
            "capabilities": {
                "entity": "product",
                "grain": "product_period",
                "measures": ["total_units_sold", "total_revenue", "profit_margin_pct", "profit_margin_real_pct", "profit_per_unit", "total_profit", "estimated_cost", "price"],
                "dimensions": ["id", "name", "category", "classification"],
                "supported_operations": ["aggregate", "filter", "rank", "sort", "limit", "diagnose"],
                "default_rank": ["total_units_sold", "total_revenue", "profit_margin_pct"],
                "active_condition": ["total_units_sold", "total_revenue"],
                "supports_period": true,
                "semantic_aliases": {
                    "total_units_sold": ["vendieron mucho", "mas vendidos", "unidades vendidas", "volumen"],
                    "total_revenue": ["ingresos", "revenue", "valor vendido"],
                    "profit_margin_pct": ["margen", "margen bajo", "rentabilidad"],
                    "profit_per_unit": ["ganancia por unidad", "utilidad unitaria"],
                    "classification": ["estrella", "perro", "vaca", "incognita"]
                },
                "answer_patterns": ["productos con bajo margen", "productos que venden mucho", "rentabilidad de productos", "menu engineering"],
                "join_keys": ["id", "name"],
                "cannot_answer": ["customer_ranking"]
            }
        }),
        "analytics food-cost" => json!({
            "domain": "analytics",
            "description": "Food cost y margen por producto con datos de costo, precio, venta y rentabilidad.",
            "tags": ["analytics", "food_cost", "products", "margin", "cost"],
            "examples": ["food cost por producto", "productos con bajo margen", "costo de productos", "rentabilidad por producto"],
            "capabilities": {
                "entity": "product",
                "grain": "product_period",
                "measures": ["profit_margin_pct", "profit_margin_real_pct", "estimated_cost", "profit_per_unit", "total_profit", "total_revenue", "total_units_sold"],
                "dimensions": ["id", "name", "category", "classification"],
                "supported_operations": ["filter", "rank", "sort", "limit", "diagnose"],
                "default_rank": ["profit_margin_pct", "total_units_sold"],
                "active_condition": ["total_units_sold", "total_revenue"],
                "supports_period": true,
                "semantic_aliases": {
                    "profit_margin_pct": ["margen", "bajo margen", "rentabilidad"],
                    "estimated_cost": ["costo", "food cost", "costo estimado"],
                    "total_units_sold": ["vendieron mucho", "unidades vendidas", "volumen"],
                    "total_revenue": ["ingresos", "valor vendido"]
                },
                "answer_patterns": ["productos vendidos mucho con bajo margen", "food cost", "margen de productos"],
                "join_keys": ["id", "name"],
                "cannot_answer": ["customer_ranking"]
            }
        }),
        "analytics alerts" => json!({
            "domain": "analytics",
            "description": "Alertas analiticas y recomendaciones accionables.",
            "tags": ["analytics", "alerts", "recommendations"],
            "examples": ["alertas del negocio", "recomendaciones", "problemas detectados"],
            "capabilities": {
                "entity": "alert",
                "grain": "alert",
                "measures": [],
                "dimensions": ["id", "type", "title", "description", "action"],
                "supported_operations": ["list", "summarize", "diagnose"],
                "default_rank": [],
                "active_condition": [],
                "supports_period": false,
                "semantic_aliases": {
                    "action": ["accion", "recomendacion"],
                    "type": ["tipo de alerta", "categoria"]
                },
                "answer_patterns": ["alertas", "recomendaciones", "diagnostico"],
                "join_keys": ["id"],
                "cannot_answer": ["raw_sales_metrics"]
            }
        }),
        "analytics data-quality" => json!({
            "domain": "analytics",
            "description": "Alertas de calidad de datos en compras, ingredientes, proveedores y desviaciones de costo.",
            "tags": ["analytics", "data_quality", "purchases", "ingredients", "cost"],
            "examples": ["problemas de calidad de datos", "compras con desviacion", "alertas de costos"],
            "capabilities": {
                "entity": "data_quality_alert",
                "grain": "alert",
                "measures": ["actual_value", "expected_value", "deviation_pct", "rolling_avg"],
                "dimensions": ["ingredient_id", "ingredient_name", "supplier_name", "purchase_date", "severity", "resolved"],
                "supported_operations": ["filter", "list", "diagnose", "summarize"],
                "default_rank": ["severity", "deviation_pct"],
                "active_condition": ["resolved=false"],
                "supports_period": false,
                "semantic_aliases": {
                    "deviation_pct": ["desviacion", "variacion", "anomalia"],
                    "severity": ["severidad", "critico", "alerta"],
                    "resolved": ["resuelto", "pendiente"]
                },
                "answer_patterns": ["calidad de datos", "alertas de compras", "costos anomalos"],
                "join_keys": ["ingredient_id", "purchase_id"],
                "cannot_answer": ["customer_ranking", "sales_total"]
            }
        }),
        "analytics cohort" => json!({
            "domain": "analytics",
            "description": "Matriz de cohortes de retencion por semana o mes, con tamano inicial y porcentaje de regreso por periodo.",
            "tags": ["analytics", "retention", "cohort", "customers"],
            "examples": ["retencion por cohortes", "cohortes semanales", "clientes que regresan por mes"],
            "capabilities": {
                "entity": "customer",
                "grain": "cohort_period",
                "measures": ["cohort_size", "retention_count", "retention_pct"],
                "dimensions": ["cohort_date", "cohort_label", "period", "retention_period"],
                "supported_operations": ["aggregate", "compare", "summarize", "diagnose"],
                "default_rank": ["cohort_date"],
                "active_condition": ["cohort_size", "retention_pct"],
                "supports_period": true,
                "semantic_aliases": {
                    "cohort_size": ["tamano de cohorte", "clientes iniciales"],
                    "retention_pct": ["retencion", "porcentaje que regresa", "clientes que vuelven"],
                    "period": ["semanal", "mensual", "cohorte"]
                },
                "answer_patterns": ["retencion por cohortes", "cohortes de clientes", "clientes que regresan"],
                "join_keys": ["cohort_date"],
                "cannot_answer": ["product_margin_analysis", "single_customer_lookup"]
            }
        }),
        "analytics waros" => json!({
            "domain": "analytics",
            "description": "Analitica del programa WAROS: puntos emitidos, redimidos, tasa de redencion, miembros activos y agrupacion por dia, semana o cliente.",
            "tags": ["analytics", "waros", "loyalty", "points", "redemption"],
            "examples": ["waros emitidos", "redenciones por semana", "clientes con mas waros", "tasa de redencion"],
            "capabilities": {
                "entity": "loyalty_transaction",
                "grain": "period_or_customer",
                "measures": ["total_issued", "total_redeemed", "redemption_rate_pct", "active_members", "transaction_count"],
                "dimensions": ["period", "customer", "name", "groupBy"],
                "supported_operations": ["aggregate", "group", "rank", "compare", "summarize"],
                "default_rank": ["total_issued", "total_redeemed"],
                "active_condition": ["total_issued", "total_redeemed", "active_members"],
                "supports_period": true,
                "semantic_aliases": {
                    "total_issued": ["waros emitidos", "puntos ganados", "puntos entregados"],
                    "total_redeemed": ["waros redimidos", "puntos usados", "redenciones"],
                    "redemption_rate_pct": ["tasa de redencion", "porcentaje redimido"],
                    "active_members": ["miembros activos", "clientes activos"]
                },
                "answer_patterns": ["analitica waros", "redencion de puntos", "waros por cliente", "waros por periodo"],
                "join_keys": ["customer_id", "period"],
                "cannot_answer": ["sales_without_loyalty_context", "product_margin_analysis"]
            }
        }),
        "analytics rfm" => json!({
            "domain": "analytics",
            "description": "Segmentacion RFM de clientes por recencia, frecuencia y valor monetario.",
            "tags": ["analytics", "rfm", "customers", "segmentation", "retention"],
            "examples": ["clientes champions", "segmentacion rfm", "clientes en riesgo", "clientes perdidos"],
            "capabilities": {
                "entity": "customer",
                "grain": "customer_period_segment",
                "measures": ["r_score", "f_score", "m_score", "order_count", "total_spent", "recency_days"],
                "dimensions": ["customer_id", "customer_name", "segment", "last_order_date"],
                "supported_operations": ["segment", "filter", "rank", "summarize", "diagnose"],
                "default_rank": ["segment", "m_score", "f_score"],
                "active_condition": ["order_count", "total_spent", "last_order_date"],
                "supports_period": true,
                "semantic_aliases": {
                    "segment": ["segmento", "champions", "loyal", "at risk", "hibernating", "lost"],
                    "r_score": ["recencia", "que tan reciente compro"],
                    "f_score": ["frecuencia", "que tanto compra"],
                    "m_score": ["monetario", "valor comprado", "gasto"],
                    "total_spent": ["valor comprado", "dinero comprado"]
                },
                "answer_patterns": ["segmentacion rfm", "clientes champions", "clientes en riesgo", "clientes perdidos"],
                "join_keys": ["customer_id"],
                "cannot_answer": ["product_margin_analysis"]
            }
        }),
        "analytics churn-risk" => json!({
            "domain": "analytics",
            "description": "Clientes en riesgo de churn por silencio relativo a su intervalo historico de visita.",
            "tags": ["analytics", "churn", "customers", "retention", "risk"],
            "examples": ["clientes en riesgo de abandono", "clientes que no han vuelto", "churn risk", "clientes silenciosos"],
            "capabilities": {
                "entity": "customer",
                "grain": "customer_risk",
                "measures": ["risk_score", "days_since_last_order", "avg_visit_interval_days", "order_count", "lifetime_value"],
                "dimensions": ["customer_id", "name", "phone", "risk"],
                "supported_operations": ["filter", "rank", "limit", "diagnose", "summarize"],
                "default_rank": ["risk_score", "lifetime_value", "days_since_last_order"],
                "active_condition": ["risk_score", "days_since_last_order", "order_count"],
                "supports_period": false,
                "semantic_aliases": {
                    "risk_score": ["riesgo", "churn", "abandono"],
                    "days_since_last_order": ["dias sin comprar", "silencio", "no ha vuelto"],
                    "avg_visit_interval_days": ["intervalo promedio", "frecuencia historica"],
                    "lifetime_value": ["valor de vida", "ltv", "valor comprado historico"]
                },
                "answer_patterns": ["clientes en riesgo de churn", "clientes que no han vuelto", "riesgo de abandono"],
                "join_keys": ["customer_id"],
                "cannot_answer": ["product_margin_analysis", "period_sales_total"]
            }
        }),
        "financial products" => json!({
            "domain": "financial",
            "description": "Analisis financiero de productos con ventas, margen, costo, ganancia, clasificacion e impacto.",
            "tags": ["financial", "products", "margin", "profit", "revenue", "cost"],
            "examples": ["productos de bajo margen", "productos rentables", "productos que mas ingresos generan", "productos con alto costo"],
            "capabilities": {
                "entity": "product",
                "grain": "product_period",
                "measures": ["sales", "margin", "cost", "profit", "price", "order_count", "tirImpact"],
                "dimensions": ["id", "name", "category", "classification", "last_order_date"],
                "supported_operations": ["filter", "rank", "sort", "limit", "diagnose", "compare"],
                "default_rank": ["margin", "sales", "profit"],
                "active_condition": ["sales", "order_count"],
                "supports_period": true,
                "semantic_aliases": {
                    "sales": ["ventas", "vendieron mucho", "ingresos", "revenue"],
                    "margin": ["margen", "bajo margen", "rentabilidad"],
                    "cost": ["costo", "food cost"],
                    "profit": ["ganancia", "utilidad"],
                    "order_count": ["ordenes", "frecuencia"]
                },
                "answer_patterns": ["productos vendidos mucho con bajo margen", "rentabilidad de productos", "ranking financiero de productos"],
                "join_keys": ["id", "name"],
                "cannot_answer": ["customer_ranking"]
            }
        }),
        "waros estimate" => json!({
            "domain": "waros",
            "description": "Estimacion de puntos WAROS ganados por una compra.",
            "tags": ["waros", "loyalty", "points"],
            "examples": ["cuantos waros gana una compra", "estimar puntos"],
            "capabilities": {
                "entity": "loyalty_estimate",
                "grain": "purchase",
                "measures": ["earned", "total", "tier"],
                "dimensions": ["customer-id"],
                "supported_operations": ["calculate"],
                "default_rank": [],
                "active_condition": [],
                "supports_period": false,
                "semantic_aliases": {
                    "earned": ["puntos ganados", "waros ganados"],
                    "total": ["total compra", "monto"]
                },
                "answer_patterns": ["estimar waros", "puntos por compra"],
                "join_keys": ["customer-id"],
                "cannot_answer": ["customer_balance_without_profile"]
            }
        }),
        "waros balances" => json!({
            "domain": "waros",
            "description": "Saldo WAROS para una lista de perfiles.",
            "tags": ["waros", "loyalty", "balances"],
            "examples": ["saldo waros", "balances de clientes"],
            "capabilities": {
                "entity": "loyalty_balance",
                "grain": "customer",
                "measures": ["balance"],
                "dimensions": ["profile_id"],
                "supported_operations": ["lookup", "list"],
                "default_rank": ["balance"],
                "active_condition": ["balance"],
                "supports_period": false,
                "semantic_aliases": {
                    "balance": ["saldo", "puntos", "waros disponibles"]
                },
                "answer_patterns": ["saldo waros", "balance de puntos"],
                "join_keys": ["profile_id"],
                "cannot_answer": ["sales_metrics"]
            }
        }),
        "waros customer" => json!({
            "domain": "waros",
            "description": "Resumen WAROS de un cliente especifico.",
            "tags": ["waros", "loyalty", "customer"],
            "examples": ["resumen waros de cliente", "tier del cliente"],
            "capabilities": {
                "entity": "loyalty_customer",
                "grain": "customer",
                "measures": ["balance", "tier"],
                "dimensions": ["profile_id", "customer"],
                "supported_operations": ["lookup", "summarize"],
                "default_rank": [],
                "active_condition": ["balance"],
                "supports_period": false,
                "semantic_aliases": {
                    "balance": ["saldo", "puntos", "waros"],
                    "tier": ["nivel", "categoria", "tier"]
                },
                "answer_patterns": ["resumen waros", "tier cliente", "saldo cliente"],
                "join_keys": ["profile_id"],
                "cannot_answer": ["sales_metrics"]
            }
        }),
        "queries schema" => json!({
            "domain": "queries",
            "description": "Lista datasets, dimensiones, medidas, filtros y campos ordenables del contrato QuerySpec seguro.",
            "tags": ["queries", "queryspec", "schema", "analytics"],
            "examples": ["datasets disponibles", "campos para queryspec", "schema queries"],
            "capabilities": {
                "entity": "query_dataset",
                "grain": "dataset",
                "measures": [],
                "dimensions": ["name", "label", "description", "required_scope", "dimensions", "measures", "filters", "sortable_fields"],
                "supported_operations": ["discover", "list"],
                "default_rank": ["name"],
                "active_condition": [],
                "supports_period": false,
                "semantic_aliases": {
                    "datasets": ["datasets", "tablas disponibles", "fuentes disponibles"],
                    "measures": ["metricas", "medidas", "valores"],
                    "dimensions": ["dimensiones", "campos", "agrupaciones"],
                    "filters": ["filtros"],
                    "sortable_fields": ["ordenar", "sort", "ranking"]
                },
                "answer_patterns": ["descubrir queryspec", "listar datasets", "ver campos disponibles"],
                "join_keys": ["name"],
                "cannot_answer": ["execute_query_without_run"]
            }
        }),
        "queries run" => json!({
            "domain": "queries",
            "description": "Ejecuta un QuerySpec seguro validado por la API y devuelve filas analiticas normalizadas.",
            "tags": ["queries", "queryspec", "analytics", "dynamic"],
            "examples": ["productos mas vendidos con margen", "clientes por ticket promedio", "productos con utilidad baja"],
            "capabilities": {
                "entity": "query_row",
                "grain": "dynamic_dataset_row",
                "measures": ["quantity_sold", "revenue", "orders_count", "avg_price", "order_count", "total_spent", "avg_ticket", "waros_balance", "profit_per_unit", "profit_margin_pct", "profit_margin_real_pct", "profit_margin_operativo_pct", "total_profit"],
                "dimensions": ["product", "product_id", "category", "day", "customer", "customer_id", "classification", "cost_source"],
                "supported_operations": ["filter", "aggregate", "group", "rank", "sort", "limit", "compare"],
                "default_rank": ["revenue", "quantity_sold", "total_profit", "total_spent"],
                "active_condition": ["limit", "dataset"],
                "supports_period": true,
                "semantic_aliases": {
                    "quantity_sold": ["unidades vendidas", "cantidad vendida", "volumen"],
                    "revenue": ["ventas", "ingresos", "facturacion"],
                    "total_profit": ["utilidad", "ganancia", "profit"],
                    "profit_margin_pct": ["margen", "rentabilidad"],
                    "total_spent": ["valor comprado", "gasto cliente"],
                    "avg_ticket": ["ticket promedio"],
                    "classification": ["clasificacion", "estrella", "plowhorse", "puzzle", "dog"],
                    "cost_source": ["fuente de costo", "origen del costo", "costo real", "costo estimado"]
                },
                "answer_patterns": ["analisis dinamico", "ranking con metricas", "comparar dimensiones", "diagnostico queryspec"],
                "join_keys": ["product_id", "customer_id"],
                "cannot_answer": ["raw_sql", "write_operations", "unallowlisted_fields"]
            }
        }),
        _ => return None,
    };
    Some(metadata)
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
const ANALYTICS_COHORT_FIELDS: &[&str] =
    &["cohort_date", "cohort_label", "cohort_size", "retention"];
const ANALYTICS_WAROS_FIELDS: &[&str] = &[
    "active_members",
    "name",
    "period",
    "redemption_rate_pct",
    "total_earned",
    "total_issued",
    "total_redeemed",
    "transaction_count",
];
const ANALYTICS_RFM_FIELDS: &[&str] = &[
    "customer_id",
    "customer_name",
    "f_score",
    "last_order_date",
    "m_score",
    "order_count",
    "r_score",
    "segment",
    "total_spent",
];
const ANALYTICS_CHURN_RISK_FIELDS: &[&str] = &[
    "avg_visit_interval_days",
    "customer_id",
    "days_since_last_order",
    "lifetime_value",
    "name",
    "order_count",
    "phone",
    "risk_score",
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
const QUERIES_SCHEMA_FIELDS: &[&str] = &[
    "default_limit",
    "description",
    "dimensions",
    "filters",
    "label",
    "max_limit",
    "measures",
    "name",
    "required_scope",
    "sortable_fields",
];
const QUERIES_RUN_FIELDS: &[&str] = &[
    "avg_price",
    "avg_ticket",
    "category",
    "classification",
    "cost_source",
    "customer",
    "customer_id",
    "day",
    "last_order_date",
    "order_count",
    "orders_count",
    "product",
    "product_id",
    "profit_margin_operativo_pct",
    "profit_margin_pct",
    "profit_margin_real_pct",
    "profit_per_unit",
    "quantity_sold",
    "revenue",
    "total_profit",
    "total_spent",
    "waros_balance",
];

const DATA_WRAPPER_KEYS: &[&str] = &["data", "meta", "pagination", "success"];
const DATA_META_SUCCESS_KEYS: &[&str] = &["data", "meta", "success"];
const CUSTOMER_LIST_KEYS: &[&str] = &["data", "limit", "offset", "total"];
const CUSTOMER_ORDERS_KEYS: &[&str] = &["items", "limit", "offset", "total"];
const CUSTOMER_METRICS_KEYS: &[&str] = &["summary", "top_customers"];
const CUSTOMER_SERIES_KEYS: &[&str] = &["summary", "series"];
const FINANCIAL_PRODUCTS_KEYS: &[&str] =
    &["categories", "filters", "insights", "metrics", "products"];
const ANALYTICS_COHORT_KEYS: &[&str] = &["cohorts", "period", "periods"];
const ANALYTICS_WAROS_KEYS: &[&str] = &["groups", "summary"];
const ANALYTICS_RFM_KEYS: &[&str] = &["data"];
const ANALYTICS_CHURN_RISK_KEYS: &[&str] = &[
    "customers",
    "min_orders",
    "threshold_multiplier",
    "total_count",
];
const WAROS_BALANCES_KEYS: &[&str] = &["balances"];
const QUERIES_TOP_LEVEL_KEYS: &[&str] = &["data", "success"];

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
        command: "analytics cohort",
        method: "POST",
        path: "/v1/analytics/cohort",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::TopLevelRows,
        row_path: "cohorts",
        fields: ANALYTICS_COHORT_FIELDS,
        default_fields: ANALYTICS_COHORT_KEYS,
        top_level_keys: ANALYTICS_COHORT_KEYS,
    },
    CommandContract {
        command: "analytics waros",
        method: "POST",
        path: "/v1/analytics/waros",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::TopLevelRows,
        row_path: "groups",
        fields: ANALYTICS_WAROS_FIELDS,
        default_fields: ANALYTICS_WAROS_KEYS,
        top_level_keys: ANALYTICS_WAROS_KEYS,
    },
    CommandContract {
        command: "analytics rfm",
        method: "POST",
        path: "/v1/analytics/rfm",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.customers",
        fields: ANALYTICS_RFM_FIELDS,
        default_fields: ANALYTICS_RFM_KEYS,
        top_level_keys: ANALYTICS_RFM_KEYS,
    },
    CommandContract {
        command: "analytics churn-risk",
        method: "POST",
        path: "/v1/analytics/churn-risk",
        scope: "analytics:read",
        paginates: false,
        shape: ResponseShape::TopLevelRows,
        row_path: "customers",
        fields: ANALYTICS_CHURN_RISK_FIELDS,
        default_fields: ANALYTICS_CHURN_RISK_KEYS,
        top_level_keys: ANALYTICS_CHURN_RISK_KEYS,
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
    CommandContract {
        command: "queries schema",
        method: "GET",
        path: "/v1/queries/schema",
        scope: "read",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.datasets",
        fields: QUERIES_SCHEMA_FIELDS,
        default_fields: QUERIES_SCHEMA_FIELDS,
        top_level_keys: QUERIES_TOP_LEVEL_KEYS,
    },
    CommandContract {
        command: "queries run",
        method: "POST",
        path: "/v1/queries/run",
        scope: "dataset_scope",
        paginates: false,
        shape: ResponseShape::NestedRows,
        row_path: "data.rows",
        fields: QUERIES_RUN_FIELDS,
        default_fields: QUERIES_RUN_FIELDS,
        top_level_keys: QUERIES_TOP_LEVEL_KEYS,
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
