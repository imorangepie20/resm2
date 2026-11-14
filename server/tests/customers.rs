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
async fn customers_list_search_and_type_filter() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-list@example.com", "pw123456").await;
    let client = Client::new();

    // 고객 생성
    client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "김철수", "phone": "010-1111-2222", "types": ["매도인"] }))
        .send()
        .await
        .unwrap();

    client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "이영희", "phone": "010-3333-4444", "types": ["매수인"] }))
        .send()
        .await
        .unwrap();

    client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "박민수", "phone": "010-5555-6666", "types": ["단순방문"] }))
        .send()
        .await
        .unwrap();

    // 검색 (이름 부분일치)
    let res = client
        .get(format!("{BASE_URL}/customers?q=철수"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customers: Vec<Value> = res.json().await.unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "김철수");

    // 검색 (전화 부분일치)
    let res = client
        .get(format!("{BASE_URL}/customers?q=3333"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customers: Vec<Value> = res.json().await.unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "이영희");

    // 유형 필터
    let res = client
        .get(format!("{BASE_URL}/customers?type=매도인"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customers: Vec<Value> = res.json().await.unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "김철수");
    assert!(customers[0]["types"].as_array().unwrap().contains(&json!("매도인")));

    // 유형 필터 - 매수인
    let res = client
        .get(format!("{BASE_URL}/customers?type=매수인"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customers: Vec<Value> = res.json().await.unwrap();
    assert_eq!(customers.len(), 1);
    assert_eq!(customers[0]["name"], "이영희");
}

#[tokio::test]
async fn customers_create_normalizes_types() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-create@example.com", "pw123456").await;
    let client = Client::new();

    // 당사자 유형만 있는 경우 -> 당사자 유형만 남음
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "테스트1", "types": ["매도인", "단순방문"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = client
        .get(format!("{BASE_URL}/customers/{id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["types"], json!(["매도인"]));

    // 빈 배열 -> 단순방문
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "테스트2", "types": [] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = client
        .get(format!("{BASE_URL}/customers/{id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["types"], json!(["단순방문"]));

    // 유효하지 않은 유형은 무시됨
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "테스트3", "types": ["없는유형", "매수인"] }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let id = body["id"].as_str().unwrap();

    let res = client
        .get(format!("{BASE_URL}/customers/{id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["types"], json!(["매수인"]));
}

#[tokio::test]
async fn customers_create_with_property_id_links_party() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-prop@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // propertyId와 함께 고객 생성 -> 당사자 연결됨
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "매도인고객", "phone": "010-9999-8888", "types": ["매도인"], "propertyId": prop_id }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let cust_id = body["id"].as_str().unwrap();

    let res = client
        .get(format!("{BASE_URL}/customers/{cust_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["property_id"], json!(prop_id));
    assert!(customer["parties"].as_array().unwrap().len() > 0);
    let party = &customer["parties"][0];
    assert_eq!(party["role"], "매도인");
}

#[tokio::test]
async fn customers_update_changes_property_relink() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-update@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id1 = create_property(&client, &token, &cookies, "A1", "A01").await;
    let prop_id2 = create_property(&client, &token, &cookies, "B1", "A01").await;

    // 첫 번째 매물과 연결
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "업데이트테스트", "propertyId": prop_id1 }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let cust_id = body["id"].as_str().unwrap();

    // 두 번째 매물로 변경
    let res = client
        .patch(format!("{BASE_URL}/customers/{cust_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "propertyId": prop_id2 }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    let res = client
        .get(format!("{BASE_URL}/customers/{cust_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["property_id"], json!(prop_id2));
}

#[tokio::test]
async fn customers_delete_owned_only() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-del@example.com", "pw123456").await;
    let client = Client::new();

    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "삭제테스트" }))
        .send()
        .await
        .unwrap();
    let body: Value = res.json().await.unwrap();
    let cust_id = body["id"].as_str().unwrap();

    // 본인 고객 삭제
    let res = client
        .delete(format!("{BASE_URL}/customers/{cust_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // 이미 삭제됨
    let res = client
        .delete(format!("{BASE_URL}/customers/{cust_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    // 다른 사용자의 고객은 생성 불가 (404 - 남의 데이터)
    // 다른 사용자로 로그인해서 테스트하려면 별도 사용자 필요
}

#[tokio::test]
async fn customers_gender_validation() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-gender@example.com", "pw123456").await;
    let client = Client::new();

    // 유효한 성별
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "성별테스트", "gender": "남" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 유효한 성별 - 여
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "성별테스트2", "gender": "여" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 유효한 성별 - 미지정
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "성별테스트3", "gender": "미지정" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);

    // 잘못된 성별 -> 400
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "성별테스트4", "gender": "기타" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn customers_name_required() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-name@example.com", "pw123456").await;
    let client = Client::new();

    // 이름 없음 -> 400
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "phone": "010-1234-5678" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);

    // 빈 이름 -> 400
    let res = client
        .post(format!("{BASE_URL}/customers"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "name": "   " }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn customers_draft_from_property() {
    let pool = sqlx::PgPool::connect(
        &std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL"),
    )
    .await
    .expect("DB connect");

    let (token, cookies) = login(&pool, "test-cust-draft@example.com", "pw123456").await;
    let client = Client::new();

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    let res = client
        .get(format!("{BASE_URL}/customers/draft-from-property/{prop_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let draft: Value = res.json().await.unwrap();
    assert_eq!(draft["property_id"], json!(prop_id));
    assert_eq!(draft["name"], "홍길동"); // customerName from property
}