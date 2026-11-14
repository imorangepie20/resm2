// 월 그리드·한국어 명칭·날짜키 (순수). month는 0-indexed(0=1월).
export const MONTH_NAMES_KO = ["1월", "2월", "3월", "4월", "5월", "6월", "7월", "8월", "9월", "10월", "11월", "12월"];
export const WEEKDAYS_KO = ["일", "월", "화", "수", "목", "금", "토"];

export type GridDay = { year: number; month: number; day: number; isCurrentMonth: boolean };

const daysInMonth = (year: number, month: number) => new Date(year, month + 1, 0).getDate();
const firstWeekday = (year: number, month: number) => new Date(year, month, 1).getDay();

export function buildMonthGrid(year: number, month: number): GridDay[] {
  const first = firstWeekday(year, month);
  const total = daysInMonth(year, month);
  const grid: GridDay[] = [];
  const prevM = month === 0 ? 11 : month - 1;
  const prevY = month === 0 ? year - 1 : year;
  const prevDays = daysInMonth(prevY, prevM);
  for (let i = first - 1; i >= 0; i--) grid.push({ year: prevY, month: prevM, day: prevDays - i, isCurrentMonth: false });
  for (let d = 1; d <= total; d++) grid.push({ year, month, day: d, isCurrentMonth: true });
  const nextM = month === 11 ? 0 : month + 1;
  const nextY = month === 11 ? year + 1 : year;
  let nd = 1;
  while (grid.length < 42) grid.push({ year: nextY, month: nextM, day: nd++, isCurrentMonth: false });
  return grid;
}

const pad = (n: number) => String(n).padStart(2, "0");
export const ymd = (year: number, month: number, day: number) => `${year}${pad(month + 1)}${pad(day)}`;
export const monthPrefix = (year: number, month: number) => `${year}${pad(month + 1)}`;
