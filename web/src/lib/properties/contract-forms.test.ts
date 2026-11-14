import { describe, it, expect } from "vitest";
import { formsFor, FORM_BY_ID } from "./contract-forms";

const fids = (g: string, t: string) => formsFor(g, t).map((f) => f.id);

describe("formsFor", () => {
  it("아파트 매매: 매매계약서·주거확인설명서·영수증·위임장, 임대차계약서 미포함", () => {
    const r = fids("A01", "A1");
    expect(r).toEqual(expect.arrayContaining(["sale_contract", "confirm_residential", "receipt", "power_of_attorney"]));
    expect(r).not.toContain("lease_contract");
    expect(r).not.toContain("confirm_nonresidential");
  });
  it("상가 월세: 임대차계약서·비주거확인설명서, 매매계약서·주거확인설명서 미포함", () => {
    const r = fids("D02", "B2");
    expect(r).toEqual(expect.arrayContaining(["lease_contract", "confirm_nonresidential"]));
    expect(r).not.toContain("sale_contract");
    expect(r).not.toContain("confirm_residential");
  });
  it("토지 매매: 비주거 확인설명서", () => {
    expect(fids("E03", "A1")).toContain("confirm_nonresidential");
  });
});

describe("FORM_BY_ID 슬롯", () => {
  it("매매계약서에 매매대금 슬롯(sourceKey=price) 존재", () => {
    const slots = FORM_BY_ID.sale_contract.slots;
    expect(slots.some((s) => s.sourceKey === "price")).toBe(true);
  });
  it("임대차계약서에 월세 슬롯(sourceKey=rentPrice) 존재", () => {
    expect(FORM_BY_ID.lease_contract.slots.some((s) => s.sourceKey === "rentPrice")).toBe(true);
  });
  it("상대 당사자 슬롯은 sourceKey 없음(수기란)", () => {
    const slots = FORM_BY_ID.sale_contract.slots;
    expect(slots.some((s) => s.sourceKey === undefined)).toBe(true);
  });
});
