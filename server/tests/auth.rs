mod common;

use reqwest::header::{COOKIE, SET_COOKIE};
use serde_json::{Value, json};

#[tokio::test]
async fn session_row_uses_hashed_token_and_30_day_expiry() {
    let app = common::spawn_app().await;
    let uid = common::insert_user(&app.db, &common::unique_email(), "abcd1234").await;
    let (token, expires) = resm_server::auth::session::create(&app.db, &uid)
        .await
        .unwrap();
    let hash: String =
        sqlx::query_scalar(r#"SELECT "tokenHash" FROM "Session" WHERE "userId" = $1"#)
            .bind(&uid)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert_eq!(hash, resm_server::auth::session::hash_token(&token));
    let days = (expires - chrono::Utc::now().naive_utc()).num_days();
    assert!((29..=30).contains(&days));
}

fn signup_body(email: &str, name: &str) -> Value {
    json!({
        "agencyName": "테스트 부동산",
        "agencyZipcode": "  06236 ",
        "agencyAddress": "",
        "agencyPhone": null,
        "name": name,
        "phone": " 010-1234-5678 ",
        "email": email,
        "password": "abcd1234",
        "passwordConfirm": "abcd1234",
        "agree": true
    })
}

async fn post_json(app: &common::TestApp, path: &str, body: &Value) -> reqwest::Response {
    app.client
        .post(format!("{}/api/v1{path}", app.addr))
        .json(body)
        .send()
        .await
        .unwrap()
}

async fn me(app: &common::TestApp, token: &str) -> reqwest::Response {
    app.client
        .get(format!("{}/api/v1/auth/me", app.addr))
        .bearer_auth(token)
        .send()
        .await
        .unwrap()
}

fn session_cookie_pair(resp: &reqwest::Response) -> String {
    let raw = resp
        .headers()
        .get(SET_COOKIE)
        .expect("Set-Cookie 없음")
        .to_str()
        .unwrap();
    raw.split(';').next().unwrap().to_string()
}

async fn ensure_superadmin(db: &sqlx::PgPool) {
    let count: i64 =
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM "User" WHERE "role" = 'superadmin'"#)
            .fetch_one(db)
            .await
            .unwrap();
    if count == 0 {
        let uid = common::insert_user(db, &common::unique_email(), "abcd1234").await;
        sqlx::query(r#"UPDATE "User" SET "role" = 'superadmin' WHERE "id" = $1"#)
            .bind(uid)
            .execute(db)
            .await
            .unwrap();
    }
}

#[tokio::test]
async fn signup_then_me_with_bearer() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    let resp = post_json(&app, "/auth/signup", &signup_body(&email, "홍길동")).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["user"]["email"], email);
    assert_eq!(body["user"]["name"], "홍길동");
    let token = body["token"].as_str().unwrap();

    let resp = me(&app, token).await;
    assert_eq!(resp.status(), 200);
    let user: Value = resp.json().await.unwrap();
    assert_eq!(user["email"], email);
    assert_eq!(user["id"], body["user"]["id"]);
    assert_eq!(user["agencyId"], body["user"]["agencyId"]);

    // 선택 필드: trim, 빈 문자열/null 은 NULL
    type Cols = (
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
    );
    let (zip, addr, phone, uphone): Cols = sqlx::query_as(
        r#"SELECT a."zipcode", a."address", a."phone", u."phone"
           FROM "User" u JOIN "Agency" a ON a."id" = u."agencyId" WHERE u."email" = $1"#,
    )
    .bind(&email)
    .fetch_one(&app.db)
    .await
    .unwrap();
    assert_eq!(zip.as_deref(), Some("06236"));
    assert_eq!(addr, None);
    assert_eq!(phone, None);
    assert_eq!(uphone.as_deref(), Some("010-1234-5678"));
}

#[tokio::test]
async fn signup_sets_session_cookie() {
    let app = common::spawn_app().await;
    let resp = post_json(
        &app,
        "/auth/signup",
        &signup_body(&common::unique_email(), "쿠키"),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let raw = resp.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
    assert!(raw.starts_with("session="), "{raw}");
    assert!(raw.contains("HttpOnly"), "{raw}");
    assert!(raw.contains("SameSite=Lax"), "{raw}");
    assert!(raw.contains("Path=/"), "{raw}");
    assert!(raw.contains("Expires="), "{raw}");
    assert!(!raw.contains("Secure"), "{raw}");
}

#[tokio::test]
async fn login_accepts_cookie_auth() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    common::insert_user(&app.db, &email, "abcd1234").await;
    let resp = post_json(
        &app,
        "/auth/login",
        &json!({"email": email, "password": "abcd1234"}),
    )
    .await;
    assert_eq!(resp.status(), 200);
    let cookie = session_cookie_pair(&resp);
    let body: Value = resp.json().await.unwrap();
    assert!(body["token"].is_string());
    assert_eq!(body["user"]["role"], "member");

    let resp = app
        .client
        .get(format!("{}/api/v1/auth/me", app.addr))
        .header(COOKIE, cookie)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let user: Value = resp.json().await.unwrap();
    assert_eq!(user["email"], email);
}

#[tokio::test]
async fn login_wrong_password_401_message() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    common::insert_user(&app.db, &email, "abcd1234").await;
    for (e, p) in [
        (email.as_str(), "wrong-pass"),
        ("nobody@example.com", "abcd1234"),
    ] {
        let resp = post_json(&app, "/auth/login", &json!({"email": e, "password": p})).await;
        assert_eq!(resp.status(), 401);
        let body: Value = resp.json().await.unwrap();
        assert_eq!(body["error"], "이메일 또는 비밀번호가 올바르지 않습니다");
    }
}

#[tokio::test]
async fn signup_validation_messages() {
    let app = common::spawn_app().await;
    type Mutate = Box<dyn Fn(&mut Value)>;
    let cases: Vec<(Mutate, &str)> = vec![
        (
            Box::new(|b| b["agencyName"] = json!("")),
            "상호명을 입력하세요",
        ),
        (Box::new(|b| b["name"] = json!("   ")), "이름을 입력하세요"),
        (
            Box::new(|b| b["email"] = json!("not-an-email")),
            "올바른 이메일을 입력하세요",
        ),
        (
            Box::new(|b| b["password"] = json!("short")),
            "비밀번호는 8자 이상이어야 합니다",
        ),
        (
            Box::new(|b| b["passwordConfirm"] = json!("different1")),
            "비밀번호가 일치하지 않습니다",
        ),
        (
            Box::new(|b| b["agree"] = json!(false)),
            "이용약관에 동의해주세요",
        ),
    ];
    for (mutate, expected) in cases {
        let mut body = signup_body(&common::unique_email(), "이름");
        mutate(&mut body);
        let resp = post_json(&app, "/auth/signup", &body).await;
        assert_eq!(resp.status(), 400, "{expected}");
        let v: Value = resp.json().await.unwrap();
        assert_eq!(v["error"], expected);
    }

    // 먼저 실패하는 규칙의 문구가 반환된다.
    let mut body = signup_body("bad", "이름");
    body["agencyName"] = json!("");
    body["agree"] = json!(false);
    let v: Value = post_json(&app, "/auth/signup", &body)
        .await
        .json()
        .await
        .unwrap();
    assert_eq!(v["error"], "상호명을 입력하세요");
}

#[tokio::test]
async fn duplicate_email_is_400_not_500() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    let body = signup_body(&email, "동시가입");
    let (a, b) = tokio::join!(
        post_json(&app, "/auth/signup", &body),
        post_json(&app, "/auth/signup", &body)
    );
    let mut statuses = [a.status().as_u16(), b.status().as_u16()];
    statuses.sort();
    assert_eq!(statuses, [200, 400]);
    let loser = if a.status() == 400 { a } else { b };
    let v: Value = loser.json().await.unwrap();
    assert_eq!(v["error"], "이미 가입된 이메일입니다");

    // 순차 중복도 같은 400
    let resp = post_json(&app, "/auth/signup", &body).await;
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn non_first_signup_is_member_and_notifies() {
    let app = common::spawn_app().await;
    ensure_superadmin(&app.db).await;
    let name = format!("알림{}", common::unique_email());
    let title = format!("새 회원 가입: {name}");
    let email = common::unique_email();
    let resp = post_json(&app, "/auth/signup", &signup_body(&email, &name)).await;
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert_eq!(body["user"]["role"], "member");

    let admins: i64 =
        sqlx::query_scalar(r#"SELECT COUNT(*) FROM "User" WHERE "role" = 'superadmin'"#)
            .fetch_one(&app.db)
            .await
            .unwrap();
    assert!(admins >= 1);
    let rows: Vec<(String, Option<String>, Option<String>, bool)> = sqlx::query_as(
        r#"SELECT "type", "description", "link", "read" FROM "Notification" WHERE "title" = $1"#,
    )
    .bind(&title)
    .fetch_all(&app.db)
    .await
    .unwrap();
    assert_eq!(rows.len() as i64, admins);
    for (kind, desc, link, read) in &rows {
        assert_eq!(kind, "member");
        assert_eq!(
            desc.as_deref(),
            Some(format!("{email} · 테스트 부동산").as_str())
        );
        assert_eq!(link.as_deref(), Some("/settings/members"));
        assert!(!read);
    }
}

#[tokio::test]
async fn logout_invalidates_token() {
    let app = common::spawn_app().await;
    let resp = post_json(
        &app,
        "/auth/signup",
        &signup_body(&common::unique_email(), "로그아웃"),
    )
    .await;
    let body: Value = resp.json().await.unwrap();
    let token = body["token"].as_str().unwrap();
    assert_eq!(me(&app, token).await.status(), 200);

    let resp = app
        .client
        .post(format!("{}/api/v1/auth/logout", app.addr))
        .bearer_auth(token)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
    let raw = resp.headers().get(SET_COOKIE).unwrap().to_str().unwrap();
    assert!(raw.starts_with("session=;"), "{raw}");
    assert!(raw.contains("Max-Age=0"), "{raw}");
    assert_eq!(me(&app, token).await.status(), 401);
}

#[tokio::test]
async fn expired_session_is_401_and_row_deleted() {
    let app = common::spawn_app().await;
    let uid = common::insert_user(&app.db, &common::unique_email(), "abcd1234").await;
    let (token, _) = resm_server::auth::session::create(&app.db, &uid)
        .await
        .unwrap();
    sqlx::query(r#"UPDATE "Session" SET "expiresAt" = $1 WHERE "userId" = $2"#)
        .bind(chrono::Utc::now().naive_utc() - chrono::Duration::hours(1))
        .bind(&uid)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(me(&app, &token).await.status(), 401);
    let n: i64 = sqlx::query_scalar(r#"SELECT COUNT(*) FROM "Session" WHERE "userId" = $1"#)
        .bind(&uid)
        .fetch_one(&app.db)
        .await
        .unwrap();
    assert_eq!(n, 0);
}

#[tokio::test]
async fn session_of_deleted_user_is_401() {
    let app = common::spawn_app().await;
    let uid = common::insert_user(&app.db, &common::unique_email(), "abcd1234").await;
    let (token, _) = resm_server::auth::session::create(&app.db, &uid)
        .await
        .unwrap();
    assert_eq!(me(&app, &token).await.status(), 200);
    sqlx::query(r#"DELETE FROM "User" WHERE "id" = $1"#)
        .bind(&uid)
        .execute(&app.db)
        .await
        .unwrap();
    assert_eq!(me(&app, &token).await.status(), 401);
}

#[tokio::test]
async fn garbage_bearer_is_401() {
    let app = common::spawn_app().await;
    assert_eq!(me(&app, "xxx").await.status(), 401);
}

#[tokio::test]
async fn cross_origin_cookie_post_is_403() {
    let app = common::spawn_app().await;
    let resp = post_json(
        &app,
        "/auth/signup",
        &signup_body(&common::unique_email(), "오리진"),
    )
    .await;
    let cookie = session_cookie_pair(&resp);
    let url = format!("{}/api/v1/auth/logout", app.addr);

    let resp = app
        .client
        .post(&url)
        .header(COOKIE, &cookie)
        .header("Origin", "https://evil.example")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["error"], "권한이 없습니다");

    // 같은 출처(Host 일치)는 통과
    let host = app.addr.trim_start_matches("http://");
    let resp = app
        .client
        .post(&url)
        .header(COOKIE, &cookie)
        .header("Origin", format!("http://{host}"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn bearer_post_with_foreign_origin_is_allowed() {
    let app = common::spawn_app().await;
    let resp = post_json(
        &app,
        "/auth/signup",
        &signup_body(&common::unique_email(), "베어러"),
    )
    .await;
    let body: Value = resp.json().await.unwrap();
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/logout", app.addr))
        .bearer_auth(body["token"].as_str().unwrap())
        .header("Origin", "https://tauri.localhost")
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 204);
}

#[tokio::test]
async fn preflight_allows_configured_origin() {
    let app = common::spawn_app().await;
    let url = format!("{}/api/v1/auth/login", app.addr);
    let resp = app
        .client
        .request(reqwest::Method::OPTIONS, &url)
        .header("Origin", "https://tauri.localhost")
        .header("Access-Control-Request-Method", "POST")
        .header(
            "Access-Control-Request-Headers",
            "content-type,authorization",
        )
        .send()
        .await
        .unwrap();
    assert!(resp.status().is_success(), "{}", resp.status());
    assert_eq!(
        resp.headers()["access-control-allow-origin"],
        "https://tauri.localhost"
    );
    assert!(
        resp.headers()
            .get("access-control-allow-credentials")
            .is_none()
    );

    let resp = app
        .client
        .request(reqwest::Method::OPTIONS, &url)
        .header("Origin", "https://evil.example")
        .header("Access-Control-Request-Method", "POST")
        .send()
        .await
        .unwrap();
    assert!(resp.headers().get("access-control-allow-origin").is_none());
}

#[tokio::test]
async fn login_from_allowed_origin_without_bearer_is_200() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    common::insert_user(&app.db, &email, "abcd1234").await;
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/login", app.addr))
        .header("Origin", "https://tauri.localhost")
        .json(&json!({"email": email, "password": "abcd1234"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["token"].is_string());
}

#[tokio::test]
async fn signup_from_allowed_origin_without_bearer_is_200() {
    let app = common::spawn_app().await;
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/signup", app.addr))
        .header("Origin", "https://tauri.localhost")
        .json(&signup_body(&common::unique_email(), "허용출처"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();
    assert!(body["token"].is_string());
}

#[tokio::test]
async fn post_from_unlisted_origin_without_bearer_is_403() {
    let app = common::spawn_app().await;
    let email = common::unique_email();
    common::insert_user(&app.db, &email, "abcd1234").await;
    let resp = app
        .client
        .post(format!("{}/api/v1/auth/login", app.addr))
        .header("Origin", "https://evil.example")
        .json(&json!({"email": email, "password": "abcd1234"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 403);
    let v: Value = resp.json().await.unwrap();
    assert_eq!(v["error"], "권한이 없습니다");
}
