use crate::customers::model::normalize_customer_types;
use crate::error::ApiError;
use crate::parties::model::{customer_match_key, PartyRow};
use sqlx::PgPool;
use sqlx::Row;

pub async fn list_parties(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
) -> Result<Vec<PartyRow>, ApiError> {
    // 본인 매물인지 확인
    let owned = sqlx::query(r#"SELECT 1 FROM "Property" WHERE id = $1 AND "userId" = $2"#)
        .bind(property_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    if owned.is_none() {
        return Err(ApiError::NotFound);
    }

    let rows = sqlx::query(
        r#"
        SELECT pp.id, pp."customerId", pp.role,
               c.name, c.phone
        FROM "PropertyParty" pp
        JOIN "Customer" c ON pp."customerId" = c.id
        WHERE pp."propertyId" = $1
        ORDER BY
            CASE pp.role
                WHEN '매도인' THEN 1
                WHEN '매수인' THEN 2
                WHEN '임대인' THEN 3
                WHEN '임차인' THEN 4
                WHEN '중개' THEN 5
                ELSE 6
            END,
            pp."createdAt" ASC
        "#,
    )
    .bind(property_id)
    .fetch_all(pool)
    .await?;

    let results: Vec<PartyRow> = rows
        .into_iter()
        .map(|r| PartyRow {
            id: r.get("id"),
            customer_id: r.get("customerId"),
            name: r.get("name"),
            phone: r.get("phone"),
            role: r.get("role"),
        })
        .collect();

    Ok(results)
}

pub async fn add_party(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
    role: &str,
    name: &str,
    phone: Option<String>,
) -> Result<(String, String), ApiError> {
    // 본인 매물인지 확인
    let owned = sqlx::query(r#"SELECT 1 FROM "Property" WHERE id = $1 AND "userId" = $2"#)
        .bind(property_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    if owned.is_none() {
        return Err(ApiError::NotFound);
    }

    // 고객 매칭 키로 기존 고객 검색
    let match_key = customer_match_key(name, phone.as_deref().unwrap_or(""));
    let phone_clean = phone.as_deref().unwrap_or("");

    let existing = sqlx::query(
        r#"
        SELECT c.id, c.name, c.phone, c.types
        FROM "Customer" c
        WHERE c."userId" = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut customer_id = None;
    let mut customer_types: Option<Vec<String>> = None;

    for row in existing {
        let c_name: String = row.get("name");
        let c_phone: Option<String> = row.get("phone");
        let c_types: Vec<String> = row.get("types");
        let c_key = customer_match_key(&c_name, c_phone.as_deref().unwrap_or(""));
        if c_key == match_key {
            customer_id = Some(row.get("id"));
            customer_types = Some(c_types);
            break;
        }
    }

    let customer_id = if let Some(cid) = customer_id {
        // 기존 고객에 역할 추가
        if let Some(mut types) = customer_types {
            if !types.contains(&role.to_string()) {
                types.push(role.to_string());
                let normalized = normalize_customer_types(&types);
                sqlx::query(r#"UPDATE "Customer" SET types = $1 WHERE id = $2"#)
                    .bind(&normalized)
                    .bind(&cid)
                    .execute(pool)
                    .await?;
            }
        }
        cid
    } else {
        // 신규 고객 생성
        let new_id = cuid2::create_id();
        let types = normalize_customer_types(&[role.to_string()]);
        sqlx::query(
            r#"
            INSERT INTO "Customer" (id, "userId", types, name, phone, "createdAt", "updatedAt")
            VALUES ($1, $2, $3, $4, $5, now() at time zone 'utc', now() at time zone 'utc')
            "#,
        )
        .bind(&new_id)
        .bind(user_id)
        .bind(&types)
        .bind(name.trim())
        .bind(if phone_clean.is_empty() { None } else { Some(phone_clean.to_string()) })
        .execute(pool)
        .await?;
        new_id
    };

    // PropertyParty upsert
    let party_id = cuid2::create_id();
    sqlx::query(
        r#"
        INSERT INTO "PropertyParty" (id, "propertyId", "customerId", role)
        VALUES ($1, $2, $3, $4)
        ON CONFLICT ("propertyId", "customerId", role) DO NOTHING
        "#,
    )
    .bind(&party_id)
    .bind(property_id)
    .bind(&customer_id)
    .bind(role)
    .execute(pool)
    .await?;

    // upsert 후 실제 생성된 row의 id 조회 (이미 존재했다면 기존 id 반환)
    let row = sqlx::query(r#"SELECT id FROM "PropertyParty" WHERE "propertyId" = $1 AND "customerId" = $2 AND role = $3"#)
        .bind(property_id)
        .bind(&customer_id)
        .bind(role)
        .fetch_one(pool)
        .await?;
    let final_party_id: String = row.get("id");

    Ok((final_party_id, customer_id))
}

pub async fn remove_party(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
    party_id: &str,
) -> Result<(), ApiError> {
    // 본인 매물인지 확인
    let owned = sqlx::query(r#"SELECT 1 FROM "Property" WHERE id = $1 AND "userId" = $2"#)
        .bind(property_id)
        .bind(user_id)
        .fetch_optional(pool)
        .await?;
    if owned.is_none() {
        return Err(ApiError::NotFound);
    }

    let result = sqlx::query(
        r#"
        DELETE FROM "PropertyParty"
        WHERE id = $1 AND "propertyId" = $2
        "#,
    )
    .bind(party_id)
    .bind(property_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}