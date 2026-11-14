use reqwest::Client;
use serde_json::{json, Value};
use sqlx::PgPool;

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

#[tokio::test]
async fn forms_list_residential_sale() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-rsale@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "forms list failed: {:?}", res.text().await);

    let forms: Vec<Value> = res.json().await.unwrap();
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
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-rlease@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "B1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "forms list failed: {:?}", res.text().await);

    let forms: Vec<Value> = res.json().await.unwrap();
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
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-csale@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "A1", "D01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "forms list failed: {:?}", res.text().await);

    let forms: Vec<Value> = res.json().await.unwrap();
    let ids: Vec<_> = forms.iter().map(|f| f["id"].as_str().unwrap()).collect();

    assert!(ids.contains(&"sale_contract"));
    assert!(ids.contains(&"confirm_nonresidential"));
    assert!(!ids.contains(&"confirm_residential"));
    assert!(!ids.contains(&"lease_contract"));
}

#[tokio::test]
async fn forms_get_html_sale_contract() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-html@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}/sale_contract"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "get form html failed: {:?}", res.text().await);

    let html = res.text().await.unwrap();
    assert!(html.contains("부동산 매매계약서"));
    assert!(html.contains("소재지"));
    assert!(html.contains("매매대금"));
}

#[tokio::test]
async fn forms_get_html_lease_contract() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-lease-html@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "B1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}/lease_contract"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "get form html failed: {:?}", res.text().await);

    let html = res.text().await.unwrap();
    assert!(html.contains("부동산 임대차계약서"));
    assert!(html.contains("보증금"));
    assert!(html.contains("월세"));
}

#[tokio::test]
async fn forms_get_html_not_applicable_returns_404() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-forms-404@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "B1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/forms/{prop_id}/sale_contract"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404, "should be 404 for non-applicable form");
}