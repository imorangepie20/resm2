// 매물 일정 날짜 → 캘린더 파생 항목(읽기전용). YYYYMMDD 값이 있는 필드만.
export type PropForEvents = { id: string; name: string | null; complexName: string | null } & Record<string, unknown>;
export type PropertyDateEvent = { date: string; label: string; propertyId: string; field: string };

export const PROPERTY_DATE_FIELDS: { key: string; label: string }[] = [
  { key: "contractHopeDate", label: "계약희망일" },
  { key: "contractDate", label: "계약일" },
  { key: "interim1Date", label: "중도금1" },
  { key: "interim2Date", label: "중도금2" },
  { key: "balanceDate", label: "잔금일" },
  { key: "moveInHopeDate", label: "입주희망일" },
  { key: "moveInDate", label: "입주일" },
];

export function propertyCalendarEvents(properties: PropForEvents[]): PropertyDateEvent[] {
  const out: PropertyDateEvent[] = [];
  for (const p of properties) {
    const name = (p.name || p.complexName || "매물") as string;
    for (const f of PROPERTY_DATE_FIELDS) {
      const v = p[f.key];
      if (typeof v === "string" && /^\d{8}$/.test(v)) {
        out.push({ date: v, label: `${name} ${f.label}`, propertyId: p.id, field: f.key });
      }
    }
  }
  return out;
}
