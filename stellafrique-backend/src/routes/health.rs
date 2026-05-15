use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    environment: String,
}

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(
    axum::extract::State(state): axum::extract::State<AppState>,
) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        environment: state.config.app_env,
    })
}
