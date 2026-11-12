import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { buildPropertyBody, createProperty, updateProperty } from "./api"

const fetchMock = vi.fn()
beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock)
  fetchMock.mockReset()
})
afterEach(() => vi.unstubAllGlobals())

const sentBody = () => JSON.parse(fetchMock.mock.calls[0][1].body as string)

describe("buildPropertyBody — 폼 문자열 → 타입 맞춘 본문", () => {
  const values: Record<string, string> = {
    name: "래미안 102동", // text
    status: "계약진행", // select
    source: "엑셀", // 폼에서 숨긴 필드 → 보내지 않음
    areaExclusive: "84.5", // float
    siteArea: "100", // float(정수 모양이어도 숫자)
    totalHouseholds: "1,234", // integer
    parkingCount: "12.9", // integer(원본처럼 버림)
    isPreSale: "true", // bool
    price: "12,000", // money(만원) → 원 문자열
    rentPrice: "1.5",
    contractDate: "20261112", // date 문자열
    memo: "", // 비움 → null
    landArea: "", // float 비움 → null
    customerPhone: "",
  }

  it("필드 종류별 JSON 타입", async () => {
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ id: "p1" }), { status: 201 }))
    await createProperty(buildPropertyBody(values))
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe("/api/v1/properties")
    expect(init.method).toBe("POST")
    const b = sentBody()
    expect(b.name).toBe("래미안 102동")
    expect(b.status).toBe("계약진행")
    expect("source" in b).toBe(false)
    expect(b.areaExclusive).toBe(84.5)
    expect(b.siteArea).toBe(100)
    expect(b.totalHouseholds).toBe(1234)
    expect(b.parkingCount).toBe(12)
    expect(b.isPreSale).toBe(true)
    expect(b.price).toBe("120000000")
    expect(b.rentPrice).toBe("15000")
    expect(b.contractDate).toBe("20261112")
    expect(b.memo).toBeNull()
    expect(b.landArea).toBeNull()
    expect(b.customerPhone).toBeNull()
    // 폼이 모르는 키는 null 로 채워진다(원본처럼 전 필드 전송)
    expect(b.heating).toBeNull()
    expect(b.isFavorite).toBeUndefined()
  })

  it("체크 해제된 bool 은 false, 빈 status 는 생략", () => {
    const b = buildPropertyBody({ isPreSale: "", status: "", price: "" })
    expect(b.isPreSale).toBe(false)
    expect("status" in b).toBe(false)
    expect(b.price).toBeNull()
  })

  it("금액 형식 오류는 요청 전에 막는다", () => {
    expect(() => buildPropertyBody({ price: "12만" })).toThrow("금액 형식이 올바르지 않습니다")
  })

  it("PATCH 는 /properties/{id} 로 본문 그대로", async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 204 }))
    await updateProperty("abc", { price: null, isFavorite: true })
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe("/api/v1/properties/abc")
    expect(init.method).toBe("PATCH")
    expect(sentBody()).toEqual({ price: null, isFavorite: true })
  })
})
