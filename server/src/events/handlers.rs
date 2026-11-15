// Calendar Event handlers

use crate::auth::extract::CurrentUser;
use crate::events::model::*;
use crate::events::repo::*;
use crate::{ApiResult, AppState};
use axum::extract::{Path, Query, State};
use axum::routing::{get, patch};
use axum::{Json, Router};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct CalendarQuery {
    year: i32,
    month: u32, // 0-indexed
}

// ─── GET /events?year=&month= ──────────────────────────────────────

pub async fn load_calendar_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<CalendarQuery>,
) -> ApiResult<Json<CalendarResponse>> {
    let res = load_calendar(&state.db, &user.id, q.year, q.month).await?;
    Ok(Json(res))
}

// ─── POST /events ──────────────────────────────────────────────────

#[derive(Deserialize)]
pub struct CreateEventRequest {
    title: String,
    date: String,
    #[serde(default, rename = "startTime")]
    start_time: Option<String>,
    category: String,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default, rename = "propertyId")]
    property_id: Option<String>,
    #[serde(default, rename = "customerId")]
    customer_id: Option<String>,
}

pub async fn create_event_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(input): Json<CreateEventRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let input = CreateEventInput {
        title: input.title,
        date: input.date,
        start_time: input.start_time,
        category: input.category,
        memo: input.memo,
        property_id: input.property_id,
        customer_id: input.customer_id,
    };
    input.validate_input()?;
    let id = create_event(&state.db, &user.id, input).await?;
    Ok(Json(serde_json::json!({ "id": id })))
}

// ─── PATCH /events/{id} ────────────────────────────────────────────

#[derive(Deserialize)]
pub struct UpdateEventRequest {
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    date: Option<String>,
    #[serde(default, rename = "startTime")]
    start_time: Option<String>,
    #[serde(default)]
    category: Option<String>,
    #[serde(default)]
    memo: Option<String>,
    #[serde(default, rename = "propertyId")]
    property_id: Option<String>,
    #[serde(default, rename = "customerId")]
    customer_id: Option<String>,
}

pub async fn update_event_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(event_id): Path<String>,
    Json(input): Json<UpdateEventRequest>,
) -> ApiResult<axum::response::Response> {
    let input = UpdateEventInput {
        title: input.title,
        date: input.date,
        start_time: input.start_time,
        category: input.category,
        memo: input.memo,
        property_id: input.property_id,
        customer_id: input.customer_id,
    };
    input.validate_input()?;
    update_event(&state.db, &user.id, &event_id, input).await?;
    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap())
}

// ─── DELETE /events/{id} ───────────────────────────────────────────

pub async fn delete_event_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(event_id): Path<String>,
) -> ApiResult<axum::response::Response> {
    delete_event(&state.db, &user.id, &event_id).await?;
    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/events", get(load_calendar_handler).post(create_event_handler))
        .route(
            "/events/{eventId}",
            patch(update_event_handler).delete(delete_event_handler),
        )
}