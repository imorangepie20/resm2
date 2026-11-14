// 매물 거래유형 → 매물 소유자 측 당사자 역할(매물의 1차 고객은 보통 내놓은 사람).
// 매매=매도인, 전세/월세/단기임대=임대인. 상대측(매수인/임차인)은 매물 화면에서 직접 추가.
export function ownerRoleFromTrade(tradeType?: string | null): string {
  if (tradeType === "A1") return "매도인";
  if (tradeType === "B1" || tradeType === "B2" || tradeType === "B3") return "임대인";
  return "매도인";
}
