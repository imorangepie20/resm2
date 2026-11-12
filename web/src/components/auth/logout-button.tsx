import { Button } from "@/components/ui/button"
import { useLogout } from "@/lib/auth"

export function LogoutButton() {
  const logout = useLogout()
  return (
    <Button type="button" variant="ghost" size="sm" onClick={() => logout.mutate()} disabled={logout.isPending}>
      로그아웃
    </Button>
  )
}
