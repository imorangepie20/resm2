// 거래유형 단일 정의 — 하드코딩 금지. 여기서만 정의하고 import해서 쓴다.
// 네이버 코드: A1 매매 / B1 전세 / B2 월세 / B3 단기임대

export type TradeOption = { value: string; label: string };

/** 단일선택(라디오) 옵션 — 사용자가 고르는 거래유형 */
export const TRADE_OPTIONS: TradeOption[] = [
  { value: "A1", label: "매매" },
  { value: "B1", label: "전세" },
  { value: "B2", label: "월세" },
  { value: "B3", label: "단기임대" },
];

/** 표시용 코드→라벨 */
export const TRADE_LABEL: Record<string, string> = Object.fromEntries(
  TRADE_OPTIONS.map((o) => [o.value, o.label]),
);

/** 기본 거래유형 (매매) */
export const DEFAULT_TRADE = TRADE_OPTIONS[0].value;
