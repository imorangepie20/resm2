import { useState, useMemo, useCallback, useEffect } from "react"
import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useSearchParams } from "react-router"
import { format, startOfMonth, endOfMonth, startOfWeek, endOfWeek, addDays, isSameMonth, isToday, isSameDay, getDay, getDate, getMonth, getYear, subMonths, addMonths, eachDayOfInterval } from "date-fns"
import { ko } from "date-fns/locale"
import { Plus, ChevronLeft, ChevronRight, Trash2, Edit2, Calendar } from "lucide-react"
import { Dialog, DialogTrigger, DialogContent, DialogHeader, DialogTitle, DialogFooter } from "@/components/ui/dialog"
import { Button } from "@/components/ui/button"
import { Input } from "@/components/ui/input"
import { Textarea } from "@/components/ui/textarea"
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Badge } from "@/components/ui/badge"
import { Separator } from "@/components/ui/separator"
import { Label } from "@/components/ui/label"
import { loadCalendar, createEvent, updateEvent, deleteEvent, type EventRow, type CalendarResponse, type CalendarOption, type CreateEventInput, type UpdateEventInput } from "@/lib/properties/api"

const CATEGORIES = ["미팅", "임장", "계약", "기타"]
const CATEGORY_COLORS: Record<string, string> = {
  미팅: "bg-blue-100 text-blue-800 dark:bg-blue-900/30 dark:text-blue-300",
  임장: "bg-green-100 text-green-800 dark:bg-green-900/30 dark:text-green-300",
  계약: "bg-purple-100 text-purple-800 dark:bg-purple-900/30 dark:text-purple-300",
  기타: "bg-gray-100 text-gray-800 dark:bg-gray-900/30 dark:text-gray-300",
}

function dateToYMD(d: Date): string {
  return format(d, "yyyyMMdd")
}

function ymdToDate(s: string): Date {
  return new Date(Number(s.slice(0, 4)), Number(s.slice(4, 6)) - 1, Number(s.slice(6, 8)))
}

export function CalendarPage() {
  const qc = useQueryClient()
  const [searchParams, setSearchParams] = useSearchParams()
  const [viewDate, setViewDate] = useState(() => {
    const y = searchParams.get("y")
    const m = searchParams.get("m")
    if (y && m) {
      const year = parseInt(y, 10)
      const month = parseInt(m, 10)
      if (!isNaN(year) && !isNaN(month) && month >= 0 && month <= 11) {
        return new Date(year, month, 1)
      }
    }
    return new Date()
  })
  const [dialogOpen, setDialogOpen] = useState(false)
  const [editingEvent, setEditingEvent] = useState<EventRow | null>(null)
  const [form, setForm] = useState<CreateEventInput>({
    title: "", date: "", startTime: "", category: "미팅", memo: "", propertyId: "", customerId: "",
  })

  // URL 동기화: viewDate 변경 시 URL 업데이트
  useEffect(() => {
    console.log("[CalendarPage] viewDate changed:", viewDate)
    const params = new URLSearchParams()
    params.set("y", String(viewDate.getFullYear()))
    params.set("m", String(viewDate.getMonth()))
    setSearchParams(params, { replace: true })
    console.log("[CalendarPage] URL updated")
  }, [viewDate, setSearchParams])

  const { data: calendar, isLoading } = useQuery({
    queryKey: ["calendar", viewDate.getFullYear(), viewDate.getMonth()],
    queryFn: () => loadCalendar(viewDate.getFullYear(), viewDate.getMonth()),
  })

  const createMut = useMutation({
    mutationFn: createEvent,
    onSuccess: () => {
      toastSuccess("일정 추가됨")
      closeDialog()
      qc.invalidateQueries({ queryKey: ["calendar", viewDate.getFullYear(), viewDate.getMonth()] })
    },
    onError: (e: Error) => toastError(e.message),
  })

  const updateMut = useMutation({
    mutationFn: ({ id, input }: { id: string; input: UpdateEventInput }) => updateEvent(id, input),
    onSuccess: () => {
      toastSuccess("일정 수정됨")
      closeDialog()
      qc.invalidateQueries({ queryKey: ["calendar", viewDate.getFullYear(), viewDate.getMonth()] })
    },
    onError: (e: Error) => toastError(e.message),
  })

  const deleteMut = useMutation({
    mutationFn: deleteEvent,
    onSuccess: () => {
      toastSuccess("일정 삭제됨")
      qc.invalidateQueries({ queryKey: ["calendar", viewDate.getFullYear(), viewDate.getMonth()] })
    },
    onError: (e: Error) => toastError(e.message),
  })

  function toastSuccess(msg: string) {
    import("sonner").then(({ toast }) => toast.success(msg))
  }
  function toastError(msg: string) {
    import("sonner").then(({ toast }) => toast.error(msg))
  }

  const days = useMemo(() => {
    const monthStart = startOfMonth(viewDate)
    const monthEnd = endOfMonth(viewDate)
    const calStart = startOfWeek(monthStart, { weekStartsOn: 0 })
    const calEnd = endOfWeek(monthEnd, { weekStartsOn: 0 })
    return eachDayOfInterval({ start: calStart, end: calEnd })
  }, [viewDate])

  const eventsByDay = useMemo(() => {
    const map = new Map<string, EventRow[]>()
    calendar?.events.forEach((e) => {
      const arr = map.get(e.date) ?? []
      arr.push(e)
      map.set(e.date, arr)
    })
    return map
  }, [calendar?.events])

  const propDatesByDay = useMemo(() => {
    const map = new Map<string, string[]>()
    calendar?.propertyDates.forEach((pd) => {
      const arr = map.get(pd.date) ?? []
      arr.push(pd.label)
      map.set(pd.date, arr)
    })
    return map
  }, [calendar?.propertyDates])

  const openCreate = () => {
    setEditingEvent(null)
    setForm({ title: "", date: dateToYMD(viewDate), startTime: "", category: "미팅", memo: "", propertyId: "", customerId: "" })
    setDialogOpen(true)
  }

  const openEdit = (e: EventRow) => {
    setEditingEvent(e)
    setForm({ title: e.title, date: e.date, startTime: e.startTime ?? "", category: e.category, memo: e.memo ?? "", propertyId: e.propertyId ?? "", customerId: e.customerId ?? "" })
    setDialogOpen(true)
  }

  const closeDialog = () => {
    setDialogOpen(false)
    setEditingEvent(null)
  }

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    const payload = { ...form, propertyId: form.propertyId || undefined, customerId: form.customerId || undefined }
    if (editingEvent) {
      updateMut.mutate({ id: editingEvent.id, input: payload })
    } else {
      createMut.mutate(payload)
    }
  }

  return (
    <div className="flex flex-col gap-4">
      <div className="flex items-center justify-between">
        <h1 className="text-xl font-semibold">일정</h1>
        <Button onClick={openCreate}><Plus className="mr-2 h-4 w-4" />일정 추가</Button>
      </div>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between">
          <CardTitle>
            <Calendar className="mr-2 h-4 w-4 inline" />
            {format(viewDate, "yyyy년 M월", { locale: ko })}
          </CardTitle>
          <div className="flex gap-2">
            <Button variant="outline" size="icon" onClick={() => setViewDate(subMonths(viewDate, 1))}><ChevronLeft className="h-4 w-4" /></Button>
            <Button variant="outline" size="icon" onClick={() => setViewDate(new Date())}>오늘</Button>
            <Button variant="outline" size="icon" onClick={() => setViewDate(addMonths(viewDate, 1))}><ChevronRight className="h-4 w-4" /></Button>
          </div>
        </CardHeader>
        <CardContent>
          {isLoading ? (
            <div className="flex justify-center py-12">불러오는 중...</div>
          ) : (
            <div className="grid grid-cols-7 gap-0">
              {["일", "월", "화", "수", "목", "금", "토"].map((d) => (
                <div key={d} className="text-center text-sm font-medium text-muted-foreground py-2 border-b">{d}</div>
              ))}
              {days.map((day) => {
                const ymd = dateToYMD(day)
                const dayEvents = eventsByDay.get(ymd) ?? []
                const dayPropDates = propDatesByDay.get(ymd) ?? []
                const today = isToday(day)
                const currentMonth = isSameMonth(day, viewDate)
                return (
                  <div
                    key={ymd}
                    data-date={ymd}
                    className={`relative min-h-[100px] border-r border-b p-1 transition-colors ${
                      today ? "bg-primary/5" : ""
                    } ${!currentMonth ? "text-muted-foreground/50 bg-muted/30" : ""}`}
                    style={{ borderRight: day.getDay() === 6 ? "none" : undefined }}
                  >
                    <div className={`text-xs font-medium ${today ? "text-primary" : ""}`}>{getDate(day)}</div>
                    {dayPropDates.map((label, i) => (
                      <div key={i} className="mt-1 text-[10px] px-1.5 py-0.5 rounded bg-amber-100 text-amber-800 dark:bg-amber-900/30 dark:text-amber-300 truncate">{label}</div>
                    ))}
                    {dayEvents.map((ev) => (
                      <div
                        key={ev.id}
                        className={`mt-1 text-[11px] px-1.5 py-0.5 rounded truncate cursor-pointer ${CATEGORY_COLORS[ev.category]}`}
                        onClick={() => openEdit(ev)}
                      >
                        {ev.startTime && <span className="mr-1 font-mono">{ev.startTime.slice(0, 5)}</span>}
                        {ev.title}
                      </div>
                    ))}
                    {(dayEvents.length + dayPropDates.length) > 3 && (
                      <div className="mt-1 text-[10px] text-muted-foreground">
                        +{(dayEvents.length + dayPropDates.length) - 3}개 더
                      </div>
                    )}
                  </div>
                )
              })}
            </div>
          )}
        </CardContent>
      </Card>

      <Dialog open={dialogOpen} onOpenChange={setDialogOpen}>
        <DialogContent className="max-w-md">
          <DialogHeader>
            <DialogTitle>{editingEvent ? "일정 수정" : "일정 추가"}</DialogTitle>
          </DialogHeader>
          <form onSubmit={handleSubmit}>
            <div className="grid gap-4 py-4">
              <div>
                <Label htmlFor="title">제목</Label>
                <Input id="title" value={form.title} onChange={(e) => setForm({ ...form, title: e.target.value })} placeholder="미팅, 계약 등" required />
              </div>
              <div>
                <Label htmlFor="date">날짜</Label>
                <Input id="date" type="date" value={format(ymdToDate(form.date), "yyyy-MM-dd")} onChange={(e) => setForm({ ...form, date: dateToYMD(new Date(e.target.value)) })} required />
              </div>
              <div>
                <Label htmlFor="startTime">시작 시간</Label>
                <Input id="startTime" type="time" value={form.startTime} onChange={(e) => setForm({ ...form, startTime: e.target.value })} />
              </div>
              <div>
                <Label htmlFor="category">카테고리</Label>
                <NativeSelect value={form.category} onChange={(e) => setForm({ ...form, category: e.target.value })}>
                  {CATEGORIES.map((c) => <NativeSelectOption key={c} value={c}>{c}</NativeSelectOption>)}
                </NativeSelect>
              </div>
              <div>
                <Label htmlFor="propertyId">매물 (선택)</Label>
                <NativeSelect value={form.propertyId} onChange={(e) => setForm({ ...form, propertyId: e.target.value })}>
                  <NativeSelectOption value="">선택 안 함</NativeSelectOption>
                  {calendar?.properties.map((p) => <NativeSelectOption key={p.id} value={p.id}>{p.label}</NativeSelectOption>)}
                </NativeSelect>
              </div>
              <div>
                <Label htmlFor="customerId">고객 (선택)</Label>
                <NativeSelect value={form.customerId} onChange={(e) => setForm({ ...form, customerId: e.target.value })}>
                  <NativeSelectOption value="">선택 안 함</NativeSelectOption>
                  {calendar?.customers.map((c) => <NativeSelectOption key={c.id} value={c.id}>{c.label}</NativeSelectOption>)}
                </NativeSelect>
              </div>
              <div>
                <Label htmlFor="memo">메모</Label>
                <Textarea id="memo" value={form.memo} onChange={(e) => setForm({ ...form, memo: e.target.value })} placeholder="메모" rows={3} />
              </div>
            </div>
            <DialogFooter className="gap-2">
              <Button type="button" variant="outline" onClick={closeDialog}>취소</Button>
              {editingEvent && (
                <Button type="button" variant="destructive" onClick={() => { if (confirm("정말 삭제하시겠습니까?")) { deleteMut.mutate(editingEvent.id); closeDialog() } }}>
                  <Trash2 className="mr-2 h-4 w-4" />삭제
                </Button>
              )}
              <Button type="submit" disabled={createMut.isPending || updateMut.isPending}>
                {editingEvent ? "수정" : "추가"}
              </Button>
            </DialogFooter>
          </form>
        </DialogContent>
      </Dialog>
    </div>
  )
}