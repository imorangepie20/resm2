use axum::Router;
use axum::extract::rejection::JsonRejection;
use axum::extract::{Json, Path, Query, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use serde::Deserialize;
use serde_json::{Value, json};

use super::model::{MSG_BAD_BODY, PropertyDto, parse_changes, parse_source, text_of};
use super::repo;
use crate::auth::extract::CurrentUser;
use crate::{ApiError, ApiResult, AppState, notify};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/properties", get(list).post(create))
        .route("/properties/delete", post(delete_many))
        .route("/properties/{id}", get(get_one).patch(patch_one))
}

fn bad_body(_: JsonRejection) -> ApiError {
    ApiError::BadRequest(MSG_BAD_BODY.to_string())
}

#[derive(Deserialize)]
struct ListQuery {
    view: Option<String>,
}

async fn list(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> ApiResult<Json<Vec<PropertyDto>>> {
    let view = q.view.as_deref().unwrap_or("all");
    let filter = repo::view_filter(view)
        .ok_or_else(|| ApiError::BadRequest("view 값이 올바르지 않습니다".to_string()))?;
    Ok(Json(repo::list(&state.db, &user.id, filter).await?))
}

async fn get_one(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<PropertyDto>> {
    repo::get(&state.db, &user.id, &id)
        .await?
        .map(Json)
        .ok_or(ApiError::NotFound)
}

async fn create(
    user: CurrentUser,
    State(state): State<AppState>,
    body: Result<Json<Value>, JsonRejection>,
) -> ApiResult<(StatusCode, Json<Value>)> {
    let Json(body) = body.map_err(bad_body)?;
    let changes = parse_changes(&body)?;
    let source = parse_source(&body)?;
    let label = text_of(&changes, "name")
        .or_else(|| text_of(&changes, "complexName"))
        .unwrap_or("매물")
        .to_string();
    let id = repo::insert(&state.db, &user.id, &source, changes).await?;

    // 알림 실패가 등록을 되돌리지는 않는다.
    if let Err(err) = notify::notify_superadmins(
        &state.db,
        "property",
        &format!("새 매물 등록: {label}"),
        Some(&format!(
            "{}님이 등록",
            user.name.as_deref().unwrap_or(&user.email)
        )),
        Some(&format!("/properties/{id}/edit")),
    )
    .await
    {
        tracing::error!(error = ?err, "매물 등록 알림 기록 실패");
    }
    Ok((StatusCode::CREATED, Json(json!({ "id": id }))))
}

async fn patch_one(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    body: Result<Json<Value>, JsonRejection>,
) -> ApiResult<StatusCode> {
    let Json(body) = body.map_err(bad_body)?;
    let changes = parse_changes(&body)?;
    let found = if changes.is_empty() {
        repo::exists(&state.db, &user.id, &id).await?
    } else {
        repo::update(&state.db, &user.id, &id, changes).await?
    };
    if found {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::NotFound)
    }
}

#[derive(Deserialize)]
struct DeleteRequest {
    ids: Vec<String>,
}

async fn delete_many(
    user: CurrentUser,
    State(state): State<AppState>,
    body: Result<Json<DeleteRequest>, JsonRejection>,
) -> ApiResult<Json<Value>> {
    let Json(req) = body.map_err(bad_body)?;
    let count = repo::delete_many(&state.db, &user.id, &req.ids).await?;
    Ok(Json(json!({ "count": count })))
}
