use crate::contract::model::PropertyDto;
use crate::error::ApiResult;
use serde_json::{Map, Value};
use sqlx::{PgPool, Row};

// ─── 매물 조회 (소유자만) ──────────────────────────────────────
pub async fn find_property_for_contract(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
) -> ApiResult<PropertyDto> {
    let row = sqlx::query(
        r#"
        SELECT
            id, "isFavorite" as is_favorite, "colorTag" as color_tag, status, source, "createdAt" as created_at,
            name, "articleNo" as article_no, "complexName" as complex_name, address,
            "realEstateType" as real_estate_type, "tradeType" as trade_type, heating,
            "customerName" as customer_name, "customerPhone" as customer_phone,
            "partnerName" as partner_name, "partnerPhone" as partner_phone,
            "partnerManager" as partner_manager, manager,
            "contractHopeDate" as contract_hope_date, "contractDate" as contract_date,
            "moveInHopeDate" as move_in_hope_date, "moveInDate" as move_in_date,
            "interim1Date" as interim1_date, "interim2Date" as interim2_date,
            "balanceDate" as balance_date, "approvalDate" as approval_date,
            note, memo,
            "siteArea" as site_area, "areaExclusive" as area_exclusive, "areaSupply" as area_supply,
            "landArea" as land_area, "buildingArea" as building_area, area,
            "totalHouseholds" as total_households, "parkingCount" as parking_count,
            "isPreSale" as is_pre_sale,
            price::text, "rentPrice"::text as rent_price, "dealAmount"::text as deal_amount,
            "contractChecklist"::text as contract_checklist
        FROM "Property"
        WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(crate::error::ApiError::NotFound)?;

    let contract_checklist: Option<String> = row.try_get("contract_checklist").ok().flatten();
    let contract_checklist = contract_checklist
        .and_then(|s| serde_json::from_str(&s).ok());

    Ok(PropertyDto {
        id: row.get("id"),
        is_favorite: row.get("is_favorite"),
        color_tag: row.get("color_tag"),
        status: row.get("status"),
        source: row.get("source"),
        created_at: row.get("created_at"),
        name: row.get("name"),
        article_no: row.get("article_no"),
        complex_name: row.get("complex_name"),
        address: row.get("address"),
        real_estate_type: row.get("real_estate_type"),
        trade_type: row.get("trade_type"),
        heating: row.get("heating"),
        customer_name: row.get("customer_name"),
        customer_phone: row.get("customer_phone"),
        partner_name: row.get("partner_name"),
        partner_phone: row.get("partner_phone"),
        partner_manager: row.get("partner_manager"),
        manager: row.get("manager"),
        contract_hope_date: row.get("contract_hope_date"),
        contract_date: row.get("contract_date"),
        move_in_hope_date: row.get("move_in_hope_date"),
        move_in_date: row.get("move_in_date"),
        interim1_date: row.get("interim1_date"),
        interim2_date: row.get("interim2_date"),
        balance_date: row.get("balance_date"),
        approval_date: row.get("approval_date"),
        note: row.get("note"),
        memo: row.get("memo"),
        site_area: row.get("site_area"),
        area_exclusive: row.get("area_exclusive"),
        area_supply: row.get("area_supply"),
        land_area: row.get("land_area"),
        building_area: row.get("building_area"),
        area: row.get("area"),
        total_households: row.get("total_households"),
        parking_count: row.get("parking_count"),
        is_pre_sale: row.get("is_pre_sale"),
        price: row.get("price"),
        rent_price: row.get("rent_price"),
        deal_amount: row.get("deal_amount"),
        contract_checklist,
    })
}

// ─── 체크리스트 토글 (JSONB 업데이트) ──────────────────────────
pub async fn toggle_checklist_item(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
    item_id: &str,
    checked: bool,
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;

    // 소유자 확인 + 현재 contractChecklist 조회
    let row = sqlx::query(
        r#"
        SELECT "contractChecklist"::text FROM "Property"
        WHERE id = $1 AND "userId" = $2
        FOR UPDATE
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::error::ApiError::NotFound)?;

    let checklist_text: Option<String> = row.try_get("contractChecklist").ok().flatten();
    let mut checklist: Map<String, Value> = checklist_text
        .and_then(|s| serde_json::from_str(&s).ok())
        .and_then(|v: Value| v.as_object().cloned())
        .unwrap_or_default();

    if checked {
        checklist.insert(
            item_id.to_string(),
            Value::String(chrono::Utc::now().to_rfc3339()),
        );
    } else {
        checklist.remove(item_id);
    }

    let checklist_json = serde_json::to_string(&Value::Object(checklist))?;

    sqlx::query(
        r#"
        UPDATE "Property"
        SET "contractChecklist" = $1::jsonb, "updatedAt" = now() at time zone 'utc'
        WHERE id = $2 AND "userId" = $3
        "#,
    )
    .bind(checklist_json)
    .bind(property_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ─── 체크리스트 전체 체크/해제 ────────────────────────────────
pub async fn set_all_checklist(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
    checked: bool,
    valid_item_ids: &[String],
) -> ApiResult<()> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        SELECT "contractChecklist"::text FROM "Property"
        WHERE id = $1 AND "userId" = $2
        FOR UPDATE
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::error::ApiError::NotFound)?;

    let checklist_text: Option<String> = row.try_get("contractChecklist").ok().flatten();
    let mut checklist: Map<String, Value> = checklist_text
        .and_then(|s| serde_json::from_str(&s).ok())
        .and_then(|v: Value| v.as_object().cloned())
        .unwrap_or_default();

    if checked {
        let now = chrono::Utc::now().to_rfc3339();
        for id in valid_item_ids {
            checklist.insert(id.clone(), Value::String(now.clone()));
        }
    } else {
        for id in valid_item_ids {
            checklist.remove(id);
        }
    }

    let checklist_json = serde_json::to_string(&Value::Object(checklist))?;

    sqlx::query(
        r#"
        UPDATE "Property"
        SET "contractChecklist" = $1::jsonb, "updatedAt" = now() at time zone 'utc'
        WHERE id = $2 AND "userId" = $3
        "#,
    )
    .bind(checklist_json)
    .bind(property_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(())
}

// ─── 상태 변경: 진행 → 계약진행 ──────────────────────────────
pub async fn start_contract(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
) -> ApiResult<PropertyDto> {
    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        SELECT status, name, "complexName" as complex_name
        FROM "Property"
        WHERE id = $1 AND "userId" = $2
        FOR UPDATE
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::error::ApiError::NotFound)?;

    let status: String = row.get("status");
    if status != "진행" {
        return Err(crate::error::ApiError::BadRequest(
            "진행 상태인 매물만 계약진행으로 전환할 수 있습니다".into(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE "Property"
        SET status = '계약진행', "updatedAt" = now() at time zone 'utc'
        WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    find_property_for_contract(pool, user_id, property_id).await
}

// ─── 상태 변경: 계약진행 → 계약완료 ──────────────────────────
pub async fn complete_contract(
    pool: &PgPool,
    user_id: &str,
    property_id: &str,
    progress_complete: bool,
) -> ApiResult<PropertyDto> {
    if !progress_complete {
        return Err(crate::error::ApiError::BadRequest(
            "필수 항목·데이터가 모두 충족되지 않았습니다".into(),
        ));
    }

    let mut tx = pool.begin().await?;

    let row = sqlx::query(
        r#"
        SELECT status, name, "complexName" as complex_name
        FROM "Property"
        WHERE id = $1 AND "userId" = $2
        FOR UPDATE
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or(crate::error::ApiError::NotFound)?;

    let status: String = row.get("status");
    if status != "계약진행" {
        return Err(crate::error::ApiError::BadRequest(
            "계약진행 상태인 매물만 계약완료로 전환할 수 있습니다".into(),
        ));
    }

    sqlx::query(
        r#"
        UPDATE "Property"
        SET status = '계약완료', "updatedAt" = now() at time zone 'utc'
        WHERE id = $1 AND "userId" = $2
        "#,
    )
    .bind(property_id)
    .bind(user_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    find_property_for_contract(pool, user_id, property_id).await
}