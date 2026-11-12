import { useState } from "react"
import { useMutation } from "@tanstack/react-query"
import { Link } from "react-router"

import { signup, type SignupInput } from "@/lib/auth"
import { AuthCard } from "@/components/auth/auth-card"
import { Button } from "@/components/ui/button"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Label } from "@/components/ui/label"
import { formatTel } from "@/lib/properties/format"
import { PostcodeSearch } from "@/components/postcode-search"

export function RegisterPage() {
  const [zipcode, setZipcode] = useState("")
  const [address, setAddress] = useState("")
  const [agencyPhone, setAgencyPhone] = useState("")
  const [phone, setPhone] = useState("")
  const [agreed, setAgreed] = useState(false)
  // 오류는 아래에서 인라인으로 보여 주므로 전역 토스트는 끈다(meta.silent).
  const mutation = useMutation({ mutationFn: (v: SignupInput) => signup(v), meta: { silent: true } })

  function onSubmit(e: React.FormEvent<HTMLFormElement>) {
    e.preventDefault()
    const fd = new FormData(e.currentTarget)
    const str = (k: string) => String(fd.get(k) ?? "")
    const opt = (k: string) => str(k).trim() || undefined
    mutation.mutate({
      agencyName: str("agencyName"),
      agencyZipcode: opt("agencyZipcode"),
      agencyAddress: opt("agencyAddress"),
      agencyPhone: opt("agencyPhone"),
      name: str("name"),
      phone: opt("phone"),
      email: str("email"),
      password: str("password"),
      passwordConfirm: str("passwordConfirm"),
      agree: agreed,
    })
  }

  return (
    <AuthCard
      title="회원가입"
      footer={
        <span className="text-muted-foreground">
          이미 계정이 있으신가요?{" "}
          <Link to="/login" className="text-foreground underline-offset-4 hover:underline font-medium">
            로그인
          </Link>
        </span>
      }
    >
      <form onSubmit={onSubmit} className="flex flex-col gap-4">
        {/* 상호명 (Agency) */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="agencyName">상호명</Label>
          <Input id="agencyName" name="agencyName" type="text" placeholder="○○공인중개사" autoComplete="organization" required />
        </div>

        {/* 중개사무소 주소 (우편번호 검색) */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="agencyAddress">중개사무소 주소</Label>
          <div className="flex items-center gap-2">
            <Input name="agencyZipcode" value={zipcode} readOnly placeholder="우편번호" className="w-28" />
            <PostcodeSearch onComplete={({ zonecode, address: addr }) => { setZipcode(zonecode); setAddress(addr) }} />
          </div>
          <Input id="agencyAddress" name="agencyAddress" value={address} onChange={(e) => setAddress(e.target.value)} placeholder="도로명주소 + 상세주소" autoComplete="street-address" />
        </div>

        {/* 중개사무소 전화번호 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="agencyPhone">중개사무소 전화번호</Label>
          <Input id="agencyPhone" name="agencyPhone" type="tel" inputMode="tel" value={agencyPhone} onChange={(e) => setAgencyPhone(formatTel(e.target.value))} placeholder="031-000-0000" autoComplete="tel" />
        </div>

        {/* 이름 (필수) */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="name">이름</Label>
          <Input id="name" name="name" type="text" placeholder="홍길동" autoComplete="name" required />
        </div>

        {/* 개인 전화번호 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="phone">개인 전화번호</Label>
          <Input id="phone" name="phone" type="tel" inputMode="tel" value={phone} onChange={(e) => setPhone(formatTel(e.target.value))} placeholder="010-0000-0000" autoComplete="tel" />
        </div>

        {/* 이메일 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="email">이메일</Label>
          <Input id="email" name="email" type="email" placeholder="m@example.com" autoComplete="email" required />
        </div>

        {/* 비밀번호 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="password">비밀번호</Label>
          <Input id="password" name="password" type="password" autoComplete="new-password" placeholder="8자 이상" required />
        </div>

        {/* 비밀번호 확인 */}
        <div className="flex flex-col gap-1.5">
          <Label htmlFor="passwordConfirm">비밀번호 확인</Label>
          <Input id="passwordConfirm" name="passwordConfirm" type="password" autoComplete="new-password" required />
        </div>

        {/* 약관 동의 */}
        <div className="flex items-start gap-2">
          <Checkbox id="agree" checked={agreed} onCheckedChange={(c) => setAgreed(c)} className="mt-0.5" />
          <Label htmlFor="agree" className="text-sm font-normal leading-snug text-muted-foreground">
            <Link to="/terms" target="_blank" className="font-medium text-foreground underline underline-offset-2">이용약관</Link> 및 <Link to="/privacy" target="_blank" className="font-medium text-foreground underline underline-offset-2">개인정보 수집·이용</Link>에 동의합니다. (필수)
          </Label>
        </div>

        {mutation.error && <p className="text-sm text-destructive">{mutation.error.message}</p>}

        <Button type="submit" disabled={mutation.isPending || !agreed} className="w-full mt-1">
          {mutation.isPending ? "처리 중..." : "가입하기"}
        </Button>
      </form>
    </AuthCard>
  )
}
