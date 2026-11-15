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
async fn parties_add_reuses_existing_customer() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // First party (seller)
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "김철수", "phone": "010-1111-2222" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let customer_id_1 = res.1["customerId"].as_str().unwrap().to_string();
    let party_id_1 = res.1["partyId"].as_str().unwrap().to_string();

    // Second party with same name+phone (buyer) -> reuses customer
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매수인", "name": "김철수", "phone": "010-1111-2222" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let customer_id_2 = res.1["customerId"].as_str().unwrap().to_string();
    let party_id_2 = res.1["partyId"].as_str().unwrap().to_string();

    assert_eq!(customer_id_1, customer_id_2, "같은 고객이어야 함");
    assert_ne!(party_id_1, party_id_2, "당사자 ID는 달라야 함");

    // Customer types should have both roles
    let res = call(&app, &token, Method::GET, &format!("/customers/{customer_id_1}"), None).await;
    assert_eq!(res.0, 200);
    let types = res.1["types"].as_array().unwrap();
    assert!(types.iter().any(|t| t == "매도인"));
    assert!(types.iter().any(|t| t == "매수인"));
}

#[tokio::test]
async fn parties_add_creates_new_customer() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "B1", "A01").await;

    // No matching customer -> creates new
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "임대인", "name": "신규고객", "phone": "010-9999-8888" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let customer_id = res.1["customerId"].as_str().unwrap();

    // Customer should have only landlord type
    let res = call(&app, &token, Method::GET, &format!("/customers/{customer_id}"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["types"], json!(["임대인"]));
    assert_eq!(res.1["name"], "신규고객");
    assert!(res.1["phone"].is_string());
}

#[tokio::test]
async fn parties_add_upserts_property_party() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // First call
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "업서트테스트", "phone": "010-3333-4444" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let party_id_1 = res.1["partyId"].as_str().unwrap();

    // Second call with same data
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "업서트테스트", "phone": "010-3333-4444" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let party_id_2 = res.1["partyId"].as_str().unwrap();

    // Same partyId returned (upsert)
    assert_eq!(party_id_1, party_id_2);

    // List should have only 1 seller
    let res = call(&app, &token, Method::GET, &format!("/properties/{prop_id}/parties"), None).await;
    assert_eq!(res.0, 200);
    let parties = res.1.as_array().unwrap();
    let seller_count = parties.iter().filter(|p| p["role"] == "매도인").count();
    assert_eq!(seller_count, 1);
}

#[tokio::test]
async fn parties_add_invalid_role_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // Invalid role
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "기타", "name": "테스트", "phone": "010-1111-2222" })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn parties_add_name_required() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // Missing name
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "phone": "010-1111-2222" })),
    )
    .await;
    assert!(res.0.as_u16() == 400 || res.0.as_u16() == 422);

    // Empty name
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "   ", "phone": "010-1111-2222" })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn parties_list_returns_role_order() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // Add in reverse order
    call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "중개", "name": "중개사", "phone": "010-0000-0000" })),
    )
    .await;
    call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "임차인", "name": "임차인", "phone": "010-1111-1111" })),
    )
    .await;
    call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "임대인", "name": "임대인", "phone": "010-2222-2222" })),
    )
    .await;
    call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매수인", "name": "매수인", "phone": "010-3333-3333" })),
    )
    .await;
    call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "매도인", "phone": "010-4444-4444" })),
    )
    .await;

    // List should return ordered: 매도인, 매수인, 임대인, 임차인, 중개
    let res = call(&app, &token, Method::GET, &format!("/properties/{prop_id}/parties"), None).await;
    assert_eq!(res.0, 200);
    let parties = res.1.as_array().unwrap();
    assert_eq!(parties.len(), 5);
    assert_eq!(parties[0]["role"], "매도인");
    assert_eq!(parties[1]["role"], "매수인");
    assert_eq!(parties[2]["role"], "임대인");
    assert_eq!(parties[3]["role"], "임차인");
    assert_eq!(parties[4]["role"], "중개");
}

#[tokio::test]
async fn parties_remove_deletes_link_only() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // Add party
    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "삭제테스트", "phone": "010-5555-6666" })),
    )
    .await;
    assert_eq!(res.0, 200);
    let party_id = res.1["partyId"].as_str().unwrap();
    let customer_id = res.1["customerId"].as_str().unwrap();

    // Delete party
    let res = call(
        &app,
        &token,
        Method::DELETE,
        &format!("/properties/{prop_id}/parties/{party_id}"),
        None,
    )
    .await;
    assert_eq!(res.0, 204);

    // Gone from list
    let res = call(&app, &token, Method::GET, &format!("/properties/{prop_id}/parties"), None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1.as_array().unwrap().len(), 0);

    // Customer still exists
    let res = call(&app, &token, Method::GET, &format!("/customers/{customer_id}"), None).await;
    assert_eq!(res.0, 200);
}

#[tokio::test]
async fn parties_other_users_property_404() {
    let app = common::spawn_app().await;
    let (token1, _) = common::signup(&app).await;
    let (token2, _) = common::signup(&app).await;

    // User 1's property
    let prop_id = create_property(&app, &token1, "A1", "A01").await;

    // User 2 tries to add party -> 404
    let res = call(
        &app,
        &token2,
        Method::POST,
        &format!("/properties/{prop_id}/parties"),
        Some(&json!({ "role": "매도인", "name": "타인매물", "phone": "010-7777-8888" })),
    )
    .await;
    assert_eq!(res.0, 404);

    // User 2 tries to list parties -> 404
    let res = call(&app, &token2, Method::GET, &format!("/properties/{prop_id}/parties"), None).await;
    assert_eq!(res.0, 404);

    // User 2 tries to delete party -> 404
    let res = call(
        &app,
        &token2,
        Method::DELETE,
        &format!("/properties/{prop_id}/parties/some-id"),
        None,
    )
    .await;
    assert_eq!(res.0, 404);
}