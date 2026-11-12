import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { Link } from "react-router"
import { Eye, EyeOff } from "lucide-react"

import { login } from "@/lib/auth"
import { AuthCard } from "@/components/auth/auth-card"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"

export function LoginPage() {
  const [showPassword, setShowPassword] = useState(false)
  // 오류는 아래에서 인라인으로 보여 주므로 전역 토스트는 끈다(meta.silent).
  const mutation = useMutation({
    mutationFn: (v: { email: string; password: string }) => login(v.email, v.password),
    meta: { silent: true },
  })

  function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    mutation.mutate({ email: String(fd.get("email") ?? ""), password: String(fd.get("password") ?? "") })
  }

  return (
    <AuthCard
      title="로그인"
      footer={
        <span className="text-muted-foreground">
          계정이 없으신가요?{" "}
          <Link to="/register" className="text-foreground underline-offset-4 hover:underline font-medium">
            회원가입
          </Link>
        </span>
      }
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4">
        {/* 이메일 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="email">이메일</Label>
          <Input id="email" name="email" type="email" placeholder="m@example.com" autoComplete="email" required />
        </div>

        {/* 비밀번호 */}
        <div className="flex flex-col gap-1.5">
          <div className="flex items-center justify-between">
            <Label htmlFor="password">비밀번호</Label>
          </div>
          <div className="relative">
            <Input
              id="password"
              name="password"
              type={showPassword ? "text" : "password"}
              autoComplete="current-password"
              className="pr-9"
              required
            />
            <button
              type="button"
              onClick={() => setShowPassword((v) => !v)}
              className="absolute right-2.5 top-1/2 -translate-y-1/2 text-muted-foreground hover:text-foreground transition-colors"
              aria-label={showPassword ? "비밀번호 숨기기" : "비밀번호 표시"}
            >
              {showPassword ? <EyeOff className="size-4" /> : <Eye className="size-4" />}
            </button>
          </div>
        </div>

        {mutation.error && <p className="text-sm text-destructive">{mutation.error.message}</p>}

        <Button type="submit" disabled={mutation.isPending} className="w-full mt-1">
          {mutation.isPending ? "처리 중..." : "로그인"}
        </Button>
      </form>
    </AuthCard>
  )
}
