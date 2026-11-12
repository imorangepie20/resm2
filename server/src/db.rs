use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;

/// 모든 연결의 세션 시간대를 UTC 로 고정한다. Prisma 는 시간대 없는 `TIMESTAMP(3)` 에 UTC 를 저장하므로,
/// `DEFAULT CURRENT_TIMESTAMP` 도 UTC 로 기록되게 해야 한다.
pub async fn connect(database_url: &str) -> sqlx::Result<PgPool> {
    PgPoolOptions::new()
        .max_connections(10)
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                sqlx::query("SET TIME ZONE 'UTC'").execute(conn).await?;
                Ok(())
            })
        })
        .connect(database_url)
        .await
}
