pub mod ai;
pub mod analysis;
pub mod auth;
pub mod config;
pub mod db;
pub mod entity;
pub mod error;
pub mod health;
pub mod import;
pub mod knowledge;
pub mod problem;
pub mod state;
pub mod submission;
pub mod tag;

use axum::{middleware, Router};
use std::sync::Arc;
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn build_router(state: state::AppState) -> Router {
    let auth_layer = middleware::from_fn_with_state(
        state.clone(),
        auth::middleware::require_auth,
    );

    // General rate limit for all routes
    let general_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(state.rate_limit_per_second)
            .burst_size(state.rate_limit_burst)
            .use_headers()
            .finish()
            .unwrap(),
    );

    // Stricter rate limit for auth endpoints (prevent brute-force)
    let auth_config = Arc::new(
        GovernorConfigBuilder::default()
            .per_second(2)
            .burst_size(5)
            .use_headers()
            .finish()
            .unwrap(),
    );

    let api_v1 = Router::new()
        .merge(
            auth::route::public_router()
                .layer(GovernorLayer { config: auth_config }),
        )
        .merge(auth::route::protected_router().route_layer(auth_layer.clone()))
        .merge(problem::route::protected_router().route_layer(auth_layer.clone()))
        .merge(submission::route::protected_router().route_layer(auth_layer.clone()))
        .merge(knowledge::route::protected_router().route_layer(auth_layer.clone()))
        .merge(tag::route::protected_router().route_layer(auth_layer.clone()))
        .merge(analysis::route::protected_router().route_layer(auth_layer.clone()))
        .merge(import::route::protected_router().route_layer(auth_layer.clone()))
        .merge(ai::route::protected_router().route_layer(auth_layer));

    Router::new()
        .merge(health::router())
        .nest("/api/v1", api_v1)
        .layer(GovernorLayer { config: general_config })
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
