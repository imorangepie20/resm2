use std::sync::Arc;

use resm_server::{AppState, Config, build_router};
use sqlx::postgres::PgPoolOptions;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"));
    tracing_subscriber::fmt()
        .with_env_filter(
            EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| EnvFilter::new("info,tower_http=info")),
        )
        .init();

    let cfg = Config::from_env()?;
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&cfg.database_url)
        .await?;
    let listener = tokio::net::TcpListener::bind(cfg.bind).await?;
    tracing::info!("listening on {}", listener.local_addr()?);

    let state = AppState {
        db,
        cfg: Arc::new(cfg),
    };
    axum::serve(listener, build_router(state)).await?;
    Ok(())
}
