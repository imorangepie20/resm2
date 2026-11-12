// 금액 변환의 단일 위치. API·DB 는 원(정수 문자열), 화면은 만원(입력·표시).
// 큰 값도 잃지 않도록 Number 곱셈·나눗셈 없이 문자열 자릿수 연산만 한다.

const INVALID = "금액 형식이 올바르지 않습니다"

/** 원 문자열 → 만원 문자열. "15000" → "1.5", null → "". 음수는 원본이 저장했을 수 있어 부호를 유지한다. */
export function wonToManwon(won: string | null): string {
  if (won == null) return ""
  const m = /^(-?)(\d+)$/.exec(won)
  if (!m) throw new Error(INVALID)
  const digits = m[2].replace(/^0+(?=\d)/, "").padStart(5, "0")
  const int = digits.slice(0, -4).replace(/^0+(?=\d)/, "")
  const frac = digits.slice(-4).replace(/0+$/, "")
  const body = frac ? `${int}.${frac}` : int
  return body === "0" ? "0" : m[1] + body
}

/** 만원 입력 → 원 문자열. 콤마·공백 제거, 빈 값은 null. 소수 4자리를 넘는 부분은 원본처럼 버린다. */
export function manwonToWon(input: string): string | null {
  const s = input.replace(/[,\s]/g, "")
  if (s === "") return null
  const m = /^(\d*)(?:\.(\d*))?$/.exec(s)
  if (!m || (m[1] === "" && !m[2])) throw new Error(INVALID)
  const frac = (m[2] ?? "").slice(0, 4).padEnd(4, "0")
  return (m[1] + frac).replace(/^0+(?=\d)/, "")
}

/** 원 문자열 → 화면 표시용 만원(정수부 천단위 콤마). null → "". */
export function formatManwon(won: string | null): string {
  const v = wonToManwon(won)
  if (v === "") return ""
  const [int, frac] = v.split(".")
  const grouped = int.replace(/\B(?=(\d{3})+(?!\d))/g, ",")
  return frac ? `${grouped}.${frac}` : grouped
}
