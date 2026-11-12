import { Link } from "react-router"
import { ArrowLeft } from "lucide-react"

const SECTIONS: { h: string; p: string }[] = [
  { h: "제1조 (목적)", p: "본 약관은 회사가 제공하는 부동산 매물관리 서비스(이하 \"서비스\")의 이용과 관련하여 회사와 회원 간의 권리·의무 및 책임사항, 이용조건 및 절차를 규정함을 목적으로 합니다." },
  { h: "제2조 (정의)", p: "\"회원\"이란 본 약관에 동의하고 회사와 이용계약을 체결하여 서비스를 이용하는 자를 말합니다. 본 약관에서 정하지 아니한 용어의 정의는 관계 법령 및 일반 관례에 따릅니다." },
  { h: "제3조 (약관의 효력 및 변경)", p: "본 약관은 서비스 화면에 게시함으로써 효력이 발생합니다. 회사는 관계 법령을 위반하지 않는 범위에서 본 약관을 변경할 수 있으며, 약관을 변경하는 경우 적용일자 및 변경사유를 명시하여 사전에 공지합니다." },
  { h: "제4조 (서비스의 제공)", p: "회사는 매물·고객·일정 관리, 실거래가 조회 등 부동산 중개 업무에 필요한 기능을 제공합니다. 회사는 운영상·기술상의 필요에 따라 제공하는 서비스의 전부 또는 일부를 변경할 수 있습니다." },
  { h: "제5조 (회원가입)", p: "이용자는 회사가 정한 가입 양식에 정보를 기입하고 본 약관에 동의함으로써 회원가입을 신청하며, 회사가 이를 승낙함으로써 이용계약이 성립합니다. 회사는 허위 정보 기재 등의 사유가 있는 경우 가입을 거부하거나 사후에 이용계약을 해지할 수 있습니다." },
  { h: "제6조 (회원의 의무)", p: "회원은 관계 법령과 본 약관의 규정을 준수하여야 하며, 타인의 정보를 도용하거나 서비스의 정상적인 운영을 방해하는 행위를 하여서는 안 됩니다. 계정 정보의 관리 책임은 회원에게 있습니다." },
  { h: "제7조 (개인정보의 보호)", p: "회사는 관계 법령에 따라 회원의 개인정보를 보호하기 위하여 노력하며, 개인정보의 처리에 관한 자세한 사항은 개인정보처리방침에 따릅니다." },
  { h: "제8조 (서비스 이용 제한 및 해지)", p: "회원이 본 약관을 위반하는 경우 회사는 서비스 이용을 제한하거나 이용계약을 해지할 수 있습니다. 회원은 언제든지 서비스 내 절차에 따라 이용계약의 해지(회원 탈퇴)를 요청할 수 있습니다." },
  { h: "제9조 (책임의 제한)", p: "회사는 천재지변, 회원의 귀책사유 등 회사의 통제를 벗어난 사유로 발생한 손해에 대하여 책임을 지지 않습니다. 서비스에서 제공하는 실거래가 등 외부 출처 데이터의 정확성·완전성은 보장되지 않으며 참고용으로 제공됩니다." },
  { h: "부칙", p: "본 약관은 2026년 6월 27일부터 시행합니다." },
]

export function TermsPage() {
  return (
    <div className="mx-auto max-w-3xl px-4 py-10 sm:py-16">
      <Link to="/register" className="mb-6 inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground">
        <ArrowLeft className="size-4" />돌아가기
      </Link>
      <h1 className="text-2xl font-semibold">이용약관</h1>
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
