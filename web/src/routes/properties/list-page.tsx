import { useQuery } from "@tanstack/react-query"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyTitle } from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { listProperties, type PropertyView } from "@/lib/properties/api"
import { PropertyList, VIEW_TITLE } from "./property-list"

const PAGE_TITLE: Record<PropertyView, string> = { all: "매물 관리", favorites: "관심 매물", "in-progress": "계약진행", contracted: "계약완료" }

// 원본의 4개 page.tsx(서버 렌더) → 한 컴포넌트. 경로별 view 는 router 가 준다.
export function PropertyListPage({ view }: { view: PropertyView }) {
  const query = useQuery({ queryKey: ["properties", view], queryFn: () => listProperties(view) })
  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold">{PAGE_TITLE[view]}</h1>
      {query.data ? (
        <PropertyList rows={query.data} view={view} />
      ) : (
        // 원본은 서버 렌더라 로딩 화면이 없었다. 같은 Card 틀 안에 Skeleton 을 둬 레이아웃이 튀지 않게 한다.
        <Card className="gap-0">
          <CardHeader className="border-b"><CardTitle>{VIEW_TITLE[view]}</CardTitle></CardHeader>
          <CardContent className="p-0">
            {query.isError ? (
              <Empty className="border-0 py-10">
                <EmptyHeader>
                  <EmptyTitle>목록을 불러오지 못했습니다</EmptyTitle>
                  <EmptyDescription>{query.error.message}</EmptyDescription>
                </EmptyHeader>
                <EmptyContent>
                  <Button size="sm" variant="outline" onClick={() => void query.refetch()} disabled={query.isFetching}>다시 시도</Button>
                </EmptyContent>
              </Empty>
            ) : (
              <div className="flex flex-col gap-2 p-3" aria-busy="true">
                {Array.from({ length: 8 }, (_, i) => <Skeleton key={i} className="h-8 w-full" />)}
              </div>
            )}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
