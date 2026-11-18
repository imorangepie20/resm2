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
  console.log("[api] request:", path, init.method ?? "GET")
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
    console.log("[api] fetch completed:", res.status, res.ok)
  } catch (e) {
    console.log("[api] fetch error:", e)
    throw new ApiError(0, "서버에 연결할 수 없습니다")
  }

  if (res.ok) {
    // 204 No Content 또는 본문 없는 200 응답 처리
    const contentLength = res.headers.get("content-length")
    console.log("[api] response ok:", res.status, "content-length:", contentLength)
    if (res.status === 204 || contentLength === "0") return undefined as T
    const text = await res.text()
    console.log("[api] response text:", text)
    if (!text) return undefined as T
    return JSON.parse(text) as T
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

// ─── 계약 ──────────────────────────────────────────────
export type ChecklistItem = {
  id: string
  label: string
  party: string
  kind: "서류" | "처리" | "신고" | "세금"
  required: boolean
}
export type RequiredField = { key: string; label: string; filled: boolean }
export type Progress = { done: number; total: number; complete: boolean }
export type ContractData = {
  property: PropertyDto
  checklist: ChecklistItem[]
  checked: Record<string, string>
  requiredFields: RequiredField[]
  progress: Progress
  status: string
}
export type PropertyDto = {
  id: string
  isFavorite: boolean
  colorTag: string | null
  status: string
  source: string
  createdAt: string
  name: string | null
  articleNo: string | null
  complexName: string | null
  address: string | null
  realEstateType: string | null
  tradeType: string | null
  heating: string | null
  customerName: string | null
  customerPhone: string | null
  partnerName: string | null
  partnerPhone: string | null
  partnerManager: string | null
  manager: string | null
  contractHopeDate: string | null
  contractDate: string | null
  moveInHopeDate: string | null
  moveInDate: string | null
  interim1Date: string | null
  interim2Date: string | null
  balanceDate: string | null
  approvalDate: string | null
  note: string | null
  memo: string | null
  siteArea: number | null
  areaExclusive: number | null
  areaSupply: number | null
  landArea: number | null
  buildingArea: number | null
  area: number | null
  totalHouseholds: number | null
  parkingCount: number | null
  isPreSale: boolean | null
  price: string | null
  rentPrice: string | null
  dealAmount: string | null
  contractChecklist: Record<string, string> | null
}

export async function getContract(propertyId: string): Promise<ContractData> {
  return api(`/contract/${propertyId}`)
}
export async function toggleChecklist(propertyId: string, itemId: string, checked: boolean): Promise<void> {
  await api(`/contract/${propertyId}/checklist/${itemId}`, { method: "PATCH", body: { checked } })
}
export async function checklistAll(propertyId: string, checked: boolean): Promise<void> {
  await api(`/contract/${propertyId}/checklist/all`, { method: "POST", body: { checked } })
}
export async function startContract(propertyId: string): Promise<ContractData> {
  return api(`/contract/${propertyId}/start`, { method: "POST" })
}
export async function completeContract(propertyId: string): Promise<ContractData> {
  return api(`/contract/${propertyId}/complete`, { method: "POST" })
}

// ─── 양식 인쇄 ──────────────────────────────────────────
export type FormSummary = { id: string; label: string }
export async function getForms(propertyId: string): Promise<FormSummary[]> {
  return api(`/forms/${propertyId}`)
}
export async function getFormHtml(propertyId: string, formId: string): Promise<string> {
  const res = await fetch(`${API_BASE}/forms/${propertyId}/${formId}`, {
    headers: isTauri() ? { Authorization: `Bearer ${await tokenStore.get()}` } : undefined,
    credentials: isTauri() ? undefined : "same-origin",
  })
  if (!res.ok) throw new ApiError(res.status, "양식을 불러올 수 없습니다")
  return res.text()
}

// ─── 고객 ───────────────────────────────────────────────
export type CustomerRow = {
  id: string
  types: string[]
  name: string
  zipcode: string | null
  phone: string | null
  address: string | null
  email: string | null
  gender: string | null
  memo: string | null
  propertyId: string | null
  propertyName: string | null
  parties: { propertyId: string; propertyLabel: string; role: string }[]
  updatedAt: string
}
export type CustomerInput = {
  name: string
  zipcode?: string | null
  phone?: string | null
  address?: string | null
  email?: string | null
  gender?: string | null
  memo?: string | null
  types?: string[]
  propertyId?: string | null
}
export async function listCustomers(q?: string, type?: string): Promise<CustomerRow[]> {
  const params = new URLSearchParams()
  if (q) params.set("q", q)
  if (type) params.set("type", type)
  return api(`/customers?${params.toString()}`)
}
export async function getCustomer(id: string): Promise<CustomerRow> {
  return api(`/customers/${id}`)
}
export async function createCustomer(input: CustomerInput): Promise<{ id: string }> {
  return api("/customers", { method: "POST", body: input })
}
export async function updateCustomer(id: string, input: CustomerInput): Promise<void> {
  console.log("[updateCustomer] called:", id, input)
  try {
    await api(`/customers/${id}`, { method: "PATCH", body: input })
    console.log("[updateCustomer] completed successfully")
  } catch (e) {
    console.log("[updateCustomer] error:", e)
    throw e
  }
}
export async function deleteCustomer(id: string): Promise<void> {
  console.log("[deleteCustomer] called:", id)
  try {
    await api(`/customers/${id}`, { method: "DELETE" })
    console.log("[deleteCustomer] completed successfully")
  } catch (e) {
    console.log("[deleteCustomer] error:", e)
    throw e
  }
}

// ─── 당사자 ─────────────────────────────────────────────
export type PartyRow = { id: string; customerId: string; name: string; phone: string | null; role: string }
export async function listParties(propertyId: string): Promise<PartyRow[]> {
  return api(`/properties/${propertyId}/parties`)
}
export async function addParty(propertyId: string, role: string, name: string, phone: string): Promise<{ partyId: string; customerId: string }> {
  return api(`/properties/${propertyId}/parties`, { method: "POST", body: { role, name, phone } })
}
export async function removeParty(propertyId: string, partyId: string): Promise<void> {
  await api(`/properties/${propertyId}/parties/${partyId}`, { method: "DELETE" })
}

// ─── 일정 ───────────────────────────────────────────────
export type EventRow = {
  id: string
  title: string
  date: string
  startTime: string | null
  category: string
  memo: string | null
  propertyId: string | null
  propertyLabel: string | null
  customerId: string | null
  customerLabel: string | null
}
export type PropertyDateEvent = { date: string; label: string; propertyId: string; field: string }
export type CalendarOption = { id: string; label: string }
export type CalendarData = {
  events: EventRow[]
  propertyDates: PropertyDateEvent[]
  properties: CalendarOption[]
  customers: CalendarOption[]
}
export type EventInput = {
  title: string
  date: string
  startTime?: string | null
  category: string
  memo?: string | null
  propertyId?: string | null
  customerId?: string | null
}
export async function loadCalendar(year: number, month: number): Promise<CalendarData> {
  return api(`/events?year=${year}&month=${month}`)
}
export async function createEvent(input: EventInput): Promise<{ id: string }> {
  return api("/events", { method: "POST", body: input })
}
export async function updateEvent(id: string, input: EventInput): Promise<void> {
  await api(`/events/${id}`, { method: "PATCH", body: input })
}
export async function deleteEvent(id: string): Promise<void> {
  await api(`/events/${id}`, { method: "DELETE" })
}
