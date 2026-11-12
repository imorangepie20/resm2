import { MutationCache, QueryCache, QueryClient } from "@tanstack/react-query"
import { toast } from "sonner"
import { ApiError } from "@/lib/api"

declare module "@tanstack/react-query" {
  interface Register {
    queryMeta: { silent?: boolean }
    mutationMeta: { silent?: boolean }
  }
}

// 폼이 인라인으로 보여 주는 오류(meta.silent)와 401(리다이렉트가 처리)은 토스트를 띄우지 않는다.
function notify(error: Error, meta: { silent?: boolean } | undefined) {
  if (meta?.silent) return
  if (error instanceof ApiError && error.status === 401) return
  toast.error(error.message)
}

export const queryClient = new QueryClient({
  queryCache: new QueryCache({ onError: (error, query) => notify(error, query.meta) }),
  mutationCache: new MutationCache({ onError: (error, _v, _c, mutation) => notify(error, mutation.meta) }),
  defaultOptions: { queries: { retry: false, refetchOnWindowFocus: false } },
})
