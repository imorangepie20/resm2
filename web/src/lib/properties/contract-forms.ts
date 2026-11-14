// 계약 양식 단일 소스 — 거래/유형별 적용 양식 + 슬롯↔Property 키 매핑(MVP: 보유분만 자동기입).
import { GROUPS } from "./contract-checklist";

export type FormFieldSlot = { slot: string; sourceKey?: string }; // sourceKey 없으면 인쇄물 수기 기입란

export type ContractForm = {
  id: string;
  label: string;
  applies: (realEstateType: string, tradeType: string) => boolean;
  slots: FormFieldSlot[];
};

const RESIDENTIAL = [...GROUPS.RESI, ...GROUPS.RESI_SINGLE];
const NONRESI = [...GROUPS.COMMERCIAL, ...GROUPS.LAND, ...GROUPS.FACTORY];

// 부동산 표시·당사자 공통 슬롯
const PROPERTY_SLOTS: FormFieldSlot[] = [
  { slot: "소재지(주소)", sourceKey: "address" },
  { slot: "단지명", sourceKey: "complexName" },
  { slot: "매물명", sourceKey: "name" },
  { slot: "전용면적", sourceKey: "areaExclusive" },
  { slot: "공급면적", sourceKey: "areaSupply" },
  { slot: "대지면적", sourceKey: "landArea" },
];
const PARTY_SLOTS: FormFieldSlot[] = [
  { slot: "갑(일방) 성명", sourceKey: "customerName" },
  { slot: "갑(일방) 연락처", sourceKey: "customerPhone" },
  { slot: "을(상대) 성명" }, // 수기
  { slot: "을(상대) 주소" }, // 수기
  { slot: "을(상대) 연락처" }, // 수기
];
const SCHEDULE_SLOTS: FormFieldSlot[] = [
  { slot: "계약일", sourceKey: "contractDate" },
  { slot: "중도금일", sourceKey: "interim1Date" },
  { slot: "잔금일", sourceKey: "balanceDate" },
];

export const CONTRACT_FORMS: ContractForm[] = [
  {
    id: "sale_contract",
    label: "부동산 매매계약서",
    applies: (_g, t) => t === "A1",
    slots: [
      ...PROPERTY_SLOTS,
      { slot: "매매대금", sourceKey: "price" },
      { slot: "계약금(수기)" },
      { slot: "잔금(수기)" },
      ...SCHEDULE_SLOTS,
      ...PARTY_SLOTS,
    ],
  },
  {
    id: "lease_contract",
    label: "부동산 임대차계약서",
    applies: (_g, t) => ["B1", "B2", "B3"].includes(t),
    slots: [
      ...PROPERTY_SLOTS,
      { slot: "보증금", sourceKey: "price" },
      { slot: "월세", sourceKey: "rentPrice" },
      ...SCHEDULE_SLOTS,
      ...PARTY_SLOTS,
    ],
  },
  {
    id: "confirm_residential",
    label: "중개대상물 확인·설명서[주거용]",
    applies: (g) => RESIDENTIAL.includes(g),
    slots: [...PROPERTY_SLOTS, { slot: "난방방식", sourceKey: "heating" }, { slot: "총세대수", sourceKey: "totalHouseholds" }],
  },
  {
    id: "confirm_nonresidential",
    label: "중개대상물 확인·설명서[비주거용]",
    applies: (g) => NONRESI.includes(g),
    slots: [...PROPERTY_SLOTS, { slot: "용도지역(수기)" }, { slot: "이용현황(수기)" }],
  },
  {
    id: "receipt",
    label: "영수증",
    applies: () => true,
    slots: [{ slot: "금액(수기)" }, { slot: "받은 사람", sourceKey: "customerName" }, { slot: "일자", sourceKey: "contractDate" }],
  },
  {
    id: "power_of_attorney",
    label: "위임장",
    applies: () => true,
    slots: [{ slot: "위임인(수기)" }, { slot: "수임인(수기)" }, { slot: "부동산 표시", sourceKey: "complexName" }],
  },
];

export const FORM_BY_ID: Record<string, ContractForm> = Object.fromEntries(CONTRACT_FORMS.map((f) => [f.id, f]));

export function formsFor(realEstateType: string, tradeType: string): ContractForm[] {
  return CONTRACT_FORMS.filter((f) => f.applies(realEstateType, tradeType));
}
