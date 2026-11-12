use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Unauthorized,
    /// 401 이지만 문구가 다른 경우(로그인 실패 등).
    UnauthorizedWith(&'static str),
    Forbidden,
    NotFound,
    Internal(anyhow::Error),
}

pub type ApiResult<T> = Result<T, ApiError>;

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "인증이 필요합니다".to_string()),
            ApiError::UnauthorizedWith(msg) => (StatusCode::UNAUTHORIZED, msg.to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "권한이 없습니다".to_string()),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "찾을 수 없습니다".to_string()),
            ApiError::Internal(err) => {
                tracing::error!(error = ?err, "internal error");
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "서버 오류가 발생했습니다".to_string(),
                )
            }
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        ApiError::Internal(err.into())
    }
}
