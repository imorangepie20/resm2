use crate::contract::validate::{ChecklistItem, Progress};
use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize, Serializer};
use serde_json::Value;
use std::collections::HashMap;

fn ser_datetime<S: Serializer>(v: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&v.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
}

// ─── PropertyDto (properties API와 공유) ──────────────────────
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyDto {
    pub id: String,
    pub is_favorite: bool,
    pub color_tag: Option<String>,
    pub status: String,
    pub source: String,
    #[serde(serialize_with = "ser_datetime")]
    pub created_at: NaiveDateTime,
    pub name: Option<String>,
    pub article_no: Option<String>,
    pub complex_name: Option<String>,
    pub address: Option<String>,
    pub real_estate_type: Option<String>,
    pub trade_type: Option<String>,
    pub heating: Option<String>,
    pub customer_name: Option<String>,
    pub customer_phone: Option<String>,
    pub partner_name: Option<String>,
    pub partner_phone: Option<String>,
    pub partner_manager: Option<String>,
    pub manager: Option<String>,
    pub contract_hope_date: Option<String>,
    pub contract_date: Option<String>,
    pub move_in_hope_date: Option<String>,
    pub move_in_date: Option<String>,
    pub interim1_date: Option<String>,
    pub interim2_date: Option<String>,
    pub balance_date: Option<String>,
    pub approval_date: Option<String>,
    pub note: Option<String>,
    pub memo: Option<String>,
    pub site_area: Option<f64>,
    pub area_exclusive: Option<f64>,
    pub area_supply: Option<f64>,
    pub land_area: Option<f64>,
    pub building_area: Option<f64>,
    pub area: Option<f64>,
    pub total_households: Option<i32>,
    pub parking_count: Option<i32>,
    pub is_pre_sale: Option<bool>,
    pub price: Option<String>,
    pub rent_price: Option<String>,
    pub deal_amount: Option<String>,
    pub contract_checklist: Option<Value>,
}

// ─── ContractData (GET /contract/{id} 응답) ───────────────────
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ContractData {
    pub property: PropertyDto,
    pub checklist: Vec<ChecklistItem>,
    pub checked: HashMap<String, String>,
    pub required_fields: Vec<RequiredField>,
    pub progress: Progress,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequiredField {
    pub key: String,
    pub label: String,
    pub filled: bool,
}

// ─── 요청/응답 DTO ────────────────────────────────────────────
#[derive(Debug, Deserialize)]
pub struct ToggleChecklistReq {
    pub checked: bool,
}

#[derive(Debug, Deserialize)]
pub struct ChecklistAllReq {
    pub checked: bool,
}

// ─── 내부 변환 헬퍼 ───────────────────────────────────────────
impl PropertyDto {
    pub fn real_estate_type(&self) -> &str {
        self.real_estate_type.as_deref().unwrap_or("")
    }

    pub fn trade_type(&self) -> &str {
        self.trade_type.as_deref().unwrap_or("")
    }

    pub fn contract_checklist_map(&self) -> HashMap<String, String> {
        self.contract_checklist
            .as_ref()
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn to_json_value(&self) -> Value {
        serde_json::to_value(self).unwrap_or(Value::Null)
    }
}