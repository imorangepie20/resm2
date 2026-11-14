import { describe, it, expect } from "vitest";
import { propertyCalendarEvents, PROPERTY_DATE_FIELDS } from "./property-events";

describe("propertyCalendarEvents", () => {
  const props = [
    { id: "p1", name: "정자동 자이 102-1503", complexName: "정자동 자이", contractDate: "20260610", balanceDate: "20260701" },
    { id: "p2", name: null, complexName: "래미안", moveInDate: "20260615" },
    { id: "p3", name: "값없음", complexName: null }, // 날짜 없음 → 항목 없음
  ];
  const events = propertyCalendarEvents(props);
  it("날짜 있는 필드마다 1건·라벨=매물명+필드", () => {
    expect(events).toHaveLength(3);
    const e = events.find((x) => x.propertyId === "p1" && x.field === "balanceDate")!;
    expect(e.date).toBe("20260701");
    expect(e.label).toBe("정자동 자이 102-1503 잔금일");
  });
  it("매물명 없으면 단지명 사용", () => {
    const e = events.find((x) => x.propertyId === "p2")!;
    expect(e.label).toBe("래미안 입주일");
  });
  it("PROPERTY_DATE_FIELDS 7종(사용승인일 제외)", () => {
    expect(PROPERTY_DATE_FIELDS).toHaveLength(7);
    expect(PROPERTY_DATE_FIELDS.map((f) => f.key)).not.toContain("approvalDate");
  });
});
