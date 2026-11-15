// Calendar Event DTOs & validation

use crate::ApiError;
use serde::{Deserialize, Serialize};
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateEventInput {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub date: String,
    #[validate(length(min = 0, max = 5))]
    pub start_time: Option<String>,
    #[validate(length(min = 1))]
    pub category: String,
    #[validate(length(min = 0, max = 1000))]
    pub memo: Option<String>,
    pub property_id: Option<String>,
    pub customer_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateEventInput {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub date: Option<String>,
    #[validate(length(min = 0, max = 5))]
    pub start_time: Option<String>,
    #[validate(length(min = 1))]
    pub category: Option<String>,
    #[validate(length(min = 0, max = 1000))]
    pub memo: Option<String>,
    pub property_id: Option<String>,
    pub customer_id: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventRow {
    pub id: String,
    pub title: String,
    pub date: String,
    #[serde(rename = "startTime")]
    pub start_time: Option<String>,
    pub category: String,
    pub memo: Option<String>,
    #[serde(rename = "propertyId")]
    pub property_id: Option<String>,
    #[serde(rename = "propertyLabel")]
    pub property_label: Option<String>,
    #[serde(rename = "customerId")]
    pub customer_id: Option<String>,
    #[serde(rename = "customerLabel")]
    pub customer_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyDateEvent {
    pub date: String,
    pub label: String,
    #[serde(rename = "propertyId")]
    pub property_id: String,
    pub field: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarOption {
    pub id: String,
    pub label: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarResponse {
    pub events: Vec<EventRow>,
    #[serde(rename = "propertyDates")]
    pub property_dates: Vec<PropertyDateEvent>,
    pub properties: Vec<CalendarOption>,
    pub customers: Vec<CalendarOption>,
}

pub const VALID_CATEGORIES: &[&str] = &["미팅", "임장", "계약", "기타"];

impl CreateEventInput {
    pub fn validate_input(&self) -> Result<(), ApiError> {
        use validator::Validate;
        self.validate()?;
        // date format YYYYMMDD
        if !regex::Regex::new(r"^\d{8}$").unwrap().is_match(&self.date) {
            return Err(ApiError::BadRequest("날짜는 YYYYMMDD 형식이어야 합니다".into()));
        }
        if !VALID_CATEGORIES.contains(&self.category.as_str()) {
            return Err(ApiError::BadRequest(format!("유효하지 않은 카테고리: {}", self.category)));
        }
        if let Some(ref st) = self.start_time {
            if !st.is_empty() && !regex::Regex::new(r"^\d{2}:\d{2}$").unwrap().is_match(st) {
                return Err(ApiError::BadRequest("시작 시간은 HH:MM 형식이어야 합니다".into()));
            }
        }
        // title must not be whitespace-only
        if self.title.trim().is_empty() {
            return Err(ApiError::BadRequest("제목은 공백일 수 없습니다".into()));
        }
        Ok(())
    }
}

impl UpdateEventInput {
    pub fn validate_input(&self) -> Result<(), ApiError> {
        use validator::Validate;
        self.validate()?;
        if let Some(ref d) = self.date {
            if !regex::Regex::new(r"^\d{8}$").unwrap().is_match(d) {
                return Err(ApiError::BadRequest("날짜는 YYYYMMDD 형식이어야 합니다".into()));
            }
        }
        if let Some(ref cat) = self.category {
            if !VALID_CATEGORIES.contains(&cat.as_str()) {
                return Err(ApiError::BadRequest(format!("유효하지 않은 카테고리: {}", cat)));
            }
        }
        if let Some(ref st) = self.start_time {
            if !st.is_empty() && !regex::Regex::new(r"^\d{2}:\d{2}$").unwrap().is_match(st) {
                return Err(ApiError::BadRequest("시작 시간은 HH:MM 형식이어야 합니다".into()));
            }
        }
        if let Some(ref t) = self.title {
            if t.trim().is_empty() {
                return Err(ApiError::BadRequest("제목은 공백일 수 없습니다".into()));
            }
        }
        Ok(())
    }
}