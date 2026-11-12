import { describe, expect, it } from "vitest"
import { formatManwon, formatManwonInput, manwonToWon, wonToManwon } from "./money"

describe("money (brief)", () => {
  it("원 ↔ 만원", () => {
    expect(wonToManwon("120000000")).toBe("12000")
    expect(wonToManwon("15000")).toBe("1.5")
    expect(wonToManwon(null)).toBe("")
    expect(manwonToWon("12,000")).toBe("120000000")
    expect(manwonToWon("1.5")).toBe("15000")
    expect(manwonToWon("")).toBeNull()
    expect(manwonToWon("900719925474.0993")).toBe("9007199254740993")
    expect(() => manwonToWon("abc")).toThrow("금액 형식이 올바르지 않습니다")
  })
})

describe("wonToManwon", () => {
  it("만원 배수가 아니면 소수 4자리까지 그대로", () => {
    expect(wonToManwon("12345")).toBe("1.2345")
    expect(wonToManwon("1")).toBe("0.0001")
    expect(wonToManwon("10001")).toBe("1.0001")
  })
  it("끝의 0은 지운다", () => {
    expect(wonToManwon("12300")).toBe("1.23")
    expect(wonToManwon("10000")).toBe("1")
    expect(wonToManwon("0")).toBe("0")
  })
  it("큰 값도 손실 없음", () => {
    expect(wonToManwon("12345678901230000")).toBe("1234567890123")
    expect(wonToManwon("9007199254740993")).toBe("900719925474.0993")
  })
  it("음수(원본이 저장했을 수 있음)는 부호 유지", () => {
    expect(wonToManwon("-15000")).toBe("-1.5")
  })
  it("숫자 문자열이 아니면 오류", () => {
    expect(() => wonToManwon("abc")).toThrow("금액 형식이 올바르지 않습니다")
    expect(() => wonToManwon("1.5")).toThrow("금액 형식이 올바르지 않습니다")
  })
})

describe("manwonToWon", () => {
  it("공백·콤마 제거, 공백뿐이면 null", () => {
    expect(manwonToWon(" 1 2,000 ")).toBe("120000000")
    expect(manwonToWon("   ")).toBeNull()
  })
  it("소수 4자리 넘는 부분은 원본처럼 버린다(Math.trunc)", () => {
    expect(manwonToWon("1.23456")).toBe("12345")
  })
  it("앞자리 0·소수점만 있는 형태", () => {
    expect(manwonToWon("0.5")).toBe("5000")
    expect(manwonToWon(".5")).toBe("5000")
    expect(manwonToWon("1.")).toBe("10000")
    expect(manwonToWon("007")).toBe("70000")
    expect(manwonToWon("0")).toBe("0")
  })
  it("음수·기호·지수 표기는 오류", () => {
    for (const bad of ["-1", "+1", "1e3", "1.2.3", ".", "1만"]) {
      expect(() => manwonToWon(bad)).toThrow("금액 형식이 올바르지 않습니다")
    }
  })
})

describe("formatManwon", () => {
  it("정수부 천단위 콤마, 소수 유지, null은 빈 문자열", () => {
    expect(formatManwon("120000000")).toBe("12,000")
    expect(formatManwon("12345678901230000")).toBe("1,234,567,890,123")
    expect(formatManwon("123456789")).toBe("12,345.6789")
    expect(formatManwon("-15000")).toBe("-1.5")
    expect(formatManwon(null)).toBe("")
  })
})

describe("formatManwonInput — 입력 중인 만원 표시", () => {
  it("콤마·소수점·소수 4자리", () => {
    expect(formatManwonInput("12000")).toBe("12,000")
    expect(formatManwonInput("12,000")).toBe("12,000")
    expect(formatManwonInput("1.5")).toBe("1.5")
    expect(formatManwonInput("1.")).toBe("1.")
    expect(formatManwonInput("1.50")).toBe("1.50")
    expect(formatManwonInput("1.23456")).toBe("1.2345")
    expect(formatManwonInput("1.2.3")).toBe("1.23")
    expect(formatManwonInput("abc12")).toBe("12")
    expect(formatManwonInput("")).toBe("")
    expect(formatManwonInput("900719925474.0993")).toBe("900,719,925,474.0993")
  })
  it("정책: 정수부 앞자리 0은 지우고(하나는 남김), 정수부 없이 점으로 시작하면 0을 붙인다", () => {
    expect(formatManwonInput("0012")).toBe("12")
    expect(formatManwonInput("000")).toBe("0")
    expect(formatManwonInput("0.5")).toBe("0.5")
    expect(formatManwonInput(".5")).toBe("0.5")
  })
})

describe("manwonToWon 자릿수 상한", () => {
  it("만원 정수부 14자리까지, 넘으면 오류", () => {
    expect(manwonToWon("99999999999999.9999")).toBe("999999999999999999")
    expect(() => manwonToWon("100000000000000")).toThrow("금액 형식이 올바르지 않습니다")
    expect(manwonToWon("000000000000001")).toBe("10000")
  })
})
