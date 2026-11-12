import { cn } from "@/lib/utils"

// 빈 목록용 하우스 라인 일러스트. 템플릿 <EmptyMedia> 안에 넣어 쓴다(매물·관심·검색 등 공용). 색은 currentColor 상속.
export function EmptyIllustration({ className }: { className?: string }) {
  return (
    <svg
      viewBox="0 0 72 64"
      fill="none"
      aria-hidden="true"
      className={cn("size-16 text-muted-foreground", className)}
    >
      <path d="M14 30 38 13 62 30" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M20 27v25h32V27" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M31 52V40h10v12" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round" strokeLinejoin="round" />
      <path d="M9 58h54" stroke="currentColor" strokeWidth="2" strokeLinecap="round" strokeDasharray="3 6" opacity="0.5" />
    </svg>
  )
}
