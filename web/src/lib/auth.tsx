import { useMutation, useQuery } from "@tanstack/react-query"
import { ServerOff } from "lucide-react"
import { Navigate, Outlet } from "react-router"
import { Button } from "@/components/ui/button"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Spinner } from "@/components/ui/spinner"
import { ApiError, api } from "@/lib/api"
import { queryClient } from "@/lib/query-client"
import { tokenStore } from "@/lib/platform"

export type Me = { id: string; email: string; name: string | null; role: string; agencyId: string }

export type SignupInput = {
  agencyName: string
  agencyZipcode?: string
  agencyAddress?: string
  agencyPhone?: string
  name: string
  phone?: string
  email: string
  password: string
  passwordConfirm: string
  agree: boolean
}

type AuthResponse = { token: string; user: Me }

const ME_KEY = ["me"] as const

// 비로그인(401)은 오류가 아니라 정상 상태이므로 null 로 푼다.
export function useMe() {
  return useQuery({
    queryKey: ME_KEY,
    meta: { silent: true },
    queryFn: async (): Promise<Me | null> => {
      try {
        return await api<Me>("/auth/me", { skipUnauthorized: true })
      } catch (e) {
        if (e instanceof ApiError && e.status === 401) return null
        throw e
      }
    },
  })
}

async function startSession(res: AuthResponse): Promise<Me> {
  await tokenStore.set(res.token)
  queryClient.setQueryData(ME_KEY, res.user)
  return res.user
}

export async function login(email: string, password: string): Promise<Me> {
  const res = await api<AuthResponse>("/auth/login", {
    method: "POST",
    body: { email, password },
    skipUnauthorized: true,
  })
  return startSession(res)
}

export async function signup(input: SignupInput): Promise<Me> {
  const res = await api<AuthResponse>("/auth/signup", { method: "POST", body: input })
  return startSession(res)
}

export async function logout(): Promise<void> {
  await api<void>("/auth/logout", { method: "POST" })
  await tokenStore.clear()
  queryClient.setQueryData(ME_KEY, null)
}

export function useLogout() {
  return useMutation({ mutationFn: logout })
}

function FullPageSpinner() {
  return (
    <div className="flex min-h-svh items-center justify-center">
      <Spinner className="size-6 text-muted-foreground" />
    </div>
  )
}

function ServerDown({ error, onRetry }: { error: Error; onRetry: () => void }) {
  const unreachable = error instanceof ApiError && error.status === 0
  return (
    <div className="flex min-h-svh p-4">
      <Empty>
        <EmptyHeader>
          <EmptyMedia variant="icon">
            <ServerOff />
          </EmptyMedia>
          <EmptyTitle>{unreachable ? "서버에 연결할 수 없습니다" : error.message}</EmptyTitle>
          {unreachable && <EmptyDescription>네트워크 상태를 확인한 뒤 다시 시도하세요.</EmptyDescription>}
        </EmptyHeader>
        <EmptyContent>
          <Button onClick={onRetry}>다시 시도</Button>
        </EmptyContent>
      </Empty>
    </div>
  )
}

// me 조회 결과에 따라 화면을 정한다: 로딩·서버 불통은 여기서 처리하고, 나머지는 render 로 넘긴다.
function AuthGate({ render }: { render: (me: Me | null) => React.ReactNode }) {
  const me = useMe()
  if (me.data !== undefined) return render(me.data)
  if (me.isError) return <ServerDown error={me.error} onRetry={() => void me.refetch()} />
  return <FullPageSpinner />
}

export function RequireAuth() {
  return <AuthGate render={(me) => (me ? <Outlet /> : <Navigate to="/login" replace />)} />
}

export function RedirectIfAuthed() {
  return <AuthGate render={(me) => (me ? <Navigate to="/properties" replace /> : <Outlet />)} />
}
