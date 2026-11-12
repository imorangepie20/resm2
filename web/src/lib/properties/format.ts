// 폼 입력 표시 포맷 헬퍼(순수). 저장값과 표시값 변환.

/** 숫자 문자열에 천단위 콤마. 빈 값/비숫자는 그대로 빈 문자열. */
export function groupDigits(s: string): string {
  const d = String(s).replace(/[^\d]/g, "");
  return d === "" ? "" : Number(d).toLocaleString("en-US");
}

/** 숫자만 남김. */
export function stripDigits(s: string): string {
  return String(s).replace(/[^\d]/g, "");
}

/** 전화번호 자동 하이픈(010-0000-0000 / 02-000-0000 등 11·10자리). */
export function formatTel(s: string): string {
  const d = String(s).replace(/[^\d]/g, "").slice(0, 11);
  if (d.length < 4) return d;
  if (d.length < 8) return `${d.slice(0, 3)}-${d.slice(3)}`;
  if (d.length === 10) return `${d.slice(0, 3)}-${d.slice(3, 6)}-${d.slice(6)}`;
  return `${d.slice(0, 3)}-${d.slice(3, 7)}-${d.slice(7)}`;
}

/** 저장값(YYYYMMDD) → date input 값(YYYY-MM-DD). 8자리 아니면 빈 값. */
export function toDateInput(s: string): string {
  const d = String(s).replace(/[^\d]/g, "");
  return d.length === 8 ? `${d.slice(0, 4)}-${d.slice(4, 6)}-${d.slice(6, 8)}` : "";
}

/** date input 값(YYYY-MM-DD) → 저장값(YYYYMMDD). */
export function fromDateInput(s: string): string {
  return String(s).replace(/[^\d]/g, "");
}
