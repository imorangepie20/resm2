mod common;

use reqwest::{Method, StatusCode};
use serde_json::{Value, json};

use common::TestApp;

async fn call(
    app: &TestApp,
    token: Option<&str>,
    method: Method,
    path: &str,
    body: Option<&Value>,
) -> (StatusCode, Value) {
    let mut req = app
        .client
        .request(method, format!("{}/api/v1{path}", app.addr));
    if let Some(t) = token {
        req = req.bearer_auth(t);
    }
    if let Some(b) = body {
        req = req.json(b);
    }
    let resp = req.send().await.unwrap();
    let status = resp.status();
    let text = resp.text().await.unwrap();
    (status, serde_json::from_str(&text).unwrap_or(Value::Null))
}

async fn create(app: &TestApp, token: &str, body: Value) -> String {
    let (status, v) = call(app, Some(token), Method::POST, "/properties", Some(&body)).await;
    assert_eq!(status, StatusCode::CREATED, "{v}");
    v["id"].as_str().unwrap().to_string()
}

async fn get(app: &TestApp, token: &str, id: &str) -> (StatusCode, Value) {
    call(
        app,
        Some(token),
        Method::GET,
        &format!("/properties/{id}"),
        None,
    )
    .await
}

async fn patch(app: &TestApp, token: &str, id: &str, body: Value) -> (StatusCode, Value) {
    call(
        app,
        Some(token),
        Method::PATCH,
        &format!("/properties/{id}"),
        Some(&body),
    )
    .await
}

async fn list(app: &TestApp, token: &str, query: &str) -> (StatusCode, Value) {
    call(
        app,
        Some(token),
        Method::GET,
        &format!("/properties{query}"),
        None,
    )
    .await
}

#[tokio::test]
async fn create_defaults_and_get() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({ "name": "테스트" })).await;
    let (status, p) = get(&app, &token, &id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(p["id"], id);
    assert_eq!(p["name"], "테스트");
    assert_eq!(p["status"], "진행");
    assert_eq!(p["source"], "수기");
    assert_eq!(p["isFavorite"], false);
    assert!(p["createdAt"].as_str().unwrap().ends_with('Z'));
    for key in [
        "colorTag",
        "articleNo",
        "complexName",
        "address",
        "realEstateType",
        "tradeType",
        "heating",
        "customerName",
        "customerPhone",
        "partnerName",
        "partnerPhone",
        "partnerManager",
        "manager",
        "contractHopeDate",
        "contractDate",
        "moveInHopeDate",
        "moveInDate",
        "interim1Date",
        "interim2Date",
        "balanceDate",
        "approvalDate",
        "note",
        "memo",
        "siteArea",
        "areaExclusive",
        "areaSupply",
        "landArea",
        "buildingArea",
        "area",
        "totalHouseholds",
        "parkingCount",
        "isPreSale",
        "price",
        "rentPrice",
    ] {
        assert!(
            p.get(key).is_some_and(Value::is_null),
            "{key} should be null"
        );
    }
    assert!(p.get("dealAmount").is_none());
    assert!(p.get("userId").is_none());
}

#[tokio::test]
async fn money_roundtrips_as_string() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(
        &app,
        &token,
        json!({ "price": "9007199254740993", "rentPrice": 500000 }),
    )
    .await;
    let (_, p) = get(&app, &token, &id).await;
    assert_eq!(p["price"], "9007199254740993");
    assert_eq!(p["rentPrice"], "500000");
}

#[tokio::test]
async fn money_rejects_non_numeric() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    for bad in [
        json!("1.5"),
        json!("abc"),
        json!("-1"),
        json!(1.5),
        json!(-1),
    ] {
        let (status, v) = call(
            &app,
            Some(&token),
            Method::POST,
            "/properties",
            Some(&json!({ "price": bad })),
        )
        .await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{bad}");
        assert_eq!(v["error"], "금액 형식이 올바르지 않습니다");
    }
    let id = create(&app, &token, json!({ "name": "a" })).await;
    let (status, v) = patch(&app, &token, &id, json!({ "rentPrice": "abc" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(v["error"], "금액 형식이 올바르지 않습니다");
}

#[tokio::test]
async fn patch_absent_keeps_null_clears() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({ "name": "단지", "memo": "메모" })).await;
    let (status, _) = patch(&app, &token, &id, json!({ "memo": null })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, p) = get(&app, &token, &id).await;
    assert!(p["memo"].is_null());
    assert_eq!(p["name"], "단지");
}

#[tokio::test]
async fn patch_null_status_ignored_bad_status_400() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({ "status": "계약진행" })).await;
    let (status, _) = patch(&app, &token, &id, json!({ "status": null })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let (_, p) = get(&app, &token, &id).await;
    assert_eq!(p["status"], "계약진행");
    let (status, v) = patch(&app, &token, &id, json!({ "status": "끝" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(v["error"], "상태 값이 올바르지 않습니다");
    let (_, p) = get(&app, &token, &id).await;
    assert_eq!(p["status"], "계약진행");
}

#[tokio::test]
async fn patch_color_tag_validated() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({})).await;
    let (status, v) = patch(&app, &token, &id, json!({ "colorTag": "red" })).await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(v["error"], "색상 값이 올바르지 않습니다");
    let (status, _) = patch(&app, &token, &id, json!({ "colorTag": "blue" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(get(&app, &token, &id).await.1["colorTag"], "blue");
    let (status, _) = patch(&app, &token, &id, json!({ "colorTag": null })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert!(get(&app, &token, &id).await.1["colorTag"].is_null());
}

async fn updated_at(app: &TestApp, id: &str) -> chrono::NaiveDateTime {
    sqlx::query_scalar(r#"SELECT "updatedAt" FROM "Property" WHERE "id" = $1"#)
        .bind(id)
        .fetch_one(&app.db)
        .await
        .unwrap()
}

#[tokio::test]
async fn patch_sets_updated_at() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({ "name": "a" })).await;
    let before = updated_at(&app, &id).await;
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;
    let (status, _) = patch(&app, &token, &id, json!({ "name": "b" })).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    let after = updated_at(&app, &id).await;
    assert!(after > before, "{after} > {before}");

    // 빈 패치는 updatedAt 을 건드리지 않는다.
    let (status, _) = patch(&app, &token, &id, json!({})).await;
    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(updated_at(&app, &id).await, after);
}

#[tokio::test]
async fn views_filter() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    create(&app, &token, json!({ "isFavorite": true })).await;
    create(&app, &token, json!({ "status": "계약진행" })).await;
    create(&app, &token, json!({ "status": "계약완료" })).await;
    create(&app, &token, json!({})).await;
    for (view, n) in [
        ("all", 4),
        ("favorites", 1),
        ("in-progress", 1),
        ("contracted", 1),
    ] {
        let (status, v) = list(&app, &token, &format!("?view={view}")).await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(v.as_array().unwrap().len(), n, "{view}");
    }
    assert_eq!(list(&app, &token, "").await.1.as_array().unwrap().len(), 4);
    let (status, _) = list(&app, &token, "?view=xyz").await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn list_is_created_desc() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let a = create(&app, &token, json!({ "name": "첫째" })).await;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let b = create(&app, &token, json!({ "name": "둘째" })).await;
    tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    let c = create(&app, &token, json!({ "name": "셋째" })).await;
    let (_, v) = list(&app, &token, "").await;
    let ids: Vec<&str> = v
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["id"].as_str().unwrap())
        .collect();
    assert_eq!(ids, vec![c.as_str(), b.as_str(), a.as_str()]);
}

#[tokio::test]
async fn other_users_property_is_404() {
    let app = common::spawn_app().await;
    let (token_a, _) = common::signup(&app).await;
    let (token_b, _) = common::signup(&app).await;
    let id = create(&app, &token_a, json!({ "name": "A의 매물" })).await;

    assert_eq!(get(&app, &token_b, &id).await.0, StatusCode::NOT_FOUND);
    let (status, _) = patch(&app, &token_b, &id, json!({ "name": "탈취" })).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, _) = patch(&app, &token_b, &id, json!({})).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    let (status, v) = call(
        &app,
        Some(&token_b),
        Method::POST,
        "/properties/delete",
        Some(&json!({ "ids": [id] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["count"], 0);
    assert!(
        list(&app, &token_b, "")
            .await
            .1
            .as_array()
            .unwrap()
            .is_empty()
    );

    let (status, p) = get(&app, &token_a, &id).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(p["name"], "A의 매물");

    let (status, v) = call(
        &app,
        Some(&token_a),
        Method::POST,
        "/properties/delete",
        Some(&json!({ "ids": [id] })),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(v["count"], 1);
    assert_eq!(get(&app, &token_a, &id).await.0, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn unauthenticated_is_401() {
    let app = common::spawn_app().await;
    let (status, _) = call(&app, None, Method::GET, "/properties", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = call(&app, None, Method::POST, "/properties", Some(&json!({}))).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let (status, _) = call(&app, None, Method::GET, "/properties/abc", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn create_notifies_superadmins() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let admin_id = common::insert_user(&app.db, &common::unique_email(), "abcd1234").await;
    sqlx::query(r#"UPDATE "User" SET "role" = 'superadmin' WHERE "id" = $1"#)
        .bind(&admin_id)
        .execute(&app.db)
        .await
        .unwrap();
    let name = format!("테스트-{}", cuid2::create_id());
    let id = create(&app, &token, json!({ "name": name })).await;
    let (title, link): (String, Option<String>) = sqlx::query_as(
        r#"SELECT "title", "link" FROM "Notification" WHERE "userId" = $1 AND "title" = $2"#,
    )
    .bind(&admin_id)
    .bind(format!("새 매물 등록: {name}"))
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(title, format!("새 매물 등록: {name}"));
    assert_eq!(link.unwrap(), format!("/properties/{id}/edit"));
}

#[tokio::test]
async fn unknown_keys_ignored_wrong_types_400() {
    let app = common::spawn_app().await;
    let (token, user_id) = common::signup(&app).await;
    let (status, v) = call(
        &app,
        Some(&token),
        Method::POST,
        "/properties",
        Some(&json!({ "bogus": 1, "name": "a", "id": "evil", "userId": "evil" })),
    )
    .await;
    assert_eq!(status, StatusCode::CREATED);
    assert_ne!(v["id"], "evil");
    let owner: String = sqlx::query_scalar(r#"SELECT "userId" FROM "Property" WHERE "id" = $1"#)
        .bind(v["id"].as_str().unwrap())
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(owner, user_id);

    for body in [
        json!({ "name": {} }),
        json!({ "isFavorite": null }),
        json!([1]),
    ] {
        let (status, v) = call(&app, Some(&token), Method::POST, "/properties", Some(&body)).await;
        assert_eq!(status, StatusCode::BAD_REQUEST, "{body}");
        assert_eq!(v["error"], "요청 형식이 올바르지 않습니다");
    }
    // 본문이 JSON 이 아님
    let resp = app
        .client
        .post(format!("{}/api/v1/properties", app.addr))
        .bearer_auth(&token)
        .header("content-type", "application/json")
        .body("{nope")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::BAD_REQUEST);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["error"], "요청 형식이 올바르지 않습니다");
}

#[tokio::test]
async fn created_at_is_utc_now() {
    let app = common::spawn_app().await;
    let (token, _) = common::signup(&app).await;
    let id = create(&app, &token, json!({ "name": "시각" })).await;
    let (_, p) = get(&app, &token, &id).await;
    let s = p["createdAt"].as_str().unwrap();
    assert!(s.ends_with('Z'));
    let at = chrono::DateTime::parse_from_rfc3339(s).unwrap();
    let diff = (chrono::Utc::now() - at.with_timezone(&chrono::Utc)).num_seconds();
    assert!(diff.abs() < 60, "{s}");
}
