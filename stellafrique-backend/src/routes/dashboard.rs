use std::collections::{BTreeMap, HashMap};

use axum::{routing::get, Json, Router};
use chrono::{Duration, Utc};
use sea_orm::{EntityTrait, QueryOrder};
use serde::Serialize;

use crate::{
    entities::{categories, order_items, orders, product_variants, products},
    error::AppError,
    state::AppState,
};

#[derive(Serialize)]
pub struct AdminDashboardResponse {
    pub overview: DashboardOverview,
    pub sales_series: Vec<SalesSeriesPoint>,
    pub fulfilment_breakdown: Vec<BreakdownPoint>,
    pub payment_breakdown: Vec<BreakdownPoint>,
    pub payment_method_breakdown: Vec<PaymentMethodPoint>,
    pub top_products: Vec<TopProductPoint>,
    pub top_categories: Vec<TopCategoryPoint>,
    pub inventory_alerts: Vec<InventoryAlertPoint>,
    pub recent_orders: Vec<RecentOrderPoint>,
}

#[derive(Serialize)]
pub struct DashboardOverview {
    pub gross_revenue: sea_orm::prelude::Decimal,
    pub paid_revenue: sea_orm::prelude::Decimal,
    pub average_order_value: sea_orm::prelude::Decimal,
    pub collection_rate_percentage: sea_orm::prelude::Decimal,
    pub total_orders: usize,
    pub orders_today: usize,
    pub paid_orders: usize,
    pub unpaid_orders: usize,
    pub pending_fulfilment_orders: usize,
    pub active_products: usize,
    pub low_stock_variants: usize,
}

#[derive(Serialize)]
pub struct SalesSeriesPoint {
    pub label: String,
    pub orders: usize,
    pub gross_revenue: sea_orm::prelude::Decimal,
    pub paid_revenue: sea_orm::prelude::Decimal,
}

#[derive(Serialize)]
pub struct BreakdownPoint {
    pub label: String,
    pub count: usize,
}

#[derive(Serialize)]
pub struct TopProductPoint {
    pub product_name: String,
    pub units_sold: i32,
    pub revenue: sea_orm::prelude::Decimal,
}

#[derive(Serialize)]
pub struct TopCategoryPoint {
    pub category_name: String,
    pub units_sold: i32,
    pub revenue: sea_orm::prelude::Decimal,
}

#[derive(Serialize)]
pub struct InventoryAlertPoint {
    pub variant_id: uuid::Uuid,
    pub product_name: String,
    pub variant_name: String,
    pub sku: String,
    pub stock_quantity: i32,
}

#[derive(Serialize)]
pub struct RecentOrderPoint {
    pub order_number: String,
    pub customer_name: String,
    pub status: String,
    pub payment_status: String,
    pub total_amount: sea_orm::prelude::Decimal,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct PaymentMethodPoint {
    pub label: String,
    pub count: usize,
}

pub fn admin_router() -> Router<AppState> {
    Router::new().route("/admin/dashboard", get(get_dashboard))
}

async fn get_dashboard(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Result<Json<AdminDashboardResponse>, AppError> {
    let orders_list = orders::Entity::find()
        .order_by_desc(orders::Column::CreatedAt)
        .all(&state.db)
        .await?;
    let order_items_list = order_items::Entity::find().all(&state.db).await?;
    let products_list = products::Entity::find().all(&state.db).await?;
    let categories_list = categories::Entity::find().all(&state.db).await?;
    let variants_list = product_variants::Entity::find().all(&state.db).await?;

    let today = Utc::now().date_naive();
    let start_day = today - Duration::days(13);

    let mut gross_revenue = sea_orm::prelude::Decimal::ZERO;
    let mut paid_revenue = sea_orm::prelude::Decimal::ZERO;
    let mut orders_today = 0usize;
    let mut paid_orders = 0usize;
    let mut unpaid_orders = 0usize;
    let mut pending_fulfilment_orders = 0usize;
    let mut fulfilment_breakdown: HashMap<String, usize> = HashMap::new();
    let mut payment_breakdown: HashMap<String, usize> = HashMap::new();
    let mut payment_method_breakdown: HashMap<String, usize> = HashMap::new();
    let mut sales_series: BTreeMap<String, SalesSeriesPoint> = (0..14)
        .map(|offset| {
            let day = start_day + Duration::days(offset);
            let label = day.format("%b %d").to_string();
            (
                label.clone(),
                SalesSeriesPoint {
                    label,
                    orders: 0,
                    gross_revenue: sea_orm::prelude::Decimal::ZERO,
                    paid_revenue: sea_orm::prelude::Decimal::ZERO,
                },
            )
        })
        .collect();

    let order_lookup: HashMap<uuid::Uuid, orders::Model> = orders_list
        .iter()
        .cloned()
        .map(|order| (order.id, order))
        .collect();

    for order in &orders_list {
        *fulfilment_breakdown.entry(order.status.clone()).or_default() += 1;
        *payment_breakdown.entry(order.payment_status.clone()).or_default() += 1;
        *payment_method_breakdown
            .entry(
                order
                    .payment_method
                    .clone()
                    .unwrap_or_else(|| "manual".to_owned()),
            )
            .or_default() += 1;

        if order.status != "cancelled" {
            gross_revenue += order.total_amount;
        }
        if order.payment_status == "paid" {
            paid_revenue += order.total_amount;
            paid_orders += 1;
        } else {
            unpaid_orders += 1;
        }

        let order_day = order.created_at.date_naive();
        if order_day == today {
            orders_today += 1;
        }
        if order.status != "fulfilled" && order.status != "cancelled" {
            pending_fulfilment_orders += 1;
        }
        if order_day >= start_day {
            let label = order_day.format("%b %d").to_string();
            if let Some(point) = sales_series.get_mut(&label) {
                point.orders += 1;
                if order.status != "cancelled" {
                    point.gross_revenue += order.total_amount;
                }
                if order.payment_status == "paid" {
                    point.paid_revenue += order.total_amount;
                }
            }
        }
    }

    let total_orders = orders_list.len();
    let average_order_value = if total_orders == 0 {
        sea_orm::prelude::Decimal::ZERO
    } else {
        gross_revenue / sea_orm::prelude::Decimal::from(total_orders as i64)
    };
    let collection_rate_percentage = if gross_revenue.is_zero() {
        sea_orm::prelude::Decimal::ZERO
    } else {
        (paid_revenue / gross_revenue) * sea_orm::prelude::Decimal::from(100)
    };

    let active_products = products_list
        .iter()
        .filter(|product| product.status == "active")
        .count();

    let product_name_by_id: HashMap<uuid::Uuid, String> = products_list
        .iter()
        .map(|product| (product.id, product.name.clone()))
        .collect();
    let category_name_by_id: HashMap<uuid::Uuid, String> = categories_list
        .iter()
        .map(|category| (category.id, category.name.clone()))
        .collect();
    let product_category_by_id: HashMap<uuid::Uuid, Option<uuid::Uuid>> = products_list
        .iter()
        .map(|product| (product.id, product.category_id))
        .collect();

    let low_stock_variants = variants_list
        .iter()
        .filter(|variant| variant.is_active && variant.stock_quantity <= 5)
        .count();

    let mut inventory_alerts = variants_list
        .iter()
        .filter(|variant| variant.is_active && variant.stock_quantity <= 5)
        .filter_map(|variant| {
            Some(InventoryAlertPoint {
                variant_id: variant.id,
                product_name: product_name_by_id.get(&variant.product_id)?.clone(),
                variant_name: variant.name.clone(),
                sku: variant.sku.clone(),
                stock_quantity: variant.stock_quantity,
            })
        })
        .collect::<Vec<_>>();
    inventory_alerts.sort_by_key(|alert| alert.stock_quantity);
    inventory_alerts.truncate(6);

    let mut top_product_map: HashMap<uuid::Uuid, TopProductPoint> = HashMap::new();
    let mut top_category_map: HashMap<String, TopCategoryPoint> = HashMap::new();
    for item in &order_items_list {
        let Some(order) = order_lookup.get(&item.order_id) else {
            continue;
        };
        if order.status == "cancelled" {
            continue;
        }

        let entry = top_product_map.entry(item.product_id).or_insert_with(|| TopProductPoint {
            product_name: item.product_name.clone(),
            units_sold: 0,
            revenue: sea_orm::prelude::Decimal::ZERO,
        });
        entry.units_sold += item.quantity;
        entry.revenue += item.line_total;

        let category_name = product_category_by_id
            .get(&item.product_id)
            .and_then(|category_id| category_id.and_then(|id| category_name_by_id.get(&id).cloned()))
            .unwrap_or_else(|| "Uncategorized".to_owned());
        let category_entry = top_category_map
            .entry(category_name.clone())
            .or_insert_with(|| TopCategoryPoint {
                category_name,
                units_sold: 0,
                revenue: sea_orm::prelude::Decimal::ZERO,
            });
        category_entry.units_sold += item.quantity;
        category_entry.revenue += item.line_total;
    }
    let mut top_products = top_product_map.into_values().collect::<Vec<_>>();
    top_products.sort_by(|left, right| {
        right
            .units_sold
            .cmp(&left.units_sold)
            .then_with(|| right.product_name.cmp(&left.product_name))
    });
    top_products.truncate(5);
    let mut top_categories = top_category_map.into_values().collect::<Vec<_>>();
    top_categories.sort_by(|left, right| {
        right
            .units_sold
            .cmp(&left.units_sold)
            .then_with(|| left.category_name.cmp(&right.category_name))
    });
    top_categories.truncate(5);

    let recent_orders = orders_list
        .iter()
        .take(6)
        .map(|order| RecentOrderPoint {
            order_number: order.order_number.clone(),
            customer_name: order.customer_name.clone(),
            status: order.status.clone(),
            payment_status: order.payment_status.clone(),
            total_amount: order.total_amount,
            created_at: order.created_at,
        })
        .collect::<Vec<_>>();

    let mut fulfilment_breakdown = fulfilment_breakdown
        .into_iter()
        .map(|(label, count)| BreakdownPoint { label, count })
        .collect::<Vec<_>>();
    fulfilment_breakdown.sort_by(|left, right| right.count.cmp(&left.count).then_with(|| left.label.cmp(&right.label)));

    let mut payment_breakdown = payment_breakdown
        .into_iter()
        .map(|(label, count)| BreakdownPoint { label, count })
        .collect::<Vec<_>>();
    payment_breakdown.sort_by(|left, right| right.count.cmp(&left.count).then_with(|| left.label.cmp(&right.label)));

    let mut payment_method_breakdown = payment_method_breakdown
        .into_iter()
        .map(|(label, count)| PaymentMethodPoint { label, count })
        .collect::<Vec<_>>();
    payment_method_breakdown.sort_by(|left, right| right.count.cmp(&left.count).then_with(|| left.label.cmp(&right.label)));

    Ok(Json(AdminDashboardResponse {
        overview: DashboardOverview {
            gross_revenue,
            paid_revenue,
            average_order_value,
            collection_rate_percentage,
            total_orders,
            orders_today,
            paid_orders,
            unpaid_orders,
            pending_fulfilment_orders,
            active_products,
            low_stock_variants,
        },
        sales_series: sales_series.into_values().collect(),
        fulfilment_breakdown,
        payment_breakdown,
        payment_method_breakdown,
        top_products,
        top_categories,
        inventory_alerts,
        recent_orders,
    }))
}
