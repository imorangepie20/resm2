import { describe, it, expect } from "vitest";
import { groupDigits, stripDigits, formatTel, toDateInput, fromDateInput } from "./format";

describe("format", () => {
  it("groupDigits", () => {
    expect(groupDigits("350000000")).toBe("350,000,000");
    expect(groupDigits("350,000,000")).toBe("350,000,000");
    expect(groupDigits("")).toBe("");
  });
  it("stripDigits", () => {
    expect(stripDigits("350,000,000원")).toBe("350000000");
  });
  it("formatTel", () => {
    expect(formatTel("01012345678")).toBe("010-1234-5678");
    expect(formatTel("0212345678")).toBe("021-234-5678"); // 10자리(02 지역번호)는 휴대폰 기준 3-3-4 — best-effort
    expect(formatTel("010123")).toBe("010-123");
  });
  it("date 변환 왕복", () => {
    expect(toDateInput("20030808")).toBe("2003-08-08");
    expect(toDateInput("미정")).toBe("");
    expect(fromDateInput("2003-08-08")).toBe("20030808");
  });
});
