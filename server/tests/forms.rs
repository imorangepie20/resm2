mod common;

use common::TestApp;
use reqwest::Method;
use serde_json::{json, Value};

async fn call(
    app: &TestApp,
    token: &str,
    method: Method,
    path: &str,
    body: Option<&Value>,
) -> (reqwest::StatusCode, Value) {
    let mut req = app
        .client
        .request(method, format!("{}/api/v1{path}", app.addr));
    req = req.bearer_auth(token);
    if let Some(b) = body {
        req = req.json(b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status();
    let text = resp.text().await.unwrap();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn call_html(
    app: &TestApp,
    token: &str,
    method: Method,
    path: &str,
) -> (reqwest::StatusCode, String) {
    let mut req = app
        .client
        .request(method, format!("{}/api/v1{path}", app.addr));
    req = req.bearer_auth(token);
    let resp = req.send().await.unwrap();
    let status = resp.status();
    let text = resp.text().await.unwrap();
    (status, text)
}

async fn create_property(app: &TestApp, token: &str, trade_type: &str, real_estate_type: &str) -> String {
    let res = call(
        app,
        token,
        Method::POST,
        "/properties",
        Some(&json!({
            "name": "테스트 매물",
            "tradeType": trade_type,
            "realEstateType": real_estate_type,
            "address": "서울시 강남구",
            "source": "수기",
            "price": "100000000",
            "contractDate": "20240101",
            "balanceDate": "20240201",
            "customerName": "홍길동"
        })),
    )
    .await;
    let (status, body) = res;
    assert!(status.is_success() || status.as_u16() == 201, "create property failed ({}): {:?}", status, body);
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn forms_list_residential_sale() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    let res = call(&app, &token, Method::GET, &format!("/forms/{prop_id}"), None).await;
    assert_eq!(res.0, 200);

    let forms = res.1.as_array().unwrap();
    let ids: Vec<_> = forms.iter().map(|f| f["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"sale_contract"));
    assert!(ids.contains(&"confirm_residential"));
    assert!(ids.contains(&"receipt"));
    assert!(ids.contains(&"power_of_attorney"));
    assert!(!ids.contains(&"lease_contract"));
    assert!(!ids.contains(&"confirm_nonresidential"));
}

#[tokio::test]
async fn forms_list_residential_lease() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "B1", "A01").await;

    let res = call(&app, &token, Method::GET, &format!("/forms/{prop_id}"), None).await;
    assert_eq!(res.0, 200);

    let forms = res.1.as_array().unwrap();
    let ids: Vec<_> = forms.iter().map(|f| f["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"lease_contract"));
    assert!(ids.contains(&"confirm_residential"));
    assert!(ids.contains(&"receipt"));
    assert!(ids.contains(&"power_of_attorney"));
    assert!(!ids.contains(&"sale_contract"));
    assert!(!ids.contains(&"confirm_nonresidential"));
}

#[tokio::test]
async fn forms_list_commercial_sale() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "D01").await;

    let res = call(&app, &token, Method::GET, &format!("/forms/{prop_id}"), None).await;
    assert_eq!(res.0, 200);

    let forms = res.1.as_array().unwrap();
    let ids: Vec<_> = forms.iter().map(|f| f["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"sale_contract"));
    assert!(ids.contains(&"confirm_nonresidential"));
    assert!(!ids.contains(&"confirm_residential"));
    assert!(!ids.contains(&"lease_contract"));
}

#[tokio::test]
async fn forms_get_html_sale_contract() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    let res = call_html(&app, &token, Method::GET, &format!("/forms/{prop_id}/sale_contract")).await;
    assert_eq!(res.0, 200);

    let html = res.1;
    assert!(html.contains("부동산 매매계약서"));
    assert!(html.contains("소재지"));
    assert!(html.contains("매매대금"));
}

#[tokio::test]
async fn forms_get_html_lease_contract() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "B1", "A01").await;

    let res = call_html(&app, &token, Method::GET, &format!("/forms/{prop_id}/lease_contract")).await;
    assert_eq!(res.0, 200);

    let html = res.1;
    assert!(html.contains("부동산 전세계약서"));
    assert!(html.contains("보증금"));
    assert!(html.contains("전세"));
}

#[tokio::test]
async fn forms_get_html_not_applicable_returns_404() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "B1", "A01").await;

    let res = call(&app, &token, Method::GET, &format!("/forms/{prop_id}/sale_contract"), None).await;
    assert_eq!(res.0, 404);
}