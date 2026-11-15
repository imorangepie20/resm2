use reqwest::Client;
use serde_json::{json, Value};

const BASE_URL: &str = "http://127.0.0.1:3104/api/v1";

async fn register(client: &Client, email: &str, password: &str, name: &str) -> (String, String) {
    let res = client
        .post(format!("{BASE_URL}/auth/signup"))
        .json(&json!({
            "agencyName": "테스트 공인중개사",
            "name": name,
            "email": email,
            "password": password,
            "passwordConfirm": password,
            "agree": true
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200, "register failed: {:?}", res.text().await);

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

async fn login(client: &Client, email: &str, password: &str) -> (String, String) {
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
    let status = res.status();
    let text = res.text().await.unwrap_or_default();
    assert!(status.is_success() || status.as_u16() == 201, "create property failed ({}): {}", status, text);
    let body: Value = serde_json::from_str(&text).unwrap();
    body["id"].as_str().unwrap().to_string()
}

async fn setup_test_user(client: &Client, email: &str, password: &str) -> (String, String) {
    // 회원가입 시도 (ASCII name for compatibility)
    let res = client
        .post(format!("{BASE_URL}/auth/signup"))
        .json(&json!({
            "agencyName": "Test Agency",
            "name": "Test User",
            "email": email,
            "password": password,
            "passwordConfirm": password,
            "agree": true
        }))
        .send()
        .await
        .unwrap();

    let status = res.status();
    if status == 200 {
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
        return (token, cookies);
    }

    // 이미 가입된 경우 (400) 또는 다른 에러 -> 로그인 시도
    let body: Value = res.json().await.unwrap();
    let err = body["error"].as_str().unwrap_or("");
    if err.contains("이미 가입") || status == 400 {
        // 기존 사용자로 로그인
        login(client, email, password).await
    } else {
        panic!("signup failed: {:?}", body);
    }
}

#[tokio::test]
async fn parties_add_reuses_existing_customer() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-reuse@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 첫 번째 당사자 추가 (매도인)
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "김철수", "phone": "010-1111-2222" }))
        .send()
        .await
        .unwrap();
    let status = res.status();
    if status != 200 {
        let text = res.text().await.unwrap_or_default();
        panic!("add party failed ({}): {}", status, text);
    }
    assert_eq!(status, 200);
    let body: Value = res.json().await.unwrap();
    let customer_id_1 = body["customerId"].as_str().unwrap().to_string();
    let party_id_1 = body["partyId"].as_str().unwrap().to_string();

    // 같은 이름+전화로 두 번째 당사자 추가 (매수인) -> 기존 고객 재사용
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매수인", "name": "김철수", "phone": "010-1111-2222" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let customer_id_2 = body["customerId"].as_str().unwrap().to_string();
    let party_id_2 = body["partyId"].as_str().unwrap().to_string();

    assert_eq!(customer_id_1, customer_id_2, "같은 고객이어야 함");
    assert_ne!(party_id_1, party_id_2, "당사자 ID는 달라야 함");

    // 고객 types에 두 역할 모두 포함되어야 함
    let res = client
        .get(format!("{BASE_URL}/customers/{customer_id_1}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    let types = customer["types"].as_array().unwrap();
    assert!(types.iter().any(|t| t == "매도인"));
    assert!(types.iter().any(|t| t == "매수인"));
}

#[tokio::test]
async fn parties_add_creates_new_customer() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-new@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "B1", "A01").await;

    // 매칭되는 고객 없음 -> 신규 생성
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "임대인", "name": "신규고객", "phone": "010-9999-8888" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let customer_id = body["customerId"].as_str().unwrap();

    // 고객 조회 - types에 임대인만 있어야 함
    let res = client
        .get(format!("{BASE_URL}/customers/{customer_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let customer: Value = res.json().await.unwrap();
    assert_eq!(customer["types"], json!(["임대인"]));
    assert_eq!(customer["name"], "신규고객");
    // 전화번호 저장 확인 (숫자만 또는 포맷 유지)
    assert!(customer["phone"].is_string());
}

#[tokio::test]
async fn parties_add_upserts_property_party() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-upsert@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 같은 propertyId, customerId, role로 두 번 호출
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "업서트테스트", "phone": "010-3333-4444" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let party_id_1 = body["partyId"].as_str().unwrap();

    // 두 번째 호출
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "업서트테스트", "phone": "010-3333-4444" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let party_id_2 = body["partyId"].as_str().unwrap();

    // 같은 partyId가 반환되어야 함 (upsert)
    assert_eq!(party_id_1, party_id_2);

    // 목록에서도 1건만 있어야 함
    let res = client
        .get(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let parties: Vec<Value> = res.json().await.unwrap();
    let seller_count = parties.iter().filter(|p| p["role"] == "매도인").count();
    assert_eq!(seller_count, 1);
}

#[tokio::test]
async fn parties_add_invalid_role_400() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-invalid@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 유효하지 않은 역할
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "기타", "name": "테스트", "phone": "010-1111-2222" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn parties_add_name_required() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-name@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 이름 없음 (422: 필수 필드 누락)
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "phone": "010-1111-2222" }))
        .send()
        .await
        .unwrap();
    assert!(res.status().as_u16() == 400 || res.status().as_u16() == 422);

    // 빈 이름 (400: 검증 실패)
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "   ", "phone": "010-1111-2222" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 400);
}

#[tokio::test]
async fn parties_list_returns_role_order() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-order@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 역순으로 추가
    client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "중개", "name": "중개사", "phone": "010-0000-0000" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "임차인", "name": "임차인", "phone": "010-1111-1111" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "임대인", "name": "임대인", "phone": "010-2222-2222" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매수인", "name": "매수인", "phone": "010-3333-3333" }))
        .send()
        .await
        .unwrap();
    client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "매도인", "phone": "010-4444-4444" }))
        .send()
        .await
        .unwrap();

    // 목록 조회 - 역할 순서 확인: 매도인, 매수인, 임대인, 임차인, 중개
    let res = client
        .get(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let parties: Vec<Value> = res.json().await.unwrap();
    assert_eq!(parties.len(), 5);
    assert_eq!(parties[0]["role"], "매도인");
    assert_eq!(parties[1]["role"], "매수인");
    assert_eq!(parties[2]["role"], "임대인");
    assert_eq!(parties[3]["role"], "임차인");
    assert_eq!(parties[4]["role"], "중개");
}

#[tokio::test]
async fn parties_remove_deletes_link_only() {
    let client = Client::new();
    let (token, cookies) = setup_test_user(&client, "test-party-remove@example.com", "pw123456").await;

    let prop_id = create_property(&client, &token, &cookies, "A1", "A01").await;

    // 당사자 추가
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .json(&json!({ "role": "매도인", "name": "삭제테스트", "phone": "010-5555-6666" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let body: Value = res.json().await.unwrap();
    let party_id = body["partyId"].as_str().unwrap();
    let customer_id = body["customerId"].as_str().unwrap();

    // 당사자 삭제
    let res = client
        .delete(format!("{BASE_URL}/properties/{prop_id}/parties/{party_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 204);

    // 목록에서 사라짐
    let res = client
        .get(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
    let parties: Vec<Value> = res.json().await.unwrap();
    assert_eq!(parties.len(), 0);

    // 고객은 유지됨
    let res = client
        .get(format!("{BASE_URL}/customers/{customer_id}"))
        .bearer_auth(&token)
        .header("Cookie", &cookies)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 200);
}

#[tokio::test]
async fn parties_other_users_property_404() {
    let client = Client::new();
    let (token1, cookies1) = setup_test_user(&client, "test-party-user1@example.com", "pw123456").await;
    let (token2, cookies2) = setup_test_user(&client, "test-party-user2@example.com", "pw123456").await;

    // 사용자 1의 매물
    let prop_id = create_property(&client, &token1, &cookies1, "A1", "A01").await;

    // 사용자 2가 사용자 1의 매물에 당사자 추가 시도 -> 404
    let res = client
        .post(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token2)
        .header("Cookie", &cookies2)
        .json(&json!({ "role": "매도인", "name": "타인매물", "phone": "010-7777-8888" }))
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    // 사용자 2가 사용자 1의 매물 당사자 목록 조회 -> 404
    let res = client
        .get(format!("{BASE_URL}/properties/{prop_id}/parties"))
        .bearer_auth(&token2)
        .header("Cookie", &cookies2)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);

    // 사용자 2가 사용자 1의 매물 당사자 삭제 시도 -> 404
    let res = client
        .delete(format!("{BASE_URL}/properties/{prop_id}/parties/some-id"))
        .bearer_auth(&token2)
        .header("Cookie", &cookies2)
        .send()
        .await
        .unwrap();
    assert_eq!(res.status(), 404);
}