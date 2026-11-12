import { afterEach, describe, expect, it, vi } from "vitest"

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
const load = vi.hoisted(() => vi.fn())
vi.mock("@tauri-apps/plugin-store", () => ({ load }))

import { API_BASE, isTauri, tokenStore } from "./platform"

const w = window as unknown as Record<string, unknown>

describe("platform", () => {
  afterEach(() => {
    delete w.__TAURI_INTERNALS__
    store.data.clear()
    load.mockReset()
  })

  it("browser: not tauri, same-origin API_BASE, tokenStore is a no-op", async () => {
    load.mockResolvedValue(store)
    expect(isTauri()).toBe(false)
    expect(API_BASE).toBe("/api/v1")
    await tokenStore.set("x")
    expect(await tokenStore.get()).toBeNull()
    await tokenStore.clear()
    expect(load).not.toHaveBeenCalled()
  })

  it("tauri: tokenStore round-trips through auth.json key token", async () => {
    load.mockResolvedValue(store)
    w.__TAURI_INTERNALS__ = {}
    expect(isTauri()).toBe(true)
    await tokenStore.set("abc")
    expect(load.mock.calls[0][0]).toBe("auth.json")
    expect(store.set).toHaveBeenCalledWith("token", "abc")
    expect(await tokenStore.get()).toBe("abc")
    await tokenStore.clear()
    expect(store.delete).toHaveBeenCalledWith("token")
    expect(await tokenStore.get()).toBeNull()
  })
})
