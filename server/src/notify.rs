use sqlx::PgPool;

/// `role = 'superadmin'` 인 모든 사용자에게 알림 행을 만든다. 대상이 없으면 아무것도 하지 않는다.
pub async fn notify_superadmins(
    db: &PgPool,
    kind: &str,
    title: &str,
    description: Option<&str>,
    link: Option<&str>,
) -> sqlx::Result<()> {
    let admin_ids: Vec<String> =
        sqlx::query_scalar(r#"SELECT "id" FROM "User" WHERE "role" = 'superadmin'"#)
            .fetch_all(db)
            .await?;
    if admin_ids.is_empty() {
        return Ok(());
    }
    let ids: Vec<String> = admin_ids.iter().map(|_| cuid2::create_id()).collect();
    sqlx::query(
        r#"INSERT INTO "Notification" ("id", "userId", "type", "title", "description", "link")
           SELECT t.id, t.user_id, $3, $4, $5, $6
           FROM UNNEST($1::text[], $2::text[]) AS t(id, user_id)"#,
    )
    .bind(&ids)
    .bind(&admin_ids)
    .bind(kind)
    .bind(title)
    .bind(description)
    .bind(link)
    .execute(db)
    .await?;
    Ok(())
}
