import { FileQuestion } from "lucide-react"
import { Link } from "react-router"
import { Button } from "@/components/ui/button"
import { Empty, EmptyContent, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"

export function NotFound() {
  return (
    <div className="flex min-h-svh p-4">
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <FileQuestion />
          </EmptyMedia>
          <EmptyTitle>페이지를 찾을 수 없습니다</EmptyTitle>
        </EmptyHeader>
        <EmptyContent>
          <Button nativeButton={false} render={<Link to="/properties" />}>매물 목록으로</Button>
        </EmptyContent>
      </Empty>
    </div>
  )
}
