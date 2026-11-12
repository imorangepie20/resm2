import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { setOnUnauthorized, api } from "./api"
import { login, logout, resetSession, type Me } from "./auth"
import { queryClient } from "./query-client"

const user: Me = { id: "u2", email: "b@example.com", name: "B", role: "agent", agencyId: "a1" }
const fetchMock = vi.fn()

function seed() {
  queryClient.setQueryData(["me"], { ...user, id: "u1" })
  queryClient.setQueryData(["properties", "all"], [{ id: "p1" }])
}

describe("session reset clears the whole query cache", () => {
  beforeEach(() => {
    vi.stubGlobal("fetch", fetchMock)
    fetchMock.mockReset()
    seed()
  })
  afterEach(() => {
    vi.unstubAllGlobals()
    queryClient.clear()
  })

  it("logout", async () => {
    fetchMock.mockResolvedValue(new Response(null, { status: 204 }))
    await logout()
    expect(queryClient.getQueryData(["properties", "all"])).toBeUndefined()
    expect(queryClient.getQueryData(["me"])).toBeNull()
  })

  it("onUnauthorized path", async () => {
    setOnUnauthorized(() => resetSession(null))
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ error: "x" }), { status: 401 }))
    await api("/properties").catch(() => {})
    expect(queryClient.getQueryData(["properties", "all"])).toBeUndefined()
    expect(queryClient.getQueryData(["me"])).toBeNull()
  })

  it("login", async () => {
    fetchMock.mockResolvedValue(new Response(JSON.stringify({ token: "t", user }), { status: 200 }))
    const me = await login("b@example.com", "pw")
    expect(me).toEqual(user)
    expect(queryClient.getQueryData(["properties", "all"])).toBeUndefined()
    expect(queryClient.getQueryData(["me"])).toEqual(user)
  })
})
