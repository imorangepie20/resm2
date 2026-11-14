// 고객유형·성별 단일 정의 + 유형 정규화.
export const CUSTOMER_TYPES = ["매도인", "매수인", "임대인", "임차인", "단순방문"] as const;
export type CustomerType = (typeof CUSTOMER_TYPES)[number];
export const PARTY_TYPES: CustomerType[] = ["매도인", "매수인", "임대인", "임차인"];
export const GENDERS = ["남", "여", "미지정"] as const;

// 거래 당사자(다중)가 있으면 그것만(정규 순서·중복 제거), 없으면 기본 단순방문.
export function normalizeCustomerTypes(input: string[]): string[] {
  const valid = new Set(input.filter((t): t is CustomerType => (CUSTOMER_TYPES as readonly string[]).includes(t)));
  const parties = PARTY_TYPES.filter((t) => valid.has(t));
  return parties.length ? parties : ["단순방문"];
}
