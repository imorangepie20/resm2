use crate::error::ApiError;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// 허용된 고객 유형
const CUSTOMER_TYPES: &[&str] = &["매도인", "매수인", "임대인", "임차인", "단순방문"];
const PARTY_TYPES: &[&str] = &["매도인", "매수인", "임대인", "임차인"];
const GENDERS: &[&str] = &["남", "여", "미지정"];

/// 고객 유형 정규화: 거래 당사자 유형이 있으면 그것만(순서대로·중복 제거), 없으면 기본 "단순방문"
pub fn normalize_customer_types(input: &[String]) -> Vec<String> {
    let valid: HashSet<&str> = input
        .iter()
        .map(|s| s.as_str())
        .filter(|t| CUSTOMER_TYPES.contains(t))
        .collect();
    let parties: Vec<_> = PARTY_TYPES
        .iter()
        .filter(|t| valid.contains(*t))
        .map(|s| s.to_string())
        .collect();
    if parties.is_empty() {
        vec!["단순방문".to_string()]
    } else {
        parties
    }
}

fn clean_gender(g: Option<String>) -> Option<String> {
    g.and_then(|s| {
        if GENDERS.contains(&s.as_str()) {
            Some(s)
        } else {
            None
        }
    })
}

#[derive(Debug, Deserialize)]
pub struct CustomerInput {
    pub name: String,
    #[serde(default)]
    pub zipcode: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
    #[serde(default)]
    pub property_id: Option<String>,
}

impl CustomerInput {
    pub fn to_data(self) -> Result<CustomerData, ApiError> {
        let name = self.name.trim();
        if name.is_empty() {
            return Err(ApiError::BadRequest("이름은 필수입니다".into()));
        }
        let gender = clean_gender(self.gender);
        let types = normalize_customer_types(&self.types.unwrap_or_default());
        Ok(CustomerData {
            name: name.to_string(),
            zipcode: self.zipcode.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            phone: self.phone.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            address: self.address.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            email: self.email.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            gender,
            memo: self.memo.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            types,
        })
    }
}

/// Partial update input - all fields optional for PATCH
#[derive(Debug, Deserialize, Default)]
pub struct CustomerUpdateInput {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub zipcode: Option<String>,
    #[serde(default)]
    pub phone: Option<String>,
    #[serde(default)]
    pub address: Option<String>,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub gender: Option<String>,
    #[serde(default)]
    pub memo: Option<String>,
    #[serde(default)]
    pub types: Option<Vec<String>>,
    #[serde(default)]
    pub property_id: Option<String>,
}

impl CustomerUpdateInput {
    pub fn to_data(self) -> Result<CustomerData, ApiError> {
        let name = self.name.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
        if let Some(ref n) = name {
            if n.is_empty() {
                return Err(ApiError::BadRequest("이름은 필수입니다".into()));
            }
        }
        let gender = clean_gender(self.gender);
        let types = self.types.as_deref().map(normalize_customer_types).unwrap_or_default();
        Ok(CustomerData {
            name: name.unwrap_or_default(),
            zipcode: self.zipcode.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            phone: self.phone.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            address: self.address.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            email: self.email.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            gender,
            memo: self.memo.map(|s| s.trim().to_string()).filter(|s| !s.is_empty()),
            types,
        })
    }
}

#[derive(Debug)]
pub struct CustomerData {
    pub name: String,
    pub zipcode: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub memo: Option<String>,
    pub types: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct CustomerRow {
    pub id: String,
    pub types: Vec<String>,
    pub name: String,
    pub zipcode: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub gender: Option<String>,
    pub memo: Option<String>,
    pub property_id: Option<String>,
    pub property_name: Option<String>,
    pub parties: Vec<PartyRow>,
    pub updated_at: String,
}

#[derive(Debug, Serialize)]
pub struct PartyRow {
    pub property_id: String,
    pub property_label: String,
    pub role: String,
}

#[derive(Debug, Serialize)]
pub struct CustomerDraft {
    pub name: String,
    pub phone: String,
    pub property_id: String,
    pub property_label: String,
}