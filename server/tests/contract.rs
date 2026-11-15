mod common;

use common::TestApp;
use reqwest::Method;
use serde_json::{json, Value};
use std::collections::HashMap;

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

async fn get_contract(app: &TestApp, token: &str, property_id: &str) -> Value {
    let res = call(app, token, Method::GET, &format!("/contract/{property_id}"), None).await;
    assert_eq!(res.0, 200, "get contract failed: {:?}", res.1);
    res.1
}

async fn toggle_checklist(app: &TestApp, token: &str, property_id: &str, item_id: &str, checked: bool) {
    let res = call(
        app,
        token,
        Method::PATCH,
        &format!("/contract/{property_id}/checklist/{item_id}"),
        Some(&json!({ "checked": checked })),
    )
    .await;
    assert_eq!(res.0, 200, "toggle checklist failed: {:?}", res.1);
}

async fn checklist_all(app: &TestApp, token: &str, property_id: &str, checked: bool) {
    let res = call(
        app,
        token,
        Method::POST,
        &format!("/contract/{property_id}/checklist/all"),
        Some(&json!({ "checked": checked })),
    )
    .await;
    assert_eq!(res.0, 200, "checklist all failed: {:?}", res.1);
}

async fn start_contract(app: &TestApp, token: &str, property_id: &str) -> Value {
    let res = call(
        app,
        token,
        Method::POST,
        &format!("/contract/{property_id}/start"),
        None,
    )
    .await;
    assert_eq!(res.0, 200, "start contract failed: {:?}", res.1);
    res.1
}

async fn complete_contract(app: &TestApp, token: &str, property_id: &str) -> Value {
    let res = call(
        app,
        token,
        Method::POST,
        &format!("/contract/{property_id}/complete"),
        None,
    )
    .await;
    assert_eq!(res.0, 200, "complete contract failed: {:?}", res.1);
    res.1
}

#[tokio::test]
async fn contract_checklist_residential_sale() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    let contract = get_contract(&app, &token, &property_id).await;

    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"DOC_ID"));
    assert!(ids.contains(&"DOC_REGISTER"));
    assert!(ids.contains(&"DOC_BLDG_LEDGER"));
    assert!(ids.contains(&"DOC_CONTRACT"));
    assert!(ids.contains(&"DOC_CONFIRM"));

    assert!(ids.contains(&"DOC_TITLE_DEED"));
    assert!(ids.contains(&"DOC_SEAL_SALE"));
    assert!(ids.contains(&"ACT_OWNERSHIP"));
    assert!(ids.contains(&"FILE_TX_REPORT"));
    assert!(ids.contains(&"TAX_ACQUISITION"));
    assert!(ids.contains(&"TAX_CAPITAL_GAINS"));

    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
    assert_eq!(progress["done"].as_u64().unwrap(), 4);
    assert_eq!(progress["complete"].as_bool().unwrap(), false);

    toggle_checklist(&app, &token, &property_id, "DOC_ID", true).await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(checked.contains_key("DOC_ID"));

    checklist_all(&app, &token, &property_id, true).await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    for item in &checklist {
        if item["required"].as_bool().unwrap() {
            assert!(checked.contains_key(item["id"].as_str().unwrap()));
        }
    }
}

#[tokio::test]
async fn contract_checklist_residential_lease() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "B1", "A01").await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"ACT_MOVE_IN"));
    assert!(ids.contains(&"ACT_FIXED_DATE"));
    assert!(ids.contains(&"FILE_LEASE_REPORT"));
    assert!(ids.contains(&"DOC_TAX_CLEARANCE"));

    assert!(ids.contains(&"ACT_DEPOSIT_GUARANTEE"));

    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn contract_checklist_land_sale() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "E03").await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"ACT_FARMLAND_CERT"));
    assert!(ids.contains(&"ACT_LUT_PERMIT"));

    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn contract_start_transition() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    let contract = get_contract(&app, &token, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "진행");

    let contract = start_contract(&app, &token, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약진행");

    let contract = get_contract(&app, &token, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약진행");
}

#[tokio::test]
async fn contract_start_fails_on_wrong_status() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    start_contract(&app, &token, &property_id).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/contract/{property_id}/start"),
        None,
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn contract_complete_transition() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    start_contract(&app, &token, &property_id).await;

    checklist_all(&app, &token, &property_id, true).await;

    let contract = complete_contract(&app, &token, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약완료");
    assert_eq!(contract["progress"]["complete"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn contract_complete_fails_without_progress() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    start_contract(&app, &token, &property_id).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/contract/{property_id}/complete"),
        None,
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn contract_complete_fails_on_wrong_status() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    let res = call(
        &app,
        &token,
        Method::POST,
        &format!("/contract/{property_id}/complete"),
        None,
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn contract_checklist_toggle_persists() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;

    toggle_checklist(&app, &token, &property_id, "DOC_ID", true).await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(checked.contains_key("DOC_ID"));

    toggle_checklist(&app, &token, &property_id, "DOC_ID", false).await;

    let contract = get_contract(&app, &token, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(!checked.contains_key("DOC_ID"));
}

#[tokio::test]
async fn contract_required_fields_match() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let property_id = create_property(&app, &token, "A1", "A01").await;
    let contract = get_contract(&app, &token, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));

    let property_id = create_property(&app, &token, "B1", "A01").await;
    let contract = get_contract(&app, &token, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));
    assert!(!keys.contains(&"rentPrice"));

    let property_id = create_property(&app, &token, "B2", "A01").await;
    let contract = get_contract(&app, &token, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));
    assert!(keys.contains(&"rentPrice"));
}