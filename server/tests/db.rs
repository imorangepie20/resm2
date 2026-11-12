mod common;

#[tokio::test]
async fn pool_sessions_are_utc() {
    let app = common::spawn_app().await;
    let mut conn = app.db.acquire().await.unwrap();
    let tz: String = sqlx::query_scalar("SELECT current_setting('TimeZone')")
        .fetch_one(&mut *conn)
        .await
        .unwrap();
    assert_eq!(tz, "UTC");
}

#[tokio::test]
async fn pool_session_timezone_survives_non_utc_database_default() {
    let app = common::spawn_app().await;
    let (default_ts, utc_ts): (chrono::NaiveDateTime, chrono::NaiveDateTime) = sqlx::query_as(
        "SELECT (CURRENT_TIMESTAMP)::timestamp(3), (now() at time zone 'utc')::timestamp(3)",
    )
    .fetch_one(&app.db)
    .await
    .unwrap();
    let diff = (default_ts - utc_ts).num_seconds().abs();
    assert!(diff < 2, "{default_ts} vs {utc_ts}");
}
