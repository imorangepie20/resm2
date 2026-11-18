import { useQuery, useMutation, useQueryClient } from "@tanstack/react-query"
import { useState } from "react"
import { toast } from "sonner"
import { Plus, Trash2 } from "lucide-react"

import { Button } from "@/components/ui/button"
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card"
import { Input } from "@/components/ui/input"
import { NativeSelect, NativeSelectOption } from "@/components/ui/native-select"
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table"
import { listParties, addParty, removeParty, type PartyRow, type AddPartyInput } from "@/lib/properties/api"

const ROLE_ORDER = ["매도인", "매수인", "임대인", "임차인", "중개"]

export function PartySection({ propertyId, tradeType }: { propertyId: string; tradeType?: string | null }) {
  const qc = useQueryClient()
  const [newRole, setNewRole] = useState(() => {
    if (tradeType === "A1") return "매도인"
    if (tradeType === "B1" || tradeType === "B2" || tradeType === "B3") return "임대인"
    return "매도인"
  })
  const [newName, setNewName] = useState("")
  const [newPhone, setNewPhone] = useState("")

  const { data: parties } = useQuery({
    queryKey: ["parties", propertyId],
    queryFn: () => listParties(propertyId),
    enabled: !!propertyId,
  })

  const addMut = useMutation({
    mutationFn: (input: AddPartyInput) => addParty(propertyId, input),
    onSuccess: () => {
      toast.success("당사자 추가됨")
      setNewName("")
      setNewPhone("")
      qc.invalidateQueries({ queryKey: ["parties", propertyId] })
      qc.invalidateQueries({ queryKey: ["customers"] })
    },
    onError: (e: Error) => toast.error(e.message),
  })

  const delMut = useMutation({
    mutationFn: (partyId: string) => removeParty(propertyId, partyId),
    onSuccess: () => {
      toast.success("당사자 삭제됨")
      qc.invalidateQueries({ queryKey: ["parties", propertyId] })
    },
    onError: (e: Error) => toast.error(e.message),
  })

  const handleAdd = (e: React.FormEvent) => {
    e.preventDefault()
    if (!newName.trim()) return toast.error("이름은 필수입니다")
    addMut.mutate({ role: newRole, name: newName.trim(), phone: newPhone.trim() || undefined })
  }

  const sorted = [...(parties ?? [])].sort((a, b) => ROLE_ORDER.indexOf(a.role) - ROLE_ORDER.indexOf(b.role))

  return (
    <Card>
      <CardHeader className="flex flex-row items-center justify-between">
        <CardTitle>당사자</CardTitle>
      </CardHeader>
      <CardContent className="flex flex-col gap-4">
        <form onSubmit={handleAdd} className="flex flex-wrap items-end gap-3">
          <div className="flex-1 min-w-[180px]">
            <label className="text-sm font-medium mb-1 block">역할</label>
            <NativeSelect value={newRole} onChange={(e) => setNewRole(e.target.value)}>
              <NativeSelectOption value="매도인">매도인</NativeSelectOption>
              <NativeSelectOption value="매수인">매수인</NativeSelectOption>
              <NativeSelectOption value="임대인">임대인</NativeSelectOption>
              <NativeSelectOption value="임차인">임차인</NativeSelectOption>
              <NativeSelectOption value="중개">중개</NativeSelectOption>
            </NativeSelect>
          </div>
          <div className="flex-1 min-w-[180px]">
            <label className="text-sm font-medium mb-1 block">이름</label>
            <Input value={newName} onChange={(e) => setNewName(e.target.value)} placeholder="홍길동" required />
          </div>
          <div className="flex-1 min-w-[160px]">
            <label className="text-sm font-medium mb-1 block">전화</label>
            <Input value={newPhone} onChange={(e) => setNewPhone(e.target.value)} placeholder="010-1234-5678" type="tel" inputMode="tel" />
          </div>
          <Button type="submit" disabled={addMut.isPending} className="h-10">
            <Plus className="mr-2 h-4 w-4" />추가
          </Button>
        </form>

        {sorted.length === 0 ? (
          <div className="text-center py-6 text-muted-foreground">등록된 당사자가 없습니다</div>
        ) : (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>역할</TableHead>
                <TableHead>이름</TableHead>
                <TableHead>전화</TableHead>
                <TableHead className="text-right w-24">삭제</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {sorted.map((p: PartyRow) => (
                <TableRow key={p.id}>
                  <TableCell><span className="inline-flex items-center rounded-full px-2 py-0.5 text-xs font-medium bg-primary/10 text-primary">{p.role}</span></TableCell>
                  <TableCell>{p.name}</TableCell>
                  <TableCell>{p.phone ?? "-"}</TableCell>
                  <TableCell className="text-right">
                    <Button variant="ghost" size="icon" onClick={() => delMut.mutate(p.id)} disabled={delMut.isPending}>
                      <Trash2 className="h-4 w-4 text-destructive" />
                    </Button>
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        )}
      </CardContent>
    </Card>
  )
}