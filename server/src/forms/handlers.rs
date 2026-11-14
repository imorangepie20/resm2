use crate::auth::extract::CurrentUser;
use crate::contract::repo::find_property_for_contract;
use crate::forms::engine::init_templates;
use crate::forms::model::{form_by_id, forms_for, template_name};
use crate::{ApiResult, AppState};
use axum::extract::{Path, State};
use axum::response::Html;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::json;
use std::sync::Arc;
use tera::Context;

/// 템플릿 엔진 상태
#[derive(Clone)]
pub struct FormsState {
    pub tera: Arc<tera::Tera>,
}

impl FormsState {
    pub fn new() -> Self {
        Self { tera: init_templates() }
    }
}

// ─── GET /forms/{propertyId} ────────────────────────────────────
pub async fn list_forms(
    user: CurrentUser,
    State(state): State<AppState>,
    Path(property_id): Path<String>,
) -> ApiResult<Json<Vec<serde_json::Value>>> {
    let property = find_property_for_contract(&state.db, &user.id, &property_id).await?;
    let forms = forms_for(property.real_estate_type(), property.trade_type());

    let result: Vec<_> = forms
        .into_iter()
        .map(|f| json!({ "id": f.id, "label": f.label }))
        .collect();

    Ok(Json(result))
}

// ─── GET /forms/{propertyId}/{formId} ───────────────────────────
pub async fn get_form_html(
    user: CurrentUser,
    State(state): State<AppState>,
    Path((property_id, form_id)): Path<(String, String)>,
) -> ApiResult<Html<String>> {
    let property = find_property_for_contract(&state.db, &user.id, &property_id).await?;

    let form = form_by_id(&form_id).ok_or(crate::error::ApiError::NotFound)?;
    let template = template_name(&form_id).ok_or(crate::error::ApiError::NotFound)?;

    // 적용 가능 여부 확인
    if !(form.applies)(property.real_estate_type(), property.trade_type()) {
        return Err(crate::error::ApiError::NotFound);
    }

    // Property를 JSON으로 변환
    let property_json = property.to_json_value();

    // Tera 컨텍스트 구성
    let mut context = Context::new();
    context.insert("p", &property_json);

    // 템플릿 렌더링
    let html = state.forms.tera.render(template, &context).map_err(|e| {
        tracing::error!(error = ?e, "template render failed");
        crate::error::ApiError::Internal(e.into())
    })?;

    Ok(Html(html))
}

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/forms/{propertyId}", get(list_forms))
        .route("/forms/{propertyId}/{formId}", get(get_form_html))
}