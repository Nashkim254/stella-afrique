pub mod catalog;
pub mod dashboard;
pub mod health;
pub mod orders;
pub mod payments;

use axum::{middleware, Router};

use crate::{auth, customer_auth, state::AppState};

pub fn router(state: AppState) -> Router<AppState> {
    let admin_router = Router::new()
        .merge(auth::admin_router())
        .merge(dashboard::admin_router())
        .merge(catalog::admin_router())
        .merge(orders::admin_router())
        .merge(payments::admin_router())
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::require_admin_auth,
        ));

    let customer_router = customer_auth::protected_router().route_layer(
        middleware::from_fn_with_state(state.clone(), customer_auth::require_customer_auth),
    );

    Router::new()
        .merge(health::router())
        .merge(auth::public_router())
        .merge(customer_auth::public_router())
        .merge(catalog::public_router())
        .merge(orders::public_router())
        .merge(payments::public_router())
        .merge(customer_router)
        .merge(admin_router)
}
