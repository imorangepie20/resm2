import { useState, useEffect } from "react"
import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query"
import { Plus, Search, X, Trash2, Pencil, User, Mail, Phone, MapPin, Calendar, FileText } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle, CardFooter } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { Badge } from "@/components/ui/badge"
import { Checkbox } from "@/components/ui/checkbox"
import { ConfirmDialog } from "@/components/ui/confirm-dialog"
import { Empty, EmptyContent, EmptyDescription, EmptyHeader, EmptyMedia, EmptyTitle } from "@/components/ui/empty"
import { Skeleton } from "@/components/ui/skeleton"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover"
import { Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger, DialogFooter } from "@/components/ui/dialog"
import { Label } from "@/components/ui/label"
import { Textarea } from "@/components/ui/textarea"
import { cn } from "@/lib/utils"
import { listCustomers, createCustomer, updateCustomer, deleteCustomer, getCustomer, type CustomerRow, type CustomerInput } from "@/lib/api"
import { CUSTOMER_TYPES, PARTY_TYPES } from "@/lib/properties/customer-types"

const PAGE_SIZE = 20
const CUSTOMER_TYPE_LABELS: Record<string, string> = {
  매도인: "매도인",
  매수인: "매수인",
  임대인: "임대인",
  임차인: "임차인",
  단순방문: "단순방문",
}
const GENDER_LABELS: Record<string, string> = { 남: "남", 여: "여", 미지정: "미지정" }
const GENDER_OPTIONS = ["남", "여", "미지정"]
const PARTY_ROLE_LABELS: Record<string, string> = { 매도인: "매도인", 매수인: "매수인", 임대인: "임대인", 임차인: "임차인" }

type FormInput = {
  name: string
  zipcode: string
  phone: string
  address: string
  email: string
  gender: string
  memo: string
  types: string[]
  propertyId: string | null
}

function CustomerTypeBadges({ types }: { types: string[] }) {
  if (!types.length) return <span className="text-muted-foreground">-</span>
  return (
    <div className="flex flex-wrap gap-1">
      {types.map((t) => (
        <Badge key={t} variant="outline" className="text-xs">
          {CUSTOMER_TYPE_LABELS[t] ?? t}
        </Badge>
      ))}
    </div>
  )
}

function GenderBadge({ gender }: { gender: string | null }) {
  if (!gender) return <span className="text-muted-foreground">-</span>
  return <Badge variant="secondary" className="text-xs">{GENDER_LABELS[gender] ?? gender}</Badge>
}

export function CustomersPage() {
  console.log("[CustomersPage] Component rendered")
  const qc = useQueryClient()
  const [q, setQ] = useState("")
  const [fType, setFType] = useState<string>("ALL")
  const [page, setPage] = useState(0)
  const [sel, setSel] = useState<Set<string>>(new Set())
  const [confirmOpen, setConfirmOpen] = useState(false)
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editing, setEditing] = useState<CustomerRow | null>(null)
  const [form, setForm] = useState<FormInput>({
    name: "",
    zipcode: "",
    phone: "",
    address: "",
    email: "",
    gender: "",
    memo: "",
    types: [],
    propertyId: null,
  })

  const query = useQuery({
    queryKey: ["customers", q, fType],
    queryFn: () => listCustomers(q || undefined, fType === "ALL" ? undefined : fType),
  })

  const createMut = useMutation({
    mutationFn: (input: CustomerInput) => createCustomer(input),
    onSuccess: () => {
      toast.success("고객을 등록했습니다")
      setDialogOpen(false)
      resetForm()
      qc.invalidateQueries({ queryKey: ["customers"] })
    },
    onError: (e: Error) => toast.error(e.message),
  })

  const updateMut = useMutation({
    mutationFn: ({ id, input }: { id: string; input: CustomerInput }) => updateCustomer(id, input),
    onSuccess: () => {
      console.log("[updateMut] onSuccess called")
      toast.success("고객 정보를 수정했습니다")
      setDialogOpen(false)
      setEditing(null)
      resetForm()
      qc.invalidateQueries({ queryKey: ["customers"] })
    },
    onError: (e: Error) => {
      console.log("[updateMut] onError called:", e)
      toast.error(e.message)
    },
    onSettled: () => {
      console.log("[updateMut] onSettled called")
    },
  })

  const removeMut = useMutation({
    mutationFn: async (ids: string[]) => {
      console.log("[removeMut] mutationFn called with:", ids)
      const result = await Promise.all(ids.map((id) => deleteCustomer(id)))
      console.log("[removeMut] mutationFn completed:", result)
      return result
    },
    onSuccess: (data) => {
      console.log("[removeMut] onSuccess called with data:", data)
      toast.success("삭제했습니다")
      setSel(new Set())
      setConfirmOpen(false)
      qc.invalidateQueries({ queryKey: ["customers"] })
    },
    onError: (e: Error) => {
      console.log("[removeMut] onError called:", e)
      toast.error(e.message)
    },
    onSettled: (data, error) => {
      console.log("[removeMut] onSettled called, data:", data, "error:", error)
    },
  })

  // Debug: log mutation state changes
  useEffect(() => {
    console.log("[CustomersPage] removeMut state changed:", {
      isPending: removeMut.isPending,
      isSuccess: removeMut.isSuccess,
      isError: removeMut.isError,
      status: removeMut.status,
    })
  }, [removeMut.isPending, removeMut.isSuccess, removeMut.isError, removeMut.status])

  const rows = query.data ?? []
  const kw = q.trim().toLowerCase()
  const filtered = rows.filter((c) => {
    if (fType !== "ALL" && !c.types.includes(fType)) return false
    if (kw && ![c.name, c.phone, c.email, c.address, c.propertyName].some((v) => String(v ?? "").toLowerCase().includes(kw))) return false
    return true
  })
  const pageCount = Math.max(1, Math.ceil(filtered.length / PAGE_SIZE))
  const current = Math.min(page, pageCount - 1)
  const paged = filtered.slice(current * PAGE_SIZE, current * PAGE_SIZE + PAGE_SIZE)
  const allSelected = paged.length > 0 && paged.every((c) => sel.has(c.id))
  const someSelected = paged.some((c) => sel.has(c.id)) && !allSelected
  const resetView = () => { setPage(0); setSel(new Set()) }
  const toggleAll = (c: boolean) => setSel(c ? new Set(paged.map((c) => c.id)) : new Set())
  const toggleOne = (id: string, c: boolean) => setSel((prev) => { const next = new Set(prev); if (c) next.add(id); else next.delete(id); return next })

  function openCreate() {
    setEditing(null)
    resetForm()
    setDialogOpen(true)
  }

  function openEdit(customer: CustomerRow) {
    setEditing(customer)
    setForm({
      name: customer.name,
      zipcode: customer.zipcode ?? "",
      phone: customer.phone ?? "",
      address: customer.address ?? "",
      email: customer.email ?? "",
      gender: customer.gender ?? "",
      memo: customer.memo ?? "",
      types: customer.types,
      propertyId: customer.propertyId ?? null,
    })
    setDialogOpen(true)
  }

  function resetForm() {
    setForm({ name: "", zipcode: "", phone: "", address: "", email: "", gender: "", memo: "", types: [], propertyId: null })
  }

  function handleTypeChange(type: string, checked: boolean) {
    setForm((prev) => ({
      ...prev,
      types: checked
        ? [...prev.types, type].filter((t, i, arr) => arr.indexOf(t) === i)
        : prev.types.filter((t) => t !== type),
    }))
  }

  function handleSimpleVisitChange(checked: boolean) {
    setForm((prev) => {
      if (checked) return { ...prev, types: ["단순방문"] }
      if (prev.types.length === 1 && prev.types[0] === "단순방문") return { ...prev, types: [] }
      return prev
    })
  }

  function handleSubmit(e: React.FormEvent) {
    e.preventDefault()
    const input: CustomerInput = {
      name: form.name.trim(),
      zipcode: form.zipcode.trim() || null,
      phone: form.phone.trim() || null,
      address: form.address.trim() || null,
      email: form.email.trim() || null,
      gender: form.gender || null,
      memo: form.memo.trim() || null,
      types: form.types,
      propertyId: form.propertyId || null,
    }
    console.log("[handleSubmit] input:", input, "editing:", editing?.id)
    if (!input.name) { toast.error("이름은 필수입니다"); return }
    if (editing) updateMut.mutate({ id: editing.id, input })
    else createMut.mutate(input)
  }

  if (query.isLoading) {
    return (
      <div className="flex flex-col gap-4">
        <h1 className="text-xl font-semibold">고객 관리</h1>
        <Card className="gap-0">
          <CardHeader className="border-b"><CardTitle>고객 목록</CardTitle></CardHeader>
          <CardContent className="p-0">
            <div className="flex flex-col gap-2 p-3" aria-busy="true">
              {Array.from({ length: 8 }, (_, i) => <Skeleton key={i} className="h-10 w-full" />)}
            </div>
          </CardContent>
        </Card>
      </div>
    )
  }

  return (
    <div className="flex flex-col gap-4">
      <h1 className="text-xl font-semibold">고객 관리</h1>

      <Card className="gap-0">
        <CardHeader className="border-b">
          <div className="flex flex-col gap-3 lg:flex-row lg:items-center lg:justify-between">
            <CardTitle>고객 목록</CardTitle>
            <div className="flex flex-wrap items-center gap-2">
              <div className="relative max-sm:flex-1">
                <Search className="pointer-events-none absolute left-2.5 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  value={q}
                  onChange={(e) => { setQ(e.target.value); resetView() }}
                  placeholder="이름·전화·이메일·주소·매물 검색"
                  className="w-full pl-8 sm:w-52"
                />
              </div>
              <Select value={fType} onValueChange={(v) => { if (v != null) { setFType(v); resetView() } }}>
                <SelectTrigger className="h-8 w-36"><SelectValue>{fType === "ALL" ? "유형 전체" : CUSTOMER_TYPE_LABELS[fType] ?? fType}</SelectValue></SelectTrigger>
                <SelectContent>
                  <SelectItem value="ALL">유형 전체</SelectItem>
                  {CUSTOMER_TYPES.map((t) => <SelectItem key={t} value={t}>{CUSTOMER_TYPE_LABELS[t]}</SelectItem>)}
                </SelectContent>
              </Select>
              {sel.size > 0 && (
                <Button size="sm" variant="destructive" onClick={() => setConfirmOpen(true)} disabled={removeMut.isPending}>
                  <Trash2 className="size-3.5" />삭제 {sel.size}
                </Button>
              )}
              <Button size="sm" onClick={openCreate}>
                <Plus className="size-3.5" />새 고객
              </Button>
              <span className="text-sm text-muted-foreground">{filtered.length}명</span>
            </div>
          </div>
        </CardHeader>
        <CardContent className="p-0">
          {filtered.length === 0 ? (
            <Empty className="border-0 py-10">
              <EmptyHeader>
                <EmptyMedia><User className="size-12" /></EmptyMedia>
                <EmptyTitle>고객이 없습니다</EmptyTitle>
                <EmptyDescription>{'"새 고객"으로 직접 등록하세요.'}</EmptyDescription>
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
                      <TableHead>유형</TableHead>
                      <TableHead className="min-w-40">이름</TableHead>
                      <TableHead className="min-w-32">전화</TableHead>
                      <TableHead className="min-w-36">이메일</TableHead>
                      <TableHead className="w-24">성별</TableHead>
                      <TableHead className="min-w-40">연결 매물</TableHead>
                      <TableHead className="w-10" aria-label="수정" />
                    </TableRow>
                  </TableHeader>
                  <TableBody>
                    {paged.map((c, i) => (
                      <TableRow key={c.id} data-state={sel.has(c.id) ? "selected" : undefined}>
                        <TableCell><Checkbox checked={sel.has(c.id)} onCheckedChange={(v) => toggleOne(c.id, v as boolean)} aria-label="선택" /></TableCell>
                        <TableCell className="text-right text-muted-foreground tabular-nums">{filtered.length - (current * PAGE_SIZE + i)}</TableCell>
                        <TableCell><CustomerTypeBadges types={c.types} /></TableCell>
                        <TableCell className="font-medium">{c.name}</TableCell>
                        <TableCell className="text-muted-foreground">{c.phone ?? "-"}</TableCell>
                        <TableCell className="text-muted-foreground">{c.email ?? "-"}</TableCell>
                        <TableCell><GenderBadge gender={c.gender} /></TableCell>
                        <TableCell className="text-muted-foreground">
                          {c.propertyName ? (
                            <span className="truncate block max-w-[200px]" title={c.propertyName}>{c.propertyName}</span>
                          ) : "-"}
                        </TableCell>
                        <TableCell>
                          <Button variant="ghost" size="icon" onClick={() => openEdit(c)} aria-label="수정">
                            <Pencil className="size-3.5" />
                          </Button>
                        </TableCell>
                      </TableRow>
                    ))}
                  </TableBody>
                </Table>
              </div>

              <div className="flex flex-col gap-2 p-3 lg:hidden">
                {paged.map((c) => (
                  <div
                    key={c.id}
                    onClick={(e) => {
                      if ((e.target as Element).closest("button,a,label,input,select,textarea,[role=checkbox]")) return
                      openEdit(c)
                    }}
                    className="flex cursor-pointer flex-col gap-2 rounded-lg border p-3 hover:bg-muted/50"
                  >
                    <div className="flex items-start gap-2">
                      <Checkbox checked={sel.has(c.id)} onCheckedChange={(v) => toggleOne(c.id, v as boolean)} aria-label="선택" className="mt-1" />
                      <div className="min-w-0 flex-1">
                        <div className="flex items-center gap-2">
                          <span className="truncate font-medium">{c.name}</span>
                          <CustomerTypeBadges types={c.types} />
                        </div>
                        <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-sm text-muted-foreground">
                          {c.phone && <span className="flex items-center gap-1"><Phone className="size-3" />{c.phone}</span>}
                          {c.email && <span className="flex items-center gap-1"><Mail className="size-3" />{c.email}</span>}
                          {c.address && <span className="flex items-center gap-1"><MapPin className="size-3" />{c.address}</span>}
                          {c.propertyName && <span className="flex items-center gap-1"><FileText className="size-3" />{c.propertyName}</span>}
                        </div>
                      </div>
                    </div>
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <GenderBadge gender={c.gender} />
                        {c.gender && <span className="text-xs text-muted-foreground">{GENDER_LABELS[c.gender]}</span>}
                      </div>
                      <Button variant="ghost" size="icon" onClick={(e) => { e.stopPropagation(); openEdit(c) }} aria-label="수정">
                        <Pencil className="size-3.5" />
                      </Button>
                    </div>
                  </div>
                ))}
              </div>
            </>
          )}
        </CardContent>
        {pageCount > 1 && (
          <CardFooter className="justify-between border-t">
            <span className="text-sm text-muted-foreground">총 {filtered.length}명</span>
            <div className="flex items-center gap-2">
              <Button size="sm" variant="outline" onClick={() => setPage(current - 1)} disabled={current === 0}>
                이전
              </Button>
              <span className="text-sm tabular-nums text-muted-foreground">{current + 1} / {pageCount}</span>
              <Button size="sm" variant="outline" onClick={() => setPage(current + 1)} disabled={current >= pageCount - 1}>
                다음
              </Button>
            </div>
          </CardFooter>
        )}
      </Card>

      <ConfirmDialog
        open={confirmOpen}
        onOpenChange={setConfirmOpen}
        title={`선택한 고객 ${sel.size}명을 삭제할까요?`}
        description="삭제한 고객은 되돌릴 수 없습니다."
        busy={removeMut.isPending}
        onConfirm={() => {
          removeMut.mutate([...sel])
        }}
      />

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="sm:max-w-[600px]">
          <DialogHeader>
            <DialogTitle>{editing ? "고객 수정" : "새 고객 등록"}</DialogTitle>
          </DialogHeader>
          <div className="p-4 pt-0 grid gap-4">
              <div className="grid gap-2 sm:grid-cols-2">
                <div>
                  <Label htmlFor="name">이름 <span className="text-red-500">*</span></Label>
                  <Input
                    id="name"
                    value={form.name}
                    onChange={(e) => setForm({ ...form, name: e.target.value })}
                    placeholder="홍길동"
                    required
                    disabled={createMut.isPending || updateMut.isPending}
                  />
                </div>
                <div>
                  <Label htmlFor="gender">성별</Label>
                  <Select
                    value={form.gender}
                    onValueChange={(v) => setForm({ ...form, gender: v || "" })}
                    disabled={createMut.isPending || updateMut.isPending}
                  >
                    <SelectTrigger id="gender"><SelectValue placeholder="선택">{form.gender ? GENDER_LABELS[form.gender] : ""}</SelectValue></SelectTrigger>
                    <SelectContent>
                      {GENDER_OPTIONS.map((g) => <SelectItem key={g} value={g}>{GENDER_LABELS[g]}</SelectItem>)}
                    </SelectContent>
                  </Select>
                </div>
              </div>

              <div className="grid gap-2 sm:grid-cols-2">
                <div>
                  <Label htmlFor="phone">전화번호</Label>
                  <Input
                    id="phone"
                    value={form.phone}
                    onChange={(e) => setForm({ ...form, phone: e.target.value })}
                    placeholder="010-1234-5678"
                    disabled={createMut.isPending || updateMut.isPending}
                  />
                </div>
                <div>
                  <Label htmlFor="email">이메일</Label>
                  <Input
                    id="email"
                    type="email"
                    value={form.email}
                    onChange={(e) => setForm({ ...form, email: e.target.value })}
                    placeholder="hong@example.com"
                    disabled={createMut.isPending || updateMut.isPending}
                  />
                </div>
              </div>

              <div>
                <Label htmlFor="address">주소</Label>
                <Input
                  id="address"
                  value={form.address}
                  onChange={(e) => setForm({ ...form, address: e.target.value })}
                  placeholder="서울시 강남구..."
                  disabled={createMut.isPending || updateMut.isPending}
                />
              </div>

              <div className="grid gap-2 sm:grid-cols-2">
                <div>
                  <Label htmlFor="zipcode">우편번호</Label>
                  <Input
                    id="zipcode"
                    value={form.zipcode}
                    onChange={(e) => setForm({ ...form, zipcode: e.target.value })}
                    placeholder="06000"
                    disabled={createMut.isPending || updateMut.isPending}
                  />
                </div>
                <div>
                  <Label htmlFor="propertyId">연결 매물 ID (선택)</Label>
                  <Input
                    id="propertyId"
                    value={form.propertyId ?? ""}
                    onChange={(e) => setForm({ ...form, propertyId: e.target.value || null })}
                    placeholder="자동 연결 시 입력"
                    disabled={createMut.isPending || updateMut.isPending}
                  />
                </div>
              </div>

              <fieldset className="space-y-2">
                <legend className="text-sm font-medium">고객 유형</legend>
                <div className="grid grid-cols-2 gap-2">
                  {PARTY_TYPES.map((t) => (
                    <label key={t} className="flex items-center gap-2 text-sm">
                      <Checkbox
                        checked={form.types.includes(t)}
                        onCheckedChange={(checked) => handleTypeChange(t, checked)}
                        disabled={createMut.isPending || updateMut.isPending}
                      />
                      <span>{PARTY_ROLE_LABELS[t]}</span>
                    </label>
                  ))}
                  <label className="flex items-center gap-2 text-sm sm:col-span-2">
                    <Checkbox
                      checked={form.types.includes("단순방문") && !form.types.some((t) => (PARTY_TYPES as readonly string[]).includes(t))}
                      onCheckedChange={handleSimpleVisitChange}
                      disabled={createMut.isPending || updateMut.isPending}
                    />
                    <span>단순방문 (당사자 유형이 없으면 기본)</span>
                  </label>
                </div>
              </fieldset>

              <div>
                <Label htmlFor="memo">메모</Label>
                <Textarea
                  id="memo"
                  value={form.memo}
                  onChange={(e) => setForm({ ...form, memo: e.target.value })}
                  placeholder="특이사항..."
                  rows={3}
                  disabled={createMut.isPending || updateMut.isPending}
                />
              </div>
            </div>
            <DialogFooter>
              <Button type="button" variant="outline" onClick={() => setDialogOpen(false)} disabled={createMut.isPending || updateMut.isPending}>
                취소
              </Button>
              <Button
                type="button"
                disabled={createMut.isPending || updateMut.isPending || !form.name.trim()}
                onClick={handleSubmit}
              >
                {createMut.isPending || updateMut.isPending ? "처리 중..." : (editing ? "수정 완료" : "등록")}
              </Button>
            </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}