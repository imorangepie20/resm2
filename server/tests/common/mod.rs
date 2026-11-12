#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use resm_server::{AppState, Config, build_router};
use sqlx::PgPool;

pub struct TestApp {
    pub addr: String,
    pub db: PgPool,
    pub client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    spawn_app_with_static(None).await
}

pub async fn spawn_app_with_static(static_dir: Option<std::path::PathBuf>) -> TestApp {
    let _ = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"));
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL 이 필요합니다");
    let db = resm_server::db::connect(&url)
        .await
        .expect("테스트 DB 접속 실패");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    let cfg = Config {
        database_url: url,
        bind: listener.local_addr().unwrap(),
        allowed_origins: vec!["https://tauri.localhost".to_string()],
        cookie_secure: false,
        static_dir,
    };
    let state = AppState {
        db: db.clone(),
        cfg: Arc::new(cfg),
    };
    tokio::spawn(async move {
        axum::serve(listener, build_router(state)).await.unwrap();
    });

    TestApp {
        addr,
        db,
        client: reqwest::Client::new(),
    }
}

pub fn unique_email() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let n = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("test-{}-{nanos}-{n}@example.com", std::process::id())
}

pub async fn insert_user(db: &PgPool, email: &str, pw: &str) -> String {
    let agency_id = cuid2::create_id();
    sqlx::query(r#"INSERT INTO "Agency" ("id", "name") VALUES ($1, $2)"#)
        .bind(&agency_id)
        .bind("테스트 중개사무소")
        .execute(db)
        .await
        .unwrap();
    let user_id = cuid2::create_id();
    sqlx::query(
        r#"INSERT INTO "User" ("id", "agencyId", "email", "passwordHash", "role")
           VALUES ($1, $2, $3, $4, 'member')"#,
    )
    .bind(&user_id)
    .bind(&agency_id)
    .bind(email)
    .bind(resm_server::auth::password::hash(pw).unwrap())
    .execute(db)
    .await
    .unwrap();
    user_id
}

/// 새 사용자(+사무소)와 세션을 DB에 직접 만든다(bcrypt 회피). `(bearer 토큰, user_id)` 반환.
pub async fn signup(app: &TestApp) -> (String, String) {
    let agency_id = cuid2::create_id();
    sqlx::query(r#"INSERT INTO "Agency" ("id", "name") VALUES ($1, $2)"#)
        .bind(&agency_id)
        .bind("테스트 중개사무소")
        .execute(&app.db)
        .await
        .unwrap();
    let user_id = cuid2::create_id();
    sqlx::query(
        r#"INSERT INTO "User" ("id", "agencyId", "email", "passwordHash", "role")
           VALUES ($1, $2, $3, 'x', 'member')"#,
    )
    .bind(&user_id)
    .bind(&agency_id)
    .bind(unique_email())
    .execute(&app.db)
    .await
    .unwrap();
    let (token, _) = resm_server::auth::session::create(&app.db, &user_id)
        .await
        .unwrap();
    (token, user_id)
}
