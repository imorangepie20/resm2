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
async fn customers_list_search_and_type_filter() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Create customers
    call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "김철수", "phone": "010-1111-2222", "types": ["매도인"] })),
    )
    .await;

    call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "이영희", "phone": "010-3333-4444", "types": ["매수인"] })),
    )
    .await;

    call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "박민수", "phone": "010-5555-6666", "types": ["단순방문"] })),
    )
    .await;

    // Search by name
    let res = call(&app, &token, Method::GET, "/customers?q=철수", None).await;
    assert_eq!(res.0, 200);
    let customers = res.1.as_array().unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "김철수");

    // Search by phone
    let res = call(&app, &token, Method::GET, "/customers?q=3333", None).await;
    assert_eq!(res.0, 200);
    let customers = res.1.as_array().unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "이영희");

    // Filter by type
    let res = call(&app, &token, Method::GET, "/customers?type=매도인", None).await;
    assert_eq!(res.0, 200);
    let customers = res.1.as_array().unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "김철수");
    assert!(customers[0]["types"].as_array().unwrap().contains(&json!("매도인")));

    let res = call(&app, &token, Method::GET, "/customers?type=매수인", None).await;
    assert_eq!(res.0, 200);
    let customers = res.1.as_array().unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "이영희");
}

#[tokio::test]
async fn customers_create_normalizes_types() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Party type only -> keeps party type
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "테스트1", "types": ["매도인", "단순방문"] })),
    )
    .await;
    assert_eq!(res.0, 200);
    let id = res.1["id"].as_str().unwrap();

    let res = call(&app, &token, Method::GET, &format!("/customers/{id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["types"], json!(["매도인"]));

    // Empty array -> 단순방문
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "테스트2", "types": [] })),
    )
    .await;
    assert_eq!(res.0, 200);
    let id = res.1["id"].as_str().unwrap();

    let res = call(&app, &token, Method::GET, &format!("/customers/{id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["types"], json!(["단순방문"]));

    // Invalid type ignored
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "테스트3", "types": ["없는유형", "매수인"] })),
    )
    .await;
    assert_eq!(res.0, 200);
    let id = res.1["id"].as_str().unwrap();

    let res = call(&app, &token, Method::GET, &format!("/customers/{id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["types"], json!(["매수인"]));
}

#[tokio::test]
async fn customers_create_with_property_id_links_party() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "매도인고객", "phone": "010-9999-8888", "types": ["매도인"], "property_id": prop_id })),
    )
    .await;
    assert_eq!(res.0, 200);
    let cust_id = res.1["id"].as_str().unwrap();

    let res = call(&app, &token, Method::GET, &format!("/customers/{cust_id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["property_id"], json!(prop_id));
    assert!(res.1["parties"].as_array().unwrap().len() > 0);
    let party = &res.1["parties"][0];
    assert_eq!(party["role"], "매도인");
}

#[tokio::test]
async fn customers_update_changes_property_relink() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id1 = create_property(&app, &token, "A1", "A01").await;
    let prop_id2 = create_property(&app, &token, "B1", "A01").await;

    // Create with first property
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "업데이트테스트", "property_id": prop_id1 })),
    )
    .await;
    let cust_id = res.1["id"].as_str().unwrap();

    // Update to second property
    let res = call(
        &app,
        &token,
        Method::PATCH,
        &format!("/customers/{cust_id}"),
        Some(&json!({ "property_id": prop_id2 })),
    )
    .await;
    assert!(res.0.as_u16() == 200 || res.0.as_u16() == 204);

    let res = call(&app, &token, Method::GET, &format!("/customers/{cust_id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["property_id"], json!(prop_id2));
}

#[tokio::test]
async fn customers_delete_owned_only() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "삭제테스트" })),
    )
    .await;
    let cust_id = res.1["id"].as_str().unwrap();

    // Delete own customer
    let res = call(&app, &token, Method::DELETE, &format!("/customers/{cust_id}"), None).await;
    assert_eq!(res.0, 200);

    // Already deleted
    let res = call(&app, &token, Method::DELETE, &format!("/customers/{cust_id}"), None).await;
    assert_eq!(res.0, 404);
}

#[tokio::test]
async fn customers_gender_validation() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Valid gender
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "성별테스트", "gender": "남" })),
    )
    .await;
    assert_eq!(res.0, 200);

    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "성별테스트2", "gender": "여" })),
    )
    .await;
    assert_eq!(res.0, 200);

    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "성별테스트3", "gender": "미지정" })),
    )
    .await;
    assert_eq!(res.0, 200);

    // Invalid gender -> ignored, becomes null
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "성별테스트4", "gender": "기타" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let id = res.1["id"].as_str().unwrap();

    let res = call(&app, &token, Method::GET, &format!("/customers/{id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["gender"], Value::Null);
}

#[tokio::test]
async fn customers_name_required() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Missing name -> 422
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "phone": "010-1234-5678" })),
    )
    .await;
    assert_eq!(res.0, 422);

    // Empty name -> 400 or 422
    let res = call(
        &app,
        &token,
        Method::POST,
        "/customers",
        Some(&json!({ "name": "   " })),
    )
    .await;
    assert!(res.0.as_u16() == 400 || res.0.as_u16() == 422);
}

#[tokio::test]
async fn customers_draft_from_property() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    let res = call(
        &app,
        &token,
        Method::GET,
        &format!("/customers/draft-from-property/{prop_id}"),
        None,
    )
    .await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["property_id"], json!(prop_id));
    assert_eq!(res.1["name"], "홍길동");
}