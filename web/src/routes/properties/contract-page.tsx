import { useState } from "react"
import { Link, useParams, useNavigate } from "react-router"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { Printer, AlertCircle, CheckCircle2, FileText, Calendar } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Checkbox } from "@/components/ui/checkbox"
import { Input } from "@/components/ui/input"
import { Progress } from "@/components/ui/progress"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { FileQuestion } from "lucide-react"

import { ApiError } from "@/lib/api"
import { getContract, toggleChecklist, checklistAll, startContract, completeContract, type ContractData, type ChecklistItem } from "@/lib/api"
import { updateProperty } from "@/lib/properties/api"
import { PROPERTY_LABEL } from "@/lib/naver/property-types"
import { TRADE_LABEL } from "@/lib/naver/trade-types"
import { PARTIES, PARTY_LABEL } from "@/lib/properties/contract-checklist"
import { FIELD_BY_KEY, formMeta } from "@/lib/properties/fields"
import { toDateInput, fromDateInput } from "@/lib/properties/format"
import { manwonToWon, formatManwon, wonToManwon } from "@/lib/properties/money"
import { formsFor } from "@/lib/properties/contract-forms"

const KIND_BADGE: Record<ChecklistItem["kind"], { variant: "secondary" | "destructive" | "outline"; label: string }> = {
  서류: { variant: "secondary", label: "서류" },
  처리: { variant: "outline", label: "처리" },
  신고: { variant: "destructive", label: "법정신고" },
  세금: { variant: "destructive", label: "세금" },
}

export function ContractPage() {
  const { id = "" } = useParams()
  const navigate = useNavigate()
  const qc = useQueryClient()

  const query = useQuery({
    queryKey: ["contract", id],
    queryFn: () => getContract(id),
    meta: { silent: true },
    gcTime: 0,
  })

  const notFound = query.error instanceof ApiError && query.error.status === 404

  const readOnly = query.data?.status === "계약완료"

  // 진행률 파생값
  const reqIds = query.data?.checklist.filter((i) => i.required).map((i) => i.id) ?? []
  const checked = query.data?.checked ?? {}
  const doneItems = reqIds.filter((x) => !!checked[x]).length
  const totalItems = reqIds.length
  const totalFields = query.data?.requiredFields.length ?? 0
  const filledFields = query.data?.requiredFields.filter((f) => f.filled).length ?? 0
  const done = doneItems + filledFields
  const total = totalItems + totalFields
  const complete = total > 0 && done === total
  const progressPct = total > 0 ? (done / total) * 100 : 0
  const allChecked = (query.data?.checklist.length ?? 0) > 0 && query.data!.checklist.every((i) => !!checked[i.id])

  // 체크리스트 토글
  const toggleMut = useMutation({
    mutationFn: ({ itemId, checked }: { itemId: string; checked: boolean }) => toggleChecklist(id, itemId, checked),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["contract", id] }),
    onError: (e) => toast.error(e instanceof Error ? e.message : "저장 실패"),
  })

  function onToggle(itemId: string, v: boolean) {
    if (readOnly) return
    toggleMut.mutate({ itemId, checked: v })
  }

  // 전체 체크/해제
  const allMut = useMutation({
    mutationFn: (checked: boolean) => checklistAll(id, checked),
    onSuccess: () => qc.invalidateQueries({ queryKey: ["contract", id] }),
    onError: (e) => toast.error(e instanceof Error ? e.message : "저장 실패"),
  })

  function toggleAll(v: boolean) {
    if (readOnly) return
    allMut.mutate(v)
  }

  // 필수 필드 저장
  const fieldMut = useMutation({
    mutationFn: async ({ key, value }: { key: string; value: string }) => {
      const field = FIELD_BY_KEY[key]
      let v: string | number | boolean | null
      if (field.type === "money") v = manwonToWon(value)
      else if (field.type === "date") v = fromDateInput(value)
      else v = value === "" ? null : value
      await updateProperty(id, { [key]: v })
    },
    onMutate: async ({ key, value }) => {
      await qc.cancelQueries({ queryKey: ["contract", id] })
      const prev = qc.getQueryData<ContractData>(["contract", id])
      qc.setQueryData<ContractData>(["contract", id], (old) => {
        if (!old) return old
        const next = { ...old }
        const rf = next.requiredFields.map((f) => f.key === key ? { ...f, filled: value !== "" } : f)
        return { ...next, requiredFields: rf }
      })
      return { prev }
    },
    onError: (_e, _v, ctx: { prev?: ContractData } | undefined) => { if (ctx?.prev) qc.setQueryData(["contract", id], ctx.prev) },
    onSettled: () => qc.invalidateQueries({ queryKey: ["contract", id] }),
  })

  function saveField(key: string, value: string) {
    if (readOnly) return
    fieldMut.mutate({ key, value })
  }

  // 계약 시작
  const startMut = useMutation({
    mutationFn: () => startContract(id),
    onSuccess: () => {
      toast.success("계약진행으로 전환했습니다")
      qc.invalidateQueries({ queryKey: ["contract", id] })
      qc.invalidateQueries({ queryKey: ["properties"] })
    },
    onError: (e) => toast.error(e instanceof Error ? e.message : "시작 처리 실패"),
  })

  // 계약 완료
  const completeMut = useMutation({
    mutationFn: () => completeContract(id),
    onSuccess: () => {
      toast.success("계약완료로 전환했습니다")
      qc.invalidateQueries({ queryKey: ["contract", id] })
      qc.invalidateQueries({ queryKey: ["properties"] })
      navigate("/properties")
    },
    onError: (e) => toast.error(e instanceof Error ? e.message : "완료 처리 실패"),
  })

  if (notFound) {
    return (
      <div className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold">계약 관리</h1>
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon"><FileQuestion /></EmptyMedia>
            <EmptyTitle>매물을 찾을 수 없습니다</EmptyTitle>
          </EmptyHeader>
          <EmptyContent>
            <Button nativeButton={false} render={<Link to="/properties" />}>매물 목록으로</Button>
          </EmptyContent>
        </Empty>
      </div>
    )
  }

  if (query.isLoading) {
    return (
      <div className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold">계약 관리</h1>
        <Card className="gap-0">
          <CardHeader className="border-b"><CardTitle>불러오는 중...</CardTitle></CardHeader>
          <CardContent className="flex flex-col gap-4 p-4" aria-busy="true">
            {Array.from({ length: 6 }, (_, i) => <Skeleton key={i} className="h-8 w-full" />)}
          </CardContent>
        </Card>
      </div>
    )
  }

  if (query.isError) {
    return (
      <div className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold">계약 관리</h1>
        <Empty className="border">
          <EmptyHeader>
            <EmptyMedia variant="icon"><FileQuestion /></EmptyMedia>
            <EmptyTitle>계약 정보를 불러오지 못했습니다</EmptyTitle>
            <EmptyDescription>{query.error.message}</EmptyDescription>
          </EmptyHeader>
          <EmptyContent>
            <Button nativeButton={false} render={<Link to="/properties" />}>매물 목록으로</Button>
          </EmptyContent>
        </Empty>
      </div>
    )
  }

  const data = query.data!
  const property = data.property
  const g = property.realEstateType ?? ""
  const t = property.tradeType ?? ""
  const title = property.name ?? property.complexName ?? "매물"
  const forms = formsFor(g, t)

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold">계약 관리</h1>

      <div className="space-y-4">
          {/* 경고 배너 */}
          <div className="rounded-md bg-amber-50 px-3 py-2 text-sm text-amber-800">
            ⚠️ 실무 보조용 체크리스트·양식입니다. 공식·법적 효력을 보장하지 않으며, 구체적 거래는 전문가 확인이 필요합니다.
          </div>

          {/* 카드 */}
          <Card>
            <CardHeader className="flex flex-row items-center justify-between border-b">
              <div>
                <CardTitle>{title}</CardTitle>
                <p className="mt-1 text-sm text-muted-foreground">
                  {property.complexName ?? ""} · {PROPERTY_LABEL[g] ?? g} · {TRADE_LABEL[t] ?? t}
                </p>
              </div>
              <div className="flex items-center gap-2">
                <Button size="sm" variant="outline" onClick={() => navigate("/properties")}>목록</Button>
                <Link to="/calendar">
                  <Button size="sm" variant="outline">
                    <Calendar className="size-3.5 mr-1" />일정
                  </Button>
                </Link>
                {!readOnly && data.status === "진행" && (
                  <Button size="sm" onClick={() => startMut.mutate()} disabled={startMut.isPending}>
                    {startMut.isPending ? "처리 중..." : "계약 시작"}
                  </Button>
                )}
              </div>
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
                  {readOnly && (
                    <div className="rounded-md bg-muted px-3 py-2 text-sm text-muted-foreground mb-4">
                      계약완료 매물입니다(읽기전용). 수정하려면 목록에서 상태를 하향하세요.
                    </div>
                  )}

                  {/* 진행률 */}
                  <section className="space-y-2">
                    <div className="flex justify-between text-sm">
                      <span className="font-medium">진행률</span>
                      <span className="tabular-nums text-muted-foreground">{done}/{total}</span>
                    </div>
                    <Progress value={progressPct} className="h-2" />
                    <p className="text-xs text-muted-foreground">
                      필수 체크리스트 {totalItems}개 중 {doneItems}개, 필수 데이터 {totalFields}개 중 {filledFields}개 충족
                    </p>
                  </section>

                  <Separator />

                  {/* ① 핵심 계약 데이터 */}
                  <section className="space-y-3">
                    <h3 className="text-sm font-semibold">핵심 계약 데이터</h3>
                    <div className="grid grid-cols-1 gap-3 sm:grid-cols-2">
                      {data.requiredFields.map((f) => {
                        const field = FIELD_BY_KEY[f.key]
                        const meta = formMeta(field)
                        const initial = (property as Record<string, unknown>)[f.key] as string | undefined ?? ""
                        const displayVal = meta.formInput === "money"
                          ? formatManwon(initial as string)
                          : meta.formInput === "date"
                          ? toDateInput(initial as string)
                          : initial

                        return (
                          <ContractField
                            key={f.key}
                            id={id}
                            fieldKey={f.key}
                            label={f.label}
                            initial={initial as string}
                            displayValue={displayVal as string}
                            meta={meta}
                            readOnly={readOnly}
                            onSave={saveField}
                          />
                        )
                      })}
                    </div>
                  </section>

                  <Separator />

                  {/* ② 서류 체크리스트 */}
                  <section className="space-y-3">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold">서류 체크리스트</h3>
                      {!readOnly && data.checklist.length > 0 && (
                        <Button size="sm" variant="outline" onClick={() => toggleAll(!allChecked)} disabled={allMut.isPending}>
                          {allChecked ? "전체 해제" : "전체 체크"}
                        </Button>
                      )}
                    </div>
                    {PARTIES.map((party) => {
                      const items = data.checklist.filter((i) => i.party === party.value)
                      if (!items.length) return null
                      return (
                        <div key={party.value} className="space-y-1.5">
                          <p className="text-xs font-medium text-muted-foreground">{party.label}</p>
                          {items.map((i) => {
                            const isChecked = !!checked[i.id]
                            const badge = KIND_BADGE[i.kind]
                            return (
                              <label key={i.id} className="flex items-center gap-2 text-sm">
                                <Checkbox
                                  checked={isChecked}
                                  onCheckedChange={(c) => onToggle(i.id, c as boolean)}
                                  disabled={readOnly}
                                />
                                <span className={isChecked ? "text-muted-foreground line-through" : undefined}>{i.label}</span>
                                {!i.required && (
                                  <Badge variant={badge.variant} className="text-[10px] px-1.5 py-0.5">
                                    {badge.label}
                                  </Badge>
                                )}
                              </label>
                            )
                          })}
                        </div>
                      )
                    })}
                  </section>

                  <Separator />

                  {/* ③ 양식 인쇄 */}
                  <section className="space-y-2">
                    <div className="flex items-center justify-between">
                      <h3 className="text-sm font-semibold">양식 인쇄</h3>
                      <Link
                        to={`/properties/${id}/forms`}
                        className="inline-flex items-center gap-1 text-sm text-muted-foreground hover:text-foreground"
                      >
                        <FileText className="size-3.5" />양식 관리 페이지
                      </Link>
                    </div>
                    <div className="flex flex-wrap gap-2">
                      {forms.map((f) => (
                        <Button
                          key={f.id}
                          size="sm"
                          variant="outline"
                          onClick={() => window.open(`/api/v1/forms/${id}/${f.id}`, "_blank")}
                        >
                          <Printer className="size-3.5" />{f.label}
                        </Button>
                      ))}
                      {forms.length === 0 && (
                        <span className="text-sm text-muted-foreground self-center">적용 가능한 양식이 없습니다</span>
                      )}
                    </div>
                  </section>

                  {/* 완료 버튼 */}
                  {!readOnly && (
                    <div className="flex items-center justify-end gap-3 border-t pt-4">
                      {!complete && (
                        <span className="text-sm text-muted-foreground flex items-center gap-1">
                          <AlertCircle className="size-3.5" />
                          필수 항목·데이터를 모두 채우면 활성화됩니다.
                        </span>
                      )}
                      {complete && (
                        <span className="text-sm text-emerald-600 flex items-center gap-1">
                          <CheckCircle2 className="size-3.5" />
                          모든 필수 조건 충족
                        </span>
                      )}
                      <Button onClick={() => completeMut.mutate()} disabled={!complete || completeMut.isPending}>
                        {completeMut.isPending ? "처리 중..." : "계약 완료 처리"}
                      </Button>
                    </div>
                  )}
                </>
              )}
            </CardContent>
          </Card>
        </div>
    </div>
  )
}

function ContractField({
  id,
  fieldKey,
  label,
  initial,
  displayValue,
  meta,
  readOnly,
  onSave,
}: {
  id: string
  fieldKey: string
  label: string
  initial: string
  displayValue: string
  meta: ReturnType<typeof formMeta>
  readOnly: boolean
  onSave: (key: string, value: string) => void
}) {
  const [val, setVal] = useState(displayValue)

  function handleSave(raw: string) {
    setVal(raw)
    onSave(fieldKey, raw)
  }

  return (
    <label className="block text-sm">
      <span className="mb-1 block text-muted-foreground">{label}</span>
      {meta.formInput === "date" ? (
        <Input
          type="date"
          value={val}
          disabled={readOnly}
          onChange={(e) => {
            const v = fromDateInput(e.target.value)
            setVal(toDateInput(v))
            handleSave(v)
          }}
        />
      ) : meta.formInput === "money" ? (
        <Input
          value={val}
          disabled={readOnly}
          inputMode="decimal"
          placeholder="0"
          onChange={(e) => setVal(e.target.value)}
          onBlur={(e) => handleSave(e.target.value)}
        />
      ) : (
        <Input
          value={val}
          disabled={readOnly}
          onChange={(e) => handleSave(e.target.value)}
        />
      )}
    </label>
  )
}