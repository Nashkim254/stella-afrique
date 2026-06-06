use axum::{http::{header, HeaderValue, Method}, Router};
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, routes, state::AppState};

pub fn build_app(config: Config, db: DatabaseConnection) -> Router {
    let frontend_origin = config.frontend_origin.clone();
    let state = AppState::new(config, db);
    let cors_origin = frontend_origin
        .parse::<HeaderValue>()
        .unwrap_or_else(|_| HeaderValue::from_static("http://localhost:3001"));

    Router::new()
        .nest("/api/v1", routes::router(state.clone()))
        .layer(
            CorsLayer::new()
                .allow_origin(cors_origin)
                .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE, Method::OPTIONS])
                .allow_headers([header::CONTENT_TYPE, header::COOKIE, header::SET_COOKIE])
                .allow_credentials(true)
                ,
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
