import { cn } from "@/lib/utils"

// RESM 브랜드 심볼 — 하우스. stroke=currentColor라 사이드바 chip(text-primary-foreground)·파비콘 등 어디서나 색을 상속한다.
export function Logo({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      strokeWidth={2}
      strokeLinecap="round"
      strokeLinejoin="round"
      aria-hidden="true"
      className={cn("size-4", className)}
    >
      <path d="M3.5 11 12 4 20.5 11" />
      <path d="M6 9.5V20h12V9.5" />
      <path d="M10 20v-5h4v5" />
    </svg>
  )
}
