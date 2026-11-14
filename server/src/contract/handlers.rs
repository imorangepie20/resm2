use crate::auth::extract::CurrentUser;
use crate::contract::model::{ChecklistAllReq, ContractData, ToggleChecklistReq};
use crate::contract::repo::{
    complete_contract, find_property_for_contract, set_all_checklist, start_contract,
    toggle_checklist_item,
};
use crate::contract::validate::{contract_progress, required_field_keys, resolve_checklist};
use crate::{ApiResult, AppState};
use axum::extract::{Path, State};
use axum::routing::{get, patch, post};
use axum::{Json, Router};

// ─── GET /contract/{id} ──────────────────────────────────────────
pub async fn get_contract(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ContractData>> {
    let property = find_property_for_contract(&state.db, &user.id, &id).await?;

    let checklist = resolve_checklist(property.real_estate_type(), property.trade_type());
    let checked = property.contract_checklist_map();
    let required_fields = required_field_keys(property.trade_type())
        .into_iter()
        .map(|k| crate::contract::model::RequiredField {
            key: k.to_string(),
            label: field_label(k),
            filled: property_filled(&property, k),
        })
        .collect();
    let progress = contract_progress(
        property.real_estate_type(),
        property.trade_type(),
        &checked,
        &property.to_json_value(),
    );
    let status = property.status.clone();

    Ok(Json(ContractData {
        property,
        checklist,
        checked,
        required_fields,
        progress,
        status,
    }))
}

// ─── PATCH /contract/{id}/checklist/{itemId} ────────────────────
pub async fn toggle_checklist(
    user: CurrentUser,
    State(state): State<AppState>,
    Path((id, item_id)): Path<(String, String)>,
    Json(req): Json<ToggleChecklistReq>,
) -> ApiResult<Json<serde_json::Value>> {
    toggle_checklist_item(&state.db, &user.id, &id, &item_id, req.checked).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── POST /contract/{id}/checklist/all ──────────────────────────
pub async fn checklist_all(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(req): Json<ChecklistAllReq>,
) -> ApiResult<Json<serde_json::Value>> {
    let property = find_property_for_contract(&state.db, &user.id, &id).await?;
    let item_ids: Vec<String> = resolve_checklist(property.real_estate_type(), property.trade_type())
        .into_iter()
        .map(|i| i.id)
        .collect();
    set_all_checklist(&state.db, &user.id, &id, req.checked, &item_ids).await?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

// ─── POST /contract/{id}/start ──────────────────────────────────
pub async fn start_contract_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ContractData>> {
    let property = start_contract(&state.db, &user.id, &id).await?;

    let checklist = resolve_checklist(property.real_estate_type(), property.trade_type());
    let checked = property.contract_checklist_map();
    let required_fields = required_field_keys(property.trade_type())
        .into_iter()
        .map(|k| crate::contract::model::RequiredField {
            key: k.to_string(),
            label: field_label(k),
            filled: property_filled(&property, k),
        })
        .collect();
    let progress = contract_progress(
        property.real_estate_type(),
        property.trade_type(),
        &checked,
        &property.to_json_value(),
    );
    let status = property.status.clone();

    Ok(Json(ContractData {
        property,
        checklist,
        checked,
        required_fields,
        progress,
        status,
    }))
}

// ─── POST /contract/{id}/complete ───────────────────────────────
pub async fn complete_contract_handler(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> ApiResult<Json<ContractData>> {
    let property = find_property_for_contract(&state.db, &user.id, &id).await?;
    let checked = property.contract_checklist_map();
    let progress = contract_progress(
        property.real_estate_type(),
        property.trade_type(),
        &checked,
        &property.to_json_value(),
    );
    let property = complete_contract(&state.db, &user.id, &id, progress.complete).await?;

    let checklist = resolve_checklist(property.real_estate_type(), property.trade_type());
    let checked = property.contract_checklist_map();
    let required_fields = required_field_keys(property.trade_type())
        .into_iter()
        .map(|k| crate::contract::model::RequiredField {
            key: k.to_string(),
            label: field_label(k),
            filled: property_filled(&property, k),
        })
        .collect();
    let progress = contract_progress(
        property.real_estate_type(),
        property.trade_type(),
        &checked,
        &property.to_json_value(),
    );
    let status = property.status.clone();

    Ok(Json(ContractData {
        property,
        checklist,
        checked,
        required_fields,
        progress,
        status,
    }))
}

// ─── 헬퍼 ────────────────────────────────────────────────────────
fn field_label(key: &str) -> String {
    match key {
        "customerName" => "고객명",
        "contractDate" => "계약일",
        "balanceDate" => "잔금일",
        "price" => "매매가",
        "rentPrice" => "보증금/월세",
        _ => key,
    }
    .to_string()
}

fn property_filled(property: &crate::contract::model::PropertyDto, key: &str) -> bool {
    match key {
        "customerName" => property.customer_name.as_ref().is_some_and(|v| !v.is_empty()),
        "contractDate" => property.contract_date.as_ref().is_some_and(|v| !v.is_empty()),
        "balanceDate" => property.balance_date.as_ref().is_some_and(|v| !v.is_empty()),
        "price" => property.price.as_ref().is_some_and(|v| !v.is_empty()),
        "rentPrice" => property.rent_price.as_ref().is_some_and(|v| !v.is_empty()),
        _ => false,
    }
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/contract/{id}", get(get_contract))
        .route("/contract/{id}/checklist/{itemId}", patch(toggle_checklist))
        .route("/contract/{id}/checklist/all", post(checklist_all))
        .route("/contract/{id}/start", post(start_contract_handler))
        .route("/contract/{id}/complete", post(complete_contract_handler))
}