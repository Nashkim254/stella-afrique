use axum::{
    extract::{Path, Request, State},
    http::{header, HeaderMap, HeaderValue},
    middleware::Next,
    response::{IntoResponse, Response},
    routing::{get, patch, post},
    Json, Router,
};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use chrono::{Duration, Utc};
use hmac::{Hmac, Mac};
use sea_orm::{ActiveModelTrait, ColumnTrait, EntityTrait, QueryFilter, QueryOrder, Set};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use subtle::ConstantTimeEq;
use tracing::info;

use crate::{
    config::AdminAuthConfig,
    entities::staff_users,
    error::AppError,
    state::AppState,
};

const ADMIN_COOKIE_NAME: &str = "stellafrique_admin_session";
const ADMIN_SESSION_HOURS: i64 = 12;

type HmacSha256 = Hmac<Sha256>;

#[derive(Deserialize)]
struct AdminLoginPayload {
    email: String,
    password: String,
}

#[derive(Clone, Serialize)]
pub struct AdminSessionUser {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
}

#[derive(Serialize)]
struct AdminSessionResponse {
    authenticated: bool,
    user: Option<AdminSessionUser>,
}

#[derive(Serialize)]
struct EmptyResponse;

#[derive(Serialize)]
pub struct StaffUserResponse {
    pub id: uuid::Uuid,
    pub email: String,
    pub full_name: String,
    pub role: String,
    pub is_active: bool,
    pub last_login_at: Option<sea_orm::prelude::DateTimeWithTimeZone>,
    pub created_at: sea_orm::prelude::DateTimeWithTimeZone,
}

#[derive(Deserialize)]
struct CreateStaffUserPayload {
    email: String,
    full_name: String,
    role: String,
    password: String,
}

#[derive(Deserialize)]
struct UpdateStaffUserPayload {
    full_name: Option<String>,
    role: Option<String>,
    password: Option<String>,
    is_active: Option<bool>,
}

pub fn public_router() -> Router<AppState> {
    Router::new()
        .route("/admin/auth/login", post(login))
        .route("/admin/auth/logout", post(logout))
        .route("/admin/auth/session", get(session))
}

pub fn admin_router() -> Router<AppState> {
    Router::new()
        .route("/admin/staff", get(list_staff_users).post(create_staff_user))
        .route("/admin/staff/:id", patch(update_staff_user))
}

pub async fn bootstrap_admin_staff(
    db: &sea_orm::DatabaseConnection,
    auth: Option<&AdminAuthConfig>,
) -> anyhow::Result<()> {
    let Some(auth) = auth else {
        return Ok(());
    };
    let (Some(email), Some(password)) = (
        auth.bootstrap_email.as_deref(),
        auth.bootstrap_password.as_deref(),
    ) else {
        return Ok(());
    };

    let normalized_email = normalize_email(email)
        .map_err(|error| anyhow::anyhow!("invalid ADMIN_EMAIL bootstrap value: {error}"))?;

    let existing = staff_users::Entity::find()
        .filter(staff_users::Column::Email.eq(normalized_email.clone()))
        .one(db)
        .await?;

    if existing.is_some() {
        return Ok(());
    }

    if password.trim().len() < 8 {
        return Err(anyhow::anyhow!(
            "ADMIN_PASSWORD must be at least 8 characters to bootstrap staff auth"
        ));
    }

    let now = Utc::now().fixed_offset();
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let password_hash = hash_password(password, &salt);

    staff_users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(normalized_email.clone()),
        full_name: Set(auth.bootstrap_name.trim().to_owned()),
        role: Set("owner".to_owned()),
        password_hash: Set(password_hash),
        password_salt: Set(salt),
        is_active: Set(true),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(db)
    .await?;

    info!(email = %normalized_email, "bootstrapped initial admin staff user from environment");

    Ok(())
}

pub async fn require_admin_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    resolve_authenticated_staff(&state, request.headers())
        .await?
        .ok_or(AppError::Unauthorized)?;

    Ok(next.run(request).await)
}

async fn login(
    State(state): State<AppState>,
    Json(payload): Json<AdminLoginPayload>,
) -> Result<Response, AppError> {
    let auth = state
        .config
        .admin_auth
        .as_ref()
        .ok_or(AppError::Unauthorized)?;
    let email = normalize_email(&payload.email)?;

    let user = staff_users::Entity::find()
        .filter(staff_users::Column::Email.eq(email))
        .filter(staff_users::Column::IsActive.eq(true))
        .one(&state.db)
        .await?
        .ok_or(AppError::Unauthorized)?;

    let expected_hash = hash_password(&payload.password, &user.password_salt);
    if !bool::from(expected_hash.as_bytes().ct_eq(user.password_hash.as_bytes())) {
        return Err(AppError::Unauthorized);
    }

    let mut active = staff_users::ActiveModel::from(user.clone());
    active.last_login_at = Set(Some(Utc::now().fixed_offset()));
    active.updated_at = Set(Utc::now().fixed_offset());
    let user = active.update(&state.db).await?;

    build_admin_session_response(&user, auth)
}

async fn logout() -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, clear_admin_cookie());
    (headers, Json(EmptyResponse)).into_response()
}

async fn session(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<AdminSessionResponse>, AppError> {
    let user = resolve_authenticated_staff(&state, &headers).await?;

    Ok(Json(AdminSessionResponse {
        authenticated: user.is_some(),
        user: user.map(map_staff_user),
    }))
}

pub async fn resolve_authenticated_staff(
    state: &AppState,
    headers: &HeaderMap,
) -> Result<Option<staff_users::Model>, AppError> {
    let auth = match state.config.admin_auth.as_ref() {
        Some(auth) => auth,
        None => return Ok(None),
    };
    let token = match read_cookie(headers, ADMIN_COOKIE_NAME) {
        Some(token) => token,
        None => return Ok(None),
    };
    let user_id = match verify_token(&token, &auth.session_secret) {
        Some(user_id) => user_id,
        None => return Ok(None),
    };

    staff_users::Entity::find_by_id(user_id)
        .filter(staff_users::Column::IsActive.eq(true))
        .one(&state.db)
        .await
        .map_err(Into::into)
}

pub async fn require_admin_roles(
    state: &AppState,
    headers: &HeaderMap,
    allowed_roles: &[&str],
) -> Result<staff_users::Model, AppError> {
    let staff = resolve_authenticated_staff(state, headers)
        .await?
        .ok_or(AppError::Unauthorized)?;

    if allowed_roles
        .iter()
        .any(|role| staff.role.eq_ignore_ascii_case(role))
    {
        Ok(staff)
    } else {
        Err(AppError::Unauthorized)
    }
}

async fn list_staff_users(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Vec<StaffUserResponse>>, AppError> {
    require_admin_roles(&state, &headers, &["owner"]).await?;

    let staff = staff_users::Entity::find()
        .order_by_asc(staff_users::Column::CreatedAt)
        .all(&state.db)
        .await?;

    Ok(Json(staff.into_iter().map(map_staff_response).collect()))
}

async fn create_staff_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<CreateStaffUserPayload>,
) -> Result<Json<StaffUserResponse>, AppError> {
    require_admin_roles(&state, &headers, &["owner"]).await?;

    let email = normalize_email(&payload.email)?;
    let full_name = payload.full_name.trim();
    if full_name.is_empty() {
        return Err(AppError::BadRequest("full name is required".to_owned()));
    }

    let role = normalize_staff_role(&payload.role)?;
    validate_staff_password(&payload.password)?;

    let existing = staff_users::Entity::find()
        .filter(staff_users::Column::Email.eq(email.clone()))
        .one(&state.db)
        .await?;
    if existing.is_some() {
        return Err(AppError::BadRequest("a staff user with that email already exists".to_owned()));
    }

    let now = Utc::now().fixed_offset();
    let salt = uuid::Uuid::new_v4().simple().to_string();
    let user = staff_users::ActiveModel {
        id: Set(uuid::Uuid::new_v4()),
        email: Set(email),
        full_name: Set(full_name.to_owned()),
        role: Set(role),
        password_hash: Set(hash_password(&payload.password, &salt)),
        password_salt: Set(salt),
        is_active: Set(true),
        last_login_at: Set(None),
        created_at: Set(now),
        updated_at: Set(now),
    }
    .insert(&state.db)
    .await?;

    Ok(Json(map_staff_response(user)))
}

async fn update_staff_user(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(staff_id): Path<uuid::Uuid>,
    Json(payload): Json<UpdateStaffUserPayload>,
) -> Result<Json<StaffUserResponse>, AppError> {
    let actor = require_admin_roles(&state, &headers, &["owner"]).await?;

    let staff = staff_users::Entity::find_by_id(staff_id)
        .one(&state.db)
        .await?
        .ok_or(AppError::NotFound)?;

    if payload.full_name.is_none()
        && payload.role.is_none()
        && payload.password.is_none()
        && payload.is_active.is_none()
    {
        return Err(AppError::BadRequest("no staff changes were provided".to_owned()));
    }

    if actor.id == staff.id {
        if matches!(payload.is_active, Some(false)) {
            return Err(AppError::BadRequest("you cannot deactivate your own owner account".to_owned()));
        }
        if let Some(role) = payload.role.as_deref() {
            if !role.eq_ignore_ascii_case("owner") {
                return Err(AppError::BadRequest("you cannot change your own owner role".to_owned()));
            }
        }
    }

    let mut active = staff_users::ActiveModel::from(staff);
    if let Some(full_name) = payload.full_name.as_deref() {
        let trimmed = full_name.trim();
        if trimmed.is_empty() {
            return Err(AppError::BadRequest("full name is required".to_owned()));
        }
        active.full_name = Set(trimmed.to_owned());
    }
    if let Some(role) = payload.role.as_deref() {
        active.role = Set(normalize_staff_role(role)?);
    }
    if let Some(password) = payload.password.as_deref() {
        let trimmed = password.trim();
        if !trimmed.is_empty() {
            validate_staff_password(trimmed)?;
            let salt = uuid::Uuid::new_v4().simple().to_string();
            active.password_salt = Set(salt.clone());
            active.password_hash = Set(hash_password(trimmed, &salt));
        }
    }
    if let Some(is_active) = payload.is_active {
        active.is_active = Set(is_active);
    }
    active.updated_at = Set(Utc::now().fixed_offset());

    let updated = active.update(&state.db).await?;
    Ok(Json(map_staff_response(updated)))
}

fn build_admin_session_response(
    user: &staff_users::Model,
    auth: &AdminAuthConfig,
) -> Result<Response, AppError> {
    let expires_at = Utc::now() + Duration::hours(ADMIN_SESSION_HOURS);
    let token = issue_token(user.id, &auth.session_secret, expires_at.timestamp())?;

    let mut headers = HeaderMap::new();
    headers.insert(header::SET_COOKIE, build_admin_cookie(&token, expires_at.timestamp())?);

    Ok((
        headers,
        Json(AdminSessionResponse {
            authenticated: true,
            user: Some(map_staff_user(user.clone())),
        }),
    )
        .into_response())
}

fn map_staff_user(user: staff_users::Model) -> AdminSessionUser {
    AdminSessionUser {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        role: user.role,
    }
}

fn map_staff_response(user: staff_users::Model) -> StaffUserResponse {
    StaffUserResponse {
        id: user.id,
        email: user.email,
        full_name: user.full_name,
        role: user.role,
        is_active: user.is_active,
        last_login_at: user.last_login_at,
        created_at: user.created_at,
    }
}

fn issue_token(user_id: uuid::Uuid, secret: &str, exp: i64) -> Result<String, AppError> {
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
    let exp = exp_raw.parse::<i64>().ok()?;

    if Utc::now().timestamp() >= exp {
        return None;
    }

    uuid::Uuid::parse_str(user_id_raw).ok()
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

fn build_admin_cookie(token: &str, expires_at: i64) -> Result<HeaderValue, AppError> {
    let expires_in = (expires_at - Utc::now().timestamp()).max(0);
    HeaderValue::from_str(&format!(
        "{ADMIN_COOKIE_NAME}={token}; Path=/; HttpOnly; SameSite=Lax; Max-Age={expires_in}"
    ))
    .map_err(|_| AppError::Unauthorized)
}

fn clear_admin_cookie() -> HeaderValue {
    HeaderValue::from_static(
        "stellafrique_admin_session=; Path=/; HttpOnly; SameSite=Lax; Max-Age=0",
    )
}

fn hash_password(password: &str, salt: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(password.as_bytes());
    format!("{:x}", hasher.finalize())
}

fn validate_staff_password(password: &str) -> Result<(), AppError> {
    if password.trim().len() < 8 {
        return Err(AppError::BadRequest(
            "password must be at least 8 characters".to_owned(),
        ));
    }
    Ok(())
}

fn normalize_staff_role(role: &str) -> Result<String, AppError> {
    let normalized = role.trim().to_lowercase();
    match normalized.as_str() {
        "owner" | "admin" | "catalog" | "fulfilment" | "finance" => Ok(normalized),
        _ => Err(AppError::BadRequest(
            "role must be one of: owner, admin, catalog, fulfilment, finance".to_owned(),
        )),
    }
}

fn normalize_email(email: &str) -> Result<String, AppError> {
    let normalized = email.trim().to_lowercase();
    if normalized.is_empty() || !normalized.contains('@') {
        return Err(AppError::BadRequest("a valid email is required".to_owned()));
    }
    Ok(normalized)
}
