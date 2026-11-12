// 매물유형(부동산 종류) 단일 정의 — 하드코딩 금지. 네이버 realEstateType 코드.
// 코드 첫 글자 A = 단지형(complex), 나머지 = 비단지형(article).

export type PropertyOption = { value: string; label: string; mode: "complex" | "article" };

export const PROPERTY_OPTIONS: PropertyOption[] = [
  { value: "A01", label: "아파트", mode: "complex" },
  { value: "A02", label: "오피스텔", mode: "complex" },
  { value: "A04", label: "재건축", mode: "complex" },
  { value: "C02", label: "빌라", mode: "article" },
  { value: "C01", label: "원룸", mode: "article" },
  { value: "C03", label: "단독/다가구", mode: "article" },
  { value: "C04", label: "전원주택", mode: "article" },
  { value: "D05", label: "상가주택", mode: "article" },
  { value: "D02", label: "상가", mode: "article" },
  { value: "E03", label: "토지", mode: "article" },
  { value: "D01", label: "사무실", mode: "article" },
  { value: "D03", label: "건물", mode: "article" },
  { value: "E02", label: "공장/창고", mode: "article" },
  { value: "E04", label: "지식산업센터", mode: "article" },
];

export const PROPERTY_LABEL: Record<string, string> = Object.fromEntries(
  PROPERTY_OPTIONS.map((o) => [o.value, o.label]),
);

export const DEFAULT_PROPERTY = PROPERTY_OPTIONS[0].value; // A01

export function propertyMode(code: string): "complex" | "article" {
  return PROPERTY_OPTIONS.find((o) => o.value === code)?.mode ?? "article";
}
