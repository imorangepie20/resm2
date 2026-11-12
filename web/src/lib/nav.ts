import type { LucideIcon } from "lucide-react";
import { ClipboardList, Star, CircleDashed, CircleCheck } from "lucide-react";

export type NavItem = { title: string; href: string; icon?: LucideIcon };
export type NavGroup = { label: string; items: NavItem[] };

export const navGroups: NavGroup[] = [
  {
    label: "매물 관리",
    items: [
      { title: "전체 매물", href: "/properties", icon: ClipboardList },
      { title: "관심 매물", href: "/properties/favorites", icon: Star },
      { title: "계약 진행", href: "/properties/progress", icon: CircleDashed },
      { title: "계약 완료", href: "/properties/contracted", icon: CircleCheck },
    ],
  },
];

// Flat list of every dashboard-shell route.
export const allNavItems: NavItem[] = navGroups.flatMap((g) => g.items);
