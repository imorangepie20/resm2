import { useState } from "react"
import { Link, useNavigate } from "react-router"
import { useMutation, useQueryClient } from "@tanstack/react-query"
import { Plus, Star, Trash2, Pencil, Search, X, ChevronLeft, ChevronRight } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardFooter, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Checkbox } from "@/components/ui/checkbox"
import { ConfirmDialog } from "@/components/ui/confirm-dialog"
import { Empty, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { EmptyIllustration } from "@/components/empty-illustration"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { cn } from "@/lib/utils"
import { EditCell, SelectCell } from "@/components/data-grid/editable-cell"
import { FIELD_BY_KEY, LIST_COLUMNS, STATUS_OPTIONS, type PropertyField } from "@/lib/properties/fields"
import { TRADE_LABEL, TRADE_OPTIONS } from "@/lib/naver/trade-types"
import { PROPERTY_LABEL, PROPERTY_OPTIONS } from "@/lib/naver/property-types"
import { COLOR_TAGS, COLOR_TAG_MAP, type ColorTagValue } from "@/lib/properties/color-tags"
import { formatManwon, wonToManwon } from "@/lib/properties/money"
import { deleteProperties, toApiValue, updateProperty, type ApiValue, type PropertyInput, type PropertyRow, type PropertyView } from "@/lib/properties/api"

export const VIEW_TITLE: Record<PropertyView, string> = { all: "전체 매물", favorites: "관심 매물", "in-progress": "계약진행", contracted: "계약완료" }
const PAGE_SIZE = 50
const won = (v: string | null) => (v == null || v === "" ? "-" : formatManwon(v)) // 원 저장 → 만원 표시
const ymd = (v: string | number | null) => { const s = v == null ? "" : String(v); return s.length === 8 ? `${s.slice(0, 4)}.${s.slice(4, 6)}.${s.slice(6, 8)}` : (s || "-") }

function display(field: PropertyField, value: ApiValue) {
  if (value == null || value === "") return "-"
  switch (field.type) {
    case "money": return `${won(value as string)}만원`
    case "date": return ymd(value as string)
    case "select": return field.options?.find((o) => o.value === value)?.label ?? String(value)
    case "bool": return value ? "예" : "아니오"
    default: return String(value)
  }
}

const cell = (p: PropertyRow, k: string) => (p as unknown as Record<string, ApiValue>)[k]

const UPDATE_KEY = ["property-update"]

// 낙관적 반영은 쿼리 캐시(["properties", view])에 직접 한다(원본의 로컬 data 상태 대신).
export function PropertyList({ rows: data, view }: { rows: PropertyRow[]; view: PropertyView }) {
  const qc = useQueryClient()
  const navigate = useNavigate()

  const [sel, setSel] = useState<Set<string>>(new Set())
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [fType, setFType] = useState("ALL")
  const [fTrade, setFTrade] = useState("ALL")
  const [fStatus, setFStatus] = useState("ALL")
  const [q, setQ] = useState("")
  const [fColor, setFColor] = useState<string | null>(null)
  const [page, setPage] = useState(0)

  const kw = q.trim().toLowerCase()
  const rows = data.filter((p) => {
    if (fType !== "ALL" && p.realEstateType !== fType) return false
    if (fTrade !== "ALL" && p.tradeType !== fTrade) return false
    if (fStatus !== "ALL" && p.status !== fStatus) return false
    if (view === "all" && fColor && p.colorTag !== fColor) return false
    if (kw && ![p.name, p.complexName, p.address, p.customerName, p.customerPhone].some((v) => String(v ?? "").toLowerCase().includes(kw))) return false
    return true
  })
  const pageCount = Math.max(1, Math.ceil(rows.length / PAGE_SIZE))
  const current = Math.min(page, pageCount - 1)
  const paged = rows.slice(current * PAGE_SIZE, current * PAGE_SIZE + PAGE_SIZE)
  const allSelected = paged.length > 0 && paged.every((p) => sel.has(p.id))
  const someSelected = paged.some((p) => sel.has(p.id)) && !allSelected
  // 검색·필터가 바뀌면 1페이지로 돌아가고 선택도 비운다(보이지 않는 행이 선택 삭제에 섞이지 않게).
  const resetView = () => { setPage(0); setSel(new Set()) }
  const toggleAll = (c: boolean) => setSel(c ? new Set(paged.map((p) => p.id)) : new Set())
  const toggleOne = (id: string, c: boolean) => setSel((prev) => { const next = new Set(prev); if (c) next.add(id); else next.delete(id); return next })

  // 원본의 서버 액션 + router.refresh() → API 호출 + 쿼리 무효화. 실패 오류는 전역 토스트(query-client)가 띄운다.
  const invalidate = (id?: string) => {
    void qc.invalidateQueries({ queryKey: ["properties"] })
    if (id) void qc.invalidateQueries({ queryKey: ["property", id] })
  }
  // 캐시된 모든 목록에서 한 행의 필드를 바꾼다. onlyIf 가 있으면 그 필드가 아직 onlyIf 값일 때만(다른 변경이 덮었으면 둔다).
  const patchCached = (id: string, values: PropertyInput, onlyIf?: PropertyInput) =>
    qc.setQueriesData<PropertyRow[]>({ queryKey: ["properties"] }, (list) => list?.map((r) => {
      if (r.id !== id) return r
      const next = { ...r } as unknown as Record<string, ApiValue>
      for (const [k, v] of Object.entries(values)) if (!onlyIf || next[k] === onlyIf[k]) next[k] = v
      return next as unknown as PropertyRow
    }))
  const update = useMutation({
    mutationKey: UPDATE_KEY,
    mutationFn: (v: { id: string; patch: PropertyInput }) => updateProperty(v.id, v.patch),
    onMutate: async ({ id, patch }) => {
      await qc.cancelQueries({ queryKey: ["properties"] })
      let prev: PropertyInput | undefined
      for (const [, list] of qc.getQueriesData<PropertyRow[]>({ queryKey: ["properties"] })) {
        const r = list?.find((x) => x.id === id)
        if (r) { prev = Object.fromEntries(Object.keys(patch).map((k) => [k, cell(r, k)])); break }
      }
      patchCached(id, patch)
      return { prev }
    },
    // 실패하면 이 변경이 바꾼 행·필드만 되돌린다(다른 행의 진행 중인 변경은 건드리지 않음). 오류 토스트는 전역.
    onError: (_e, { id, patch }, ctx) => { if (ctx?.prev) patchCached(id, ctx.prev, patch) },
    // 다른 매물 변경이 아직 진행 중이면 재조회를 미룬다(그 변경의 낙관적 값을 덮지 않도록). 자기 자신은 아직 pending 으로 센다.
    onSettled: (_r, _e, { id }) => { if (qc.isMutating({ mutationKey: UPDATE_KEY }) === 1) invalidate(id) },
  })
  const remove = useMutation({
    mutationFn: (ids: string[]) => deleteProperties(ids),
    onSuccess: () => { toast.success("삭제했습니다"); setSel(new Set()); setConfirmOpen(false); invalidate() },
  })

  const patchRow = (p: PropertyRow, patch: PropertyInput) => update.mutate({ id: p.id, patch })
  function saveCell(p: PropertyRow, k: string, raw: string) {
    let v: ApiValue
    try { v = toApiValue(FIELD_BY_KEY[k], raw) } catch (e) { toast.error(e instanceof Error ? e.message : "저장 실패"); return }
    patchRow(p, { [k]: v })
  }
  const toggleFav = (p: PropertyRow) => patchRow(p, { isFavorite: !p.isFavorite })
  const setColor = (p: PropertyRow, colorTag: ColorTagValue | null) => patchRow(p, { colorTag })

  return (
    <Card className="gap-0">
      <CardHeader className="border-b">
        <div className="flex flex-col gap-3 lg:flex-row lg:items-start lg:justify-between">
          <div className="flex flex-col gap-1.5">
            <CardTitle>{VIEW_TITLE[view]}</CardTitle>
            {view === "all" && (
              <div className="flex items-center gap-1.5" role="group" aria-label="색상 필터">
                {COLOR_TAGS.map((c) => (
                  <button key={c.value} type="button" title={`${c.label} 필터`} aria-pressed={fColor === c.value}
                    onClick={() => { setFColor(fColor === c.value ? null : c.value); resetView() }}
                    className={cn("size-4 rounded-full transition", c.dot, fColor === c.value ? "ring-2 ring-foreground/50 ring-offset-1" : "opacity-50 hover:opacity-100")} />
                ))}
              </div>
            )}
          </div>
          <div className="flex flex-wrap items-center gap-2">
          <div className="relative max-sm:flex-1">
            <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
            <Input value={q} onChange={(e) => { setQ(e.target.value); resetView() }} placeholder="매물명·단지·주소·고객 검색" className="w-full pl-8 sm:w-52" />
          </div>
          <Select value={fType} onValueChange={(v) => { if (v != null) { setFType(v); resetView() } }}>
            <SelectTrigger className="h-8"><SelectValue>{fType === "ALL" ? "매물유형 전체" : PROPERTY_LABEL[fType] ?? fType}</SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value="ALL">매물유형 전체</SelectItem>
              {PROPERTY_OPTIONS.map((p) => <SelectItem key={p.value} value={p.value}>{p.label}</SelectItem>)}
            </SelectContent>
          </Select>
          <Select value={fTrade} onValueChange={(v) => { if (v != null) { setFTrade(v); resetView() } }}>
            <SelectTrigger className="h-8"><SelectValue>{fTrade === "ALL" ? "거래유형 전체" : TRADE_LABEL[fTrade] ?? fTrade}</SelectValue></SelectTrigger>
            <SelectContent>
              <SelectItem value="ALL">거래유형 전체</SelectItem>
              {TRADE_OPTIONS.map((t) => <SelectItem key={t.value} value={t.value}>{t.label}</SelectItem>)}
            </SelectContent>
          </Select>
          {view !== "contracted" && view !== "in-progress" && (
            <Select value={fStatus} onValueChange={(v) => { if (v != null) { setFStatus(v); resetView() } }}>
              <SelectTrigger className="h-8"><SelectValue>{fStatus === "ALL" ? "상태 전체" : fStatus}</SelectValue></SelectTrigger>
              <SelectContent>
                <SelectItem value="ALL">상태 전체</SelectItem>
                {STATUS_OPTIONS.map((s) => <SelectItem key={s.value} value={s.value}>{s.label}</SelectItem>)}
              </SelectContent>
            </Select>
          )}
          {sel.size > 0 && (
            <Button size="sm" variant="destructive" onClick={() => setConfirmOpen(true)} disabled={remove.isPending}>
              <Trash2 className="size-3.5" />삭제 {sel.size}
            </Button>
          )}
          {view === "all" && (
            <Button size="sm" nativeButton={false} render={<Link to="/properties/new" />}>
              <Plus className="size-3.5" />새 매물
            </Button>
          )}
          <span className="text-sm text-muted-foreground">{rows.length}개</span>
          </div>
        </div>
      </CardHeader>
      <CardContent className="p-0">
        {rows.length === 0 ? (
          <Empty className="border-0 py-10">
            <EmptyHeader>
              <EmptyMedia><EmptyIllustration /></EmptyMedia>
              <EmptyTitle>매물이 없습니다</EmptyTitle>
              <EmptyDescription>{'"새 매물"로 직접 등록하세요. 셀을 더블클릭하면 바로 수정됩니다.'}</EmptyDescription>
            </EmptyHeader>
          </Empty>
        ) : (
          <>
          <div className="hidden overflow-x-auto lg:block">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-10"><Checkbox checked={allSelected} indeterminate={someSelected} onCheckedChange={(c) => toggleAll(c)} aria-label="전체 선택" /></TableHead>
                  <TableHead className="w-12 text-right">#</TableHead>
                  <TableHead className={view === "all" ? "w-16" : "w-10"} aria-label="관심" />
                  {LIST_COLUMNS.map((k) => <TableHead key={k}>{FIELD_BY_KEY[k].label}</TableHead>)}
                  <TableHead className="w-10" aria-label="수정" />
                </TableRow>
              </TableHeader>
              <TableBody>
                {paged.map((p, i) => (
                  <TableRow key={p.id} data-state={sel.has(p.id) ? "selected" : undefined}
                    className={cn(view === "all" && p.colorTag ? COLOR_TAG_MAP[p.colorTag]?.row : undefined)}>
                    <TableCell><Checkbox checked={sel.has(p.id)} onCheckedChange={(c) => toggleOne(p.id, c)} aria-label="선택" /></TableCell>
                    <TableCell className="text-right text-muted-foreground tabular-nums">{rows.length - (current * PAGE_SIZE + i)}</TableCell>
                    <TableCell>
                      <div className="flex items-center gap-1.5">
                        <button type="button" onClick={() => toggleFav(p)} aria-label="관심" className="text-muted-foreground hover:text-foreground">
                          <Star className={cn("size-4", p.isFavorite && "fill-amber-400 text-amber-400")} />
                        </button>
                        {view === "all" && <ColorSwatch value={p.colorTag} onChange={(c) => setColor(p, c)} />}
                      </div>
                    </TableCell>
                    {LIST_COLUMNS.map((k) => {
                      const f = FIELD_BY_KEY[k]
                      const v = cell(p, k)
                      if (f.type === "select") {
                        return <TableCell key={k}><SelectCell value={String(v ?? "")} label={display(f, v)} options={f.options ?? []} onSave={(nv) => saveCell(p, k, nv)} /></TableCell>
                      }
                      if (k === "customerName") {
                        // 원본의 "미등록" 아이콘(2단계)은 뺐다. 셀 폭이 원본과 같도록 감싸는 div 는 남긴다.
                        return <TableCell key={k}>
                          <div className="flex items-center gap-1">
                            <EditCell value={v as string | number | null} display={display(f, v)} onSave={(nv) => saveCell(p, k, nv)} />
                          </div>
                        </TableCell>
                      }
                      const numeric = f.type === "money" || f.type === "number" || f.type === "area"
                      const editV = f.type === "money" && v != null ? wonToManwon(v as string) : (v as string | number | null) // money는 만원으로 편집
                      return <TableCell key={k} className={cn(k === "complexName" && "min-w-40 font-medium")}><EditCell numeric={numeric} value={editV} display={display(f, v)} onSave={(nv) => saveCell(p, k, nv)} /></TableCell>
                    })}
                    <TableCell>
                      <Link to={`/properties/${p.id}/edit`} aria-label="수정" className="text-muted-foreground hover:text-foreground"><Pencil className="size-4" /></Link>
                    </TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          </div>

          <div className="flex flex-col gap-2 p-3 lg:hidden">
            {paged.map((p) => {
              const rowColor = view === "all" && p.colorTag ? COLOR_TAG_MAP[p.colorTag]?.row : undefined
              const cname = String(p.customerName ?? "")
              return (
                // 카드 본문을 누르면 수정 화면. 안쪽 컨트롤(체크박스·별·색상·수정 링크)과 텍스트 선택은 제외한다.
                // 키보드 경로는 안쪽 "수정" 링크가 맡는다(카드 전체를 버튼으로 만들지 않음).
                <div
                  key={p.id}
                  // 팝오버가 열린 채 누른 탭은 팝오버를 닫는 데만 쓴다(누르는 순간의 상태를 기억).
                  onPointerDownCapture={(e) => {
                    e.currentTarget.dataset.popoverOpen = document.querySelector('[data-slot="popover-content"]') ? "1" : ""
                  }}
                  onClick={(e) => {
                    // 포털(색상 팝오버)에서 올라온 클릭은 카드 것이 아니다.
                    if (!e.currentTarget.contains(e.target as Node)) return
                    if (e.currentTarget.dataset.popoverOpen) return
                    if ((e.target as Element).closest("a,button,input,label,select,textarea,[role=checkbox],[role=menuitem],[data-no-card-nav]")) return
                    if (window.getSelection()?.toString()) return
                    navigate(`/properties/${p.id}/edit`)
                  }}
                  className={cn("flex cursor-pointer flex-col gap-2 rounded-lg border p-3", rowColor, sel.has(p.id) && "ring-2 ring-primary")}
                >
                  <div className="flex items-start gap-2">
                    <Checkbox checked={sel.has(p.id)} onCheckedChange={(c) => toggleOne(p.id, c)} aria-label="선택" className="mt-1" />
                    <div className="min-w-0 flex-1">
                      <div className="truncate font-medium">{String(p.name || p.complexName || "(이름 없음)")}</div>
                      <div className="mt-0.5 flex flex-wrap items-center gap-x-2 gap-y-0.5 text-xs text-muted-foreground">
                        {p.name && p.complexName ? <span className="truncate">{String(p.complexName)}</span> : null}
                        {p.realEstateType ? <span>{PROPERTY_LABEL[String(p.realEstateType)] ?? String(p.realEstateType)}</span> : null}
                        {p.tradeType ? <span>{TRADE_LABEL[String(p.tradeType)] ?? String(p.tradeType)}</span> : null}
                        {p.areaExclusive ? <span>{display(FIELD_BY_KEY.areaExclusive, p.areaExclusive)}</span> : null}
                      </div>
                    </div>
                    <div className="flex shrink-0 items-center gap-1.5">
                      <button type="button" onClick={() => toggleFav(p)} aria-label="관심" className="text-muted-foreground hover:text-foreground">
                        <Star className={cn("size-4", p.isFavorite && "fill-amber-400 text-amber-400")} />
                      </button>
                      {view === "all" && <ColorSwatch value={p.colorTag} onChange={(c) => setColor(p, c)} />}
                    </div>
                  </div>
                  <div className="flex flex-wrap items-center justify-between gap-2">
                    <div className="flex items-center gap-2">
                      <Badge variant="secondary">{String(p.status ?? "진행")}</Badge>
                      <span className="text-sm font-medium tabular-nums">{display(FIELD_BY_KEY.price, p.price)}</span>
                    </div>
                    <div className="flex items-center gap-3 text-sm">
                      <Link to={`/properties/${p.id}/edit`} className="inline-flex items-center gap-1 text-muted-foreground hover:text-foreground"><Pencil className="size-3.5" />수정</Link>
                    </div>
                  </div>
                  {cname && (
                    <div className="flex items-center gap-1 text-sm text-muted-foreground">
                      고객: <span className="text-foreground">{cname}</span>
                    </div>
                  )}
                </div>
              )
            })}
          </div>
          </>
        )}
      </CardContent>
      {pageCount > 1 && (
        <CardFooter className="justify-between border-t">
          <span className="text-sm text-muted-foreground">총 {rows.length}개</span>
          <div className="flex items-center gap-2">
            <Button size="sm" variant="outline" onClick={() => setPage(current - 1)} disabled={current === 0}>
              <ChevronLeft className="size-3.5" />이전
            </Button>
            <span className="text-sm tabular-nums text-muted-foreground">{current + 1} / {pageCount}</span>
            <Button size="sm" variant="outline" onClick={() => setPage(current + 1)} disabled={current >= pageCount - 1}>
              다음<ChevronRight className="size-3.5" />
            </Button>
          </div>
        </CardFooter>
      )}
      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={`선택한 매물 ${sel.size}개를 삭제할까요?`}
        description="삭제한 매물은 되돌릴 수 없습니다."
        busy={remove.isPending}
        onConfirm={() => remove.mutate([...sel])}
      />
    </Card>
  )
}

// 행 색상 태그 스와치 — 클릭 시 팝오버로 4색 선택/해제. (전체 매물 메뉴 전용)
function ColorSwatch({ value, onChange }: { value: ColorTagValue | null; onChange: (c: ColorTagValue | null) => void }) {
  const [open, setOpen] = useState(false)
  const cur = value ? COLOR_TAG_MAP[value] : null
  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger
        render={
          <button
            type="button"
            aria-label="색상 태그"
            className={cn(
              "size-3.5 shrink-0 rounded-full border transition",
              cur ? cn(cur.dot, "border-transparent") : "border-dashed border-muted-foreground/50 hover:border-foreground",
            )}
          />
        }
      />
      <PopoverContent align="start" sideOffset={6} className="w-auto flex-row items-center gap-1.5 p-1.5">
        {COLOR_TAGS.map((c) => (
          <button
            key={c.value}
            type="button"
            title={c.label}
            onClick={() => { onChange(c.value); setOpen(false) }}
            className={cn("size-5 rounded-full ring-foreground/40 ring-offset-1 ring-offset-popover transition hover:ring-2", c.dot, value === c.value && "ring-2")}
          />
        ))}
        <button
          type="button"
          title="없음"
          onClick={() => { onChange(null); setOpen(false) }}
          className="flex size-5 items-center justify-center rounded-full border border-dashed text-muted-foreground transition hover:bg-muted"
        >
          <X className="size-3" />
        </button>
      </PopoverContent>
    </Popover>
  )
}
