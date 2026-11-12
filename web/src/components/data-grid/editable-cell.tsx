
import { type ReactNode, useState } from "react"

import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select"
import { cn } from "@/lib/utils"

// 더블클릭 → 입력 편집, blur/Enter 저장, Esc 취소
export function EditCell({ value, display, onSave, numeric }: { value: string | number | null; display: ReactNode; onSave: (v: string) => void; numeric?: boolean }) {
  const [editing, setEditing] = useState(false)
  if (editing) {
    return (
      <input
        autoFocus
        type={numeric ? "number" : "text"}
        defaultValue={value ?? ""}
        onBlur={(e) => { if (String(value ?? "") !== e.target.value) onSave(e.target.value); setEditing(false) }}
        onKeyDown={(e) => { if (e.key === "Enter") e.currentTarget.blur(); else if (e.key === "Escape") setEditing(false) }}
        className={cn("w-full min-w-16 rounded bg-background px-1 py-0.5 outline-none ring-1 ring-ring", numeric && "text-right tabular-nums")}
      />
    )
  }
  return (
    <div onDoubleClick={() => setEditing(true)} title="더블클릭하여 편집" className="cursor-text rounded px-1 py-0.5 hover:bg-muted/50">{display}</div>
  )
}

// 더블클릭 → 셀렉트 편집
export function SelectCell({ value, label, options, onSave }: { value: string; label: string; options: { value: string; label: string }[]; onSave: (v: string) => void }) {
  const [editing, setEditing] = useState(false)
  if (editing) {
    return (
      <Select defaultOpen value={value} onOpenChange={(o) => { if (!o) setEditing(false) }} onValueChange={(v) => { if (v != null) onSave(v); setEditing(false) }}>
        <SelectTrigger className="h-7 px-1"><SelectValue>{label}</SelectValue></SelectTrigger>
        <SelectContent>{options.map((o) => <SelectItem key={o.value} value={o.value}>{o.label}</SelectItem>)}</SelectContent>
      </Select>
    )
  }
  return (
    <div onDoubleClick={() => setEditing(true)} title="더블클릭하여 편집" className="cursor-text rounded px-1 py-0.5 hover:bg-muted/50">{label}</div>
  )
}
