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
  if (m[1].replace(/^0+/, "").length > 14) throw new Error(INVALID) // 원 18자리(서버 상한) = 만원 정수 14자리
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

/** 사용자가 입력 중인 만원 값 → 입력란 표시 문자열. 숫자·점 외 문자는 버리고, 점은 하나만, 소수는 4자리(=1원)까지.
 *  정수부는 천단위 콤마, 앞자리 0은 지운다(하나는 남김). 입력 중인 끝의 점·0("1.", "1.50")은 그대로 둔다. */
export function formatManwonInput(raw: string): string {
  const s = raw.replace(/[^\d.]/g, "")
  if (s === "") return ""
  const dot = s.indexOf(".")
  const intDigits = (dot < 0 ? s : s.slice(0, dot)).replace(/^0+(?=\d)/, "")
  const int = (intDigits || "0").replace(/\B(?=(\d{3})+(?!\d))/g, ",")
  if (dot < 0) return int
  return `${int}.${s.slice(dot + 1).replace(/\./g, "").slice(0, 4)}`
}
