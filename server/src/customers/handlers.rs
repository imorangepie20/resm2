use crate::auth::extract::CurrentUser;
use crate::customers::model::{CustomerInput, CustomerRow};
use crate::customers::repo as customers_repo;
use crate::{ApiResult, AppState};
use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ListCustomersQuery {
    #[serde(default)]
    q: Option<String>,
    #[serde(default, rename = "type")]
    type_: Option<String>,
}

// ─── GET /customers ────────────────────────────────────────────────
pub async fn list_customers(
    user: CurrentUser,
    State(state): State<AppState>,
    Query(params): Query<ListCustomersQuery>,
) -> ApiResult<Json<Vec<CustomerRow>>> {
    let customers = customers_repo::list_customers(&state.db, &user.id, params.q.as_deref(), params.type_.as_deref()).await?;
    Ok(Json(customers))
}

// ─── GET /customers/{id} ───────────────────────────────────────────
pub async fn get_customer(
    user: CurrentUser,
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<Json<CustomerRow>> {
    let customer = customers_repo::get_customer(&state.db, &user.id, &id).await?
        .ok_or(crate::error::ApiError::NotFound)?;
    Ok(Json(customer))
}

// ─── POST /customers ───────────────────────────────────────────────
pub async fn create_customer(
    user: CurrentUser,
    State(state): State<AppState>,
    Json(input): Json<CustomerInput>,
) -> ApiResult<Json<serde_json::Value>> {
    let property_id = input.property_id.clone();
    let data = input.to_data()?;
    let id = customers_repo::create_customer(&state.db, &user.id, data, property_id).await?;
    Ok(Json(json!({ "id": id })))
}

// ─── PATCH /customers/{id} ─────────────────────────────────────────
pub async fn update_customer(
    user: CurrentUser,
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(input): Json<CustomerInput>,
) -> ApiResult<()> {
    let property_id = input.property_id.clone();
    let data = input.to_data()?;
    customers_repo::update_customer(&state.db, &user.id, &id, data, property_id).await?;
    Ok(())
}

// ─── DELETE /customers/{id} ────────────────────────────────────────
pub async fn delete_customer(
    user: CurrentUser,
    State(state): State<AppState>,
    axum::extract::Path(id): axum::extract::Path<String>,
) -> ApiResult<()> {
    customers_repo::delete_customer(&state.db, &user.id, &id).await?;
    Ok(())
}

// ─── GET /customers/draft-from-property/{propertyId} ───────────────
pub async fn customer_draft_from_property(
    user: CurrentUser,
    State(state): State<AppState>,
    axum::extract::Path(property_id): axum::extract::Path<String>,
) -> ApiResult<Json<Option<crate::customers::model::CustomerDraft>>> {
    let draft = customers_repo::customer_draft_from_property(&state.db, &user.id, &property_id).await?;
    Ok(Json(draft))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/customers", get(list_customers).post(create_customer))
        .route(
            "/customers/{id}",
            get(get_customer).patch(update_customer).delete(delete_customer),
        )
        .route(
            "/customers/draft-from-property/{propertyId}",
            get(customer_draft_from_property),
        )
}