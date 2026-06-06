pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod health;
pub mod state;

use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

pub fn build_router(state: state::AppState) -> Router {
    let api_v1 = Router::new()
        .merge(auth::route::public_router())
        .merge(
            auth::route::protected_router().route_layer(middleware::from_fn_with_state(
                state.clone(),
                auth::middleware::require_auth,
            )),
        );
    Router::new()
        .merge(health::router())
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
