use axum::{
    body::Bytes,
    extract::{Path, Query, State},
    http::HeaderMap,
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use sea_orm::{ActiveModelTrait, ColumnTrait, Condition, EntityTrait, QueryFilter, QueryOrder, Set, TransactionTrait};
use serde::{Deserialize, Serialize};
use tracing::{error, info, warn};

use crate::{
    auth,
    customer_auth,
    entities::{orders, withdrawals},
    error::AppError,
    routes::orders::{fetch_order_detail, insert_order_event, record_order_event, send_order_update_emails, AdminOrderDetailResponse},
    state::AppState,
    velipay::BusinessWithdrawRequest,
};

#[derive(Serialize)]
pub struct PublicPaymentStatusResponse {
    pub order_number: String,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub provider_status: Option<String>,
    pub paid_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
}

#[derive(Deserialize)]
pub struct PublicPaymentStatusQuery {
    customer_email: Option<String>,
}

#[derive(Serialize)]
pub struct AdminPaymentStatusResponse {
    pub order_number: String,
    pub payment_status: String,
    pub payment_method: Option<String>,
    pub payment_reference: Option<String>,
    pub provider_status: Option<String>,
    pub paid_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
}

#[derive(Deserialize)]
pub struct RetryVelipayPaymentPayload {
    pub phone_number: Option<String>,
    pub customer_email: Option<String>,
}

#[derive(Deserialize)]
pub struct ReleaseFundsPayload {
    pub amount: Option<i64>,
    pub destination_type: Option<String>,
    pub destination: String,
}

#[derive(Serialize)]
pub struct ReleaseFundsResponse {
    pub order_number: String,
    pub status: String,
    pub release_reference: String,
    pub receipt_number: Option<String>,
    pub external_request_id: Option<String>,
    pub message: String,
}

#[derive(Serialize)]
pub struct VelipayWebhookProbeResponse {
    pub status: String,
    pub provider: String,
    pub webhook_path: String,
    pub webhook_url: Option<String>,
    pub environment: String,
    pub timestamp: String,
}

#[derive(Serialize)]
pub struct WithdrawalsOverviewResponse {
    pub balance: WithdrawalBalanceResponse,
    pub withdrawals: Vec<WithdrawalHistoryItemResponse>,
}

#[derive(Serialize)]
pub struct WithdrawalBalanceResponse {
    pub currency: String,
    pub collected_revenue: sea_orm::prelude::Decimal,
    pub successful_withdrawals: sea_orm::prelude::Decimal,
    pub pending_withdrawals: sea_orm::prelude::Decimal,
    pub available_balance: sea_orm::prelude::Decimal,
}

#[derive(Serialize)]
pub struct WithdrawalHistoryItemResponse {
    pub id: uuid::Uuid,
    pub release_reference: String,
    pub amount: sea_orm::prelude::Decimal,
    pub currency: String,
    pub destination_type: String,
    pub destination: String,
    pub status: String,
    pub requested_by_email: String,
    pub requested_by_role: String,
    pub payout_id: Option<String>,
    pub receipt_number: Option<String>,
    pub external_request_id: Option<String>,
    pub failure_reason: Option<String>,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub updated_at: sea_orm::prelude::DateTimeWithTimeZone,
    pub completed_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
}

#[derive(Deserialize)]
pub struct CreateWithdrawalPayload {
    pub amount: i64,
    pub destination_type: Option<String>,
    pub destination: String,
}

#[derive(Serialize)]
pub struct CreateWithdrawalResponse {
    pub status: String,
    pub release_reference: String,
    pub receipt_number: Option<String>,
    pub external_request_id: Option<String>,
    pub message: String,
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/payments/velipay/webhook-probe", get(get_velipay_webhook_probe))
        .route(
            "/payments/velipay/webhook",
            get(get_velipay_webhook_info).post(handle_velipay_webhook),
        )
        .route("/payments/velipay/orders/:order_number/status", get(get_public_payment_status))
        .route("/payments/velipay/orders/:order_number/retry", post(retry_public_velipay_payment))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/withdrawals", get(get_withdrawals_overview).post(create_withdrawal))
        .route("/admin/orders/:order_number/payment-status", get(get_admin_payment_status))
        .route("/admin/orders/:order_number/retry-payment", post(retry_velipay_payment))
        .route("/admin/orders/:order_number/release-funds", post(release_funds_for_order))
}

async fn get_withdrawals_overview(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<WithdrawalsOverviewResponse>, AppError> {
    auth::require_admin_roles(&state, &headers, &["owner", "admin"]).await?;
    let balance = compute_withdrawal_balance(&state).await?;
    let withdrawals = withdrawals::Entity::find()
        .order_by_desc(withdrawals::Column::CreatedAt)
        .all(&state.db)
        .await?
        .into_iter()
        .map(map_withdrawal_history_item)
        .collect();

    Ok(Json(WithdrawalsOverviewResponse { balance, withdrawals }))
}

async fn create_withdrawal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateWithdrawalPayload>,
) -> Result<Json<CreateWithdrawalResponse>, AppError> {
    let staff = auth::require_admin_roles(&state, &headers, &["owner", "admin"]).await?;
    let response = initiate_withdrawal_request(&state, &staff, &payload, None).await?;
    Ok(Json(response))
}

async fn get_velipay_webhook_probe(
    State(state): State<AppState>,
) -> Result<Json<VelipayWebhookProbeResponse>, AppError> {
    let webhook_url = state
        .config
        .velipay
        .as_ref()
        .and_then(|config| config.webhook_url.clone());

    info!(webhook_url = webhook_url.as_deref().unwrap_or("not_configured"), "velipay webhook probe requested");

    Ok(Json(VelipayWebhookProbeResponse {
        status: "ok".to_owned(),
        provider: "velipay".to_owned(),
        webhook_path: "/api/v1/payments/velipay/webhook".to_owned(),
        webhook_url,
        environment: state.config.app_env.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }))
}

async fn get_velipay_webhook_info(
    State(state): State<AppState>,
) -> Result<Json<VelipayWebhookProbeResponse>, AppError> {
    let webhook_url = state
        .config
        .velipay
        .as_ref()
        .and_then(|config| config.webhook_url.clone());

    info!(webhook_url = webhook_url.as_deref().unwrap_or("not_configured"), "velipay webhook info requested");

    Ok(Json(VelipayWebhookProbeResponse {
        status: "ok".to_owned(),
        provider: "velipay".to_owned(),
        webhook_path: "/api/v1/payments/velipay/webhook".to_owned(),
        webhook_url,
        environment: state.config.app_env.clone(),
        timestamp: Utc::now().to_rfc3339(),
    }))
}

async fn handle_velipay_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, AppError> {
    info!(body_bytes = body.len(), "velipay webhook received");
    let velipay = state.velipay.as_ref().ok_or(AppError::PaymentNotConfigured)?;
    let signature = headers
        .get("X-VeliPay-Signature")
        .and_then(|value| value.to_str().ok())
        .ok_or(AppError::Unauthorized)?;

    if !velipay.verify_signature(&body, signature) {
        error!("velipay webhook signature verification failed");
        return Err(AppError::Unauthorized);
    }
    info!("velipay webhook signature verified");

    let payload = serde_json::from_slice::<crate::velipay::VelipayWebhookPayload>(&body)
        .map_err(|_| AppError::BadRequest("invalid velipay webhook payload".to_owned()))?;
    info!(
        event = %payload.event,
        payment_id = payload.data.payment_id.as_deref().unwrap_or("none"),
        merchant_reference = payload.data.merchant_reference.as_deref().unwrap_or("none"),
        provider_status = payload.data.status.as_deref().unwrap_or("none"),
        receipt_number = payload.data.receipt_number.as_deref().unwrap_or("none"),
        "velipay webhook payload parsed"
    );

    match payload.event.as_str() {
        "payment.paid" => {
            if let Some(order_number) = locate_order_number_for_webhook(&state, &payload.data).await? {
                info!(order_number = %order_number, "velipay webhook matched paid payment to order");
                mark_order_paid_from_provider(
                    &state,
                    &order_number,
                    payload.data.payment_id.clone(),
                    payload.data.status.clone(),
                    payload.data.receipt_number.clone(),
                )
                .await?;
            } else {
                warn!("velipay webhook could not match paid payment to an order");
            }
        }
        "payment.failed" => {
            if let Some(order_number) = locate_order_number_for_webhook(&state, &payload.data).await? {
                info!(order_number = %order_number, "velipay webhook matched failed payment to order");
                record_payment_failure(
                    &state,
                    &order_number,
                    payload.data.payment_id.clone(),
                    payload.data.status.clone(),
                    payload.data.failure_reason.clone(),
                )
                .await?;
            } else {
                warn!("velipay webhook could not match failed payment to an order");
            }
        }
        "payment.settled" | "payment.payout_success" => {
            if let Some(order_number) = locate_order_number_for_webhook(&state, &payload.data).await? {
                let status = payload.data.status.clone().unwrap_or_else(|| payload.event.clone());
                info!(order_number = %order_number, event = %payload.event, provider_status = %status, "velipay webhook matched settlement update to order");
                record_order_event(
                    &state,
                    &order_number,
                    "payment_provider_update",
                    "velipay",
                    format!("Velipay reported {}.", payload.event),
                    Some(status),
                )
                .await;
            } else {
                warn!(event = %payload.event, "velipay settlement webhook could not match payment to an order");
            }
        }
        "withdrawal.success" => {
            if let Some(release_reference) = payload.data.release_reference.as_deref() {
                update_withdrawal_from_webhook(
                    &state,
                    release_reference,
                    payload.data.status.clone().unwrap_or_else(|| "succeeded".to_owned()),
                    payload.data.receipt_number.clone(),
                    None,
                    true,
                )
                .await?;
            } else {
                warn!("velipay withdrawal success webhook missing release reference");
            }
        }
        "withdrawal.failed" => {
            if let Some(release_reference) = payload.data.release_reference.as_deref() {
                update_withdrawal_from_webhook(
                    &state,
                    release_reference,
                    payload.data.status.clone().unwrap_or_else(|| "failed".to_owned()),
                    payload.data.receipt_number.clone(),
                    payload.data.failure_reason.clone(),
                    false,
                )
                .await?;
            } else {
                warn!("velipay withdrawal failed webhook missing release reference");
            }
        }
        _ => {
            info!(event = %payload.event, "velipay webhook ignored unhandled event");
        }
    }

    Ok(Json(serde_json::json!({ "received": true })))
}

async fn get_public_payment_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
    Query(query): Query<PublicPaymentStatusQuery>,
) -> Result<Json<PublicPaymentStatusResponse>, AppError> {
    let order = resolve_customer_visible_order(&state, &headers, &order_number, query.customer_email.as_deref()).await?;

    let mut provider_status = None;
    if let (Some(velipay), Some(payment_reference)) = (state.velipay.as_ref(), order.payment_reference.as_deref()) {
        match velipay.get_payment_status(payment_reference).await {
            Ok(status) => {
                provider_status = status.status.clone();
                if payment_is_collected(&status) && order.payment_status != "paid" {
                    mark_order_paid_from_provider(
                        &state,
                        &order.order_number,
                        status.payment_id.clone().or_else(|| Some(payment_reference.to_owned())),
                        status.status.clone(),
                        status.receipt_number.clone(),
                    )
                    .await?;
                }
            }
            Err(error) => {
                warn!("failed to fetch velipay status for {}: {}", order.order_number, error);
            }
        }
    }

    let refreshed = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(Json(PublicPaymentStatusResponse {
        order_number: refreshed.order_number,
        payment_status: refreshed.payment_status,
        payment_method: refreshed.payment_method,
        payment_reference: refreshed.payment_reference,
        provider_status,
        paid_at: refreshed.paid_at,
    }))
}

async fn get_admin_payment_status(
    State(state): State<AppState>,
    Path(order_number): Path<String>,
) -> Result<Json<AdminPaymentStatusResponse>, AppError> {
    let status = refresh_payment_status_for_order(&state, &order_number).await?;
    Ok(Json(status))
}

async fn retry_velipay_payment(
    State(state): State<AppState>,
    Path(order_number): Path<String>,
    Json(payload): Json<RetryVelipayPaymentPayload>,
) -> Result<Json<crate::routes::orders::CheckoutPaymentResponse>, AppError> {
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.clone()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    if order.status == "cancelled" {
        return Err(AppError::BadRequest("cancelled orders cannot be retried for payment".to_owned()));
    }
    if order.payment_status == "paid" {
        return Err(AppError::BadRequest("this order is already marked as paid".to_owned()));
    }

    let response = crate::routes::orders::initiate_velipay_payment(
        &state,
        &order,
        payload.phone_number.as_deref(),
    )
    .await?;

    Ok(Json(response))
}

async fn retry_public_velipay_payment(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
    Json(payload): Json<RetryVelipayPaymentPayload>,
) -> Result<Json<crate::routes::orders::CheckoutPaymentResponse>, AppError> {
    let order = resolve_customer_visible_order(
        &state,
        &headers,
        &order_number,
        payload.customer_email.as_deref(),
    )
    .await?;

    if order.status == "cancelled" {
        return Err(AppError::BadRequest(
            "cancelled orders cannot be retried for payment".to_owned(),
        ));
    }
    if order.payment_status == "paid" {
        return Err(AppError::BadRequest(
            "this order is already marked as paid".to_owned(),
        ));
    }
    if order.payment_method.as_deref() != Some("velipay_stk_push") {
        return Err(AppError::BadRequest(
            "only Velipay STK push orders can be retried here".to_owned(),
        ));
    }

    let response = crate::routes::orders::initiate_velipay_payment(
        &state,
        &order,
        payload.phone_number.as_deref(),
    )
    .await?;

    Ok(Json(response))
}

async fn release_funds_for_order(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
    Json(payload): Json<ReleaseFundsPayload>,
) -> Result<Json<ReleaseFundsResponse>, AppError> {
    let staff = auth::require_admin_roles(&state, &headers, &["owner", "admin"]).await?;
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.clone()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    if order.status == "cancelled" {
        return Err(AppError::BadRequest(
            "cancelled orders cannot release funds".to_owned(),
        ));
    }
    if order.payment_status != "paid" {
        return Err(AppError::BadRequest(
            "only paid orders can release funds".to_owned(),
        ));
    }
    if order.payment_method.as_deref() != Some("velipay_stk_push") {
        return Err(AppError::BadRequest(
            "release funds is only available for Velipay STK payments".to_owned(),
        ));
    }

    let requested_amount = payload
        .amount
        .unwrap_or(decimal_to_whole_number(order.total_amount)?);
    if requested_amount < 0 {
        return Err(AppError::BadRequest(
            "release amount must be zero or a positive whole number".to_owned(),
        ));
    }

    let source_payment_id = order.payment_reference.clone().ok_or(AppError::BadRequest(
        "this order does not have a payment reference to release against".to_owned(),
    ))?;
    let response = initiate_withdrawal_request(
        &state,
        &staff,
        &CreateWithdrawalPayload {
            amount: requested_amount,
            destination_type: payload.destination_type.clone(),
            destination: payload.destination.clone(),
        },
        Some(source_payment_id),
    )
    .await?;

    Ok(Json(ReleaseFundsResponse {
        order_number: order.order_number,
        status: response.status,
        release_reference: response.release_reference,
        receipt_number: response.receipt_number,
        external_request_id: response.external_request_id,
        message: response.message,
    }))
}

async fn resolve_customer_visible_order(
    state: &AppState,
    headers: &HeaderMap,
    order_number: &str,
    customer_email: Option<&str>,
) -> Result<orders::Model, AppError> {
    let authenticated_user = customer_auth::resolve_authenticated_user(state, headers).await?;

    let mut query = orders::Entity::find().filter(orders::Column::OrderNumber.eq(order_number.to_owned()));
    if let Some(user) = authenticated_user {
        query = query.filter(orders::Column::UserId.eq(user.id));
    } else if let Some(email) = customer_email.map(str::trim).filter(|value| !value.is_empty()) {
        query = query.filter(orders::Column::CustomerEmail.eq(email.to_lowercase()));
    } else {
        return Err(AppError::Unauthorized);
    }

    query.one(&state.db).await?.ok_or(AppError::NotFound)
}

async fn refresh_payment_status_for_order(
    state: &AppState,
    order_number: &str,
) -> Result<AdminPaymentStatusResponse, AppError> {
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.to_owned()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let mut provider_status = None;
    if let (Some(velipay), Some(payment_reference)) = (state.velipay.as_ref(), order.payment_reference.as_deref()) {
        match velipay.get_payment_status(payment_reference).await {
            Ok(status) => {
                provider_status = status.status.clone();
                if payment_is_collected(&status) && order.payment_status != "paid" {
                    mark_order_paid_from_provider(
                        state,
                        &order.order_number,
                        status.payment_id.clone().or_else(|| Some(payment_reference.to_owned())),
                        status.status.clone(),
                        status.receipt_number.clone(),
                    )
                    .await?;
                } else if payment_is_failed(&status) {
                    record_payment_failure(
                        state,
                        &order.order_number,
                        status.payment_id.clone().or_else(|| Some(payment_reference.to_owned())),
                        status.status.clone(),
                        None,
                    )
                    .await?;
                }
            }
            Err(error) => {
                warn!("failed to fetch velipay status for {}: {}", order.order_number, error);
                return Err(AppError::ExternalService(format!("failed to fetch velipay status: {}", error)));
            }
        }
    }

    let refreshed = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.to_owned()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    Ok(AdminPaymentStatusResponse {
        order_number: refreshed.order_number,
        payment_status: refreshed.payment_status,
        payment_method: refreshed.payment_method,
        payment_reference: refreshed.payment_reference,
        provider_status,
        paid_at: refreshed.paid_at,
    })
}

fn payment_is_collected(status: &crate::velipay::PaymentStatusResponse) -> bool {
    status.status.as_deref() == Some("paid")
        || status.amount_received.unwrap_or_default() > 0
        || status.receipt_number.as_deref().is_some_and(|value| !value.trim().is_empty())
}

fn payment_is_failed(status: &crate::velipay::PaymentStatusResponse) -> bool {
    matches!(status.status.as_deref(), Some("failed" | "cancelled" | "expired"))
        && status.amount_received.unwrap_or_default() <= 0
        && status
            .receipt_number
            .as_deref()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true)
}

async fn locate_order_number_for_webhook(
    state: &AppState,
    payload: &crate::velipay::VelipayWebhookData,
) -> Result<Option<String>, AppError> {
    if let Some(payment_id) = payload.payment_id.as_deref() {
        if let Some(order) = orders::Entity::find()
            .filter(orders::Column::PaymentReference.eq(payment_id.to_owned()))
            .one(&state.db)
            .await?
        {
            info!(order_number = %order.order_number, payment_id = %payment_id, "matched velipay payment id to order");
            return Ok(Some(order.order_number));
        }
    }

    if let Some(merchant_reference) = payload.merchant_reference.as_deref() {
        if let Some(order) = orders::Entity::find()
            .filter(Condition::any()
                .add(orders::Column::OrderNumber.eq(merchant_reference.to_owned()))
                .add(orders::Column::PaymentReference.eq(merchant_reference.to_owned())))
            .one(&state.db)
            .await?
        {
            info!(order_number = %order.order_number, merchant_reference = %merchant_reference, "matched velipay merchant reference to order");
            return Ok(Some(order.order_number));
        }
    }

    warn!(
        payment_id = payload.payment_id.as_deref().unwrap_or("none"),
        merchant_reference = payload.merchant_reference.as_deref().unwrap_or("none"),
        "unable to match velipay webhook payload to order"
    );
    Ok(None)
}

async fn compute_withdrawal_balance(
    state: &AppState,
) -> Result<WithdrawalBalanceResponse, AppError> {
    let paid_orders = orders::Entity::find()
        .filter(orders::Column::PaymentStatus.eq("paid"))
        .filter(orders::Column::Status.ne("cancelled"))
        .all(&state.db)
        .await?;
    let withdrawal_records = withdrawals::Entity::find().all(&state.db).await?;

    let collected_revenue = paid_orders
        .iter()
        .fold(sea_orm::prelude::Decimal::ZERO, |sum, order| sum + order.total_amount);
    let successful_withdrawals = withdrawal_records
        .iter()
        .filter(|withdrawal| withdrawal_is_success(&withdrawal.status))
        .fold(sea_orm::prelude::Decimal::ZERO, |sum, withdrawal| sum + withdrawal.amount);
    let pending_withdrawals = withdrawal_records
        .iter()
        .filter(|withdrawal| withdrawal_is_pending(&withdrawal.status))
        .fold(sea_orm::prelude::Decimal::ZERO, |sum, withdrawal| sum + withdrawal.amount);
    let available_balance = (collected_revenue - successful_withdrawals - pending_withdrawals)
        .max(sea_orm::prelude::Decimal::ZERO);

    Ok(WithdrawalBalanceResponse {
        currency: "KES".to_owned(),
        collected_revenue,
        successful_withdrawals,
        pending_withdrawals,
        available_balance,
    })
}

fn map_withdrawal_history_item(withdrawal: withdrawals::Model) -> WithdrawalHistoryItemResponse {
    WithdrawalHistoryItemResponse {
        id: withdrawal.id,
        release_reference: withdrawal.release_reference,
        amount: withdrawal.amount,
        currency: withdrawal.currency,
        destination_type: withdrawal.destination_type,
        destination: withdrawal.destination,
        status: withdrawal.status,
        requested_by_email: withdrawal.requested_by_email,
        requested_by_role: withdrawal.requested_by_role,
        payout_id: withdrawal.payout_id,
        receipt_number: withdrawal.receipt_number,
        external_request_id: withdrawal.external_request_id,
        failure_reason: withdrawal.failure_reason,
        created_at: withdrawal.created_at,
        updated_at: withdrawal.updated_at,
        completed_at: withdrawal.completed_at,
    }
}

async fn initiate_withdrawal_request(
    state: &AppState,
    staff: &crate::entities::staff_users::Model,
    payload: &CreateWithdrawalPayload,
    source_payment_id: Option<String>,
) -> Result<CreateWithdrawalResponse, AppError> {
    let velipay = state.velipay.as_ref().ok_or(AppError::PaymentNotConfigured)?;
    let destination = payload.destination.trim();
    if destination.is_empty() {
        return Err(AppError::BadRequest("destination is required".to_owned()));
    }
    if payload.amount <= 0 {
        return Err(AppError::BadRequest(
            "withdrawal amount must be a positive whole number".to_owned(),
        ));
    }

    let destination_type = payload
        .destination_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("phone");
    if destination_type != "phone" {
        return Err(AppError::BadRequest(
            "only phone destination releases are currently supported".to_owned(),
        ));
    }

    let balance = compute_withdrawal_balance(state).await?;
    let requested_amount_decimal = sea_orm::prelude::Decimal::from(payload.amount);
    if requested_amount_decimal > balance.available_balance {
        return Err(AppError::BadRequest(format!(
            "withdrawal amount exceeds available balance of {} {}",
            balance.currency, balance.available_balance
        )));
    }

    let release_reference = format!("WD-{}", uuid::Uuid::new_v4().simple());
    info!(
        staff_email = %staff.email,
        requested_amount = payload.amount,
        destination = destination,
        release_reference = %release_reference,
        source_payment_id = source_payment_id.as_deref().unwrap_or("none"),
        "processing pooled withdrawal request"
    );

    let response = velipay
        .withdraw_business_funds(&BusinessWithdrawRequest {
            amount: payload.amount,
            source_payment_id,
            release_reference: release_reference.clone(),
            destination_type: destination_type.to_owned(),
            destination: destination.to_owned(),
        })
        .await
        .map_err(|error| {
            error!(
                staff_email = %staff.email,
                requested_amount = payload.amount,
                destination = destination,
                release_reference = %release_reference,
                error = %error,
                "velipay pooled withdrawal request failed"
            );
            AppError::ExternalService(format!("failed to release funds: {error}"))
        })?;

    let now = Utc::now().fixed_offset();
    let status = response.status.clone().unwrap_or_else(|| "pending".to_owned());
    withdrawals::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        release_reference: Set(response.release_reference.clone().unwrap_or_else(|| release_reference.clone())),
        amount: Set(requested_amount_decimal),
        currency: Set(balance.currency.clone()),
        destination_type: Set(destination_type.to_owned()),
        destination: Set(destination.to_owned()),
        status: Set(status.clone()),
        payout_id: Set(response.payout_id.map(|value| value.to_string())),
        receipt_number: Set(response.receipt_number.clone()),
        external_request_id: Set(response.external_request_id.clone()),
        failure_reason: Set(None),
        requested_by_email: Set(staff.email.clone()),
        requested_by_role: Set(staff.role.clone()),
        created_at: Set(now),
        updated_at: Set(now),
        completed_at: Set(if withdrawal_is_success(&status) { Some(now) } else { None }),
    }
    .insert(&state.db)
    .await?;

    Ok(CreateWithdrawalResponse {
        status,
        release_reference: response.release_reference.unwrap_or(release_reference),
        receipt_number: response.receipt_number,
        external_request_id: response.external_request_id,
        message: response
            .message
            .unwrap_or_else(|| "Withdrawal request submitted to Velipay.".to_owned()),
    })
}

async fn update_withdrawal_from_webhook(
    state: &AppState,
    release_reference: &str,
    status: String,
    receipt_number: Option<String>,
    failure_reason: Option<String>,
    successful: bool,
) -> Result<(), AppError> {
    let withdrawal = withdrawals::Entity::find()
        .filter(withdrawals::Column::ReleaseReference.eq(release_reference.to_owned()))
        .one(&state.db)
        .await?;

    let Some(withdrawal) = withdrawal else {
        warn!(release_reference = %release_reference, "velipay withdrawal webhook could not match release reference to a withdrawal");
        return Ok(());
    };

    let now = Utc::now().fixed_offset();
    let mut active = withdrawals::ActiveModel::from(withdrawal);
    active.status = Set(status);
    active.receipt_number = Set(receipt_number);
    active.failure_reason = Set(failure_reason);
    active.updated_at = Set(now);
    active.completed_at = Set(if successful { Some(now) } else { None });
    active.update(&state.db).await?;
    Ok(())
}

fn withdrawal_is_success(status: &str) -> bool {
    matches!(
        status.to_lowercase().as_str(),
        "success" | "succeeded" | "completed" | "settled"
    )
}

fn withdrawal_is_pending(status: &str) -> bool {
    !withdrawal_is_success(status)
        && !matches!(
            status.to_lowercase().as_str(),
            "failed" | "cancelled" | "rejected" | "error"
        )
}

pub(crate) async fn mark_order_paid_from_provider(
    state: &AppState,
    order_number: &str,
    payment_reference: Option<String>,
    provider_status: Option<String>,
    receipt_number: Option<String>,
) -> Result<(), AppError> {
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.to_owned()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let previous_status = order.status.clone();
    let previous_payment_status = order.payment_status.clone();
    let previous_fulfilled_at = order.fulfilled_at;
    info!(
        order_number = %order.order_number,
        previous_payment_status = %previous_payment_status,
        provider_status = provider_status.as_deref().unwrap_or("none"),
        receipt_number = receipt_number.as_deref().unwrap_or("none"),
        "marking order as paid from velipay provider update"
    );

    let mut active = orders::ActiveModel::from(order.clone());
    active.payment_status = Set("paid".to_owned());
    active.payment_method = Set(Some("velipay_stk_push".to_owned()));
    if let Some(reference) = payment_reference.clone() {
        active.payment_reference = Set(Some(reference));
    }
    active.paid_at = Set(Some(Utc::now().fixed_offset()));
    active.updated_at = Set(Utc::now().fixed_offset());

    let transaction = state.db.begin().await?;
    let updated = active.update(&transaction).await?;
    insert_order_event(
        &transaction,
        updated.id,
        "payment_status_updated",
        "velipay",
        "Payment status changed to paid from Velipay callback.".to_owned(),
        Some(format!(
            "Provider status: {} · Receipt: {}",
            provider_status.unwrap_or_else(|| "paid".to_owned()),
            receipt_number.unwrap_or_else(|| "Not provided".to_owned())
        )),
    )
    .await?;
    transaction.commit().await?;

    let detail: AdminOrderDetailResponse = fetch_order_detail(&state.db, order_number).await?;
    send_order_update_emails(
        state,
        &detail,
        &previous_status,
        &previous_payment_status,
        previous_fulfilled_at,
    )
    .await;

    Ok(())
}

async fn record_payment_failure(
    state: &AppState,
    order_number: &str,
    payment_reference: Option<String>,
    provider_status: Option<String>,
    failure_reason: Option<String>,
) -> Result<(), AppError> {
    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number.to_owned()))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;
    info!(
        order_number = %order.order_number,
        provider_status = provider_status.as_deref().unwrap_or("none"),
        failure_reason = failure_reason.as_deref().unwrap_or("none"),
        "recording velipay payment failure"
    );

    let mut active = orders::ActiveModel::from(order.clone());
    active.payment_method = Set(Some("velipay_stk_push".to_owned()));
    if let Some(reference) = payment_reference {
        active.payment_reference = Set(Some(reference));
    }
    active.updated_at = Set(Utc::now().fixed_offset());

    let transaction = state.db.begin().await?;
    let updated = active.update(&transaction).await?;
    insert_order_event(
        &transaction,
        updated.id,
        "payment_failed",
        "velipay",
        "Velipay reported a failed or cancelled payment.".to_owned(),
        Some(format!(
            "Status: {} · Reason: {}",
            provider_status.unwrap_or_else(|| "failed".to_owned()),
            failure_reason.unwrap_or_else(|| "Not provided".to_owned())
        )),
    )
    .await?;
    transaction.commit().await?;

    Ok(())
}

fn decimal_to_whole_number(value: sea_orm::prelude::Decimal) -> Result<i64, AppError> {
    let parsed = value
        .round_dp(0)
        .to_string()
        .parse::<f64>()
        .map_err(|_| AppError::BadRequest("invalid payment amount".to_owned()))?;
    Ok(parsed as i64)
}
