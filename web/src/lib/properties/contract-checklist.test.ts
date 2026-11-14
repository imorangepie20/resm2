import { describe, it, expect } from "vitest";
import { resolveChecklist, requiredFieldKeys, contractProgress } from "./contract-checklist";

const ids = (g: string, t: string) => resolveChecklist(g, t).map((i) => i.id);

describe("resolveChecklist 교차", () => {
  it("아파트 전세(A01×B1): 전입·확정일자·선순위·보증보험 포함, 사업자등록 미포함", () => {
    const r = ids("A01", "B1");
    expect(r).toEqual(expect.arrayContaining(["ACT_MOVE_IN", "ACT_FIXED_DATE", "ACT_SENIOR_LIEN", "ACT_DEPOSIT_GUARANTEE"]));
    expect(r).not.toContain("ACT_BIZ_REG");
  });
  it("상가 월세(D02×B2): 사업자등록·세무서확정일자 포함, 전입신고·선순위 미포함", () => {
    const r = ids("D02", "B2");
    expect(r).toEqual(expect.arrayContaining(["ACT_BIZ_REG", "ACT_TAX_FIXED_DATE"]));
    expect(r).not.toContain("ACT_MOVE_IN");
    expect(r).not.toContain("ACT_SENIOR_LIEN");
  });
  it("상가 전세(D02×B1): 보증보험 미포함(주거 한정)", () => {
    expect(ids("D02", "B1")).not.toContain("ACT_DEPOSIT_GUARANTEE");
  });
  it("토지 매매(E03×A1): 농취증·토허 포함", () => {
    expect(ids("E03", "A1")).toEqual(expect.arrayContaining(["ACT_FARMLAND_CERT", "ACT_LUT_PERMIT"]));
  });
  it("매매(A1): 등기권리증·소유권이전·거래신고·취득세·양도세 포함", () => {
    expect(ids("A01", "A1")).toEqual(expect.arrayContaining(["DOC_TITLE_DEED", "ACT_OWNERSHIP", "FILE_TX_REPORT", "TAX_ACQUISITION", "TAX_CAPITAL_GAINS"]));
  });
  it("임대(B1/B2): 임대인 납세증명 포함, B3·공장은 공통 위주", () => {
    expect(ids("A01", "B2")).toContain("DOC_TAX_CLEARANCE");
    expect(ids("A01", "B3")).not.toContain("ACT_MOVE_IN");
    expect(ids("E02", "A1")).not.toContain("ACT_BIZ_REG"); // 공장 전용항목 0
  });
  it("공통 5종은 항상 포함", () => {
    expect(ids("A01", "B3")).toEqual(expect.arrayContaining(["DOC_ID", "DOC_REGISTER", "DOC_BLDG_LEDGER", "DOC_CONTRACT", "DOC_CONFIRM"]));
  });
});

describe("requiredFieldKeys", () => {
  it("매매=가격, 전세=보증금, 월세=보증금+월세, 단기=공통만", () => {
    expect(requiredFieldKeys("A1")).toEqual(["customerName", "contractDate", "balanceDate", "price"]);
    expect(requiredFieldKeys("B1")).toContain("price");
    expect(requiredFieldKeys("B2")).toEqual(expect.arrayContaining(["price", "rentPrice"]));
    expect(requiredFieldKeys("B3")).toEqual(["customerName", "contractDate", "balanceDate"]);
  });
});

describe("contractProgress 게이트", () => {
  const full = { customerName: "홍길동", contractDate: "20260701", balanceDate: "20260801", price: "500000000" };
  it("required 항목·필드 전부 충족 시 complete", () => {
    const items = resolveChecklist("A01", "A1").filter((i) => i.required);
    const checked = Object.fromEntries(items.map((i) => [i.id, "2026-07-01T00:00:00Z"]));
    const p = contractProgress("A01", "A1", checked, full);
    expect(p.done).toBe(p.total);
    expect(p.complete).toBe(true);
  });
  it("필드 하나 비면 미완", () => {
    const items = resolveChecklist("A01", "A1").filter((i) => i.required);
    const checked = Object.fromEntries(items.map((i) => [i.id, "x"]));
    const p = contractProgress("A01", "A1", checked, { ...full, balanceDate: "" });
    expect(p.complete).toBe(false);
    expect(p.done).toBe(p.total - 1);
  });
  it("required 아닌 항목(신고)은 게이트에 미반영", () => {
    const checkedReqOnly = Object.fromEntries(
      resolveChecklist("A01", "A1").filter((i) => i.required).map((i) => [i.id, "x"]),
    );
    expect(contractProgress("A01", "A1", checkedReqOnly, full).complete).toBe(true);
  });
});
