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
            "contractDate": "20240115",
            "balanceDate": "20240215",
            "contractHopeDate": "20240110",
            "interim1Date": "20240120",
            "moveInHopeDate": "20240201",
            "moveInDate": "20240220",
            "customerName": "홍길동"
        })),
    )
    .await;
    let (status, body) = res;
    assert!(status.is_success() || status.as_u16() == 201, "create property failed ({}): {:?}", status, body);
    body["id"].as_str().unwrap().to_string()
}

async fn create_customer(app: &TestApp, token: &str, name: &str) -> String {
    let res = call(
        app,
        token,
        Method::POST,
        "/customers",
        Some(&json!({
            "name": name,
            "phone": "010-1234-5678",
            "type": "매도인"
        })),
    )
    .await;
    let (status, body) = res;
    assert_eq!(status, 200, "create customer failed: {:?}", body);
    body["id"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn events_load_calendar_empty() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let body = res.1;
    assert_eq!(body["events"].as_array().unwrap().len(), 0);
    assert_eq!(body["propertyDates"].as_array().unwrap().len(), 0);
    assert!(body["properties"].is_array());
    assert!(body["customers"].is_array());
}

#[tokio::test]
async fn events_create_and_list() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;
    let cust_id = create_customer(&app, &token, "테스트고객").await;

    // Create event
    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "미팅 약속",
            "date": "20240115",
            "startTime": "10:00",
            "category": "미팅",
            "memo": "매물 소개",
            "propertyId": prop_id,
            "customerId": cust_id
        })),
    )
    .await;
    assert_eq!(res.0, 200);
    let event_id = res.1["id"].as_str().unwrap().to_string();

    // List events for January 2024 (month=0)
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let events = res.1["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["id"], event_id);
    assert_eq!(events[0]["title"], "미팅 약속");
    assert_eq!(events[0]["date"], "20240115");
    assert_eq!(events[0]["startTime"], "10:00");
    assert_eq!(events[0]["category"], "미팅");
    assert_eq!(events[0]["memo"], "매물 소개");
    assert_eq!(events[0]["propertyId"], prop_id);
    assert!(events[0]["propertyLabel"].is_string());
    assert_eq!(events[0]["customerId"], cust_id);
    assert!(events[0]["customerLabel"].is_string());
}

#[tokio::test]
async fn events_create_all_categories() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    for (i, cat) in ["미팅", "임장", "계약", "기타"].iter().enumerate() {
        let date = format!("202401{:02}", i + 10);
        let res = call(
            &app,
            &token,
            Method::POST,
            "/events",
            Some(&json!({
                "title": format!("{} 일정", cat),
                "date": date,
                "category": cat
            })),
        )
        .await;
        assert_eq!(res.0, 200, "create {} failed: {:?}", cat, res.1);
    }

    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let events = res.1["events"].as_array().unwrap();
    assert_eq!(events.len(), 4);
    for (i, cat) in ["미팅", "임장", "계약", "기타"].iter().enumerate() {
        assert_eq!(events[i]["category"], *cat);
    }
}

#[tokio::test]
async fn events_create_invalid_category_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "테스트",
            "date": "20240115",
            "category": "잘못된카테고리"
        })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn events_create_invalid_date_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "테스트",
            "date": "2024-01-15",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn events_create_invalid_time_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "테스트",
            "date": "20240115",
            "startTime": "10:00:00",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn events_update() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Create
    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "원본 제목",
            "date": "20240115",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 200);
    let event_id = res.1["id"].as_str().unwrap();

    // Update
    let res = call(
        &app,
        &token,
        Method::PATCH,
        &format!("/events/{}", event_id),
        Some(&json!({
            "title": "수정된 제목",
            "date": "20240120",
            "category": "계약"
        })),
    )
    .await;
    assert_eq!(res.0, 204);

    // Verify
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let events = res.1["events"].as_array().unwrap();
    assert_eq!(events.len(), 1);
    assert_eq!(events[0]["title"], "수정된 제목");
    assert_eq!(events[0]["date"], "20240120");
    assert_eq!(events[0]["category"], "계약");
}

#[tokio::test]
async fn events_delete() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Create
    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "삭제할 일정",
            "date": "20240115",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 200);
    let event_id = res.1["id"].as_str().unwrap();

    // Delete
    let res = call(&app, &token, Method::DELETE, &format!("/events/{}", event_id), None).await;
    assert_eq!(res.0, 204);

    // Verify gone
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    assert_eq!(res.1["events"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn events_other_user_404() {
    let app = common::spawn_app().await;
    let (token1, _) = common::signup(&app).await;
    let (token2, _) = common::signup(&app).await;

    // User 1 creates event
    let res = call(
        &app,
        &token1,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "유저1 일정",
            "date": "20240115",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 200);
    let event_id = res.1["id"].as_str().unwrap();

    // User 2 tries to update -> 404
    let res = call(
        &app,
        &token2,
        Method::PATCH,
        &format!("/events/{}", event_id),
        Some(&json!({ "title": "해킹 시도" })),
    )
    .await;
    assert_eq!(res.0, 404);

    // User 2 tries to delete -> 404
    let res = call(&app, &token2, Method::DELETE, &format!("/events/{}", event_id), None).await;
    assert_eq!(res.0, 404);
}

#[tokio::test]
async fn events_property_dates_derived() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Create property with several date fields
    let prop_id = create_property(&app, &token, "A1", "A01").await;

    // Load calendar for January 2024 (month=0)
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let prop_dates = res.1["propertyDates"].as_array().unwrap();

    // Should have at least contractDate(20240115), contractHopeDate(20240110), interim1Date(20240120)
    assert!(prop_dates.len() >= 3);
    let dates: Vec<&str> = prop_dates.iter().map(|d| d["date"].as_str().unwrap()).collect();
    assert!(dates.contains(&"20240115")); // contractDate
    assert!(dates.contains(&"20240110")); // contractHopeDate
    assert!(dates.contains(&"20240120")); // interim1Date

    // Check labels contain property name and field label
    for pd in prop_dates {
        assert!(pd["label"].as_str().unwrap().contains("테스트 매물"));
        assert_eq!(pd["propertyId"].as_str().unwrap(), prop_id);
        assert!(pd["field"].is_string());
    }
}

#[tokio::test]
async fn events_property_dates_filter_by_month() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    // Create property with dates in different months
    let res = call(
        &app,
        &token,
        Method::POST,
        "/properties",
        Some(&json!({
            "name": "월별매물",
            "tradeType": "A1",
            "realEstateType": "A01",
            "address": "서울시",
            "source": "수기",
            "price": "100000000",
            "contractDate": "20240115",
            "balanceDate": "20240215",
            "moveInDate": "20240315"
        })),
    )
    .await;
    assert!(res.0.is_success() || res.0.as_u16() == 201);

    // January (month=0) - should have contractDate
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);
    let prop_dates = res.1["propertyDates"].as_array().unwrap();
    let dates: Vec<&str> = prop_dates.iter().map(|d| d["date"].as_str().unwrap()).collect();
    assert!(dates.contains(&"20240115"));
    assert!(!dates.contains(&"20240215"));
    assert!(!dates.contains(&"20240315"));

    // February (month=1) - should have balanceDate
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=1", None).await;
    assert_eq!(res.0, 200);
    let prop_dates = res.1["propertyDates"].as_array().unwrap();
    let dates: Vec<&str> = prop_dates.iter().map(|d| d["date"].as_str().unwrap()).collect();
    assert!(dates.contains(&"20240215"));
    assert!(!dates.contains(&"20240115"));

    // March (month=2) - should have moveInDate
    let res = call(&app, &token, Method::GET, "/events?year=2024&month=2", None).await;
    assert_eq!(res.0, 200);
    let prop_dates = res.1["propertyDates"].as_array().unwrap();
    let dates: Vec<&str> = prop_dates.iter().map(|d| d["date"].as_str().unwrap()).collect();
    assert!(dates.contains(&"20240315"));
}

#[tokio::test]
async fn events_dropdown_options() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let prop_id = create_property(&app, &token, "A1", "A01").await;
    let cust_id = create_customer(&app, &token, "옵션고객").await;

    let res = call(&app, &token, Method::GET, "/events?year=2024&month=0", None).await;
    assert_eq!(res.0, 200);

    let props = res.1["properties"].as_array().unwrap();
    assert!(props.iter().any(|p| p["id"] == prop_id && p["label"].as_str().unwrap() == "테스트 매물"));

    let customers = res.1["customers"].as_array().unwrap();
    assert!(customers.iter().any(|c| c["id"] == cust_id && c["label"].as_str().unwrap().contains("옵션고객")));
}

#[tokio::test]
async fn events_empty_title_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "   ",
            "date": "20240115",
            "category": "미팅"
        })),
    )
    .await;
    assert_eq!(res.0, 400);
}

#[tokio::test]
async fn events_missing_date_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;

    let res = call(
        &app,
        &token,
        Method::POST,
        "/events",
        Some(&json!({
            "title": "테스트",
            "category": "미팅"
        })),
    )
    .await;
    assert!(res.0.as_u16() == 400 || res.0.as_u16() == 422);
}