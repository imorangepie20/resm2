mod common;

use std::path::PathBuf;

/// 임시 디렉터리에 `index.html`, `assets/a.js`, 그리고 바깥의 비밀 파일을 만든다.
fn make_static_dir() -> PathBuf {
    let root = std::env::temp_dir().join(format!("resm-static-{}", common::unique_email()));
    let dist = root.join("dist");
    std::fs::create_dir_all(dist.join("assets")).unwrap();
    std::fs::write(dist.join("index.html"), "<title>RESM</title>").unwrap();
    std::fs::write(dist.join("assets/a.js"), "console.log(1)").unwrap();
    std::fs::write(root.join("secret.txt"), "TOP-SECRET").unwrap();
    dist
}

async fn get(app: &common::TestApp, path: &str) -> reqwest::Response {
    app.client
        .get(format!("{}{}", app.addr, path))
        .send()
        .await
        .unwrap()
}

#[tokio::test]
async fn deep_link_serves_index() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    let res = get(&app, "/properties/abc/edit").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["cache-control"], "no-cache");
    assert!(res.text().await.unwrap().contains("RESM"));
}

#[tokio::test]
async fn root_serves_index_no_cache() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    let res = get(&app, "/").await;
    assert_eq!(res.status(), 200);
    assert_eq!(res.headers()["cache-control"], "no-cache");
    assert!(res.text().await.unwrap().contains("RESM"));
}

#[tokio::test]
async fn asset_is_served_with_long_cache() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    let res = get(&app, "/assets/a.js").await;
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.headers()["cache-control"],
        "public, max-age=31536000, immutable"
    );
    assert_eq!(res.text().await.unwrap(), "console.log(1)");
}

#[tokio::test]
async fn missing_asset_is_404_not_index() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    let res = get(&app, "/assets/nope.js").await;
    assert_eq!(res.status(), 404);
    assert!(res.headers().get("cache-control").is_none());
    assert!(!res.text().await.unwrap().contains("RESM"));
}

#[tokio::test]
async fn unknown_api_path_still_json_404() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    for path in ["/api/v1/nope", "/api/v1", "/api/nope", "/api"] {
        let res = get(&app, path).await;
        assert_eq!(res.status(), 404, "{path}");
        let ct = res.headers()["content-type"].to_str().unwrap().to_string();
        assert!(ct.starts_with("application/json"), "{path}: {ct}");
    }
}

#[tokio::test]
async fn path_like_apix_is_a_page() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    let res = get(&app, "/apix").await;
    assert_eq!(res.status(), 200);
    assert!(res.text().await.unwrap().contains("RESM"));
}

#[tokio::test]
async fn encoded_dotdot_cannot_escape_static_dir() {
    let app = common::spawn_app_with_static(Some(make_static_dir())).await;
    for path in [
        "/%2e%2e/secret.txt",
        "/assets/%2e%2e/%2e%2e/secret.txt",
        "/..%2fsecret.txt",
    ] {
        let res = get(&app, path).await;
        let body = res.text().await.unwrap();
        assert!(!body.contains("TOP-SECRET"), "{path}");
    }
}

#[tokio::test]
async fn no_static_dir_means_404_for_pages() {
    let app = common::spawn_app().await;
    let res = get(&app, "/").await;
    assert_eq!(res.status(), 404);
}
