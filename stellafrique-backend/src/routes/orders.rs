use axum::{
    extract::{Path, Query, State},
    http::HeaderMap,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use sea_orm::{
    ActiveModelTrait, ColumnTrait, Condition, ConnectionTrait, DatabaseTransaction, EntityTrait,
    QueryFilter, QueryOrder, Set, TransactionTrait,
};
use serde::{Deserialize, Serialize};
use tracing::{error, warn};

use crate::{
    customer_auth,
    email_templates,
    entities::{order_events, order_items, orders, product_images, product_variants, products},
    error::AppError,
    state::AppState,
    velipay::{CreateStkPushRequest, StkPushMetadata},
};

struct PendingOrderLine {
    product_id: uuid::Uuid,
    variant_id: uuid::Uuid,
    response: OrderLineResponse,
}

#[derive(Deserialize)]
pub struct CreateOrderPayload {
    customer_name: String,
    customer_email: String,
    customer_phone: String,
    shipping_address_line1: String,
    shipping_address_line2: Option<String>,
    shipping_city: String,
    shipping_region: Option<String>,
    shipping_postal_code: Option<String>,
    shipping_country: String,
    notes: Option<String>,
    payment_method: Option<String>,
    payment_phone_number: Option<String>,
    items: Vec<CreateOrderItemPayload>,
}

#[derive(Deserialize)]
pub struct CreateOrderItemPayload {
    slug: String,
    variant_id: Option<String>,
    quantity: i32,
}

#[derive(Serialize)]
pub struct OrderSummaryResponse {
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub currency: String,
    pub subtotal_amount: sea_orm::prelude::Decimal,
    pub total_amount: sea_orm::prelude::Decimal,
    pub customer_name: String,
    pub customer_email: String,
    pub payment: Option<CheckoutPaymentResponse>,
    pub items: Vec<OrderLineResponse>,
}

#[derive(Serialize)]
pub struct CheckoutPaymentResponse {
    pub provider: String,
    pub method: String,
    pub status: String,
    pub payment_reference: Option<String>,
    pub merchant_reference: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct AdminOrderListItem {
    order_number: String,
    status: String,
    payment_status: String,
    currency: String,
    total_amount: sea_orm::prelude::Decimal,
    customer_name: String,
    customer_email: String,
    item_count: usize,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Serialize)]
pub struct AdminOrderDetailResponse {
    pub order_number: String,
    pub status: String,
    pub payment_status: String,
    pub currency: String,
    pub subtotal_amount: sea_orm::prelude::Decimal,
    pub total_amount: sea_orm::prelude::Decimal,
    pub customer_name: String,
    pub customer_email: String,
    pub customer_phone: String,
    pub shipping_address_line1: String,
    pub shipping_address_line2: Option<String>,
    pub shipping_city: String,
    pub shipping_region: Option<String>,
    pub shipping_postal_code: Option<String>,
    pub shipping_country: String,
    pub notes: Option<String>,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub shipping_courier: Option<String>,
    pub tracking_number: Option<String>,
    pub paid_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    pub fulfilled_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub items: Vec<OrderLineResponse>,
    pub events: Vec<OrderEventResponse>,
}

#[derive(Serialize)]
pub struct OrderLineResponse {
    pub product_name: String,
    pub variant_name: Option<String>,
    pub sku: Option<String>,
    pub size: Option<String>,
    pub color: Option<String>,
    pub quantity: i32,
    pub unit_price: sea_orm::prelude::Decimal,
    pub line_total: sea_orm::prelude::Decimal,
    pub image_url: Option<String>,
}

#[derive(Serialize)]
pub struct OrderEventResponse {
    pub event_type: String,
    pub actor: String,
    pub message: String,
    pub details: Option<String>,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Deserialize)]
pub struct UpdateOrderStatusPayload {
    status: Option<String>,
    payment_status: Option<String>,
    payment_method: Option<String>,
    payment_reference: Option<String>,
    shipping_courier: Option<String>,
    tracking_number: Option<String>,
}

#[derive(Deserialize)]
pub struct AdminOrdersQuery {
    status: Option<String>,
    payment_status: Option<String>,
    search: Option<String>,
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/orders", post(create_order))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/orders", get(list_orders))
        .route("/admin/orders/:order_number", get(order_detail).patch(update_order_status))
}

async fn create_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateOrderPayload>,
) -> Result<(StatusCode, Json<OrderSummaryResponse>), AppError> {
    let requested_payment_method = payload.payment_method.clone();
    if payload.items.is_empty() {
        return Err(AppError::BadRequest("cart is empty".to_owned()));
    }
    if payload
        .payment_method
        .as_deref()
        .is_some_and(|method| !matches!(method.trim(), "velipay_stk_push" | "manual"))
    {
        return Err(AppError::BadRequest("invalid payment method".to_owned()));
    }

    error!(
        order_items = payload.items.len(),
        payment_method = requested_payment_method.as_deref().unwrap_or("none"),
        customer_email = %payload.customer_email,
        "processing checkout order"
    );

    let mut subtotal = sea_orm::prelude::Decimal::ZERO;
    let mut order_lines = Vec::with_capacity(payload.items.len());
    let transaction = state.db.begin().await?;
    error!("checkout transaction opened");
    let authenticated_user = customer_auth::resolve_authenticated_user(&state, &headers).await?;
    error!(
        authenticated = authenticated_user.is_some(),
        "checkout customer session resolved"
    );

    for requested_item in payload.items {
        if requested_item.quantity <= 0 {
            return Err(AppError::BadRequest("quantity must be at least 1".to_owned()));
        }

        let product = products::Entity::find()
            .filter(products::Column::Slug.eq(requested_item.slug))
            .filter(products::Column::Status.eq("active"))
            .one(&transaction)
            .await?
            .ok_or(AppError::BadRequest("product is no longer available".to_owned()))?;

        let variant = match requested_item.variant_id {
            Some(variant_id) => {
                let parsed_id = uuid::Uuid::parse_str(&variant_id)
                    .map_err(|_| AppError::BadRequest("invalid variant id".to_owned()))?;

                product_variants::Entity::find_by_id(parsed_id)
                    .filter(product_variants::Column::ProductId.eq(product.id))
                    .filter(product_variants::Column::IsActive.eq(true))
                    .one(&transaction)
                    .await?
            }
            None => {
                product_variants::Entity::find()
                    .filter(product_variants::Column::ProductId.eq(product.id))
                    .filter(product_variants::Column::IsActive.eq(true))
                    .one(&transaction)
                    .await?
            }
        }
        .ok_or(AppError::BadRequest("product variant is no longer available".to_owned()))?;

        if variant.stock_quantity < requested_item.quantity {
            return Err(AppError::BadRequest(format!(
                "requested quantity for {} exceeds available stock",
                product.name
            )));
        }

        let image = product_images::Entity::find()
            .filter(product_images::Column::ProductId.eq(product.id))
            .filter(product_images::Column::IsPrimary.eq(true))
            .one(&transaction)
            .await?;

        let quantity_decimal = sea_orm::prelude::Decimal::from(requested_item.quantity);
        let line_total = variant.price * quantity_decimal;
        subtotal += line_total;

        apply_stock_change(
            &transaction,
            &variant,
            requested_item.quantity,
            InventoryAction::Reserve,
        )
        .await?;

        order_lines.push(PendingOrderLine {
            product_id: product.id,
            variant_id: variant.id,
            response: OrderLineResponse {
                product_name: product.name,
                variant_name: Some(variant.name.clone()),
                sku: Some(variant.sku.clone()),
                size: variant.size.clone(),
                color: variant.color.clone(),
                quantity: requested_item.quantity,
                unit_price: variant.price,
                line_total,
                image_url: image.map(|record| record.image_url),
            },
        });
    }

    let now = Utc::now().fixed_offset();
    let order_id = uuid::Uuid::new_v4();
    let suffix = uuid::Uuid::new_v4()
        .simple()
        .to_string()
        .chars()
        .take(8)
        .collect::<String>()
        .to_uppercase();
    let order_number = format!("STL-{}-{}", Utc::now().format("%Y%m%d%H%M%S"), suffix);

    let order = orders::ActiveModel {
        id: Set(order_id),
        user_id: Set(authenticated_user.as_ref().map(|user| user.id)),
        order_number: Set(order_number.clone()),
        status: Set("pending".to_owned()),
        payment_status: Set("unpaid".to_owned()),
        currency: Set("KES".to_owned()),
        subtotal_amount: Set(subtotal),
        total_amount: Set(subtotal),
        customer_name: Set(payload.customer_name),
        customer_email: Set(payload.customer_email),
        customer_phone: Set(payload.customer_phone),
        shipping_address_line1: Set(payload.shipping_address_line1),
        shipping_address_line2: Set(payload.shipping_address_line2),
        shipping_city: Set(payload.shipping_city),
        shipping_region: Set(payload.shipping_region),
        shipping_postal_code: Set(payload.shipping_postal_code),
        shipping_country: Set(payload.shipping_country),
        notes: Set(payload.notes),
        payment_method: Set(None),
        payment_reference: Set(None),
        shipping_courier: Set(None),
        tracking_number: Set(None),
        paid_at: Set(None),
        fulfilled_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&transaction)
    .await?;
    error!(order_number = %order.order_number, "checkout order row inserted");

    for line in &order_lines {
        order_items::ActiveModel {
            id: Set(uuid::Uuid::new_v4()),
            order_id: Set(order.id),
            product_id: Set(line.product_id),
            variant_id: Set(Some(line.variant_id)),
            product_name: Set(line.response.product_name.clone()),
            variant_name: Set(line.response.variant_name.clone()),
            sku: Set(line.response.sku.clone()),
            size: Set(line.response.size.clone()),
            color: Set(line.response.color.clone()),
            unit_price: Set(line.response.unit_price),
            quantity: Set(line.response.quantity),
            line_total: Set(line.response.line_total),
            image_url: Set(line.response.image_url.clone()),
            created_at: Set(now),
        }
        .insert(&transaction)
        .await?;
    }
    error!(order_number = %order.order_number, line_items = order_lines.len(), "checkout order items inserted");

    insert_order_event(
        &transaction,
        order.id,
        "order_created",
        "customer",
        format!("Order {} was placed.", order.order_number),
        Some(format!(
            "{} line items · total {} {}",
            order_lines.len(),
            order.currency,
            order.total_amount
        )),
    )
    .await?;

    insert_order_event(
        &transaction,
        order.id,
        "inventory_reserved",
        "system",
        "Inventory reserved for the newly placed order.".to_owned(),
        Some(format!("Reserved stock for {} line items.", order_lines.len())),
    )
    .await?;

    transaction.commit().await?;
    error!(order_number = %order.order_number, "checkout transaction committed");

    let mut response = OrderSummaryResponse {
        order_number,
        status: order.status.clone(),
        payment_status: order.payment_status.clone(),
        currency: order.currency.clone(),
        subtotal_amount: order.subtotal_amount,
        total_amount: order.total_amount,
        customer_name: order.customer_name.clone(),
        customer_email: order.customer_email.clone(),
        payment: None,
        items: order_lines.into_iter().map(|line| line.response).collect(),
    };

    if payload
        .payment_method
        .as_deref()
        .is_some_and(|method| method.eq_ignore_ascii_case("velipay_stk_push"))
    {
        response.payment = Some(
            initiate_velipay_payment(
                &state,
                &order,
                payload.payment_phone_number.as_deref(),
            )
            .await?,
        );
    }

    send_order_created_email(&state, &response).await;

    Ok((
        StatusCode::CREATED,
        Json(response),
    ))
}

async fn list_orders(
    State(state): State<AppState>,
    Query(query): Query<AdminOrdersQuery>,
) -> Result<Json<Vec<AdminOrderListItem>>, AppError> {
    let mut orders_query = orders::Entity::find().order_by_desc(orders::Column::CreatedAt);

    if let Some(status) = query.status.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        orders_query = orders_query.filter(orders::Column::Status.eq(status.to_lowercase()));
    }

    if let Some(payment_status) = query
        .payment_status
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        orders_query =
            orders_query.filter(orders::Column::PaymentStatus.eq(payment_status.to_lowercase()));
    }

    if let Some(search) = query.search.as_deref().map(str::trim).filter(|value| !value.is_empty()) {
        orders_query = orders_query.filter(
            Condition::any()
                .add(orders::Column::OrderNumber.contains(search))
                .add(orders::Column::CustomerName.contains(search))
                .add(orders::Column::CustomerEmail.contains(search)),
        );
    }

    let records = orders_query.all(&state.db).await?;

    let mut response = Vec::with_capacity(records.len());
    for order in records {
        let item_count = order_items::Entity::find()
            .filter(order_items::Column::OrderId.eq(order.id))
            .all(&state.db)
            .await?
            .len();

        response.push(AdminOrderListItem {
            order_number: order.order_number,
            status: order.status,
            payment_status: order.payment_status,
            currency: order.currency,
            total_amount: order.total_amount,
            customer_name: order.customer_name,
            customer_email: order.customer_email,
            item_count,
            created_at: order.created_at,
        });
    }

    Ok(Json(response))
}

async fn order_detail(
    State(state): State<AppState>,
    Path(order_number): Path<String>,
) -> Result<Json<AdminOrderDetailResponse>, AppError> {
    Ok(Json(fetch_order_detail(&state.db, &order_number).await?))
}

async fn update_order_status(
    State(state): State<AppState>,
    Path(order_number): Path<String>,
    Json(payload): Json<UpdateOrderStatusPayload>,
) -> Result<Json<AdminOrderDetailResponse>, AppError> {
    if payload.status.is_none()
        && payload.payment_status.is_none()
        && payload.payment_method.is_none()
        && payload.payment_reference.is_none()
        && payload.shipping_courier.is_none()
        && payload.tracking_number.is_none()
    {
        return Err(AppError::BadRequest("at least one order state field is required".to_owned()));
    }

    let next_status = payload
        .status
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase);
    if let Some(status) = next_status.as_deref() {
        if !matches!(status, "pending" | "paid" | "fulfilled" | "cancelled") {
            return Err(AppError::BadRequest("invalid order status".to_owned()));
        }
    }

    let next_payment_status = payload
        .payment_status
        .as_deref()
        .map(str::trim)
        .map(str::to_lowercase);
    if let Some(payment_status) = next_payment_status.as_deref() {
        if !matches!(payment_status, "unpaid" | "paid" | "refunded") {
            return Err(AppError::BadRequest("invalid payment status".to_owned()));
        }
    }

    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.clone()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    let previous_status = order.status.clone();
    let previous_payment_status = order.payment_status.clone();
    let previous_fulfilled_at = order.fulfilled_at;
    let previous_shipping_courier = order.shipping_courier.clone();
    let previous_tracking_number = order.tracking_number.clone();
    let order_id = order.id;
    let now = Utc::now().fixed_offset();
    let payment_metadata_was_provided =
        payload.payment_method.is_some() || payload.payment_reference.is_some();
    let shipping_metadata_was_provided =
        payload.shipping_courier.is_some() || payload.tracking_number.is_some();

    let mut active = orders::ActiveModel::from(order);
    if let Some(status) = next_status.clone() {
        active.status = Set(status);
    }
    if let Some(payment_status) = next_payment_status.clone() {
        active.payment_status = Set(payment_status);
    }
    if let Some(payment_method) = payload.payment_method {
        active.payment_method = Set(if payment_method.trim().is_empty() {
            None
        } else {
            Some(payment_method.trim().to_owned())
        });
    }
    if let Some(payment_reference) = payload.payment_reference {
        active.payment_reference = Set(if payment_reference.trim().is_empty() {
            None
        } else {
            Some(payment_reference.trim().to_owned())
        });
    }
    if let Some(shipping_courier) = payload.shipping_courier {
        active.shipping_courier = Set(if shipping_courier.trim().is_empty() {
            None
        } else {
            Some(shipping_courier.trim().to_owned())
        });
    }
    if let Some(tracking_number) = payload.tracking_number {
        active.tracking_number = Set(if tracking_number.trim().is_empty() {
            None
        } else {
            Some(tracking_number.trim().to_owned())
        });
    }
    if let Some(status) = next_status.as_deref() {
        if status == "fulfilled" {
            active.fulfilled_at = Set(Some(now));
        } else if matches!(status, "pending" | "paid" | "cancelled") {
            active.fulfilled_at = Set(None);
        }
    }
    if let Some(payment_status) = next_payment_status.as_deref() {
        if payment_status == "paid" && previous_payment_status != "paid" {
            active.paid_at = Set(Some(now));
        } else if payment_status == "unpaid" {
            active.paid_at = Set(None);
        }
    }
    active.updated_at = Set(now);
    let transaction = state.db.begin().await?;

    if payload
        .status
        .as_deref()
        .is_some_and(|status| previous_status != "cancelled" && status.trim().eq_ignore_ascii_case("cancelled"))
    {
        restore_order_inventory(&transaction, &order_id).await?;
        insert_order_event(
            &transaction,
            order_id,
            "inventory_restored",
            "admin",
            "Inventory restored because the order was cancelled.".to_owned(),
            None,
        )
        .await?;
    } else if payload
        .status
        .as_deref()
        .is_some_and(|status| previous_status == "cancelled" && !status.trim().eq_ignore_ascii_case("cancelled"))
    {
        reserve_order_inventory(&transaction, &order_id).await?;
        insert_order_event(
            &transaction,
            order_id,
            "inventory_reserved",
            "admin",
            "Inventory re-reserved because the order was re-opened.".to_owned(),
            None,
        )
        .await?;
    }

    let updated_order = active.update(&transaction).await?;

    if previous_status != updated_order.status {
        insert_order_event(
            &transaction,
            order_id,
            "status_updated",
            "admin",
            format!(
                "Fulfilment status changed from {} to {}.",
                previous_status, updated_order.status
            ),
            None,
        )
        .await?;
    }

    if previous_payment_status != updated_order.payment_status {
        insert_order_event(
            &transaction,
            order_id,
            "payment_status_updated",
            "admin",
            format!(
                "Payment status changed from {} to {}.",
                previous_payment_status, updated_order.payment_status
            ),
            None,
        )
        .await?;
    }

    if payment_metadata_was_provided {
        insert_order_event(
            &transaction,
            order_id,
            "payment_metadata_updated",
            "admin",
            "Payment metadata was updated.".to_owned(),
            Some(format!(
                "Method: {} · Reference: {}",
                updated_order
                    .payment_method
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned()),
                updated_order
                    .payment_reference
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned())
            )),
        )
        .await?;
    }

    if shipping_metadata_was_provided {
        insert_order_event(
            &transaction,
            order_id,
            "shipping_metadata_updated",
            "admin",
            "Shipping courier or tracking details were updated.".to_owned(),
            Some(format!(
                "Courier: {} → {} · Tracking: {} → {}",
                previous_shipping_courier
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned()),
                updated_order
                    .shipping_courier
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned()),
                previous_tracking_number
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned()),
                updated_order
                    .tracking_number
                    .clone()
                    .unwrap_or_else(|| "Not specified".to_owned())
            )),
        )
        .await?;
    }

    transaction.commit().await?;

    let detail = fetch_order_detail(&state.db, &order_number).await?;
    send_order_update_emails(
        &state,
        &detail,
        &previous_status,
        &previous_payment_status,
        previous_fulfilled_at,
    )
    .await;

    Ok(Json(detail))
}

#[derive(Copy, Clone)]
enum InventoryAction {
    Reserve,
    Restore,
}

async fn reserve_order_inventory(
    transaction: &DatabaseTransaction,
    order_id: &uuid::Uuid,
) -> Result<(), AppError> {
    let lines = order_items::Entity::find()
        .filter(order_items::Column::OrderId.eq(*order_id))
        .all(transaction)
        .await?;

    for line in lines {
        let variant_id = line
            .variant_id
            .ok_or(AppError::BadRequest("order item is missing a variant".to_owned()))?;
        let variant = product_variants::Entity::find_by_id(variant_id)
            .one(transaction)
            .await?
            .ok_or(AppError::BadRequest("variant no longer exists".to_owned()))?;
        apply_stock_change(transaction, &variant, line.quantity, InventoryAction::Reserve).await?;
    }

    Ok(())
}

async fn restore_order_inventory(
    transaction: &DatabaseTransaction,
    order_id: &uuid::Uuid,
) -> Result<(), AppError> {
    let lines = order_items::Entity::find()
        .filter(order_items::Column::OrderId.eq(*order_id))
        .all(transaction)
        .await?;

    for line in lines {
        let Some(variant_id) = line.variant_id else {
            continue;
        };
        let Some(variant) = product_variants::Entity::find_by_id(variant_id)
            .one(transaction)
            .await? else {
            continue;
        };
        apply_stock_change(transaction, &variant, line.quantity, InventoryAction::Restore).await?;
    }

    Ok(())
}

async fn apply_stock_change<C: ConnectionTrait>(
    connection: &C,
    variant: &product_variants::Model,
    quantity: i32,
    action: InventoryAction,
) -> Result<(), AppError> {
    let next_quantity = match action {
        InventoryAction::Reserve => {
            if variant.stock_quantity < quantity {
                return Err(AppError::BadRequest(format!(
                    "requested quantity for {} exceeds available stock",
                    variant.name
                )));
            }
            variant.stock_quantity - quantity
        }
        InventoryAction::Restore => variant.stock_quantity + quantity,
    };

    let mut active_variant = product_variants::ActiveModel::from(variant.clone());
    active_variant.stock_quantity = Set(next_quantity);
    active_variant.updated_at = Set(Utc::now().fixed_offset());
    active_variant.update(connection).await?;
    Ok(())
}

pub(crate) async fn fetch_order_detail(
    db: &sea_orm::DatabaseConnection,
    order_number: &str,
) -> Result<AdminOrderDetailResponse, AppError> {
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number))
        .one(db)
        .await?
        .ok_or(AppError::NotFound)?;

    let line_items = order_items::Entity::find()
        .filter(order_items::Column::OrderId.eq(order.id))
        .order_by_asc(order_items::Column::CreatedAt)
        .all(db)
        .await?;

    let event_records = order_events::Entity::find()
        .filter(order_events::Column::OrderId.eq(order.id))
        .order_by_desc(order_events::Column::CreatedAt)
        .all(db)
        .await?;

    Ok(AdminOrderDetailResponse {
        order_number: order.order_number,
        status: order.status,
        payment_status: order.payment_status,
        currency: order.currency,
        subtotal_amount: order.subtotal_amount,
        total_amount: order.total_amount,
        customer_name: order.customer_name,
        customer_email: order.customer_email,
        customer_phone: order.customer_phone,
        shipping_address_line1: order.shipping_address_line1,
        shipping_address_line2: order.shipping_address_line2,
        shipping_city: order.shipping_city,
        shipping_region: order.shipping_region,
        shipping_postal_code: order.shipping_postal_code,
        shipping_country: order.shipping_country,
        notes: order.notes,
        payment_method: order.payment_method,
        payment_reference: order.payment_reference,
        shipping_courier: order.shipping_courier,
        tracking_number: order.tracking_number,
        paid_at: order.paid_at,
        fulfilled_at: order.fulfilled_at,
        created_at: order.created_at,
        items: line_items
            .into_iter()
            .map(|line| OrderLineResponse {
                product_name: line.product_name,
                variant_name: line.variant_name,
                sku: line.sku,
                size: line.size,
                color: line.color,
                quantity: line.quantity,
                unit_price: line.unit_price,
                line_total: line.line_total,
                image_url: line.image_url,
            })
            .collect(),
        events: event_records
            .into_iter()
            .map(|event| OrderEventResponse {
                event_type: event.event_type,
                actor: event.actor,
                message: event.message,
                details: event.details,
                created_at: event.created_at,
            })
            .collect(),
    })
}

async fn send_order_created_email(state: &AppState, order: &OrderSummaryResponse) {
    let Some(mailer) = state.mailer.as_ref() else {
        return;
    };

    let template = email_templates::order_created(order);

    if let Err(error) = mailer
        .send_email(
            &order.customer_email,
            &template.subject,
            &template.html,
            &template.text,
            &format!("order-created-{}", order.order_number),
        )
        .await
    {
        record_order_event(
            state,
            &order.order_number,
            "email_failed",
            "system",
            "Customer order confirmation email failed.".to_owned(),
            Some(error.to_string()),
        )
        .await;
        warn!("failed to send order created email: {}", error);
    } else {
        record_order_event(
            state,
            &order.order_number,
            "email_sent",
            "system",
            "Customer order confirmation email sent.".to_owned(),
            Some(order.customer_email.clone()),
        )
        .await;
    }

    let internal_template = email_templates::internal_order_created(order);
    if let Err(error) = mailer
        .send_notification_email(
            &internal_template.subject,
            &internal_template.html,
            &internal_template.text,
            &format!("internal-order-created-{}", order.order_number),
        )
        .await
    {
        record_order_event(
            state,
            &order.order_number,
            "notification_failed",
            "system",
            "Internal new-order notification email failed.".to_owned(),
            Some(error.to_string()),
        )
        .await;
        warn!("failed to send internal order created email: {}", error);
    } else {
        record_order_event(
            state,
            &order.order_number,
            "notification_sent",
            "system",
            "Internal new-order notification email sent.".to_owned(),
            None,
        )
        .await;
    }
}

pub(crate) async fn send_order_update_emails(
    state: &AppState,
    order: &AdminOrderDetailResponse,
    previous_status: &str,
    previous_payment_status: &str,
    previous_fulfilled_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
) {
    let Some(mailer) = state.mailer.as_ref() else {
        return;
    };

    if previous_payment_status != order.payment_status && order.payment_status == "paid" {
        let template = email_templates::order_paid(order);
        if let Err(error) = mailer
            .send_email(
                &order.customer_email,
                &template.subject,
                &template.html,
                &template.text,
                &format!("order-paid-{}", order.order_number),
            )
            .await
        {
            record_order_event(
                state,
                &order.order_number,
                "email_failed",
                "system",
                "Customer payment confirmation email failed.".to_owned(),
                Some(error.to_string()),
            )
            .await;
            warn!("failed to send payment email: {}", error);
        } else {
            record_order_event(
                state,
                &order.order_number,
                "email_sent",
                "system",
                "Customer payment confirmation email sent.".to_owned(),
                Some(order.customer_email.clone()),
            )
            .await;
        }
    }

    if previous_status != order.status && order.status == "fulfilled" && previous_fulfilled_at != order.fulfilled_at {
        let template = email_templates::order_fulfilled(order);
        if let Err(error) = mailer
            .send_email(
                &order.customer_email,
                &template.subject,
                &template.html,
                &template.text,
                &format!("order-fulfilled-{}", order.order_number),
            )
            .await
        {
            record_order_event(
                state,
                &order.order_number,
                "email_failed",
                "system",
                "Customer fulfilment email failed.".to_owned(),
                Some(error.to_string()),
            )
            .await;
            warn!("failed to send fulfilment email: {}", error);
        } else {
            record_order_event(
                state,
                &order.order_number,
                "email_sent",
                "system",
                "Customer fulfilment email sent.".to_owned(),
                Some(order.customer_email.clone()),
            )
            .await;
        }
    }

    if previous_status != order.status || previous_payment_status != order.payment_status {
        let internal_template =
            email_templates::internal_order_updated(order, previous_status, previous_payment_status);
        if let Err(error) = mailer
            .send_notification_email(
                &internal_template.subject,
                &internal_template.html,
                &internal_template.text,
                &format!(
                    "internal-order-updated-{}-{}-{}",
                    order.order_number, order.status, order.payment_status
                ),
            )
            .await
        {
            record_order_event(
                state,
                &order.order_number,
                "notification_failed",
                "system",
                "Internal order update notification email failed.".to_owned(),
                Some(error.to_string()),
            )
            .await;
            warn!("failed to send internal order update email: {}", error);
        } else {
            record_order_event(
                state,
                &order.order_number,
                "notification_sent",
                "system",
                "Internal order update notification email sent.".to_owned(),
                None,
            )
            .await;
        }
    }
}

pub(crate) async fn insert_order_event<C: ConnectionTrait>(
    connection: &C,
    order_id: uuid::Uuid,
    event_type: &str,
    actor: &str,
    message: String,
    details: Option<String>,
) -> Result<(), AppError> {
    order_events::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        order_id: Set(order_id),
        event_type: Set(event_type.to_owned()),
        actor: Set(actor.to_owned()),
        message: Set(message),
        details: Set(details),
        created_at: Set(Utc::now().fixed_offset()),
    }
    .insert(connection)
    .await?;

    Ok(())
}

pub(crate) async fn record_order_event(
    state: &AppState,
    order_number: &str,
    event_type: &str,
    actor: &str,
    message: String,
    details: Option<String>,
) {
    let Some(order) = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number))
        .one(&state.db)
        .await
        .ok()
        .flatten()
    else {
        return;
    };

    if let Err(error) = insert_order_event(&state.db, order.id, event_type, actor, message, details).await {
        warn!("failed to record order event: {}", error);
    }
}

pub(crate) async fn initiate_velipay_payment(
    state: &AppState,
    order: &orders::Model,
    payment_phone_number: Option<&str>,
) -> Result<CheckoutPaymentResponse, AppError> {
    let velipay = state.velipay.as_ref().ok_or(AppError::PaymentNotConfigured)?;
    let phone_number = payment_phone_number
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or(order.customer_phone.trim());
    if phone_number.is_empty() {
        return Err(AppError::BadRequest("customer phone number is required for STK push".to_owned()));
    }

    let order_total_amount = decimal_to_whole_number(order.total_amount)?;
    let amount = if state.config.app_env == "development" {
        state
            .config
            .payment_test_amount_override
            .unwrap_or(order_total_amount)
    } else {
        order_total_amount
    };
    let description = format!("Order {}", order.order_number);
    error!(
        order_number = %order.order_number,
        amount,
        order_total_amount,
        phone_number = %phone_number,
        "initiating velipay stk push"
    );
    let request = CreateStkPushRequest {
        amount,
        phone_number: phone_number.to_owned(),
        merchant_reference: order.order_number.clone(),
        description,
        settlement_mode: "auto".to_owned(),
        metadata: StkPushMetadata {
            order_id: order.order_number.clone(),
            customer_name: order.customer_name.clone(),
        },
    };

    match velipay.create_stk_push(&request).await {
        Ok(provider_response) => {
            error!(
                order_number = %order.order_number,
                provider_status = provider_response.status.as_deref().unwrap_or("unknown"),
                payment_reference = provider_response.payment_id.as_deref().or(provider_response.merchant_reference.as_deref()).unwrap_or("none"),
                "velipay stk push request completed"
            );
            let payment_reference = provider_response
                .payment_id
                .clone()
                .or_else(|| provider_response.merchant_reference.clone());
            let transaction = state.db.begin().await?;
            let mut active = orders::ActiveModel::from(order.clone());
            active.payment_method = Set(Some("velipay_stk_push".to_owned()));
            active.payment_reference = Set(payment_reference.clone());
            active.updated_at = Set(Utc::now().fixed_offset());
            let updated = active.update(&transaction).await?;
            insert_order_event(
                &transaction,
                updated.id,
                "payment_initiated",
                "system",
                "Velipay STK push initiated.".to_owned(),
                Some(format!(
                    "Status: {} · Reference: {} · Phone: {} · Charged amount: {} KES",
                    provider_response.status.clone().unwrap_or_else(|| "pending".to_owned()),
                    payment_reference.clone().unwrap_or_else(|| "Not provided".to_owned()),
                    phone_number,
                    amount
                )),
            )
            .await?;
            transaction.commit().await?;

            Ok(CheckoutPaymentResponse {
                provider: "velipay".to_owned(),
                method: "velipay_stk_push".to_owned(),
                status: provider_response.status.unwrap_or_else(|| "pending".to_owned()),
                payment_reference,
                merchant_reference: order.order_number.clone(),
                message: provider_response
                    .message
                    .unwrap_or_else(|| "STK push initiated. Ask the customer to confirm on their phone.".to_owned()),
            })
        }
        Err(error) => {
            error!(
                order_number = %order.order_number,
                error = %error,
                "velipay stk push request failed"
            );
            record_order_event(
                state,
                &order.order_number,
                "payment_initiation_failed",
                "system",
                "Velipay STK push failed to start.".to_owned(),
                Some(error.to_string()),
            )
            .await;

            Ok(CheckoutPaymentResponse {
                provider: "velipay".to_owned(),
                method: "velipay_stk_push".to_owned(),
                status: "error".to_owned(),
                payment_reference: None,
                merchant_reference: order.order_number.clone(),
                message: "The order was created, but Velipay could not start the STK push yet.".to_owned(),
            })
        }
    }
}

fn decimal_to_whole_number(value: sea_orm::prelude::Decimal) -> Result<i64, AppError> {
    let parsed = value
        .round_dp(0)
        .to_string()
        .parse::<f64>()
        .map_err(|_| AppError::BadRequest("invalid payment amount".to_owned()))?;
    Ok(parsed as i64)
}
