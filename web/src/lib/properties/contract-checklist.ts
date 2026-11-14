// 계약 서류 체크리스트 단일 소스 — (매물유형×거래유형) byCross 합성. 저장 안 함.
export type ChecklistItem = { id: string; label: string; party: string; kind: "서류" | "처리" | "신고" | "세금"; required: boolean };

export const PARTIES = [
  { value: "common", label: "공통" },
  { value: "seller", label: "매도인" },
  { value: "buyer", label: "매수인" },
  { value: "landlord", label: "임대인" },
  { value: "tenant", label: "임차인" },
  { value: "agent", label: "중개" },
];
export const PARTY_LABEL: Record<string, string> = Object.fromEntries(PARTIES.map((p) => [p.value, p.label]));

export const GROUPS = {
  RESI: ["A01", "A02", "A04", "C02", "C01"],
  RESI_SINGLE: ["C03", "C04"],
  COMMERCIAL: ["D01", "D02", "D03", "D05"],
  LAND: ["E03"],
  FACTORY: ["E02", "E04"],
};
const RESIDENTIAL = [...GROUPS.RESI, ...GROUPS.RESI_SINGLE];

const ITEMS: ChecklistItem[] = [
  { id: "DOC_ID", label: "신분증", party: "common", kind: "서류", required: true },
  { id: "DOC_REGISTER", label: "등기사항전부증명서(당일 최신본)", party: "agent", kind: "서류", required: true },
  { id: "DOC_BLDG_LEDGER", label: "건축물대장", party: "agent", kind: "서류", required: true },
  { id: "DOC_CONTRACT", label: "부동산 매매·임대차계약서", party: "agent", kind: "서류", required: true },
  { id: "DOC_CONFIRM", label: "중개대상물 확인·설명서", party: "agent", kind: "서류", required: true },
  { id: "DOC_TITLE_DEED", label: "등기권리증(등기필증)", party: "seller", kind: "서류", required: true },
  { id: "DOC_SEAL_SALE", label: "인감증명서(부동산 매도용, 3개월 이내)", party: "seller", kind: "서류", required: true },
  { id: "ACT_OWNERSHIP", label: "소유권이전등기", party: "buyer", kind: "처리", required: true },
  { id: "FILE_TX_REPORT", label: "부동산거래신고(계약+30일)", party: "agent", kind: "신고", required: false },
  { id: "TAX_ACQUISITION", label: "취득세 신고·납부", party: "buyer", kind: "세금", required: false },
  { id: "TAX_CAPITAL_GAINS", label: "양도세 예정신고", party: "seller", kind: "세금", required: false },
  { id: "DOC_TAX_CLEARANCE", label: "임대인 납세증명서·미납국세 확인", party: "landlord", kind: "서류", required: false },
  { id: "ACT_SENIOR_LIEN", label: "선순위 권리·전입세대 점검", party: "agent", kind: "처리", required: true },
  { id: "ACT_MOVE_IN", label: "전입신고(대항력)", party: "tenant", kind: "처리", required: true },
  { id: "ACT_FIXED_DATE", label: "확정일자(우선변제권)", party: "tenant", kind: "처리", required: true },
  { id: "ACT_DEPOSIT_GUARANTEE", label: "전세보증금반환보증", party: "tenant", kind: "처리", required: false },
  { id: "FILE_LEASE_REPORT", label: "주택임대차신고(30일)", party: "common", kind: "신고", required: false },
  { id: "ACT_BIZ_REG", label: "사업자등록(개업 20일 내)", party: "tenant", kind: "처리", required: true },
  { id: "ACT_TAX_FIXED_DATE", label: "세무서 확정일자", party: "tenant", kind: "처리", required: true },
  { id: "ACT_FARMLAND_CERT", label: "농지취득자격증명(농취증)", party: "buyer", kind: "처리", required: false },
  { id: "ACT_LUT_PERMIT", label: "토지거래허가", party: "buyer", kind: "처리", required: false },
];
const BY_ID: Record<string, ChecklistItem> = Object.fromEntries(ITEMS.map((i) => [i.id, i]));

function itemIds(realEstateType: string, tradeType: string): string[] {
  const g = realEstateType;
  const t = tradeType;
  const set = new Set<string>(["DOC_ID", "DOC_REGISTER", "DOC_BLDG_LEDGER", "DOC_CONTRACT", "DOC_CONFIRM"]); // 공통
  const isResi = RESIDENTIAL.includes(g);
  const isComm = GROUPS.COMMERCIAL.includes(g);
  // byDealType
  if (t === "A1") ["DOC_TITLE_DEED", "DOC_SEAL_SALE", "ACT_OWNERSHIP", "FILE_TX_REPORT", "TAX_ACQUISITION", "TAX_CAPITAL_GAINS"].forEach((x) => set.add(x));
  if (t === "B1" || t === "B2") ["FILE_LEASE_REPORT", "DOC_TAX_CLEARANCE"].forEach((x) => set.add(x));
  // B3 → 분기 없음(의도)
  // byPropertyGroup
  if (GROUPS.LAND.includes(g)) ["ACT_FARMLAND_CERT", "ACT_LUT_PERMIT"].forEach((x) => set.add(x));
  // FACTORY → 분기 없음(MVP)
  // byCross (매물군 × 거래유형)
  if (isResi && (t === "B1" || t === "B2")) ["ACT_MOVE_IN", "ACT_FIXED_DATE", "ACT_SENIOR_LIEN"].forEach((x) => set.add(x));
  if (isResi && t === "B1") set.add("ACT_DEPOSIT_GUARANTEE");
  if (isComm && (t === "B1" || t === "B2")) ["ACT_BIZ_REG", "ACT_TAX_FIXED_DATE"].forEach((x) => set.add(x));
  return [...set];
}

export function resolveChecklist(realEstateType: string, tradeType: string): ChecklistItem[] {
  return itemIds(realEstateType, tradeType).map((id) => BY_ID[id]).filter(Boolean);
}

export function requiredFieldKeys(tradeType: string): string[] {
  const base = ["customerName", "contractDate", "balanceDate"];
  if (tradeType === "A1") return [...base, "price"];
  if (tradeType === "B1") return [...base, "price"];
  if (tradeType === "B2") return [...base, "price", "rentPrice"];
  return base;
}

const filled = (v: unknown) => v != null && v !== "";

export function contractProgress(
  realEstateType: string,
  tradeType: string,
  checked: Record<string, string>,
  property: Record<string, unknown>,
): { done: number; total: number; complete: boolean } {
  const reqItems = resolveChecklist(realEstateType, tradeType).filter((i) => i.required);
  const fieldKeys = requiredFieldKeys(tradeType);
  const doneItems = reqItems.filter((i) => !!checked[i.id]).length;
  const doneFields = fieldKeys.filter((k) => filled(property[k])).length;
  const done = doneItems + doneFields;
  const total = reqItems.length + fieldKeys.length;
  return { done, total, complete: total > 0 && done === total };
}
