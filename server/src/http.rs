use axum::extract::Request;
use axum::http::header::{AUTHORIZATION, CONTENT_TYPE, HOST, ORIGIN};
use axum::http::{HeaderValue, Method};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};
use tower_http::cors::CorsLayer;

use crate::{ApiError, Config};

/// 허용 출처는 `cfg.allowed_origins` 만. 앱은 Bearer 를 쓰므로 credentials 는 허용하지 않는다.
pub fn cors_layer(cfg: &Config) -> CorsLayer {
    let origins: Vec<HeaderValue> = cfg
        .allowed_origins
        .iter()
        .filter_map(|o| HeaderValue::from_str(o).ok())
        .collect();
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH])
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
}

fn host_of_origin(origin: &str) -> &str {
    origin.split_once("://").map_or(origin, |(_, rest)| rest)
}

/// 쿠키 인증 요청(Authorization 없음)의 상태 변경 메서드는 Origin 이 Host 와 같은 출처일 때만 허용(CSRF 방어).
pub async fn origin_check(req: Request, next: Next) -> Response {
    let headers = req.headers();
    let safe = matches!(*req.method(), Method::GET | Method::HEAD | Method::OPTIONS);
    if !safe
        && !headers.contains_key(AUTHORIZATION)
        && let Some(origin) = headers.get(ORIGIN)
    {
        let same = match (origin.to_str(), headers.get(HOST).map(|h| h.to_str())) {
            (Ok(o), Some(Ok(h))) => host_of_origin(o).eq_ignore_ascii_case(h),
            _ => false,
        };
        if !same {
            return ApiError::Forbidden.into_response();
        }
    }
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strips_scheme() {
        assert_eq!(host_of_origin("https://a.example:8443"), "a.example:8443");
        assert_eq!(host_of_origin("a.example"), "a.example");
    }
}
