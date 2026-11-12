mod common;

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
