import { useQuery } from "@tanstack/react-query"
import { Link, useParams } from "react-router"
import { FileQuestion } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { ApiError } from "@/lib/api"
import { getProperty } from "@/lib/properties/api"
import { PropertyForm } from "./property-form"

// 원본의 "당사자" 섹션과 "고객으로 등록" 버튼은 2단계에서 되살린다.
export function EditPropertyPage() {
  const { id = "" } = useParams()
  const query = useQuery({
    queryKey: ["property", id],
    queryFn: () => getProperty(id),
    // 404 는 아래 빈 상태로 보여 주므로 전역 토스트를 끈다. 그 밖의 오류도 화면에 그대로 보인다.
    meta: { silent: true },
    // 캐시를 남기지 않는다: 열 때마다 최신 값으로 폼을 채우고, 입력 중에 다시 받아 폼이 바뀌는 일을 막는다.
    gcTime: 0,
  })
  const notFound = query.error instanceof ApiError && query.error.status === 404
  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold">매물 수정</h1>
      {query.data ? (
        <PropertyForm key={id} property={query.data} />
      ) : query.isError ? (
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon"><FileQuestion /></EmptyMedia>
            <EmptyTitle>{notFound ? "매물을 찾을 수 없습니다" : "매물을 불러오지 못했습니다"}</EmptyTitle>
            {!notFound && <EmptyDescription>{query.error.message}</EmptyDescription>}
          </EmptyHeader>
          <EmptyContent>
            <Button nativeButton={false} render={<Link to="/properties" />}>매물 목록으로</Button>
          </EmptyContent>
        </Empty>
      ) : (
        <Card>
          <CardHeader><CardTitle>매물 수정</CardTitle></CardHeader>
          <CardContent className="flex flex-col gap-4" aria-busy="true">
            {Array.from({ length: 6 }, (_, i) => <Skeleton key={i} className="h-8 w-full" />)}
          </CardContent>
        </Card>
      )}
    </div>
  )
}
