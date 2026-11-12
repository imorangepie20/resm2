import { QueryClientProvider } from "@tanstack/react-query"
import { cleanup, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter, Route, Routes } from "react-router"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
const toast = vi.hoisted(() => ({ error: vi.fn(), success: vi.fn() }))
vi.mock("sonner", () => ({ toast }))

import { queryClient } from "@/lib/query-client"
import { EditPropertyPage } from "./edit-page"
import { NewPropertyPage } from "./new-page"

const fetchMock = vi.fn()
const json = (status: number, body: unknown) => new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } })

const property = (over: Record<string, unknown>) => ({
  id: "p1", isFavorite: false, colorTag: null, status: "진행", source: "수기", createdAt: "2026-11-12T00:00:00Z",
  name: "매물", tradeType: "A1", price: null, rentPrice: null, ...over,
})

function mount(path: string, row?: Record<string, unknown>) {
  fetchMock.mockImplementation(async (url: string, init: RequestInit) => {
    const m = init.method ?? "GET"
    if (m === "GET" && url === "/api/v1/properties/p1") return json(200, row)
    if (m === "PATCH") return new Response(null, { status: 204 })
    if (m === "POST") return json(201, { id: "new" })
    return json(200, [])
  })
  render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[path]}>
        <Routes>
          <Route path="/properties/new" element={<NewPropertyPage />} />
          <Route path="/properties/:id/edit" element={<EditPropertyPage />} />
          <Route path="/properties" element={<div>목록</div>} />
        </Routes>
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

const sent = (method: string) => {
  const call = fetchMock.mock.calls.find(([, i]) => i.method === method)
  return call ? JSON.parse(call[1].body as string) : undefined
}
const priceInput = () => screen.getByLabelText("가격") as HTMLInputElement

beforeEach(() => { vi.stubGlobal("fetch", fetchMock); fetchMock.mockReset(); toast.error.mockReset() })
afterEach(() => { cleanup(); vi.unstubAllGlobals(); queryClient.clear() })

describe("수정 폼 금액", () => {
  it("(a) 15000원은 1.5로 보이고, 손대지 않고 저장하면 price·rentPrice 키를 보내지 않는다", async () => {
    mount("/properties/p1/edit", property({ price: "15000", rentPrice: "500000" }))
    await waitFor(() => expect(priceInput().value).toBe("1.5"))
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await screen.findByText("목록")
    const body = sent("PATCH")
    expect("price" in body).toBe(false)
    expect("rentPrice" in body).toBe(false)
    expect(body.name).toBe("매물")
  })

  it("(b) 2.5로 고쳐 저장하면 price 25000", async () => {
    mount("/properties/p1/edit", property({ price: "15000" }))
    await waitFor(() => expect(priceInput().value).toBe("1.5"))
    await userEvent.clear(priceInput())
    await userEvent.type(priceInput(), "2.5")
    expect(priceInput().value).toBe("2.5")
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await screen.findByText("목록")
    expect(sent("PATCH").price).toBe("25000")
  })

  it("(c) 12345원은 1.2345로 보인다", async () => {
    mount("/properties/p1/edit", property({ price: "12345" }))
    await waitFor(() => expect(priceInput().value).toBe("1.2345"))
  })

  it("(d) 큰 값은 모든 자릿수가 보이고 손대지 않으면 보내지 않는다", async () => {
    mount("/properties/p1/edit", property({ price: "9007199254740993" }))
    await waitFor(() => expect(priceInput().value).toBe("900,719,925,474.0993"))
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await screen.findByText("목록")
    expect("price" in sent("PATCH")).toBe(false)
  })

  it("(e) 지우고 저장하면 price null", async () => {
    mount("/properties/p1/edit", property({ price: "15000" }))
    await waitFor(() => expect(priceInput().value).toBe("1.5"))
    await userEvent.clear(priceInput())
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await screen.findByText("목록")
    expect(sent("PATCH").price).toBeNull()
  })

  it("음수처럼 money.ts 가 왕복 못 하는 저장값도 손대지 않으면 저장을 막지 않는다", async () => {
    mount("/properties/p1/edit", property({ price: "-15000" }))
    await screen.findByLabelText("가격")
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await screen.findByText("목록")
    expect("price" in sent("PATCH")).toBe(false)
    expect(toast.error).not.toHaveBeenCalled()
  })

  it("만원 정수부 14자리 초과는 요청 전에 막는다", async () => {
    mount("/properties/p1/edit", property({ price: null }))
    await screen.findByLabelText("가격")
    await userEvent.type(priceInput(), "100000000000000")
    await userEvent.click(screen.getByRole("button", { name: "수정" }))
    await waitFor(() => expect(toast.error).toHaveBeenCalledWith("금액 형식이 올바르지 않습니다"))
    expect(sent("PATCH")).toBeUndefined()
  })
})

describe("등록 폼 금액", () => {
  it("(f) 12,000 입력 → price 120000000", async () => {
    mount("/properties/new")
    await userEvent.type(priceInput(), "12,000")
    expect(priceInput().value).toBe("12,000")
    await userEvent.click(screen.getByRole("button", { name: "등록" }))
    await screen.findByText("목록")
    expect(sent("POST").price).toBe("120000000")
  })
})
