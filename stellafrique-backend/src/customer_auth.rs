use axum::{
    extract::{Path, Request, State},
    http::{header, HeaderMap, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;

use crate::{
    config::CustomerAuthConfig,
    entities::{order_items, orders, products, users},
    error::AppError,
    state::AppState,
};

const CUSTOMER_COOKIE_NAME: &str = "stellafrique_customer_session";
const CUSTOMER_SESSION_HOURS: i64 = 24 * 14;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct CustomerRegisterPayload {
    full_name: String,
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct CustomerLoginPayload {
    email: String,
    password: String,
}

#[derive(Serialize)]
struct CustomerSessionResponse {
    authenticated: bool,
    user: Option<CustomerUserResponse>,
}

#[derive(Serialize)]
struct CustomerUserResponse {
    id: uuid::Uuid,
    full_name: String,
    email: String,
    phone: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Deserialize)]
struct UpdateCustomerProfilePayload {
    full_name: Option<String>,
    phone: Option<String>,
    address_line1: Option<String>,
    address_line2: Option<String>,
    city: Option<String>,
    region: Option<String>,
    postal_code: Option<String>,
    country: Option<String>,
}

#[derive(Deserialize)]
struct ChangeCustomerPasswordPayload {
    current_password: String,
    new_password: String,
}

#[derive(Serialize)]
struct CustomerOrderListItem {
    order_number: String,
    status: String,
    payment_status: String,
    currency: String,
    total_amount: sea_orm::prelude::Decimal,
    item_count: usize,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Serialize)]
struct CustomerOrderDetailResponse {
    order_number: String,
    status: String,
    payment_status: String,
    payment_method: Option<String>,
    payment_reference: Option<String>,
    paid_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    currency: String,
    subtotal_amount: sea_orm::prelude::Decimal,
    total_amount: sea_orm::prelude::Decimal,
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
    shipping_courier: Option<String>,
    tracking_number: Option<String>,
    created_at: sea_orm::prelude::DateTimeWithTimeZone,
    items: Vec<CustomerOrderLineResponse>,
}

#[derive(Serialize)]
struct CustomerOrderLineResponse {
    product_slug: Option<String>,
    variant_id: Option<uuid::Uuid>,
    product_name: String,
    variant_name: Option<String>,
    sku: Option<String>,
    size: Option<String>,
    color: Option<String>,
    quantity: i32,
    unit_price: sea_orm::prelude::Decimal,
    line_total: sea_orm::prelude::Decimal,
    image_url: Option<String>,
}

#[derive(Serialize)]
struct EmptyResponse;

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/session", get(session))
}

pub fn protected_router() -> Router<AppState> {
    Router::new()
        .route("/account/profile", get(profile).patch(update_profile))
        .route("/account/password", post(change_password))
        .route("/account/orders", get(order_history))
        .route("/account/orders/:order_number", get(order_detail))
}

pub async fn require_customer_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    resolve_authenticated_user(&state, request.headers())
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(next.run(request).await)
}

async fn register(
    State(state): State<AppState>,
    Json(payload): Json<CustomerRegisterPayload>,
) -> Result<Response, AppError> {
    let auth = state
        .config
        .customer_auth
        .as_ref()
        .ok_or(AppError::Unauthorized)?;

    let email = normalize_email(&payload.email)?;
    let full_name = payload.full_name.trim();
    if full_name.is_empty() {
        return Err(AppError::BadRequest("full name is required".to_owned()));
    }
    if payload.password.trim().len() < 8 {
        return Err(AppError::BadRequest("password must be at least 8 characters".to_owned()));
    }

    let existing = users::Entity::find()
        .filter(users::Column::Email.eq(email.clone()))
        .one(&state.db)
        .await?;

    if existing.is_some() {
        return Err(AppError::BadRequest("an account with that email already exists".to_owned()));
    }

    let now = Utc::now().fixed_offset();
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let password_hash = hash_password(&payload.password, &salt);
    let user = users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(email.clone()),
        full_name: Set(full_name.to_owned()),
        phone: Set(None),
        address_line1: Set(None),
        address_line2: Set(None),
        city: Set(None),
        region: Set(None),
        postal_code: Set(None),
        country: Set(None),
        password_hash: Set(password_hash),
        password_salt: Set(salt),
        is_active: Set(true),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    build_customer_session_response(&user, auth)
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<CustomerLoginPayload>,
) -> Result<Response, AppError> {
    let auth = state
        .config
        .customer_auth
        .as_ref()
        .ok_or(AppError::Unauthorized)?;
    let email = normalize_email(&payload.email)?;

    let user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let expected_hash = hash_password(&payload.password, &user.password_salt);
    if !bool::from(expected_hash.as_bytes().ct_eq(user.password_hash.as_bytes())) {
        return Err(AppError::Unauthorized);
    }

    build_customer_session_response(&user, auth)
}

async fn logout() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, clear_customer_cookie());
    (headers, Json(EmptyResponse)).into_response()
}

async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CustomerSessionResponse>, AppError> {
    let user = resolve_authenticated_user(&state, &headers).await?;

    Ok(Json(CustomerSessionResponse {
        authenticated: user.is_some(),
        user: user.map(map_user),
    }))
}

async fn profile(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<CustomerUserResponse>, AppError> {
    let user = resolve_authenticated_user(&state, &headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(Json(map_user(user)))
}

async fn update_profile(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<UpdateCustomerProfilePayload>,
) -> Result<Json<CustomerUserResponse>, AppError> {
    let user = resolve_authenticated_user(&state, &headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if payload.full_name.is_none()
        && payload.phone.is_none()
        && payload.address_line1.is_none()
        && payload.address_line2.is_none()
        && payload.city.is_none()
        && payload.region.is_none()
        && payload.postal_code.is_none()
        && payload.country.is_none()
    {
        return Err(AppError::BadRequest("no profile changes were provided".to_owned()));
    }

    let mut active_user = users::ActiveModel::from(user);
    if let Some(full_name) = payload.full_name {
        let full_name = full_name.trim().to_owned();
        if full_name.is_empty() {
            return Err(AppError::BadRequest("full name is required".to_owned()));
        }
        active_user.full_name = Set(full_name);
    }
    if let Some(phone) = payload.phone {
        active_user.phone = Set(normalize_optional_string(phone));
    }
    if let Some(address_line1) = payload.address_line1 {
        active_user.address_line1 = Set(normalize_optional_string(address_line1));
    }
    if let Some(address_line2) = payload.address_line2 {
        active_user.address_line2 = Set(normalize_optional_string(address_line2));
    }
    if let Some(city) = payload.city {
        active_user.city = Set(normalize_optional_string(city));
    }
    if let Some(region) = payload.region {
        active_user.region = Set(normalize_optional_string(region));
    }
    if let Some(postal_code) = payload.postal_code {
        active_user.postal_code = Set(normalize_optional_string(postal_code));
    }
    if let Some(country) = payload.country {
        active_user.country = Set(normalize_optional_string(country));
    }
    active_user.updated_at = Set(Utc::now().fixed_offset());
    let user = active_user.update(&state.db).await?;

    Ok(Json(map_user(user)))
}

async fn change_password(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ChangeCustomerPasswordPayload>,
) -> Result<Json<EmptyResponse>, AppError> {
    let user = resolve_authenticated_user(&state, &headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if payload.new_password.trim().len() < 8 {
        return Err(AppError::BadRequest("new password must be at least 8 characters".to_owned()));
    }

    let expected_hash = hash_password(&payload.current_password, &user.password_salt);
    if !bool::from(expected_hash.as_bytes().ct_eq(user.password_hash.as_bytes())) {
        return Err(AppError::BadRequest("current password is incorrect".to_owned()));
    }

    let new_hash = hash_password(&payload.new_password, &user.password_salt);
    if bool::from(new_hash.as_bytes().ct_eq(user.password_hash.as_bytes())) {
        return Err(AppError::BadRequest("new password must be different from the current password".to_owned()));
    }

    let new_salt = uuid::Uuid::new_v4().simple().to_string();
    let mut active_user = users::ActiveModel::from(user);
    active_user.password_salt = Set(new_salt.clone());
    active_user.password_hash = Set(hash_password(&payload.new_password, &new_salt));
    active_user.updated_at = Set(Utc::now().fixed_offset());
    active_user.update(&state.db).await?;

    Ok(Json(EmptyResponse))
}

async fn order_history(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<CustomerOrderListItem>>, AppError> {
    let user = resolve_authenticated_user(&state, &headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let records = orders::Entity::find()
        .filter(orders::Column::UserId.eq(user.id))
        .order_by_desc(orders::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let mut response = Vec::with_capacity(records.len());
    for order in records {
        let item_count = order_items::Entity::find()
            .filter(order_items::Column::OrderId.eq(order.id))
            .all(&state.db)
            .await?
            .len();

        response.push(CustomerOrderListItem {
            order_number: order.order_number,
            status: order.status,
            payment_status: order.payment_status,
            currency: order.currency,
            total_amount: order.total_amount,
            item_count,
            created_at: order.created_at,
        });
    }

    Ok(Json(response))
}

async fn order_detail(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(order_number): Path<String>,
) -> Result<Json<CustomerOrderDetailResponse>, AppError> {
    let user = resolve_authenticated_user(&state, &headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let order = orders::Entity::find()
        .filter(orders::Column::OrderNumber.eq(order_number))
        .filter(orders::Column::UserId.eq(user.id))
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    let items = order_items::Entity::find()
        .filter(order_items::Column::OrderId.eq(order.id))
        .order_by_asc(order_items::Column::CreatedAt)
        .all(&state.db)
        .await?;

    let product_ids = items.iter().map(|item| item.product_id).collect::<Vec<_>>();
    let product_records = products::Entity::find()
        .filter(products::Column::Id.is_in(product_ids))
        .all(&state.db)
        .await?;
    let product_slug_map = product_records
        .into_iter()
        .map(|product| (product.id, product.slug))
        .collect::<std::collections::HashMap<_, _>>();

    Ok(Json(CustomerOrderDetailResponse {
        order_number: order.order_number,
        status: order.status,
        payment_status: order.payment_status,
        payment_method: order.payment_method,
        payment_reference: order.payment_reference,
        paid_at: order.paid_at,
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
        shipping_courier: order.shipping_courier,
        tracking_number: order.tracking_number,
        created_at: order.created_at,
        items: items
            .into_iter()
            .map(|item| CustomerOrderLineResponse {
                product_slug: product_slug_map.get(&item.product_id).cloned(),
                variant_id: item.variant_id,
                product_name: item.product_name,
                variant_name: item.variant_name,
                sku: item.sku,
                size: item.size,
                color: item.color,
                quantity: item.quantity,
                unit_price: item.unit_price,
                line_total: item.line_total,
                image_url: item.image_url,
            })
            .collect(),
    }))
}

pub async fn resolve_authenticated_user(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<users::Model>, AppError> {
    let auth = match state.config.customer_auth.as_ref() {
        Some(auth) => auth,
        None => return Ok(None),
        };
    let token = match read_cookie(headers, CUSTOMER_COOKIE_NAME) {
        Some(token) => token,
        None => return Ok(None),
    };
    let user_id = match verify_token(&token, &auth.session_secret) {
        Some(user_id) => user_id,
        None => return Ok(None),
    };

    users::Entity::find_by_id(user_id)
        .filter(users::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(Into::into)
}

fn build_customer_session_response(
    user: &users::Model,
    auth: &CustomerAuthConfig,
) -> Result<Response, AppError> {
    let expires_at = Utc::now() + Duration::hours(CUSTOMER_SESSION_HOURS);
    let token = issue_token(user.id, &auth.session_secret, expires_at.timestamp())?;

    let mut headers = HeaderMap::new();
    headers.insert(
        header::SET_COOKIE,
        build_customer_cookie(&token, expires_at.timestamp())?,
    );

    Ok((
        headers,
        Json(CustomerSessionResponse {
            authenticated: true,
            user: Some(map_user(user.clone())),
        }),
    )
        .into_response())
}

fn map_user(user: users::Model) -> CustomerUserResponse {
    CustomerUserResponse {
        id: user.id,
        full_name: user.full_name,
        email: user.email,
        phone: user.phone,
        address_line1: user.address_line1,
        address_line2: user.address_line2,
        city: user.city,
        region: user.region,
        postal_code: user.postal_code,
        country: user.country,
        created_at: user.created_at,
    }
}

fn normalize_email(email: &str) -> Result<String, AppError> {
    let email = email.trim().to_lowercase();
    if email.is_empty() || !email.contains('@') {
        return Err(AppError::BadRequest("a valid email is required".to_owned()));
    }
    Ok(email)
}

fn normalize_optional_string(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    if value.is_empty() {
        None
    } else {
        Some(value)
    }
}

fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password.as_bytes());
    hasher.update(b":");
    hasher.update(salt.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn issue_token(
    user_id: uuid::Uuid,
    secret: &str,
    exp: i64,
) -> Result<String, AppError> {
    let payload = format!("{user_id}:{exp}");
    let payload_encoded = URL_SAFE_NO_PAD.encode(payload.as_bytes());
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).map_err(|_| AppError::Unauthorized)?;
    mac.update(payload_encoded.as_bytes());
    let signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());
    Ok(format!("{payload_encoded}.{signature}"))
}

fn verify_token(token: &str, secret: &str) -> Option<uuid::Uuid> {
    let (payload_encoded, signature) = token.split_once('.')?;
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).ok()?;
    mac.update(payload_encoded.as_bytes());
    let expected_signature = URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes());

    if !bool::from(signature.as_bytes().ct_eq(expected_signature.as_bytes())) {
        return None;
    }

    let decoded = URL_SAFE_NO_PAD.decode(payload_encoded).ok()?;
    let payload = String::from_utf8(decoded).ok()?;
    let (user_id_raw, exp_raw) = payload.rsplit_once(':')?;
    let user_id = uuid::Uuid::parse_str(user_id_raw).ok()?;
    let exp = exp_raw.parse::<i64>().ok()?;

    if Utc::now().timestamp() >= exp {
        return None;
    }

    Some(user_id)
}

fn read_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie_header = headers.get(header::COOKIE)?.to_str().ok()?;
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|cookie| {
            let (cookie_name, value) = cookie.split_once('=')?;
            (cookie_name == name).then(|| value.to_owned())
        })
}

fn build_customer_cookie(token: &str, expires_at: i64) -> Result<HeaderValue, AppError> {
    let expires_in = (expires_at - Utc::now().timestamp()).max(0);
    HeaderValue::from_str(&format!(
        "{CUSTOMER_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={expires_in}"
    ))
    .map_err(|_| AppError::Unauthorized)
}

fn clear_customer_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "stellafrique_customer_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
    )
}
