import { useState, useEffect } from "react"
import { useParams, useNavigate, Link } from "react-router"
import { useQuery } from "@tanstack/react-query"
import { Printer, ChevronLeft, FileText, X, Calendar } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { FileQuestion } from "lucide-react"

import { ApiError } from "@/lib/api"
import { getForms, getFormHtml, type FormSummary } from "@/lib/api"
import { PROPERTY_LABEL } from "@/lib/naver/property-types"
import { TRADE_LABEL } from "@/lib/naver/trade-types"
import { formsFor } from "@/lib/properties/contract-forms"

export function FormsPage() {
  const { id = "" } = useParams()
  const navigate = useNavigate()

  // 계약 정보 조회 (매물 유형/거래 유형 확인용)
  const contractQuery = useQuery({
    queryKey: ["contract", id],
    queryFn: () => import("@/lib/api").then((m) => m.getContract(id)),
    meta: { silent: true },
    gcTime: 0,
  })

  // 사용 가능한 양식 목록
  const formsQuery = useQuery({
    queryKey: ["forms", id],
    queryFn: () => getForms(id),
    enabled: !!id,
    meta: { silent: true },
  })

  const notFound = contractQuery.error instanceof ApiError && contractQuery.error.status === 404

  const [selectedFormId, setSelectedFormId] = useState<string | null>(null)
  const [formHtml, setFormHtml] = useState<string | null>(null)
  const [formLoading, setFormLoading] = useState(false)

  // 양식 선택 시 HTML 로드
  useEffect(() => {
    if (!selectedFormId) {
      setFormHtml(null)
      return
    }
    let cancelled = false
    setFormLoading(true)
    getFormHtml(id, selectedFormId)
      .then((html) => {
        if (!cancelled) setFormHtml(html)
      })
      .catch((e) => {
        if (!cancelled) {
          toast.error(e instanceof Error ? e.message : "양식을 불러올 수 없습니다")
          setFormHtml(null)
        }
      })
      .finally(() => {
        if (!cancelled) setFormLoading(false)
      })
    return () => { cancelled = true }
  }, [id, selectedFormId])

  // 프린트 함수
  function handlePrint() {
    if (!formHtml) return
    const printWindow = window.open("", "_blank", "width=800,height=1000")
    if (!printWindow) {
      toast.error("팝업이 차단되었습니다. 브라우저 설정을 확인하세요.")
      return
    }
    printWindow.document.write(`
      <!DOCTYPE html>
      <html lang="ko">
      <head>
        <meta charset="UTF-8">
        <title>양식 인쇄</title>
        <style>
          @media print {
            @page { size: 210mm 297mm; margin: 18mm 14mm; }
            .no-print { display: none !important; }
            body { font-family: 'Noto Serif KR', 'Noto Serif', 'Malgun Gothic', 'Apple SD Gothic Neo', serif; font-size: 13px; line-height: 1.6; color: #000; }
            .page { width: 210mm; min-height: 297mm; padding: 16mm 18mm; margin: 0 auto; background: white; }
            h1 { text-align: center; font-size: 28px; font-weight: 700; letter-spacing: 0.4em; margin: 0 0 8px; }
            .subtitle { text-align: center; font-size: 11px; letter-spacing: 0.1em; color: #666; margin-bottom: 24px; }
            .section { margin-top: 16px; }
            .section h2 { font-size: 13px; font-weight: 700; margin-bottom: 8px; }
            table { width: 100%; border-collapse: collapse; border: 1px solid #000; font-size: 12px; }
            th, td { border: 1px solid #000; padding: 6px 8px; vertical-align: top; }
            th { background: #f5f5f5; font-weight: 700; text-align: center; }
            .fill { display: inline-block; border-bottom: 1px dotted #999; min-width: 100%; vertical-align: bottom; }
            .fill-short { display: inline-block; border-bottom: 1px dotted #999; min-width: 40%; vertical-align: bottom; }
            .fill-medium { display: inline-block; border-bottom: 1px dotted #999; min-width: 55%; vertical-align: bottom; }
            .fill-wide { display: inline-block; border-bottom: 1px dotted #999; min-width: 80%; vertical-align: bottom; }
            .text-gray-600 { color: #666; }
            .text-gray-700 { color: #444; }
            .bg-gray-100 { background: #f5f5f5; }
            .articles { font-size: 12px; line-height: 1.8; }
            .articles li { margin-bottom: 6px; }
            .articles li strong { font-weight: 700; }
            .special-terms { min-height: 72px; border: 1px solid #000; padding: 8px; line-height: 1.9; color: #999; }
            .special-terms div { border-bottom: 1px dotted #ccc; }
            .special-terms div:last-child { border-bottom: none; }
            .date-line { text-align: center; margin: 20px 0; letter-spacing: 0.5em; }
            .signatures th { width: 70px; }
            .signatures td:nth-child(2) { width: 40px; }
            .signatures td:nth-child(3) { width: 28%; }
            .signatures td:nth-child(4) { width: 40px; }
            .signatures td:nth-child(5) { width: auto; }
            .receipt-table { border: 2px solid #000; }
            .receipt-table th { width: 100px; }
            .amount-table th { width: 60px; }
            .amount-table td:nth-child(2) { width: 50%; }
            .amount-table td:nth-child(3) { width: 42%; }
          }
          @media screen {
            body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; padding: 20px; background: #f5f5f5; }
            .page { max-width: 210mm; margin: 0 auto; background: white; box-shadow: 0 4px 20px rgba(0,0,0,0.1); padding: 16mm 18mm; min-height: 297mm; }
            .no-print { position: fixed; top: 16px; right: 16px; z-index: 100; }
          }
        </style>
      </head>
      <body>
        <div class="no-print flex gap-2">
          <button onclick="window.print()" class="px-4 py-2 bg-blue-600 text-white rounded hover:bg-blue-700">인쇄</button>
          <button onclick="window.close()" class="px-4 py-2 bg-gray-200 text-gray-700 rounded hover:bg-gray-300">닫기</button>
        </div>
        ${formHtml}
      </body>
      </html>
    `)
    printWindow.document.close()
  }

  if (notFound || !contractQuery.data) {
    return (
      <div className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold">양식 인쇄</h1>
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon"><FileQuestion /></EmptyMedia>
            <EmptyTitle>{notFound ? "매물을 찾을 수 없습니다" : "계약 정보를 불러오지 못했습니다"}</EmptyTitle>
            {!notFound && contractQuery.error && <EmptyDescription>{contractQuery.error.message}</EmptyDescription>}
          </EmptyHeader>
          <EmptyContent>
            <Button nativeButton={false} render={<Link to="/properties" />}>매물 목록으로</Button>
          </EmptyContent>
        </Empty>
      </div>
    )
  }

  const contract = contractQuery.data!
  const property = contract.property
  const g = property.realEstateType ?? ""
  const t = property.tradeType ?? ""
  const title = property.name ?? property.complexName ?? "매물"
  const availableForms = formsFor(g, t)

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">양식 인쇄</h1>
        <div className="flex items-center gap-2">
          <Link to="/calendar">
            <Button variant="outline" size="sm">
              <Calendar className="size-3.5 mr-1" />일정
            </Button>
          </Link>
          <Button variant="outline" size="sm" onClick={() => navigate(`/properties/${id}/contract`)}>
            <ChevronLeft className="size-3.5 mr-1" />계약 관리
          </Button>
        </div>
      </div>

      {contractQuery.isLoading ? (
        <Card className="gap-0">
          <CardHeader className="border-b"><CardTitle>{title}</CardTitle></CardHeader>
          <CardContent className="flex flex-col gap-4 p-4" aria-busy="true">
            {Array.from({ length: 4 }, (_, i) => <Skeleton key={i} className="h-8 w-full" />)}
          </CardContent>
        </Card>
      ) : (
        <div className="space-y-4">
          {/* 안내 배너 */}
          <div className="rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-800">
            ⚠️ 실무 보조용 양식입니다. 공식·법적 효력을 보장하지 않으며, 구체적 거래는 전문가 확인이 필요합니다.
          </div>

          {/* 매물 정보 */}
          <Card>
            <CardHeader className="border-b">
              <CardTitle>{title}</CardTitle>
              <p className="mt-1 text-sm text-muted-foreground">
                {property.complexName ?? ""} · {PROPERTY_LABEL[g] ?? g} · {TRADE_LABEL[t] ?? t}
              </p>
            </CardHeader>
            <CardContent className="pt-4">
              {!g || !t ? (
                <div className="py-8 text-center text-sm text-muted-foreground">
                  먼저 매물유형·거래유형을 설정하세요.{" "}
                  <Button variant="link" size="sm" onClick={() => navigate(`/properties/${id}/edit`)}>
                    수정 폼 열기
                  </Button>
                </div>
              ) : (
                <>
                  {/* 양식 목록 */}
                  <section className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold">적용 가능한 양식 ({availableForms.length}개)</h3>
                      {formsQuery.isLoading && <Skeleton className="h-5 w-20" />}
                    </div>
                    {availableForms.length === 0 ? (
                      <div className="text-center py-8 text-sm text-muted-foreground">
                        이 매물 유형·거래 유형에 적용 가능한 양식이 없습니다.
                      </div>
                    ) : (
                      <div className="grid gap-2 sm:grid-cols-2 lg:grid-cols-3">
                        {availableForms.map((f) => (
                          <Button
                            key={f.id}
                            variant={selectedFormId === f.id ? "default" : "outline"}
                            className="h-auto py-3 text-left gap-2"
                            onClick={() => setSelectedFormId(selectedFormId === f.id ? null : f.id)}
                          >
                            <FileText className="size-4 shrink-0" />
                            <span className="font-medium">{f.label}</span>
                            {selectedFormId === f.id && <X className="size-4 ml-auto" />}
                          </Button>
                        ))}
                      </div>
                    )}
                  </section>

                  {selectedFormId && (
                    <>
                      <Separator />
                      {/* 양식 미리보기/인쇄 */}
                      <section className="space-y-3">
                        <div className="flex items-center justify-between">
                          <h3 className="text-sm font-semibold">미리보기 / 인쇄</h3>
                          <Button onClick={handlePrint} disabled={formLoading || !formHtml}>
                            <Printer className="size-3.5 mr-1" />{formLoading ? "로딩 중..." : "인쇄"}
                          </Button>
                        </div>
                        {formLoading ? (
                          <div className="aspect-[210/297] bg-gray-50 rounded border flex items-center justify-center">
                            <div className="flex flex-col gap-2 items-center">
                              <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-primary" />
                              <span className="text-sm text-muted-foreground">양식 로딩 중...</span>
                            </div>
                          </div>
                        ) : formHtml ? (
                          <div className="rounded border border-gray-200 bg-white shadow-sm overflow-hidden">
                            <div className="p-3 border-b bg-gray-50 flex items-center justify-between no-print">
                              <span className="text-sm text-muted-foreground">아래 내용이 인쇄됩니다. '인쇄' 버튼을 눌러 출력하세요.</span>
                              <Button variant="ghost" size="sm" onClick={handlePrint}>
                                <Printer className="size-3.5 mr-1" />인쇄
                              </Button>
                            </div>
                            <div
                              className="page-preview"
                              dangerouslySetInnerHTML={{ __html: formHtml }}
                            />
                          </div>
                        ) : (
                          <div className="rounded border border-red-200 bg-red-50 p-4 text-red-700 text-sm">
                            양식을 불러오지 못했습니다.
                          </div>
                        )}
                      </section>
                    </>
                  )}
                </>
              )}
            </CardContent>
          </Card>
        </div>
      )}
    </div>
  )
}

// 페이지 미리보기용 스타일 (인쇄용 CSS와 유사하지만 화면용)
const style = document.createElement("style")
style.textContent = `
  .page-preview {
    width: 210mm;
    min-height: 297mm;
    padding: 16mm 18mm;
    margin: 0 auto;
    background: white;
    font-family: 'Noto Serif KR', 'Noto Serif', 'Malgun Gothic', 'Apple SD Gothic Neo', serif;
    font-size: 13px;
    line-height: 1.6;
    color: #000;
    box-sizing: border-box;
  }
  .page-preview h1 { text-align: center; font-size: 28px; font-weight: 700; letter-spacing: 0.4em; margin: 0 0 8px; }
  .page-preview .subtitle { text-align: center; font-size: 11px; letter-spacing: 0.1em; color: #666; margin-bottom: 24px; }
  .page-preview .section { margin-top: 16px; }
  .page-preview .section h2 { font-size: 13px; font-weight: 700; margin-bottom: 8px; }
  .page-preview table { width: 100%; border-collapse: collapse; border: 1px solid #000; font-size: 12px; }
  .page-preview th, .page-preview td { border: 1px solid #000; padding: 6px 8px; vertical-align: top; }
  .page-preview th { background: #f5f5f5; font-weight: 700; text-align: center; }
  .page-preview .fill { display: inline-block; border-bottom: 1px dotted #999; min-width: 100%; vertical-align: bottom; }
  .page-preview .fill-short { display: inline-block; border-bottom: 1px dotted #999; min-width: 40%; vertical-align: bottom; }
  .page-preview .fill-medium { display: inline-block; border-bottom: 1px dotted #999; min-width: 55%; vertical-align: bottom; }
  .page-preview .fill-wide { display: inline-block; border-bottom: 1px dotted #999; min-width: 80%; vertical-align: bottom; }
  .page-preview .text-gray-600 { color: #666; }
  .page-preview .text-gray-700 { color: #444; }
  .page-preview .bg-gray-100 { background: #f5f5f5; }
  .page-preview .articles { font-size: 12px; line-height: 1.8; }
  .page-preview .articles li { margin-bottom: 6px; }
  .page-preview .articles li strong { font-weight: 700; }
  .page-preview .special-terms { min-height: 72px; border: 1px solid #000; padding: 8px; line-height: 1.9; color: #999; }
  .page-preview .special-terms div { border-bottom: 1px dotted #ccc; }
  .page-preview .special-terms div:last-child { border-bottom: none; }
  .page-preview .date-line { text-align: center; margin: 20px 0; letter-spacing: 0.5em; }
  .page-preview .signatures th { width: 70px; }
  .page-preview .signatures td:nth-child(2) { width: 40px; }
  .page-preview .signatures td:nth-child(3) { width: 28%; }
  .page-preview .signatures td:nth-child(4) { width: 40px; }
  .page-preview .signatures td:nth-child(5) { width: auto; }
  .page-preview .receipt-table { border: 2px solid #000; }
  .page-preview .receipt-table th { width: 100px; }
  .page-preview .amount-table th { width: 60px; }
  .page-preview .amount-table td:nth-child(2) { width: 50%; }
  .page-preview .amount-table td:nth-child(3) { width: 42%; }
`
if (typeof document !== "undefined" && !document.head.contains(style)) {
  document.head.appendChild(style)
}