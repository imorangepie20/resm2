import { QueryClientProvider } from "@tanstack/react-query"
import { cleanup, render, screen, waitFor, within } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter } from "react-router"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
const toast = vi.hoisted(() => ({ error: vi.fn(), success: vi.fn() }))
vi.mock("sonner", () => ({ toast }))

import { queryClient } from "@/lib/query-client"
import type { PropertyRow, PropertyView } from "@/lib/properties/api"
import { PropertyListPage } from "./list-page"

const fetchMock = vi.fn()

function json(status: number, body: unknown) {
  return new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } })
}

function row(i: number, over: Partial<PropertyRow> = {}): PropertyRow {
  return {
    id: `p${i}`, isFavorite: false, colorTag: null, status: "진행", source: "수기", createdAt: "2026-11-12T00:00:00Z",
    name: `매물${i}`, articleNo: null, complexName: null, address: null, realEstateType: "A01", tradeType: "A1",
    heating: null, customerName: null, customerPhone: null, partnerName: null, partnerPhone: null, partnerManager: null,
    manager: null, contractHopeDate: null, contractDate: null, moveInHopeDate: null, moveInDate: null,
    interim1Date: null, interim2Date: null, balanceDate: null, approvalDate: null, note: null, memo: null,
    siteArea: null, areaExclusive: null, areaSupply: null, landArea: null, buildingArea: null, area: null,
    totalHouseholds: null, parkingCount: null, isPreSale: null, price: "120000000", rentPrice: null,
    ...over,
  }
}

// lg 미만 카드 목록은 CSS 로만 숨겨져 jsdom 에서는 표·카드가 모두 렌더된다. 표 안에서만 찾는다.
const table = () => within(screen.getByRole("table"))
const pageInfo = () => screen.queryByText(/^\d+ \/ \d+$/)?.textContent

function mount(view: PropertyView, rows: PropertyRow[]) {
  fetchMock.mockImplementation(async (url: string, init: RequestInit) => {
    if ((init.method ?? "GET") === "GET" && url.startsWith("/api/v1/properties?view=")) return json(200, rows)
    throw new Error(`unexpected ${init.method} ${url}`)
  })
  render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter><PropertyListPage view={view} /></MemoryRouter>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  toast.error.mockReset()
  toast.success.mockReset()
  vi.stubGlobal("fetch", fetchMock)
  fetchMock.mockReset()
})
afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  queryClient.clear()
})

describe("PropertyList 검색·필터·페이징", () => {
  it("검색이 행을 줄이고, 검색·필터 변경 시 1페이지로 돌아간다", async () => {
    const rows = Array.from({ length: 60 }, (_, i) =>
      row(60 - i, i === 5 ? { customerName: "홍길동", customerPhone: "010-1111-2222" } : i < 10 ? { colorTag: "blue" } : {}))
    mount("all", rows)
    expect(await screen.findByText("60개")).toBeTruthy()
    expect(pageInfo()).toBe("1 / 2")
    expect(table().getAllByRole("row")).toHaveLength(51) // 머리행 + 50

    await userEvent.click(screen.getByRole("button", { name: /다음/ }))
    expect(pageInfo()).toBe("2 / 2")
    expect(table().getAllByRole("row")).toHaveLength(11)

    // 색상 필터(전체 뷰 전용) → 1페이지로, 9건(i<10 중 i=5 제외)
    await userEvent.click(screen.getByTitle("파랑 필터"))
    expect(screen.getByText("9개")).toBeTruthy()
    expect(pageInfo()).toBeUndefined() // 한 페이지면 페이저 없음
    await userEvent.click(screen.getByTitle("파랑 필터"))

    await userEvent.click(screen.getByRole("button", { name: /다음/ }))
    expect(pageInfo()).toBe("2 / 2")
    // 고객 전화로 검색 → 1건, 페이지 초기화
    await userEvent.type(screen.getByPlaceholderText("매물명·단지·주소·고객 검색"), "1111-2222")
    expect(screen.getByText("1개")).toBeTruthy()
    expect(table().getByText("매물55")).toBeTruthy()
    await userEvent.clear(screen.getByPlaceholderText("매물명·단지·주소·고객 검색"))
    expect(pageInfo()).toBe("1 / 2")

    // 상태 필터(Select) → 결과 없음 빈 상태
    await userEvent.click(screen.getByText("상태 전체"))
    await userEvent.click(await screen.findByRole("option", { name: "계약완료" }))
    expect(screen.getByText("0개")).toBeTruthy()
    expect(screen.getByText("매물이 없습니다")).toBeTruthy()
  })

  it("계약진행·계약완료 뷰는 상태 필터와 새 매물 버튼이 없고, 관심 뷰도 새 매물 버튼이 없다", async () => {
    mount("in-progress", [row(1, { status: "계약진행" })])
    expect(await screen.findByText("1개")).toBeTruthy()
    expect(screen.queryByText("상태 전체")).toBeNull()
    expect(screen.queryByText("새 매물")).toBeNull()
    expect(screen.queryByRole("group", { name: "색상 필터" })).toBeNull()
  })
})

describe("인라인 편집", () => {
  it("더블클릭 → 입력 → Enter 저장(만원 입력은 원으로), Esc 는 취소", async () => {
    mount("all", [row(1)])
    expect(await screen.findByText("12,000만원", { selector: "td div" })).toBeTruthy()
    let server = row(1) // 서버 상태: PATCH 를 반영하고 재조회에 돌려준다
    fetchMock.mockImplementation(async (_url: string, init: RequestInit) => {
      if (init.method === "PATCH") { server = { ...server, ...JSON.parse(init.body as string) }; return new Response(null, { status: 204 }) }
      return json(200, [server])
    })

    // Esc: 저장하지 않음
    await userEvent.dblClick(table().getByText("매물1"))
    const input = table().getByDisplayValue("매물1")
    await userEvent.clear(input)
    await userEvent.type(input, "버릴값{Escape}")
    expect(table().getByText("매물1")).toBeTruthy()
    expect(fetchMock.mock.calls.filter(([, i]) => i.method === "PATCH")).toHaveLength(0)

    // 금액: 만원 1.5 → 원 "15000"
    await userEvent.dblClick(table().getByText("12,000만원"))
    const money = table().getByDisplayValue("12000")
    await userEvent.clear(money)
    await userEvent.type(money, "1.5{Enter}")
    expect(table().getByText("1.5만원")).toBeTruthy()
    const patch = fetchMock.mock.calls.find(([, i]) => i.method === "PATCH")!
    expect(patch[0]).toBe("/api/v1/properties/p1")
    expect(JSON.parse(patch[1].body)).toEqual({ price: "15000" })

    // 텍스트: blur 로 저장
    await userEvent.dblClick(table().getByText("매물1"))
    const name = table().getByDisplayValue("매물1")
    await userEvent.clear(name)
    await userEvent.type(name, "새이름")
    await userEvent.click(document.body)
    const patches = fetchMock.mock.calls.filter(([, i]) => i.method === "PATCH")
    expect(JSON.parse(patches[1][1].body)).toEqual({ name: "새이름" })
    await waitFor(() => expect(table().getByText("새이름")).toBeTruthy())
  })
})

describe("관심 토글 낙관적 반영", () => {
  it("PATCH 가 실패하면 별을 되돌리고 오류 토스트를 띄운다", async () => {
    mount("all", [row(1)])
    await screen.findByText("1개")
    let release: (r: Response) => void = () => {}
    fetchMock.mockImplementation((_url: string, init: RequestInit) => {
      if (init.method === "PATCH") return new Promise<Response>((r) => { release = r })
      return Promise.resolve(json(200, [row(1)]))
    })
    const star = () => table().getByRole("button", { name: "관심" }).querySelector("svg")!
    expect(star().getAttribute("class")).not.toContain("fill-amber-400")
    await userEvent.click(table().getByRole("button", { name: "관심" }))
    expect(star().getAttribute("class")).toContain("fill-amber-400") // 즉시 반영
    expect(JSON.parse(fetchMock.mock.calls.find(([, i]) => i.method === "PATCH")![1].body)).toEqual({ isFavorite: true })
    release(json(500, { error: "저장하지 못했습니다" }))
    await waitFor(() => expect(star().getAttribute("class")).not.toContain("fill-amber-400"))
    expect(toast.error).toHaveBeenCalledWith("저장하지 못했습니다")
  })
})

describe("겹치는 낙관적 변경", () => {
  const starOf = (name: string) => table().getByText(name).closest("tr")!.querySelector('button[aria-label="관심"] svg')!
  const starred = (name: string) => (starOf(name).getAttribute("class") ?? "").includes("fill-amber-400")
  const clickStar = (name: string) => userEvent.click(within(table().getByText(name).closest("tr")!).getByRole("button", { name: "관심" }))

  it("PATCH 가 모두 실패하면 A·B 둘 다 원래대로 돌아가고 오류 토스트가 뜬다", async () => {
    mount("all", [row(2), row(1)])
    await screen.findByText("2개")
    const pending: ((r: Response) => void)[] = []
    fetchMock.mockImplementation((_url: string, init: RequestInit) => {
      if (init.method === "PATCH") return new Promise<Response>((r) => pending.push(r))
      return Promise.resolve(json(200, [row(2), row(1)]))
    })
    await clickStar("매물1")
    await clickStar("매물2")
    expect(starred("매물1") && starred("매물2")).toBe(true)
    pending.forEach((r) => r(json(500, { error: "저장하지 못했습니다" })))
    await waitFor(() => expect(starred("매물1")).toBe(false))
    await waitFor(() => expect(starred("매물2")).toBe(false))
    expect(toast.error).toHaveBeenCalledWith("저장하지 못했습니다")
  })

  it("A 가 성공해도 아직 진행 중인 B 의 낙관적 값은 유지된다", async () => {
    mount("all", [row(2), row(1)])
    await screen.findByText("2개")
    const pending = new Map<string, (r: Response) => void>()
    fetchMock.mockImplementation((url: string, init: RequestInit) => {
      if (init.method === "PATCH") return new Promise<Response>((r) => pending.set(url, r))
      return Promise.resolve(json(200, [row(2), row(1)])) // 서버는 아직 B 를 모른다
    })
    await clickStar("매물1")
    await clickStar("매물2")
    pending.get("/api/v1/properties/p1")!(new Response(null, { status: 204 }))
    await new Promise((r) => setTimeout(r, 50))
    expect(starred("매물2")).toBe(true)
    expect(fetchMock.mock.calls.filter(([, i]) => (i.method ?? "GET") === "GET")).toHaveLength(1) // 재조회 보류
    pending.get("/api/v1/properties/p2")!(new Response(null, { status: 204 }))
    await waitFor(() => expect(fetchMock.mock.calls.filter(([, i]) => (i.method ?? "GET") === "GET").length).toBeGreaterThan(1))
  })
})
