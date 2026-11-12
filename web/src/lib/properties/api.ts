// 매물 API 호출의 유일한 위치. 서버는 쓰기 본문의 타입을 엄격히 검사하므로
// 화면이 들고 있는 문자열 값을 여기서 fields.ts 의 필드 타입대로 바꿔 보낸다.
import { api } from "@/lib/api"
import type { ColorTagValue } from "@/lib/properties/color-tags"
import { PROPERTY_FIELDS, coerceField, formMeta, type PropertyField } from "@/lib/properties/fields"
import { manwonToWon } from "@/lib/properties/money"

export type PropertyView = "all" | "favorites" | "in-progress" | "contracted"

type Text = string | null
export type PropertyRow = {
  id: string
  isFavorite: boolean
  colorTag: ColorTagValue | null
  status: string
  source: string
  createdAt: string
  name: Text; articleNo: Text; complexName: Text; address: Text; realEstateType: Text; tradeType: Text
  heating: Text; customerName: Text; customerPhone: Text; partnerName: Text; partnerPhone: Text
  partnerManager: Text; manager: Text
  contractHopeDate: Text; contractDate: Text; moveInHopeDate: Text; moveInDate: Text
  interim1Date: Text; interim2Date: Text; balanceDate: Text; approvalDate: Text
  note: Text; memo: Text
  siteArea: number | null; areaExclusive: number | null; areaSupply: number | null
  landArea: number | null; buildingArea: number | null; area: number | null
  totalHouseholds: number | null; parkingCount: number | null
  isPreSale: boolean | null
  /** 원 단위 정수 문자열 */
  price: Text
  /** 원 단위 정수 문자열 */
  rentPrice: Text
}

export type ApiValue = string | number | boolean | null
export type PropertyInput = Record<string, ApiValue>

export const listProperties = (view: PropertyView) => api<PropertyRow[]>(`/properties?view=${view}`)
export const getProperty = (id: string) => api<PropertyRow>(`/properties/${encodeURIComponent(id)}`)
export const createProperty = (input: PropertyInput) => api<{ id: string }>("/properties", { method: "POST", body: input })
export const updateProperty = (id: string, patch: PropertyInput) =>
  api<void>(`/properties/${encodeURIComponent(id)}`, { method: "PATCH", body: patch })
export const deleteProperties = (ids: string[]) => api<{ count: number }>("/properties/delete", { method: "POST", body: { ids } })

/** 화면 문자열 값 → API 값. 빈 값은 null(비움). money 는 만원 입력을 원 문자열로(형식 오류면 throw). */
export function toApiValue(field: PropertyField, raw: string): ApiValue {
  switch (field.type) {
    case "money": return manwonToWon(raw)
    case "number":
    case "area": return coerceField(field.type, raw)
    case "bool": return raw === "true"
    default: return raw === "" ? null : raw
  }
}

/** 폼 값 전체 → 등록·수정 본문. 폼에서 숨긴 필드(source)는 보내지 않고, 빈 status 는 생략한다(NOT NULL). */
export function buildPropertyBody(values: Record<string, string>): PropertyInput {
  const body: PropertyInput = {}
  for (const f of PROPERTY_FIELDS) {
    if (formMeta(f).formHidden) continue
    const v = toApiValue(f, values[f.key] ?? "")
    if (f.key === "status" && v == null) continue
    body[f.key] = v
  }
  return body
}
