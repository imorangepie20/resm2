// Calendar Event repository (PostgreSQL via sqlx)

use crate::events::model::*;
use crate::error::ApiError;
use sqlx::{PgPool, Row};

/// 월 접두사(YYYYMM) 생성 - month는 0-indexed
fn month_prefix(year: i32, month: u32) -> String {
    format!("{year}{:02}", month + 1)
}

/// YYYYMMDD 날짜 키 생성
fn ymd(year: i32, month: u32, day: u32) -> String {
    format!("{year}{:02}{:02}", month + 1, day)
}

/// 매물 날짜 필드 (파생 일정용)
const PROPERTY_DATE_FIELDS: &[(&str, &str)] = &[
    ("contractHopeDate", "계약희망일"),
    ("contractDate", "계약일"),
    ("interim1Date", "중도금1"),
    ("interim2Date", "중도금2"),
    ("balanceDate", "잔금일"),
    ("moveInHopeDate", "입주희망일"),
    ("moveInDate", "입주일"),
];

// ─── Manual Events ────────────────────────────────────────────────

pub async fn list_events(
    pool: &PgPool,
    user_id: &str,
    year: i32,
    month: u32,
) -> Result<Vec<EventRow>, ApiError> {
    let prefix = month_prefix(year, month);
    let rows = sqlx::query(
        r#"
        SELECT
            e.id, e.title, e.date, e."startTime", e.category, e.memo,
            e."propertyId", p.name AS property_name, p."complexName" AS property_complex_name,
            e."customerId", c.name AS customer_name
        FROM "Event" e
        LEFT JOIN "Property" p ON e."propertyId" = p.id
        LEFT JOIN "Customer" c ON e."customerId" = c.id
        WHERE e."userId" = $1 AND e.date LIKE $2 || '%'
        ORDER BY e.date ASC, e."startTime" ASC NULLS FIRST
        "#,
    )
    .bind(user_id)
    .bind(prefix)
    .fetch_all(pool)
    .await?;

    let mut events = Vec::with_capacity(rows.len());
    for row in rows {
        events.push(EventRow {
            id: row.get("id"),
            title: row.get("title"),
            date: row.get("date"),
            start_time: row.get("startTime"),
            category: row.get("category"),
            memo: row.get("memo"),
            property_id: row.get("propertyId"),
            property_label: row
                .try_get::<Option<String>, _>("property_name")
                .ok()
                .flatten()
                .or_else(|| row.try_get::<Option<String>, _>("property_complex_name").ok().flatten()),
            customer_id: row.get("customerId"),
            customer_label: row.try_get::<Option<String>, _>("customer_name").ok().flatten(),
        });
    }
    Ok(events)
}

pub async fn create_event(
    pool: &PgPool,
    user_id: &str,
    input: CreateEventInput,
) -> Result<String, ApiError> {
    let id = cuid2::create_id();
    sqlx::query(
        r#"
        INSERT INTO "Event" (id, "userId", title, date, "startTime", category, memo, "propertyId", "customerId", "createdAt", "updatedAt")
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, now(), now())
        "#,
    )
    .bind(&id)
    .bind(user_id)
    .bind(&input.title)
    .bind(&input.date)
    .bind(input.start_time.as_deref().filter(|s| !s.is_empty()))
    .bind(&input.category)
    .bind(input.memo.as_deref().filter(|s| !s.is_empty()))
    .bind(input.property_id.as_deref().filter(|s| !s.is_empty()))
    .bind(input.customer_id.as_deref().filter(|s| !s.is_empty()))
    .execute(pool)
    .await?;
    Ok(id)
}

pub async fn update_event(
    pool: &PgPool,
    user_id: &str,
    event_id: &str,
    input: UpdateEventInput,
) -> Result<(), ApiError> {
    let result = sqlx::query(
        r#"
        UPDATE "Event" SET
            title = COALESCE($3, title),
            date = COALESCE($4, date),
            "startTime" = COALESCE($5, "startTime"),
            category = COALESCE($6, category),
            memo = COALESCE($7, memo),
            "propertyId" = COALESCE($8, "propertyId"),
            "customerId" = COALESCE($9, "customerId"),
            "updatedAt" = now()
        WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(event_id)
    .bind(user_id)
    .bind(input.title)
    .bind(input.date)
    .bind(input.start_time.as_deref().filter(|s| !s.is_empty()))
    .bind(input.category)
    .bind(input.memo.as_deref().filter(|s| !s.is_empty()))
    .bind(input.property_id.as_deref().filter(|s| !s.is_empty()))
    .bind(input.customer_id.as_deref().filter(|s| !s.is_empty()))
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

pub async fn delete_event(pool: &PgPool, user_id: &str, event_id: &str) -> Result<(), ApiError> {
    let result = sqlx::query(
        r#"
        DELETE FROM "Event" WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(event_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(ApiError::NotFound);
    }
    Ok(())
}

// ─── Property-derived events (read-only) ──────────────────────────

pub async fn list_property_dates(
    pool: &PgPool,
    user_id: &str,
    year: i32,
    month: u32,
) -> Result<Vec<PropertyDateEvent>, ApiError> {
    let prefix = month_prefix(year, month);

    let rows = sqlx::query(
        r#"
        SELECT id, name, "complexName",
            "contractHopeDate", "contractDate", "interim1Date", "interim2Date",
            "balanceDate", "moveInHopeDate", "moveInDate"
        FROM "Property"
        WHERE "userId" = $1
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut out = Vec::new();
    for row in rows {
        let name: Option<String> = row.get("name");
        let complex_name: Option<String> = row.get("complexName");
        let display_name = name.or(complex_name).unwrap_or_else(|| "매물".to_string());
        let property_id: String = row.get("id");

        for (col, label) in PROPERTY_DATE_FIELDS {
            let val: Option<String> = row.get(*col);
            if let Some(v) = val {
                if v.len() == 8 && v.chars().all(|c| c.is_ascii_digit()) && v.starts_with(&prefix) {
                    out.push(PropertyDateEvent {
                        date: v,
                        label: format!("{} {}", display_name, label),
                        property_id: property_id.clone(),
                        field: (*col).to_string(),
                    });
                }
            }
        }
    }
    Ok(out)
}

// ─── Dropdown options ─────────────────────────────────────────────

pub async fn list_property_options(pool: &PgPool, user_id: &str) -> Result<Vec<CalendarOption>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, "complexName" FROM "Property"
        WHERE "userId" = $1
        ORDER BY "createdAt" DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut opts = Vec::with_capacity(rows.len());
    for row in rows {
        let name: Option<String> = row.get("name");
        let complex_name: Option<String> = row.get("complexName");
        let label = name.or(complex_name).unwrap_or_else(|| "매물".to_string());
        opts.push(CalendarOption {
            id: row.get("id"),
            label,
        });
    }
    Ok(opts)
}

pub async fn list_customer_options(pool: &PgPool, user_id: &str) -> Result<Vec<CalendarOption>, ApiError> {
    let rows = sqlx::query(
        r#"
        SELECT id, name, phone FROM "Customer"
        WHERE "userId" = $1
        ORDER BY "createdAt" DESC
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut opts = Vec::with_capacity(rows.len());
    for row in rows {
        let name: String = row.get("name");
        let phone: Option<String> = row.get("phone");
        let label = if let Some(p) = phone.filter(|s| !s.is_empty()) {
            format!("{} ({})", name, p)
        } else {
            name
        };
        opts.push(CalendarOption {
            id: row.get("id"),
            label,
        });
    }
    Ok(opts)
}

// ─── Combined calendar load ───────────────────────────────────────

pub async fn load_calendar(
    pool: &PgPool,
    user_id: &str,
    year: i32,
    month: u32,
) -> Result<CalendarResponse, ApiError> {
    let events = list_events(pool, user_id, year, month).await?;
    let property_dates = list_property_dates(pool, user_id, year, month).await?;
    let properties = list_property_options(pool, user_id).await?;
    let customers = list_customer_options(pool, user_id).await?;

    Ok(CalendarResponse {
        events,
        property_dates,
        properties,
        customers,
    })
}