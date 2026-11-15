use crate::auth::extract::CurrentUser;
use crate::parties::model::{AddPartyInput, PartyRow};
use crate::parties::repo::{add_party, list_parties, remove_party};
use crate::{ApiResult, AppState};
use axum::extract::{Path, State};
use axum::routing::{delete, get, post};
use axum::{Json, Router};
use serde_json::json;

#[derive(serde::Deserialize)]
pub struct AddPartyRequest {
    role: String,
    name: String,
    #[serde(default)]
    phone: Option<String>,
}

// ─── GET /properties/{propertyId}/parties ──────────────────────────
pub async fn list_parties_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(property_id): Path<String>,
) -> ApiResult<Json<Vec<PartyRow>>> {
    let parties = list_parties(&state.db, &user.id, &property_id).await?;
    Ok(Json(parties))
}

// ─── POST /properties/{propertyId}/parties ─────────────────────────
pub async fn add_party_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(property_id): Path<String>,
    Json(input): Json<AddPartyRequest>,
) -> ApiResult<Json<serde_json::Value>> {
    let input = AddPartyInput {
        role: input.role,
        name: input.name,
        phone: input.phone,
    }
    .validate()?;
    let (party_id, customer_id) = add_party(
        &state.db,
        &user.id,
        &property_id,
        &input.role,
        &input.name,
        input.phone,
    )
    .await?;
    Ok(Json(json!({ "partyId": party_id, "customerId": customer_id })))
}

// ─── DELETE /properties/{propertyId}/parties/{partyId} ─────────────
pub async fn remove_party_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path((property_id, party_id)): Path<(String, String)>,
) -> ApiResult<axum::response::Response> {
    remove_party(&state.db, &user.id, &property_id, &party_id).await?;
    Ok(axum::response::Response::builder()
        .status(axum::http::StatusCode::NO_CONTENT)
        .body(axum::body::Body::empty())
        .unwrap())
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route(
            "/properties/{propertyId}/parties",
            get(list_parties_handler).post(add_party_handler),
        )
        .route(
            "/properties/{propertyId}/parties/{partyId}",
            delete(remove_party_handler),
        )
}