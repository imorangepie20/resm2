import { Link } from "react-router"
import { ArrowLeft } from "lucide-react"

const SECTIONS: { h: string; p: string }[] = [
  { h: "1. 수집하는 개인정보 항목", p: "회원가입 시 이메일, 비밀번호, 이름, 개인 전화번호, 상호명, 중개사무소 주소·전화번호를 수집합니다. 서비스 이용 과정에서 접속 로그 등 서비스 이용 기록이 자동으로 생성·수집될 수 있습니다." },
  { h: "2. 개인정보의 수집 및 이용 목적", p: "회원 식별 및 가입 의사 확인, 서비스의 제공 및 운영, 고객 문의 응대, 서비스 품질 개선을 위하여 개인정보를 이용합니다." },
  { h: "3. 개인정보의 보유 및 이용 기간", p: "수집한 개인정보는 회원 탈퇴 시까지 보유·이용합니다. 다만 관계 법령에 따라 보존이 필요한 경우 해당 법령에서 정한 기간 동안 보관합니다." },
  { h: "4. 개인정보의 제3자 제공", p: "회사는 회원의 동의 없이 개인정보를 외부에 제공하지 않습니다. 다만 법령에 근거하거나 수사기관의 적법한 요청이 있는 경우에는 예외로 합니다." },
  { h: "5. 개인정보 처리의 위탁", p: "회사는 서비스 운영에 필요한 경우 개인정보 처리 업무의 일부를 외부에 위탁할 수 있으며, 위탁하는 경우 위탁 대상 및 업무 내용을 본 방침을 통해 고지합니다." },
  { h: "6. 개인정보의 파기", p: "개인정보의 보유 기간이 경과하거나 처리 목적이 달성된 경우 지체 없이 해당 개인정보를 파기합니다." },
  { h: "7. 이용자의 권리", p: "회원은 언제든지 자신의 개인정보를 조회·수정할 수 있으며, 개인정보의 처리 정지 또는 삭제(회원 탈퇴)를 요청할 수 있습니다." },
  { h: "8. 개인정보 보호책임자", p: "개인정보 처리에 관한 문의·불만·피해구제는 서비스 운영자에게 요청할 수 있으며, 회사는 신속하게 답변·처리합니다." },
  { h: "부칙", p: "본 개인정보처리방침은 2026년 6월 27일부터 시행합니다." },
]

export function PrivacyPage() {
  return (
    <div className="mx-auto max-w-3xl px-4 py-10 sm:py-16">
      <Link to="/register" className="mb-6 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="size-4" />돌아가기
      </Link>
      <h1 className="text-2xl font-semibold">개인정보처리방침</h1>
      <p className="mt-1 text-sm text-muted-foreground">시행일: 2026년 6월 27일</p>
      <div className="mt-8 flex flex-col gap-6 text-sm leading-relaxed text-foreground">
        {SECTIONS.map((s) => (
          <section key={s.h}>
            <h2 className="mb-1.5 font-semibold">{s.h}</h2>
            <p className="text-muted-foreground">{s.p}</p>
          </section>
        ))}
      </div>
    </div>
  )
}
