import { describe, it, expect } from "vitest";
import { normalizeCustomerTypes, CUSTOMER_TYPES } from "./types";

describe("normalizeCustomerTypes", () => {
  it("단순방문은 배타 — 거래 당사자가 있으면 단순방문 제거", () => {
    expect(normalizeCustomerTypes(["단순방문", "매도인"])).toEqual(["매도인"]);
  });
  it("거래 당사자 다중 유지·정규 순서·중복 제거", () => {
    expect(normalizeCustomerTypes(["매수인", "매도인", "매수인"])).toEqual(["매도인", "매수인"]);
  });
  it("빈 입력·무효값만 → 기본 단순방문", () => {
    expect(normalizeCustomerTypes([])).toEqual(["단순방문"]);
    expect(normalizeCustomerTypes(["xyz"])).toEqual(["단순방문"]);
  });
  it("단순방문 단독 유지", () => {
    expect(normalizeCustomerTypes(["단순방문"])).toEqual(["단순방문"]);
  });
  it("CUSTOMER_TYPES 5종", () => {
    expect(CUSTOMER_TYPES).toHaveLength(5);
  });
});
