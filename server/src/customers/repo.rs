use crate::contract::validate::owner_role_from_trade;
use crate::customers::model::{CustomerData, CustomerDraft, CustomerRow, PartyRow};
use crate::error::ApiError;
use sqlx::PgPool;
use sqlx::Row;
use sqlx::QueryBuilder;

pub async fn list_customers(
    pool: &PgPool,
    user_id: &str,
    q: Option<&str>,
    type_: Option<&str>,
) -> Result<Vec<CustomerRow>, ApiError> {
    let mut qb: QueryBuilder<sqlx::Postgres> = QueryBuilder::new(
        r#"
        SELECT c.id, c.types, c.name, c.zipcode, c.phone, c.address, c.email, c.gender, c.memo,
               c."propertyId", c."updatedAt",
               p.name as property_name, p."complexName" as complex_name
        FROM "Customer" c
        LEFT JOIN "Property" p ON c."propertyId" = p.id
        WHERE c."userId" = "#,
    );
    qb.push_bind(user_id);

    if let Some(q) = q {
        qb.push(" AND (c.name ILIKE ");
        let like = format!("%{}%", q);
        qb.push_bind(like.clone());
        qb.push(" OR c.phone ILIKE ");
        qb.push_bind(like);
        qb.push(")");
    }
    if let Some(t) = type_ {
        qb.push(" AND ");
        qb.push_bind(t);
        qb.push(" = ANY(c.types)");
    }
    qb.push(" ORDER BY c.\"updatedAt\" DESC");

    let rows = qb.build().fetch_all(pool).await?;

    let mut results = Vec::new();
    for row in rows {
        let customer_id: String = row.get("id");
        let types: Vec<String> = row.get("types");
        let property_id: Option<String> = row.get("propertyId");
        let property_name: Option<String> = row.get("property_name");
        let complex_name: Option<String> = row.get("complex_name");
        let prop_name = property_name.or(complex_name);

        // 당사자 조회
        let parties = sqlx::query(
            r#"
            SELECT pp."propertyId", pp.role,
                   prop.name as property_name, prop."complexName" as complex_name
            FROM "PropertyParty" pp
            JOIN "Property" prop ON pp."propertyId" = prop.id
            WHERE pp."customerId" = $1
            ORDER BY pp."createdAt" ASC
            "#,
        )
        .bind(&customer_id)
        .fetch_all(pool)
        .await?;

        let party_rows: Vec<PartyRow> = parties
            .into_iter()
            .map(|p| {
                let prop_id: String = p.get("propertyId");
                let role: String = p.get("role");
                let pname: Option<String> = p.get("property_name");
                let cname: Option<String> = p.get("complex_name");
                PartyRow {
                    property_id: prop_id,
                    property_label: pname.or(cname).unwrap_or_else(|| "매물".into()),
                    role,
                }
            })
            .collect();

        let updated_at: chrono::NaiveDateTime = row.get("updatedAt");
        results.push(CustomerRow {
            id: customer_id,
            types,
            name: row.get("name"),
            zipcode: row.get("zipcode"),
            phone: row.get("phone"),
            address: row.get("address"),
            email: row.get("email"),
            gender: row.get("gender"),
            memo: row.get("memo"),
            property_id,
            property_name: prop_name,
            parties: party_rows,
            updated_at: updated_at.format("%Y-%m-%d").to_string(),
        });
    }

    Ok(results)
}

pub async fn get_customer(
    pool: &PgPool,
    user_id: &str,
    id: &str,
) -> Result<Option<CustomerRow>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT c.id, c.types, c.name, c.zipcode, c.phone, c.address, c.email, c.gender, c.memo,
               c."propertyId", c."updatedAt",
               p.name as property_name, p."complexName" as complex_name
        FROM "Customer" c
        LEFT JOIN "Property" p ON c."propertyId" = p.id
        WHERE c.id = $1 AND c."userId" = $2
        "#,
    )
    .bind(id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let customer_id: String = row.get("id");
        let types: Vec<String> = row.get("types");
        let property_id: Option<String> = row.get("propertyId");
        let property_name: Option<String> = row.get("property_name");
        let complex_name: Option<String> = row.get("complex_name");
        let prop_name = property_name.or(complex_name);

        let parties = sqlx::query(
            r#"
            SELECT pp."propertyId", pp.role,
                   prop.name as property_name, prop."complexName" as complex_name
            FROM "PropertyParty" pp
            JOIN "Property" prop ON pp."propertyId" = prop.id
            WHERE pp."customerId" = $1
            ORDER BY pp."createdAt" ASC
            "#,
        )
        .bind(&customer_id)
        .fetch_all(pool)
        .await?;

        let party_rows: Vec<PartyRow> = parties
            .into_iter()
            .map(|p| {
                let prop_id: String = p.get("propertyId");
                let role: String = p.get("role");
                let pname: Option<String> = p.get("property_name");
                let cname: Option<String> = p.get("complex_name");
                PartyRow {
                    property_id: prop_id,
                    property_label: pname.or(cname).unwrap_or_else(|| "매물".into()),
                    role,
                }
            })
            .collect();

        let updated_at: chrono::NaiveDateTime = row.get("updatedAt");
        Ok(Some(CustomerRow {
            id: customer_id,
            types,
            name: row.get("name"),
            zipcode: row.get("zipcode"),
            phone: row.get("phone"),
            address: row.get("address"),
            email: row.get("email"),
            gender: row.get("gender"),
            memo: row.get("memo"),
            property_id,
            property_name: prop_name,
            parties: party_rows,
            updated_at: updated_at.format("%Y-%m-%d").to_string(),
        }))
    } else {
        Ok(None)
    }
}

pub async fn owned_property_id(
    pool: &PgPool,
    user_id: &str,
    property_id: Option<&str>,
) -> Result<Option<String>, ApiError> {
    if let Some(pid) = property_id {
        let row = sqlx::query(r#"SELECT id FROM "Property" WHERE id = $1 AND "userId" = $2"#)
            .bind(pid)
            .bind(user_id)
            .fetch_optional(pool)
            .await?;
        Ok(row.map(|r| r.get("id")))
    } else {
        Ok(None)
    }
}

pub async fn create_customer(
    pool: &PgPool,
    user_id: &str,
    data: CustomerData,
    property_id: Option<String>,
) -> Result<String, ApiError> {
    let property_id = owned_property_id(pool, user_id, property_id.as_deref()).await?;
    let customer_id = cuid2::create_id();

    sqlx::query(
        r#"
        INSERT INTO "Customer" (id, "userId", "propertyId", types, name, zipcode, phone, address, email, gender, memo)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
        "#,
    )
    .bind(&customer_id)
    .bind(user_id)
    .bind(&property_id)
    .bind(&data.types)
    .bind(&data.name)
    .bind(&data.zipcode)
    .bind(&data.phone)
    .bind(&data.address)
    .bind(&data.email)
    .bind(&data.gender)
    .bind(&data.memo)
    .execute(pool)
    .await?;

    if let Some(pid) = &property_id {
        let prop = sqlx::query(r#"SELECT "tradeType" FROM "Property" WHERE id = $1"#)
            .bind(pid)
            .fetch_one(pool)
            .await?;
        let trade_type: String = prop.get("tradeType");
        let role = owner_role_from_trade(Some(&trade_type));

        sqlx::query(
            r#"
            INSERT INTO "PropertyParty" ("propertyId", "customerId", role)
            VALUES ($1, $2, $3)
            ON CONFLICT ("propertyId", "customerId", role) DO NOTHING
            "#,
        )
        .bind(pid)
        .bind(&customer_id)
        .bind(&role)
        .execute(pool)
        .await?;
    }

    Ok(customer_id)
}

pub async fn update_customer(
    pool: &PgPool,
    user_id: &str,
    id: &str,
    data: CustomerData,
    property_id: Option<String>,
) -> Result<(), ApiError> {
    let property_id = owned_property_id(pool, user_id, property_id.as_deref()).await?;

    let result = sqlx::query(
        r#"
        UPDATE "Customer"
        SET types = $1, name = $2, zipcode = $3, phone = $4, address = $5, email = $6, gender = $7, memo = $8,
            "propertyId" = $9, "updatedAt" = now() at time zone 'utc'
        WHERE id = $10 AND "userId" = $11
        "#,
    )
    .bind(&data.types)
    .bind(&data.name)
    .bind(&data.zipcode)
    .bind(&data.phone)
    .bind(&data.address)
    .bind(&data.email)
    .bind(&data.gender)
    .bind(&data.memo)
    .bind(&property_id)
    .bind(id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn delete_customer(pool: &PgPool, user_id: &str, id: &str) -> Result<(), ApiError> {
    let result = sqlx::query(r#"DELETE FROM "Customer" WHERE id = $1 AND "userId" = $2"#)
        .bind(id)
        .bind(user_id)
        .execute(pool)
        .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn customer_draft_from_property(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
) -> Result<Option<CustomerDraft>, ApiError> {
    let row = sqlx::query(
        r#"
        SELECT id, name, "complexName", "customerName", "customerPhone"
        FROM "Property"
        WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?;

    if let Some(row) = row {
        let id: String = row.get("id");
        let name: Option<String> = row.get("name");
        let complex_name: Option<String> = row.get("complexName");
        let customer_name: Option<String> = row.get("customerName");
        let customer_phone: Option<String> = row.get("customerPhone");

        Ok(Some(CustomerDraft {
            name: customer_name.unwrap_or_default(),
            phone: customer_phone.unwrap_or_default(),
            property_id: id,
            property_label: name.or(complex_name).unwrap_or_else(|| "매물".into()),
        }))
    } else {
        Ok(None)
    }
}