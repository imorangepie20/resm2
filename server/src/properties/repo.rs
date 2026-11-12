use sqlx::{AssertSqlSafe, PgPool, Postgres, QueryBuilder};

use super::model::{PropertyDto, SELECT_COLUMNS, Val};

/// 목록 필터. 알 수 없는 값이면 `None`.
pub fn view_filter(view: &str) -> Option<&'static str> {
    match view {
        "all" => Some(""),
        "favorites" => Some(r#" AND "isFavorite""#),
        "in-progress" => Some(r#" AND "status" = '계약진행'"#),
        "contracted" => Some(r#" AND "status" = '계약완료'"#),
        _ => None,
    }
}

pub async fn list(
    db: &PgPool,
    user_id: &str,
    filter: &'static str,
) -> sqlx::Result<Vec<PropertyDto>> {
    sqlx::query_as(AssertSqlSafe(format!(
        r#"SELECT {SELECT_COLUMNS} FROM "Property" WHERE "userId" = $1{filter}
           ORDER BY "createdAt" DESC, "id" DESC"#
    )))
    .bind(user_id)
    .fetch_all(db)
    .await
}

pub async fn get(db: &PgPool, user_id: &str, id: &str) -> sqlx::Result<Option<PropertyDto>> {
    sqlx::query_as(AssertSqlSafe(format!(
        r#"SELECT {SELECT_COLUMNS} FROM "Property" WHERE "id" = $1 AND "userId" = $2"#
    )))
    .bind(id)
    .bind(user_id)
    .fetch_optional(db)
    .await
}

fn push_val(qb: &mut QueryBuilder<Postgres>, val: Val) {
    match val {
        Val::Text(v) => qb.push_bind(v),
        Val::Float(v) => qb.push_bind(v),
        Val::Int(v) => qb.push_bind(v),
        Val::Bool(v) => qb.push_bind(v),
        Val::Big(v) => qb.push_bind(v),
    };
}

/// `changes` 의 컬럼명은 `model::FIELDS` 등 코드 안의 고정 목록에서만 온다.
pub async fn insert(
    db: &PgPool,
    user_id: &str,
    source: &str,
    changes: Vec<(&'static str, Val)>,
) -> sqlx::Result<String> {
    let id = cuid2::create_id();
    let mut qb =
        QueryBuilder::new(r#"INSERT INTO "Property" ("id", "userId", "source", "updatedAt""#);
    for (col, _) in &changes {
        qb.push(", \"").push(*col).push("\"");
    }
    qb.push(") VALUES (");
    qb.push_bind(id.clone());
    qb.push(", ").push_bind(user_id.to_string());
    qb.push(", ").push_bind(source.to_string());
    qb.push(", now() at time zone 'utc'");
    for (_, val) in changes {
        qb.push(", ");
        push_val(&mut qb, val);
    }
    qb.push(")");
    qb.build().execute(db).await?;
    Ok(id)
}

/// 본인 소유 행이 있는지.
pub async fn exists(db: &PgPool, user_id: &str, id: &str) -> sqlx::Result<bool> {
    sqlx::query_scalar(
        r#"SELECT EXISTS(SELECT 1 FROM "Property" WHERE "id" = $1 AND "userId" = $2)"#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_one(db)
    .await
}

/// `changes` 가 비어 있지 않아야 한다. 갱신된 행이 있으면 true.
pub async fn update(
    db: &PgPool,
    user_id: &str,
    id: &str,
    changes: Vec<(&'static str, Val)>,
) -> sqlx::Result<bool> {
    let mut qb =
        QueryBuilder::new(r#"UPDATE "Property" SET "updatedAt" = now() at time zone 'utc'"#);
    for (col, val) in changes {
        qb.push(", \"").push(col).push("\" = ");
        push_val(&mut qb, val);
    }
    qb.push(r#" WHERE "id" = "#).push_bind(id.to_string());
    qb.push(r#" AND "userId" = "#)
        .push_bind(user_id.to_string());
    Ok(qb.build().execute(db).await?.rows_affected() > 0)
}

pub async fn delete_many(db: &PgPool, user_id: &str, ids: &[String]) -> sqlx::Result<u64> {
    Ok(
        sqlx::query(r#"DELETE FROM "Property" WHERE "id" = ANY($1) AND "userId" = $2"#)
            .bind(ids)
            .bind(user_id)
            .execute(db)
            .await?
            .rows_affected(),
    )
}
