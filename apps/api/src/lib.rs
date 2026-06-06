pub mod analysis;
pub mod auth;
pub mod config;
pub mod db;
pub mod error;
pub mod health;
pub mod knowledge;
pub mod problem;
pub mod state;
pub mod submission;
pub mod tag;

use axum::{middleware, Router};
use tower_http::trace::TraceLayer;

pub fn build_router(state: state::AppState) -> Router {
    let auth_layer = middleware::from_fn_with_state(
        state.clone(),
        auth::middleware::require_auth,
    );

    let api_v1 = Router::new()
        .merge(auth::route::public_router())
        .merge(auth::route::protected_router().route_layer(auth_layer.clone()))
        .merge(problem::route::protected_router().route_layer(auth_layer.clone()))
        .merge(submission::route::protected_router().route_layer(auth_layer.clone()))
        .merge(knowledge::route::protected_router().route_layer(auth_layer.clone()))
        .merge(tag::route::protected_router().route_layer(auth_layer.clone()))
        .merge(analysis::route::protected_router().route_layer(auth_layer));

    Router::new()
        .merge(health::router())
        .nest("/api/v1", api_v1)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
