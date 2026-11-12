pub mod auth;
pub mod config;
pub mod error;

use std::sync::Arc;

use axum::routing::get;
use axum::{Json, Router, extract::State};
use sqlx::PgPool;
use tower_http::trace::TraceLayer;

pub use config::Config;
pub use error::{ApiError, ApiResult};

#[derive(Clone)]
pub struct AppState {
    pub db: PgPool,
    pub cfg: Arc<Config>,
}

async fn health(State(state): State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    sqlx::query("SELECT 1").execute(&state.db).await?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn api_not_found() -> ApiError {
    ApiError::NotFound
}

fn api_v1() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .fallback(api_not_found)
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .nest("/api/v1", api_v1())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
