export function isTauri(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window
}

// 브라우저: 같은 출처의 /api/v1. 앱: 빌드 시점 설정값.
export const API_BASE: string = isTauri()
  ? (import.meta.env.VITE_API_BASE as string)
  : "/api/v1"

const STORE_FILE = "auth.json"
const TOKEN_KEY = "token"

// 플러그인은 Tauri 분기 안에서만 동적으로 불러온다(브라우저 빌드에 영향 없음).
async function openStore() {
  const { load } = await import("@tauri-apps/plugin-store")
  return load(STORE_FILE, { defaults: {} })
}

// 브라우저는 쿠키가 세션을 맡으므로 아무것도 저장하지 않는다.
export const tokenStore = {
  async get(): Promise<string | null> {
    if (!isTauri()) return null
    const store = await openStore()
    return (await store.get<string>(TOKEN_KEY)) ?? null
  },
  async set(t: string): Promise<void> {
    if (!isTauri()) return
    const store = await openStore()
    await store.set(TOKEN_KEY, t)
    await store.save()
  },
  async clear(): Promise<void> {
    if (!isTauri()) return
    const store = await openStore()
    await store.delete(TOKEN_KEY)
    await store.save()
  },
}
