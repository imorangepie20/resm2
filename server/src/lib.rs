pub mod auth;
pub mod config;
pub mod error;
pub mod http;
pub mod notify;

use std::sync::Arc;

use axum::middleware::from_fn_with_state;
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

fn api_v1(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(auth::handlers::routes())
        .fallback(api_not_found)
        .layer(from_fn_with_state(state, http::origin_check))
}

pub fn build_router(state: AppState) -> Router {
    let cors = http::cors_layer(&state.cfg);
    Router::new()
        .nest("/api/v1", api_v1(state.clone()).layer(cors))
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
