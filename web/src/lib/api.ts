import { API_BASE, isTauri, tokenStore } from "./platform"

export class ApiError extends Error {
  status: number
  constructor(status: number, message: string) {
    super(message)
    this.name = "ApiError"
    this.status = status
  }
}

export type ApiInit = {
  method?: string
  body?: unknown
  // true 면 401 에서 전역 onUnauthorized 를 부르지 않는다(로그인 실패·/auth/me 처럼 401 이 정상 응답인 호출).
  skipUnauthorized?: boolean
}

let onUnauthorized: () => void = () => {}

export function setOnUnauthorized(fn: () => void) {
  onUnauthorized = fn
}

export async function api<T>(path: string, init: ApiInit = {}): Promise<T> {
  const headers: Record<string, string> = {}
  if (init.body !== undefined) headers["Content-Type"] = "application/json"

  const requestInit: RequestInit = { method: init.method ?? "GET", headers }
  if (init.body !== undefined) requestInit.body = JSON.stringify(init.body)

  if (isTauri()) {
    const token = await tokenStore.get()
    if (token) headers.Authorization = `Bearer ${token}`
  } else {
    requestInit.credentials = "same-origin"
  }

  let res: Response
  try {
    res = await fetch(`${API_BASE}${path}`, requestInit)
  } catch {
    throw new ApiError(0, "서버에 연결할 수 없습니다")
  }

  if (res.ok) {
    if (res.status === 204) return undefined as T
    return (await res.json()) as T
  }

  let message: string | null = null
  try {
    const body = (await res.json()) as { error?: unknown }
    if (typeof body.error === "string") message = body.error
  } catch {
    // 본문이 JSON 이 아니다.
  }
  // 서버는 오류를 항상 {error} JSON 으로 준다. 본문 없는 5xx 는 앞단 프록시가 서버에 닿지 못했다는 뜻이다.
  if (message === null && res.status >= 500) throw new ApiError(0, "서버에 연결할 수 없습니다")
  message ??= `요청에 실패했습니다 (${res.status})`

  if (res.status === 401) {
    await tokenStore.clear()
    if (!init.skipUnauthorized) onUnauthorized()
  }
  throw new ApiError(res.status, message)
}
