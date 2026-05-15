pub mod health;
pub mod storefront;

use axum::Router;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(storefront::router())
}
