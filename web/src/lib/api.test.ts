import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

const store = vi.hoisted(() => {
  const data = new Map<string, string>()
  return {
    data,
    get: vi.fn(async (k: string) => data.get(k)),
    set: vi.fn(async (k: string, v: string) => void data.set(k, v)),
    delete: vi.fn(async (k: string) => void data.delete(k)),
    save: vi.fn(async () => {}),
  }
})
vi.mock("@tauri-apps/plugin-store", () => ({ load: vi.fn(async () => store) }))

import { ApiError, api, setOnUnauthorized } from "./api"
import { tokenStore } from "./platform"

function jsonResponse(status: number, body?: unknown) {
  return new Response(body === undefined ? null : JSON.stringify(body), {
    status,
    headers: body === undefined ? {} : { "content-type": "application/json" },
  })
}

// 거부(reject)된 ApiError 를 돌려준다. 성공하면 테스트를 실패시킨다.
function rejection(p: Promise<unknown>) {
  return p.then(
    () => { throw new Error("resolved unexpectedly") },
    (e) => e as ApiError,
  )
}

describe("api", () => {
  const fetchMock = vi.fn()
  const onUnauthorized = vi.fn()

  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock)
    fetchMock.mockReset()
    onUnauthorized.mockReset()
    setOnUnauthorized(onUnauthorized)
  })
  afterEach(() => {
    vi.unstubAllGlobals()
    delete (window as unknown as Record<string, unknown>).__TAURI_INTERNALS__
    store.data.clear()
  })

  it("returns parsed json on 200", async () => {
    fetchMock.mockResolvedValue(jsonResponse(200, { ok: 1 }))
    await expect(api<{ ok: number }>("/auth/me")).resolves.toEqual({ ok: 1 })
    const [url, init] = fetchMock.mock.calls[0]
    expect(url).toBe("/api/v1/auth/me")
    expect(init.credentials).toBe("same-origin")
    expect(init.headers.Authorization).toBeUndefined()
  })

  it("sends json body with method", async () => {
    fetchMock.mockResolvedValue(jsonResponse(200, {}))
    await api("/auth/login", { method: "POST", body: { a: 1 } })
    const [, init] = fetchMock.mock.calls[0]
    expect(init.method).toBe("POST")
    expect(init.body).toBe('{"a":1}')
    expect(init.headers["Content-Type"]).toBe("application/json")
  })

  it("throws ApiError with server message on 400", async () => {
    fetchMock.mockResolvedValue(jsonResponse(400, { error: "상호명을 입력하세요" }))
    const err = await rejection(api("/auth/signup", { method: "POST", body: {} }))
    expect(err).toBeInstanceOf(ApiError)
    expect(err.message).toBe("상호명을 입력하세요")
    expect(err.status).toBe(400)
    expect(onUnauthorized).not.toHaveBeenCalled()
  })

  it("401 clears token and calls onUnauthorized", async () => {
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    await tokenStore.set("t")
    fetchMock.mockResolvedValue(jsonResponse(401, { error: "로그인이 필요합니다" }))
    const err = await rejection(api("/properties"))
    expect(err.status).toBe(401)
    expect(store.delete).toHaveBeenCalledWith("token")
    expect(onUnauthorized).toHaveBeenCalledTimes(1)
  })

  it("401 with skipUnauthorized throws without calling onUnauthorized", async () => {
    fetchMock.mockResolvedValue(jsonResponse(401, { error: "이메일 또는 비밀번호가 올바르지 않습니다" }))
    const err = await rejection(api("/auth/login", { method: "POST", body: {}, skipUnauthorized: true }))
    expect(err).toBeInstanceOf(ApiError)
    expect(err.status).toBe(401)
    expect(err.message).toBe("이메일 또는 비밀번호가 올바르지 않습니다")
    expect(onUnauthorized).not.toHaveBeenCalled()
  })

  it('network failure becomes ApiError status 0 with "서버에 연결할 수 없습니다"', async () => {
    fetchMock.mockRejectedValue(new TypeError("Failed to fetch"))
    const err = await rejection(api("/auth/me"))
    expect(err).toBeInstanceOf(ApiError)
    expect(err.status).toBe(0)
    expect(err.message).toBe("서버에 연결할 수 없습니다")
  })

  it("502 from a proxy (non-JSON body) becomes status 0 server-unreachable", async () => {
    fetchMock.mockResolvedValue(new Response("Bad Gateway", { status: 502 }))
    const err = await rejection(api("/auth/me"))
    expect(err.status).toBe(0)
    expect(err.message).toBe("서버에 연결할 수 없습니다")
  })

  it("204 returns undefined", async () => {
    fetchMock.mockResolvedValue(jsonResponse(204))
    await expect(api("/auth/logout", { method: "POST" })).resolves.toBeUndefined()
  })

  it("in Tauri sends Bearer token and no cookie credentials", async () => {
    ;(window as unknown as Record<string, unknown>).__TAURI_INTERNALS__ = {}
    await tokenStore.set("abc")
    fetchMock.mockResolvedValue(jsonResponse(200, {}))
    await api("/auth/me")
    const [, init] = fetchMock.mock.calls[0]
    expect(init.headers.Authorization).toBe("Bearer abc")
    expect(init.credentials).toBeUndefined()
  })
})
