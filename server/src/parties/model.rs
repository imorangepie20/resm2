use crate::error::ApiError;
use serde::{Deserialize, Serialize};

const VALID_ROLES: &[&str] = &["매도인", "매수인", "임대인", "임차인", "중개"];

pub fn validate_role(role: &str) -> Result<(), ApiError> {
    if !VALID_ROLES.contains(&role) {
        return Err(ApiError::BadRequest("유효하지 않은 역할입니다".into()));
    }
    Ok(())
}

/// 이름+전화(숫자만)로 고객 매칭 키 생성
pub fn customer_match_key(name: &str, phone: &str) -> String {
    let clean_name = name.trim();
    let clean_phone: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    format!("{}|{}", clean_name, clean_phone)
}

#[derive(Debug, Deserialize)]
pub struct AddPartyInput {
    pub role: String,
    pub name: String,
    #[serde(default)]
    pub phone: Option<String>,
}

impl AddPartyInput {
    pub fn validate(self) -> Result<Self, ApiError> {
        validate_role(&self.role)?;
        if self.name.trim().is_empty() {
            return Err(ApiError::BadRequest("이름은 필수입니다".into()));
        }
        Ok(self)
    }
}

#[derive(Debug, Serialize)]
pub struct PartyRow {
    pub id: String,
    pub customer_id: String,
    pub name: String,
    pub phone: Option<String>,
    pub role: String,
}