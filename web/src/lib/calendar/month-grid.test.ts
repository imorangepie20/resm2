import { describe, it, expect } from "vitest";
import { buildMonthGrid, ymd, monthPrefix, MONTH_NAMES_KO, WEEKDAYS_KO } from "./month-grid";

describe("buildMonthGrid (2026년 6월=month 5)", () => {
  const grid = buildMonthGrid(2026, 5);
  it("항상 42칸", () => {
    expect(grid).toHaveLength(42);
  });
  it("6월 1일은 월요일(인덱스 1) — 2026-06-01", () => {
    const june1 = grid.find((c) => c.isCurrentMonth && c.day === 1)!;
    expect(grid.indexOf(june1)).toBe(1);
    expect(june1.month).toBe(5);
  });
  it("선행/후행 칸은 isCurrentMonth=false", () => {
    expect(grid[0].isCurrentMonth).toBe(false);
    expect(grid[41].isCurrentMonth).toBe(false);
  });
});

describe("ymd / monthPrefix", () => {
  it("month 0-indexed → YYYYMMDD/YYYYMM", () => {
    expect(ymd(2026, 5, 7)).toBe("20260607");
    expect(ymd(2026, 11, 25)).toBe("20261225");
    expect(monthPrefix(2026, 5)).toBe("202606");
  });
});

describe("한국어 명칭", () => {
  it("12개월·7요일", () => {
    expect(MONTH_NAMES_KO).toHaveLength(12);
    expect(MONTH_NAMES_KO[5]).toBe("6월");
    expect(WEEKDAYS_KO).toEqual(["일", "월", "화", "수", "목", "금", "토"]);
  });
});
