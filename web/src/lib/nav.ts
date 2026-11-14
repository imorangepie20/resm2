import type { LucideIcon } from "lucide-react";
import {
  ClipboardList,
  Star,
  CircleDashed,
  CircleCheck,
  Users,
  Calendar,
  FileText,
} from "lucide-react";

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
  {
    label: "계약 관리",
    items: [
      { title: "계약 진행", href: "/properties/progress", icon: CircleDashed },
      { title: "계약 완료", href: "/properties/contracted", icon: CircleCheck },
    ],
  },
  {
    label: "고객 관리",
    items: [
      { title: "고객 목록", href: "/customers", icon: Users },
    ],
  },
  {
    label: "일정",
    items: [
      { title: "일정 관리", href: "/calendar", icon: Calendar },
    ],
  },
];

// Flat list of every dashboard-shell route.
export const allNavItems: NavItem[] = navGroups.flatMap((g) => g.items);