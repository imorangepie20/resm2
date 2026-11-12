mod common;

#[tokio::test]
async fn health_reports_db_ok() {
    let app = common::spawn_app().await;
    let res = app
        .client
        .get(format!("{}/api/v1/health", app.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    assert_eq!(
        res.json::<serde_json::Value>().await.unwrap()["status"],
        "ok"
    );
}

#[tokio::test]
async fn unknown_api_path_is_json_404() {
    let app = common::spawn_app().await;
    let res = app
        .client
        .get(format!("{}/api/v1/nope", app.addr))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
    assert!(res.json::<serde_json::Value>().await.unwrap()["error"].is_string());
}
