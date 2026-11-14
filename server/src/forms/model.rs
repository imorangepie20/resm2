use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FormFieldSlot {
    pub slot: String,
    pub source_key: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ContractForm {
    pub id: String,
    pub label: String,
    pub applies: fn(&str, &str) -> bool,
    pub slots: Vec<FormFieldSlot>,
}

fn applies_sale(_g: &str, t: &str) -> bool {
    t == "A1"
}

fn applies_lease(_g: &str, t: &str) -> bool {
    ["B1", "B2", "B3"].contains(&t)
}

fn applies_residential(g: &str, _t: &str) -> bool {
    RESIDENTIAL.contains(&g)
}

fn applies_nonresidential(g: &str, _t: &str) -> bool {
    NONRESIDENTIAL.contains(&g)
}

fn applies_always(_g: &str, _t: &str) -> bool {
    true
}

const RESIDENTIAL: &[&str] = &["A01", "A02", "A04", "C02", "C01", "C03", "C04"];
const NONRESIDENTIAL: &[&str] = &[
    "D01", "D02", "D03", "D05", "E02", "E03", "E04",
];

fn base_property_slots() -> Vec<FormFieldSlot> {
    vec![
        FormFieldSlot { slot: "소재지(주소)".into(), source_key: Some("address".into()) },
        FormFieldSlot { slot: "단지명".into(), source_key: Some("complexName".into()) },
        FormFieldSlot { slot: "매물명".into(), source_key: Some("name".into()) },
        FormFieldSlot { slot: "전용면적".into(), source_key: Some("areaExclusive".into()) },
        FormFieldSlot { slot: "공급면적".into(), source_key: Some("areaSupply".into()) },
        FormFieldSlot { slot: "대지면적".into(), source_key: Some("landArea".into()) },
    ]
}

fn base_party_slots() -> Vec<FormFieldSlot> {
    vec![
        FormFieldSlot { slot: "갑(일방) 성명".into(), source_key: Some("customerName".into()) },
        FormFieldSlot { slot: "갑(일방) 연락처".into(), source_key: Some("customerPhone".into()) },
        FormFieldSlot { slot: "을(상대) 성명".into(), source_key: None },
        FormFieldSlot { slot: "을(상대) 주소".into(), source_key: None },
        FormFieldSlot { slot: "을(상대) 연락처".into(), source_key: None },
    ]
}

fn base_schedule_slots() -> Vec<FormFieldSlot> {
    vec![
        FormFieldSlot { slot: "계약일".into(), source_key: Some("contractDate".into()) },
        FormFieldSlot { slot: "중도금일".into(), source_key: Some("interim1Date".into()) },
        FormFieldSlot { slot: "잔금일".into(), source_key: Some("balanceDate".into()) },
    ]
}

fn build_forms() -> Vec<ContractForm> {
    vec![
        ContractForm {
            id: "sale_contract".into(),
            label: "부동산 매매계약서".into(),
            applies: applies_sale,
            slots: {
                let mut s = base_property_slots();
                s.push(FormFieldSlot { slot: "매매대금".into(), source_key: Some("price".into()) });
                s.push(FormFieldSlot { slot: "계약금(수기)".into(), source_key: None });
                s.push(FormFieldSlot { slot: "잔금(수기)".into(), source_key: None });
                s.extend(base_schedule_slots());
                s.extend(base_party_slots());
                s
            },
        },
        ContractForm {
            id: "lease_contract".into(),
            label: "부동산 임대차계약서".into(),
            applies: applies_lease,
            slots: {
                let mut s = base_property_slots();
                s.push(FormFieldSlot { slot: "보증금".into(), source_key: Some("price".into()) });
                s.push(FormFieldSlot { slot: "월세".into(), source_key: Some("rentPrice".into()) });
                s.extend(base_schedule_slots());
                s.extend(base_party_slots());
                s
            },
        },
        ContractForm {
            id: "confirm_residential".into(),
            label: "중개대상물 확인·설명서[주거용]".into(),
            applies: applies_residential,
            slots: {
                let mut s = base_property_slots();
                s.push(FormFieldSlot { slot: "난방방식".into(), source_key: Some("heating".into()) });
                s.push(FormFieldSlot { slot: "총세대수".into(), source_key: Some("totalHouseholds".into()) });
                s
            },
        },
        ContractForm {
            id: "confirm_nonresidential".into(),
            label: "중개대상물 확인·설명서[비주거용]".into(),
            applies: applies_nonresidential,
            slots: {
                let mut s = base_property_slots();
                s.push(FormFieldSlot { slot: "용도지역(수기)".into(), source_key: None });
                s.push(FormFieldSlot { slot: "이용현황(수기)".into(), source_key: None });
                s
            },
        },
        ContractForm {
            id: "receipt".into(),
            label: "영수증".into(),
            applies: applies_always,
            slots: vec![
                FormFieldSlot { slot: "금액(수기)".into(), source_key: None },
                FormFieldSlot { slot: "받은 사람".into(), source_key: Some("customerName".into()) },
                FormFieldSlot { slot: "일자".into(), source_key: Some("contractDate".into()) },
            ],
        },
        ContractForm {
            id: "power_of_attorney".into(),
            label: "위임장".into(),
            applies: applies_always,
            slots: vec![
                FormFieldSlot { slot: "위임인(수기)".into(), source_key: None },
                FormFieldSlot { slot: "수임인(수기)".into(), source_key: None },
                FormFieldSlot { slot: "부동산 표시".into(), source_key: Some("complexName".into()) },
            ],
        },
    ]
}

pub fn forms_for(real_estate_type: &str, trade_type: &str) -> Vec<ContractForm> {
    build_forms()
        .into_iter()
        .filter(|f| (f.applies)(real_estate_type, trade_type))
        .collect()
}

pub fn form_by_id(form_id: &str) -> Option<ContractForm> {
    build_forms().into_iter().find(|f| f.id == form_id)
}

pub fn template_name(form_id: &str) -> Option<&'static str> {
    match form_id {
        "sale_contract" => Some("forms/sale_contract.html"),
        "lease_contract" => Some("forms/lease_contract.html"),
        "confirm_residential" => Some("forms/confirm_residential.html"),
        "confirm_nonresidential" => Some("forms/confirm_nonresidential.html"),
        "receipt" => Some("forms/receipt.html"),
        "power_of_attorney" => Some("forms/power_of_attorney.html"),
        _ => None,
    }
}

// ─── 헬퍼 함수들 ────────────────────────────────────────────────

pub fn format_won(value: &Option<serde_json::Value>) -> String {
    value.as_ref()
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| {
            let num: i64 = s.parse().unwrap_or(0);
            let s = num.to_string();
            let mut result = String::new();
            for (i, ch) in s.chars().rev().enumerate() {
                if i > 0 && i % 3 == 0 {
                    result.push(',');
                }
                result.push(ch);
            }
            result.chars().rev().collect()
        })
        .unwrap_or_default()
}

pub fn format_ymd(value: &Option<serde_json::Value>) -> String {
    value.as_ref()
        .and_then(|v| v.as_str())
        .filter(|s| s.len() == 8)
        .map(|s| format!("{}년 {}월 {}일", &s[0..4], &s[4..6], &s[6..8]))
        .unwrap_or_default()
}

pub fn fill(value: &Option<String>, width: &str) -> String {
    match value {
        Some(v) if !v.is_empty() => format!("<span class=\"font-medium\">{}</span>", v),
        _ => format!("<span class=\"fill\" style=\"min-width: {}\">&nbsp;</span>", width),
    }
}

pub fn property_slots(p: &serde_json::Value) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for slot in base_property_slots() {
        if let Some(key) = &slot.source_key {
            if let Some(val) = p.get(key) {
                map.insert(slot.slot.clone(), val.as_str().unwrap_or("").to_string());
            }
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn forms_for_residential_sale() {
        let forms = forms_for("A01", "A1");
        let ids: Vec<_> = forms.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"sale_contract"));
        assert!(ids.contains(&"confirm_residential"));
        assert!(ids.contains(&"receipt"));
        assert!(ids.contains(&"power_of_attorney"));
        assert!(!ids.contains(&"lease_contract"));
    }

    #[test]
    fn forms_for_residential_lease() {
        let forms = forms_for("A01", "B1");
        let ids: Vec<_> = forms.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"lease_contract"));
        assert!(ids.contains(&"confirm_residential"));
        assert!(ids.contains(&"receipt"));
        assert!(ids.contains(&"power_of_attorney"));
        assert!(!ids.contains(&"sale_contract"));
    }

    #[test]
    fn forms_for_commercial() {
        let forms = forms_for("D01", "A1");
        let ids: Vec<_> = forms.iter().map(|f| f.id.as_str()).collect();
        assert!(ids.contains(&"sale_contract"));
        assert!(ids.contains(&"confirm_nonresidential"));
        assert!(!ids.contains(&"confirm_residential"));
    }
}