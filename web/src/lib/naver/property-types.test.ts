import { describe, expect, it } from "vitest";

import { PROPERTY_OPTIONS, PROPERTY_LABEL, DEFAULT_PROPERTY, propertyMode } from "./property-types";

describe("property-types", () => {
  it("14종 매물유형 + 단지형/비단지형 모드", () => {
    expect(PROPERTY_OPTIONS).toHaveLength(14);
    expect(DEFAULT_PROPERTY).toBe("A01");
    expect(PROPERTY_LABEL.A01).toBe("아파트");
    expect(PROPERTY_LABEL.D02).toBe("상가");
  });

  it("A* = 단지형, 나머지 = 비단지형", () => {
    expect(propertyMode("A01")).toBe("complex");
    expect(propertyMode("A02")).toBe("complex");
    expect(propertyMode("A04")).toBe("complex");
    expect(propertyMode("C02")).toBe("article");
    expect(propertyMode("D02")).toBe("article");
    expect(propertyMode("E03")).toBe("article");
    expect(propertyMode("ZZZ")).toBe("article"); // 미지정은 비단지형 기본
  });
});
