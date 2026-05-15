use axum::Router;
use sea_orm::DatabaseConnection;
use tower_http::{cors::CorsLayer, trace::TraceLayer};

use crate::{config::Config, routes, state::AppState};

pub fn build_app(config: Config, db: DatabaseConnection) -> Router {
    let state = AppState::new(config, db);

    Router::new()
        .nest("/api/v1", routes::router())
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
