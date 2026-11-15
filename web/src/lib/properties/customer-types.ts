// 서버와 동일하게 허용된 고객 유형
export const CUSTOMER_TYPES = ["매도인", "매수인", "임대인", "임차인", "단순방문"] as const
export const PARTY_TYPES = ["매도인", "매수인", "임대인", "임차인"] as const
export const GENDERS = ["남", "여", "미지정"] as const

export type CustomerType = typeof CUSTOMER_TYPES[number]
export type PartyType = typeof PARTY_TYPES[number]
export type Gender = typeof GENDERS[number]