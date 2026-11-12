#![allow(dead_code)]

use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

use resm_server::{AppState, Config, build_router};
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

pub struct TestApp {
    pub addr: String,
    pub db: PgPool,
    pub client: reqwest::Client,
}

pub async fn spawn_app() -> TestApp {
    let _ = dotenvy::from_path(concat!(env!("CARGO_MANIFEST_DIR"), "/../.env"));
    let url = std::env::var("TEST_DATABASE_URL").expect("TEST_DATABASE_URL 이 필요합니다");
    let db = PgPoolOptions::new()
        .max_connections(5)
        .connect(&url)
        .await
        .expect("테스트 DB 접속 실패");

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = format!("http://{}", listener.local_addr().unwrap());
    let cfg = Config {
        database_url: url,
        bind: listener.local_addr().unwrap(),
        allowed_origins: vec![],
        cookie_secure: false,
        static_dir: None,
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
