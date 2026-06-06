use acmind_api::{build_router, config::Config, db, state::AppState};
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .with(tracing_subscriber::fmt::layer().with_target(true))
        .init();

    let cfg = Config::from_env()?;
    let db = db::connect(&cfg.database_url).await?;
    db::run_migrations(&db).await?;

    let state = AppState {
        db,
        jwt_secret: Arc::new(cfg.jwt_secret),
        jwt_expires_in: cfg.jwt_expires_in,
        allow_register: cfg.allow_register,
    };

    let app = build_router(state);
    let addr = format!("0.0.0.0:{}", cfg.api_port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(%addr, "acmind-api listening");
    axum::serve(listener, app).await?;
    Ok(())
}
