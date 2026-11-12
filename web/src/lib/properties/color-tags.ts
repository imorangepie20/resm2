// 매물 색상 태그(관리자 특성) — 전체 매물 메뉴에서만 사용. dot=스와치/필터 점, row=행 배경 틴트.
// 클래스는 전체 리터럴로 둬야 Tailwind가 인식한다(동적 조합 금지).
export type ColorTagValue = "purple" | "blue" | "orange" | "gray";

export const COLOR_TAGS: { value: ColorTagValue; label: string; dot: string; row: string }[] = [
  { value: "purple", label: "보라", dot: "bg-violet-600", row: "bg-violet-200 dark:bg-violet-900/70" },
  { value: "blue", label: "파랑", dot: "bg-blue-600", row: "bg-blue-200 dark:bg-blue-900/70" },
  { value: "orange", label: "주황", dot: "bg-orange-600", row: "bg-orange-200 dark:bg-orange-900/70" },
  { value: "gray", label: "회색", dot: "bg-gray-500", row: "bg-gray-300 dark:bg-gray-800/70" },
];

export const COLOR_TAG_MAP: Record<string, { value: ColorTagValue; label: string; dot: string; row: string }> =
  Object.fromEntries(COLOR_TAGS.map((c) => [c.value, c]));

export const COLOR_TAG_VALUES = COLOR_TAGS.map((c) => c.value);
