use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── 체크리스트 아이템 ────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChecklistItem {
    pub id: String,
    pub label: String,
    pub party: String,
    pub kind: ChecklistKind,
    pub required: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ChecklistKind {
    서류,
    처리,
    신고,
    세금,
}

// ─── 그룹 정의 ───────────────────────────────────────────────
const PARTIES: &[(&str, &str)] = &[
    ("common", "공통"),
    ("seller", "매도인"),
    ("buyer", "매수인"),
    ("landlord", "임대인"),
    ("tenant", "임차인"),
    ("agent", "중개"),
];

const GROUPS_RESI: &[&str] = &["A01", "A02", "A04", "C02", "C01"];
const GROUPS_RESI_SINGLE: &[&str] = &["C03", "C04"];
const GROUPS_COMMERCIAL: &[&str] = &["D01", "D02", "D03", "D05"];
const GROUPS_LAND: &[&str] = &["E03"];
const GROUPS_FACTORY: &[&str] = &["E02", "E04"];

const RESIDENTIAL: &[&str] = &[
    "A01", "A02", "A04", "C02", "C01", "C03", "C04",
];

// ─── 체크리스트 아이템 마스터 데이터 ──────────────────────────
const ITEMS: &[(&str, &str, &str, ChecklistKind, bool)] = &[
    ("DOC_ID", "신분증", "common", ChecklistKind::서류, true),
    ("DOC_REGISTER", "등기사항전부증명서(당일 최신본)", "agent", ChecklistKind::서류, true),
    ("DOC_BLDG_LEDGER", "건축물대장", "agent", ChecklistKind::서류, true),
    ("DOC_CONTRACT", "부동산 매매·임대차계약서", "agent", ChecklistKind::서류, true),
    ("DOC_CONFIRM", "중개대상물 확인·설명서", "agent", ChecklistKind::서류, true),
    ("DOC_TITLE_DEED", "등기권리증(등기필증)", "seller", ChecklistKind::서류, true),
    ("DOC_SEAL_SALE", "인감증명서(부동산 매도용, 3개월 이내)", "seller", ChecklistKind::서류, true),
    ("ACT_OWNERSHIP", "소유권이전등기", "buyer", ChecklistKind::처리, true),
    ("FILE_TX_REPORT", "부동산거래신고(계약+30일)", "agent", ChecklistKind::신고, false),
    ("TAX_ACQUISITION", "취득세 신고·납부", "buyer", ChecklistKind::세금, false),
    ("TAX_CAPITAL_GAINS", "양도세 예정신고", "seller", ChecklistKind::세금, false),
    ("DOC_TAX_CLEARANCE", "임대인 납세증명서·미납국세 확인", "landlord", ChecklistKind::서류, false),
    ("ACT_SENIOR_LIEN", "선순위 권리·전입세대 점검", "agent", ChecklistKind::처리, true),
    ("ACT_MOVE_IN", "전입신고(대항력)", "tenant", ChecklistKind::처리, true),
    ("ACT_FIXED_DATE", "확정일자(우선변제권)", "tenant", ChecklistKind::처리, true),
    ("ACT_DEPOSIT_GUARANTEE", "전세보증금반환보증", "tenant", ChecklistKind::처리, false),
    ("FILE_LEASE_REPORT", "주택임대차신고(30일)", "common", ChecklistKind::신고, false),
    ("ACT_BIZ_REG", "사업자등록(개업 20일 내)", "tenant", ChecklistKind::처리, true),
    ("ACT_TAX_FIXED_DATE", "세무서 확정일자", "tenant", ChecklistKind::처리, true),
    ("ACT_FARMLAND_CERT", "농지취득자격증명(농취증)", "buyer", ChecklistKind::처리, false),
    ("ACT_LUT_PERMIT", "토지거래허가", "buyer", ChecklistKind::처리, false),
];

// ─── 헬퍼 ────────────────────────────────────────────────────
fn is_residential(g: &str) -> bool {
    RESIDENTIAL.iter().any(|&x| x == g)
}

fn in_groups(g: &str, groups: &[&str]) -> bool {
    groups.iter().any(|&x| x == g)
}

fn item_ids(real_estate_type: &str, trade_type: &str) -> Vec<String> {
    let g = real_estate_type;
    let t = trade_type;
    let mut set = std::collections::HashSet::new();

    // 공통
    ["DOC_ID", "DOC_REGISTER", "DOC_BLDG_LEDGER", "DOC_CONTRACT", "DOC_CONFIRM"]
        .iter()
        .for_each(|x| { set.insert(*x); });

    let is_resi = is_residential(g);
    let is_comm = in_groups(g, GROUPS_COMMERCIAL);

    // 거래유형별
    if t == "A1" {
        ["DOC_TITLE_DEED", "DOC_SEAL_SALE", "ACT_OWNERSHIP", "FILE_TX_REPORT", "TAX_ACQUISITION", "TAX_CAPITAL_GAINS"]
            .iter()
            .for_each(|x| { set.insert(*x); });
    }
    if t == "B1" || t == "B2" {
        ["FILE_LEASE_REPORT", "DOC_TAX_CLEARANCE"]
            .iter()
            .for_each(|x| { set.insert(*x); });
    }
    // B3 분기 없음

    // 매물군별
    if in_groups(g, GROUPS_LAND) {
        ["ACT_FARMLAND_CERT", "ACT_LUT_PERMIT"]
            .iter()
            .for_each(|x| { set.insert(*x); });
    }
    // FACTORY 분기 없음

    // 교차 (매물군 × 거래유형)
    if is_resi && (t == "B1" || t == "B2") {
        ["ACT_MOVE_IN", "ACT_FIXED_DATE", "ACT_SENIOR_LIEN"]
            .iter()
            .for_each(|x| { set.insert(*x); });
    }
    if is_resi && t == "B1" {
        set.insert("ACT_DEPOSIT_GUARANTEE");
    }
    if is_comm && (t == "B1" || t == "B2") {
        ["ACT_BIZ_REG", "ACT_TAX_FIXED_DATE"]
            .iter()
            .for_each(|x| { set.insert(*x); });
    }

    set.into_iter().map(String::from).collect()
}

/// 매물 유형·거래 유형에 해당하는 체크리스트 반환
pub fn resolve_checklist(real_estate_type: &str, trade_type: &str) -> Vec<ChecklistItem> {
    let ids = item_ids(real_estate_type, trade_type);
    ITEMS
        .iter()
        .filter(|(id, _, _, _, _)| ids.contains(&(*id).to_string()))
        .map(|(id, label, party, kind, required)| ChecklistItem {
            id: (*id).to_string(),
            label: (*label).to_string(),
            party: (*party).to_string(),
            kind: *kind,
            required: *required,
        })
        .collect()
}

/// 거래유형별 필수 필드 키 반환
pub fn required_field_keys(trade_type: &str) -> Vec<&'static str> {
    let base = ["customerName", "contractDate", "balanceDate"];
    match trade_type {
        "A1" => vec!["customerName", "contractDate", "balanceDate", "price"],
        "B1" => vec!["customerName", "contractDate", "balanceDate", "price"],
        "B2" => vec!["customerName", "contractDate", "balanceDate", "price", "rentPrice"],
        _ => base.to_vec(),
    }
}

fn filled(v: &Option<&serde_json::Value>) -> bool {
    v.is_some_and(|x| !x.is_null() && x != "")
}

/// 진행률 계산
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub done: usize,
    pub total: usize,
    pub complete: bool,
}

pub fn contract_progress(
    real_estate_type: &str,
    trade_type: &str,
    checked: &HashMap<String, String>,
    property: &serde_json::Value,
) -> Progress {
    let req_items = resolve_checklist(real_estate_type, trade_type)
        .into_iter()
        .filter(|i| i.required)
        .collect::<Vec<_>>();
    let field_keys = required_field_keys(trade_type);

    let done_items = req_items.iter().filter(|i| checked.contains_key(&i.id)).count();
    let done_fields = field_keys
        .iter()
        .filter(|k| filled(&property.get(*k)))
        .count();

    let done = done_items + done_fields;
    let total = req_items.len() + field_keys.len();
    Progress {
        done,
        total,
        complete: total > 0 && done == total,
    }
}

/// 거래유형 → 소유자 측 당사자 역할
pub fn owner_role_from_trade(trade_type: Option<&str>) -> &'static str {
    match trade_type {
        Some("A1") => "매도인",
        Some("B1") | Some("B2") | Some("B3") => "임대인",
        _ => "매도인",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn resolve_checklist_residential_sale() {
        let list = resolve_checklist("A01", "A1");
        assert!(list.iter().any(|i| i.id == "DOC_TITLE_DEED"));
        assert!(list.iter().any(|i| i.id == "ACT_OWNERSHIP"));
        assert!(list.iter().any(|i| i.id == "DOC_ID"));
    }

    #[test]
    fn resolve_checklist_residential_lease() {
        let list = resolve_checklist("A01", "B1");
        assert!(list.iter().any(|i| i.id == "ACT_MOVE_IN"));
        assert!(list.iter().any(|i| i.id == "ACT_FIXED_DATE"));
        assert!(list.iter().any(|i| i.id == "FILE_LEASE_REPORT"));
    }

    #[test]
    fn resolve_checklist_land_sale() {
        let list = resolve_checklist("E03", "A1");
        assert!(list.iter().any(|i| i.id == "ACT_FARMLAND_CERT"));
        assert!(list.iter().any(|i| i.id == "ACT_LUT_PERMIT"));
    }

    #[test]
    fn required_field_keys_match() {
        assert_eq!(required_field_keys("A1"), vec!["customerName", "contractDate", "balanceDate", "price"]);
        assert_eq!(required_field_keys("B1"), vec!["customerName", "contractDate", "balanceDate", "price"]);
        assert_eq!(required_field_keys("B2"), vec!["customerName", "contractDate", "balanceDate", "price", "rentPrice"]);
        assert_eq!(required_field_keys("B3"), vec!["customerName", "contractDate", "balanceDate"]);
    }

    #[test]
    fn contract_progress_calculates_correctly() {
        let mut checked = HashMap::new();
        checked.insert("DOC_ID".to_string(), "2024-01-01T00:00:00Z".to_string());
        let property = json!({
            "customerName": "홍길동",
            "contractDate": "20240101",
            "balanceDate": "20240201",
            "price": "100000000"
        });
        let prog = contract_progress("A01", "A1", &checked, &property);
        assert!(prog.done > 0);
        assert!(prog.total > 0);
    }

    #[test]
    fn owner_role_from_trade_works() {
        assert_eq!(owner_role_from_trade(Some("A1")), "매도인");
        assert_eq!(owner_role_from_trade(Some("B1")), "임대인");
        assert_eq!(owner_role_from_trade(Some("B2")), "임대인");
        assert_eq!(owner_role_from_trade(Some("B3")), "임대인");
        assert_eq!(owner_role_from_trade(None), "매도인");
    }
}