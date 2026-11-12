use std::path::Path;

use axum::Router;
use axum::http::{HeaderValue, header::CACHE_CONTROL};
use axum::middleware::map_response;
use axum::response::Response;
use axum::routing::any;
use tower_http::services::{ServeDir, ServeFile};

use crate::api_not_found;

/// 빌드된 SPA 제공. 파일이 있으면 그 파일, 없으면 `index.html`(200) — 새로고침·딥링크용.
/// `/api` 와 `/api/...` 는 JSON 404 로 막아 SPA 가 가리지 않게 한다.
/// `assets/` 는 해시 파일명이라 영구 캐시, 나머지(index.html 포함)는 `no-cache`.
pub fn service(dir: &Path) -> Router {
    let assets = ServeDir::new(dir.join("assets"));
    let pages = ServeDir::new(dir).fallback(ServeFile::new(dir.join("index.html")));
    // 성공(2xx) 응답에만 붙인다. 없는 해시 파일의 404 가 1년 캐시되면 안 된다.
    let with_cache = |svc: Router, value: &'static str| {
        svc.layer(map_response(move |mut res: Response| async move {
            if res.status().is_success() {
                res.headers_mut()
                    .insert(CACHE_CONTROL, HeaderValue::from_static(value));
            }
            res
        }))
    };
    Router::new()
        .nest_service(
            "/assets",
            with_cache(
                Router::new().fallback_service(assets),
                "public, max-age=31536000, immutable",
            ),
        )
        .route("/api", any(api_not_found))
        .route("/api/{*rest}", any(api_not_found))
        .fallback_service(with_cache(
            Router::new().fallback_service(pages),
            "no-cache",
        ))
}
