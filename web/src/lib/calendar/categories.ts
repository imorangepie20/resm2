// 일정 유형·색 단일정의. 색은 Tailwind 토큰 클래스(임의 hex 금지).
export type CalendarCategory = "미팅" | "임장" | "계약" | "기타";

export const CALENDAR_CATEGORIES: { value: CalendarCategory; dot: string; text: string }[] = [
  { value: "미팅", dot: "bg-blue-500", text: "text-blue-600" },
  { value: "임장", dot: "bg-emerald-500", text: "text-emerald-600" },
  { value: "계약", dot: "bg-amber-500", text: "text-amber-600" },
  { value: "기타", dot: "bg-slate-400", text: "text-slate-500" },
];

export const PROPERTY_EVENT_STYLE = { dot: "bg-red-500", text: "text-red-600" };

export function categoryStyle(value: string): { dot: string; text: string } {
  return CALENDAR_CATEGORIES.find((c) => c.value === value) ?? CALENDAR_CATEGORIES[3];
}
