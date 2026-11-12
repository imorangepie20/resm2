use chrono::NaiveDateTime;
use serde::{Serialize, Serializer};
use serde_json::{Map, Value};
use sqlx::FromRow;

use crate::{ApiError, ApiResult};

pub const MSG_BAD_BODY: &str = "요청 형식이 올바르지 않습니다";
const MSG_BAD_STATUS: &str = "상태 값이 올바르지 않습니다";
const MSG_BAD_COLOR: &str = "색상 값이 올바르지 않습니다";
const MSG_BAD_MONEY: &str = "금액 형식이 올바르지 않습니다";

const STATUSES: [&str; 3] = ["진행", "계약진행", "계약완료"];
const COLOR_TAGS: [&str; 4] = ["purple", "blue", "orange", "gray"];

/// 조회 SELECT 목록. `PropertyDto` 필드와 같은 순서·이름이며 `dealAmount`·`contractChecklist` 는 제외한다.
pub const SELECT_COLUMNS: &str = r#""id", "isFavorite", "colorTag", "status", "source", "createdAt",
    "name", "articleNo", "complexName", "address", "realEstateType", "tradeType", "heating",
    "customerName", "customerPhone", "partnerName", "partnerPhone", "partnerManager", "manager",
    "contractHopeDate", "contractDate", "moveInHopeDate", "moveInDate", "interim1Date",
    "interim2Date", "balanceDate", "approvalDate", "note", "memo",
    "siteArea", "areaExclusive", "areaSupply", "landArea", "buildingArea", "area",
    "totalHouseholds", "parkingCount", "isPreSale", "price", "rentPrice""#;

#[derive(Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[sqlx(rename_all = "camelCase")]
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
    #[serde(serialize_with = "ser_money")]
    pub price: Option<i64>,
    #[serde(serialize_with = "ser_money")]
    pub rent_price: Option<i64>,
}

fn ser_datetime<S: Serializer>(v: &NaiveDateTime, s: S) -> Result<S::Ok, S::Error> {
    s.collect_str(&v.format("%Y-%m-%dT%H:%M:%S%.3fZ"))
}

/// BIGINT(원) → 십진 문자열. JS 의 2^53 한계를 피하려고 항상 문자열로 내보낸다.
fn ser_money<S: Serializer>(v: &Option<i64>, s: S) -> Result<S::Ok, S::Error> {
    match v {
        Some(n) => s.collect_str(n),
        None => s.serialize_none(),
    }
}

/// 요청 값 → BIGINT. `^\d{1,18}$` 문자열 또는 음수 아닌 JSON 정수만 허용.
fn parse_money(v: &Value) -> ApiResult<Option<i64>> {
    let bad = || ApiError::BadRequest(MSG_BAD_MONEY.to_string());
    match v {
        Value::Null => Ok(None),
        Value::String(s)
            if (1..=18).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_digit()) =>
        {
            s.parse().map(Some).map_err(|_| bad())
        }
        Value::Number(n) => n
            .as_u64()
            .and_then(|u| i64::try_from(u).ok())
            .map(Some)
            .ok_or_else(bad),
        _ => Err(bad()),
    }
}

/// 바인딩할 컬럼 값.
pub enum Val {
    Text(Option<String>),
    Float(Option<f64>),
    Int(Option<i32>),
    Bool(Option<bool>),
    Big(Option<i64>),
}

enum Kind {
    Text,
    Float,
    Int,
    Bool,
    Money,
}

/// 요청으로 쓸 수 있는 컬럼. 컬럼명은 SQL 에 들어가므로 반드시 이 고정 목록에서만 온다.
const FIELDS: &[(&str, Kind)] = &[
    ("name", Kind::Text),
    ("articleNo", Kind::Text),
    ("complexName", Kind::Text),
    ("address", Kind::Text),
    ("realEstateType", Kind::Text),
    ("tradeType", Kind::Text),
    ("heating", Kind::Text),
    ("customerName", Kind::Text),
    ("customerPhone", Kind::Text),
    ("partnerName", Kind::Text),
    ("partnerPhone", Kind::Text),
    ("partnerManager", Kind::Text),
    ("manager", Kind::Text),
    ("contractHopeDate", Kind::Text),
    ("contractDate", Kind::Text),
    ("moveInHopeDate", Kind::Text),
    ("moveInDate", Kind::Text),
    ("interim1Date", Kind::Text),
    ("interim2Date", Kind::Text),
    ("balanceDate", Kind::Text),
    ("approvalDate", Kind::Text),
    ("note", Kind::Text),
    ("memo", Kind::Text),
    ("siteArea", Kind::Float),
    ("areaExclusive", Kind::Float),
    ("areaSupply", Kind::Float),
    ("landArea", Kind::Float),
    ("buildingArea", Kind::Float),
    ("area", Kind::Float),
    ("totalHouseholds", Kind::Int),
    ("parkingCount", Kind::Int),
    ("isPreSale", Kind::Bool),
    ("price", Kind::Money),
    ("rentPrice", Kind::Money),
];

fn bad_body() -> ApiError {
    ApiError::BadRequest(MSG_BAD_BODY.to_string())
}

fn parse_field(kind: &Kind, v: &Value) -> ApiResult<Val> {
    Ok(match (kind, v) {
        (Kind::Money, v) => Val::Big(parse_money(v)?),
        (Kind::Text, Value::Null) => Val::Text(None),
        // 원본의 toData 처럼 빈 문자열은 NULL.
        (Kind::Text, Value::String(s)) => Val::Text(Some(s.clone()).filter(|s| !s.is_empty())),
        (Kind::Float, Value::Null) => Val::Float(None),
        (Kind::Float, Value::Number(n)) => Val::Float(n.as_f64()),
        (Kind::Int, Value::Null) => Val::Int(None),
        (Kind::Int, Value::Number(n)) => Val::Int(Some(
            n.as_i64()
                .and_then(|i| i32::try_from(i).ok())
                .ok_or_else(bad_body)?,
        )),
        (Kind::Bool, Value::Null) => Val::Bool(None),
        (Kind::Bool, Value::Bool(b)) => Val::Bool(Some(*b)),
        _ => return Err(bad_body()),
    })
}

/// 본문 객체에서 쓸 컬럼을 뽑는다. 키 없음 = 목록에 없음(변경 없음), `null` = NULL.
/// 모르는 키는 무시. `status: null` 은 무시(NOT NULL 컬럼). `isFavorite` 는 불리언 필수.
pub fn parse_changes(body: &Value) -> ApiResult<Vec<(&'static str, Val)>> {
    let obj: &Map<String, Value> = body.as_object().ok_or_else(bad_body)?;
    let mut out = Vec::new();
    for (col, kind) in FIELDS {
        if let Some(v) = obj.get(*col) {
            out.push((*col, parse_field(kind, v)?));
        }
    }
    match obj.get("status") {
        None | Some(Value::Null) => {}
        Some(Value::String(s)) if STATUSES.contains(&s.as_str()) => {
            out.push(("status", Val::Text(Some(s.clone()))));
        }
        Some(Value::String(_)) => return Err(ApiError::BadRequest(MSG_BAD_STATUS.to_string())),
        Some(_) => return Err(bad_body()),
    }
    match obj.get("isFavorite") {
        None => {}
        Some(Value::Bool(b)) => out.push(("isFavorite", Val::Bool(Some(*b)))),
        Some(_) => return Err(bad_body()),
    }
    match obj.get("colorTag") {
        None => {}
        Some(Value::Null) => out.push(("colorTag", Val::Text(None))),
        Some(Value::String(s)) if COLOR_TAGS.contains(&s.as_str()) => {
            out.push(("colorTag", Val::Text(Some(s.clone()))));
        }
        Some(Value::String(_)) => return Err(ApiError::BadRequest(MSG_BAD_COLOR.to_string())),
        Some(_) => return Err(bad_body()),
    }
    Ok(out)
}

/// 등록 시 `source`. 없거나 비면 "수기".
pub fn parse_source(body: &Value) -> ApiResult<String> {
    match body.get("source") {
        None | Some(Value::Null) => Ok("수기".to_string()),
        Some(Value::String(s)) if s.is_empty() => Ok("수기".to_string()),
        Some(Value::String(s)) => Ok(s.clone()),
        Some(_) => Err(bad_body()),
    }
}

/// 변경 목록에서 문자열 컬럼 값 하나를 꺼낸다.
pub fn text_of<'a>(changes: &'a [(&'static str, Val)], col: &str) -> Option<&'a str> {
    changes.iter().find_map(|(c, v)| match v {
        Val::Text(Some(s)) if *c == col => Some(s.as_str()),
        _ => None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn money_accepts_digits_and_non_negative_ints() {
        assert_eq!(parse_money(&json!("0")).unwrap(), Some(0));
        assert_eq!(
            parse_money(&json!("999999999999999999")).unwrap(),
            Some(999_999_999_999_999_999)
        );
        assert_eq!(parse_money(&json!(12)).unwrap(), Some(12));
        assert_eq!(parse_money(&Value::Null).unwrap(), None);
    }

    #[test]
    fn money_rejects_everything_else() {
        for v in [
            json!(""),
            json!("1.5"),
            json!("-1"),
            json!("+1"),
            json!(" 1"),
            json!("1234567890123456789"),
            json!(-1),
            json!(1.5),
            json!(true),
            json!({}),
        ] {
            assert!(parse_money(&v).is_err(), "{v}");
        }
    }

    #[test]
    fn money_serializes_as_string() {
        #[derive(Serialize)]
        struct W {
            #[serde(serialize_with = "ser_money")]
            v: Option<i64>,
        }
        let s = serde_json::to_string(&W {
            v: Some(9_007_199_254_740_993),
        })
        .unwrap();
        assert_eq!(s, r#"{"v":"9007199254740993"}"#);
    }
}
