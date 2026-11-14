use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;
use std::collections::HashMap;

const BASE_URL: &str = "http://127.0.0.1:3105/api/v1";

async fn login(pool: &PgPool, email: &str, password: &str) -> (String, String) {
    let client = Client::new();
    let res = client
        .post(format!("{BASE_URL}/auth/login"))
        .json(&json!({ "email": email, "password": password }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "login failed: {:?}", res.text().await);

    let cookies = res
        .headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok())
        .map(|s| s.to_string())
        .collect::<Vec<_>>()
        .join("; ");

    let body: Value = res.json().await.unwrap();
    let token = body["token"].as_str().unwrap().to_string();
    (token, cookies)
}

async fn create_property(
    client: &Client,
    token: &str,
    cookies: &str,
    trade_type: &str,
    real_estate_type: &str,
) -> String {
    let res = client
        .post(format!("{BASE_URL}/properties"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .json(&json!({
            "name": "테스트 매물",
            "tradeType": trade_type,
            "realEstateType": real_estate_type,
            "address": "서울시 강남구",
            "source": "수기",
            "price": "100000000",
            "contractDate": "20240101",
            "balanceDate": "20240201",
            "customerName": "홍길동"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "create property failed: {:?}", res.text().await);
    let body: Value = res.json().await.unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn get_contract(
    client: &Client,
    token: &str,
    cookies: &str,
    property_id: &str,
) -> Value {
    let res = client
        .get(format!("{BASE_URL}/contract/{property_id}"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "get contract failed: {:?}", res.text().await);
    res.json().await.unwrap()
}

async fn toggle_checklist(
    client: &Client,
    token: &str,
    cookies: &str,
    property_id: &str,
    item_id: &str,
    checked: bool,
) {
    let res = client
        .patch(format!("{BASE_URL}/contract/{property_id}/checklist/{item_id}"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .json(&json!({ "checked": checked }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "toggle checklist failed: {:?}", res.text().await);
}

async fn checklist_all(
    client: &Client,
    token: &str,
    cookies: &str,
    property_id: &str,
    checked: bool,
) {
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/checklist/all"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .json(&json!({ "checked": checked }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "checklist all failed: {:?}", res.text().await);
}

async fn start_contract(
    client: &Client,
    token: &str,
    cookies: &str,
    property_id: &str,
) -> Value {
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/start"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "start contract failed: {:?}", res.text().await);
    res.json().await.unwrap()
}

async fn complete_contract(
    client: &Client,
    token: &str,
    cookies: &str,
    property_id: &str,
) -> Value {
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/complete"))
        .bearer_auth(token)
        .header("Cookie", cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "complete contract failed: {:?}", res.text().await);
    res.json().await.unwrap()
}

#[tokio::test]
async fn contract_checklist_residential_sale() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    // 사용자 생성 또는 기존 사용자 사용
    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    // 주거용 매매 매물 생성
    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 계약 조회
    let contract = get_contract(&client, &token, &cookies, &property_id).await;

    // 체크리스트 검증 - 주거용 매매용 아이템들이 있어야 함
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    // 공통 필수
    assert!(ids.contains(&"DOC_ID"));
    assert!(ids.contains(&"DOC_REGISTER"));
    assert!(ids.contains(&"DOC_BLDG_LEDGER"));
    assert!(ids.contains(&"DOC_CONTRACT"));
    assert!(ids.contains(&"DOC_CONFIRM"));

    // 매매 필수
    assert!(ids.contains(&"DOC_TITLE_DEED"));
    assert!(ids.contains(&"DOC_SEAL_SALE"));
    assert!(ids.contains(&"ACT_OWNERSHIP"));
    assert!(ids.contains(&"FILE_TX_REPORT"));
    assert!(ids.contains(&"TAX_ACQUISITION"));
    assert!(ids.contains(&"TAX_CAPITAL_GAINS"));

    // 진행률 확인
    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
    assert_eq!(progress["done"].as_u64().unwrap(), 0);
    assert_eq!(progress["complete"].as_bool().unwrap(), false);

    // 체크리스트 토글 테스트
    toggle_checklist(&client, &token, &cookies, &property_id, "DOC_ID", true).await;

    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(checked.contains_key("DOC_ID"));

    // 전체 체크
    checklist_all(&client, &token, &cookies, &property_id, true).await;

    let contract = get_contract(&client, &token, &cookies, &property_id).await;
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
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    // 주거용 전세 매물 생성
    let property_id = create_property(&client, &token, &cookies, "B1", "A01").await;

    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    // 전세 필수
    assert!(ids.contains(&"ACT_MOVE_IN"));
    assert!(ids.contains(&"ACT_FIXED_DATE"));
    assert!(ids.contains(&"FILE_LEASE_REPORT"));
    assert!(ids.contains(&"DOC_TAX_CLEARANCE"));

    // 전세보증금반환보증 (B1만)
    assert!(ids.contains(&"ACT_DEPOSIT_GUARANTEE"));

    // 진행률
    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn contract_checklist_land_sale() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    // 토지 매매 매물 생성
    let property_id = create_property(&client, &token, &cookies, "A1", "E03").await;

    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let checklist: Vec<Value> = contract["checklist"].as_array().unwrap().clone();
    let ids: Vec<&str> = checklist.iter().map(|i| i["id"].as_str().unwrap()).collect();

    // 토지 필수
    assert!(ids.contains(&"ACT_FARMLAND_CERT"));
    assert!(ids.contains(&"ACT_LUT_PERMIT"));

    let progress = &contract["progress"];
    assert!(progress["total"].as_u64().unwrap() > 0);
}

#[tokio::test]
async fn contract_start_transition() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 상태 확인 - "진행"이어야 함
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "진행");

    // 계약진행 시작
    let contract = start_contract(&client, &token, &cookies, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약진행");

    // 다시 조회해도 계약진행
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약진행");
}

#[tokio::test]
async fn contract_start_fails_on_wrong_status() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 먼저 계약진행으로 변경
    start_contract(&client, &token, &cookies, &property_id).await;

    // 이미 계약진행 상태인데 다시 시작 시도 -> 400
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/start"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn contract_complete_transition() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 계약진행으로 변경
    start_contract(&client, &token, &cookies, &property_id).await;

    // 필수 필드가 모두 채워진 상태에서 완료 시도
    // price, contractDate, balanceDate, customerName은 생성 시 입력함
    // 체크리스트 전체 체크
    checklist_all(&client, &token, &cookies, &property_id, true).await;

    // 계약완료
    let contract = complete_contract(&client, &token, &cookies, &property_id).await;
    assert_eq!(contract["status"].as_str().unwrap(), "계약완료");
    assert_eq!(contract["progress"]["complete"].as_bool().unwrap(), true);
}

#[tokio::test]
async fn contract_complete_fails_without_progress() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 계약진행으로 변경
    start_contract(&client, &token, &cookies, &property_id).await;

    // 체크리스트 체크 안 하고 완료 시도 -> 400
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/complete"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn contract_complete_fails_on_wrong_status() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 진행 상태에서 바로 완료 시도 -> 400
    let res = client
        .post(format!("{BASE_URL}/contract/{property_id}/complete"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn contract_checklist_toggle_persists() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 토글 on
    toggle_checklist(&client, &token, &cookies, &property_id, "DOC_ID", true).await;

    // 다시 조회해서 확인
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(checked.contains_key("DOC_ID"));

    // 토글 off
    toggle_checklist(&client, &token, &cookies, &property_id, "DOC_ID", false).await;

    // 다시 조회해서 확인
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let checked: HashMap<String, String> = serde_json::from_value(contract["checked"].clone()).unwrap();
    assert!(!checked.contains_key("DOC_ID"));
}

#[tokio::test]
async fn contract_required_fields_match() {
    let pool = PgPool::connect(&std::env::var("DATABASE_URL").unwrap())
        .await
        .unwrap();

    let (token, cookies) = login(&pool, "test@example.com", "password123").await;
    let client = Client::new();

    // 매매
    let property_id = create_property(&client, &token, &cookies, "A1", "A01").await;
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));

    // 전세
    let property_id = create_property(&client, &token, &cookies, "B1", "A01").await;
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));
    assert!(keys.contains(&"rentPrice"));

    // 월세
    let property_id = create_property(&client, &token, &cookies, "B2", "A01").await;
    let contract = get_contract(&client, &token, &cookies, &property_id).await;
    let required: Vec<Value> = contract["required_fields"].as_array().unwrap().clone();
    let keys: Vec<&str> = required.iter().map(|r| r["key"].as_str().unwrap()).collect();
    assert!(keys.contains(&"customerName"));
    assert!(keys.contains(&"contractDate"));
    assert!(keys.contains(&"balanceDate"));
    assert!(keys.contains(&"price"));
    assert!(keys.contains(&"rentPrice"));
}