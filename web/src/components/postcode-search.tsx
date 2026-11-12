import { useEffect, useRef, useState } from "react"
import { Search } from "lucide-react"
import { toast } from "sonner"

import { Button } from "@/components/ui/button"
import { Dialog, DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog"

type PostcodeData = { zonecode: string; roadAddress: string; jibunAddress: string }
type DaumPostcode = { embed: (el: HTMLElement) => void }
declare global {
  interface Window {
    daum?: {
      Postcode: new (opts: {
        oncomplete: (data: PostcodeData) => void
        width?: string
        height?: string
      }) => DaumPostcode
    }
  }
}

const SCRIPT_SRC = "https://t1.kakaocdn.net/mapjsapi/bundle/postcode/prod/postcode.v2.js"

function loadScript(): Promise<void> {
  return new Promise((resolve, reject) => {
    if (window.daum?.Postcode) { resolve(); return }
    const existing = document.querySelector<HTMLScriptElement>(`script[src="${SCRIPT_SRC}"]`)
    if (existing) {
      existing.addEventListener("load", () => resolve())
      existing.addEventListener("error", () => reject(new Error("로드 실패")))
      return
    }
    const s = document.createElement("script")
    s.src = SCRIPT_SRC
    s.async = true
    s.onload = () => resolve()
    s.onerror = () => {
      // 실패한 태그가 남으면 재시도 때 영영 끝나지 않는 리스너만 붙으므로 떼어 낸다.
      s.remove()
      reject(new Error("우편번호 스크립트 로드 실패"))
    }
    document.body.appendChild(s)
  })
}

export function PostcodeSearch({ onComplete, className }: { onComplete: (r: { zonecode: string; address: string }) => void; className?: string }) {
  const [busy, setBusy] = useState(false)
  const [open, setOpen] = useState(false)
  const onCompleteRef = useRef(onComplete)
  useEffect(() => { onCompleteRef.current = onComplete })

  async function openSearch() {
    setBusy(true)
    try {
      await loadScript()
      setOpen(true)
    } catch {
      toast.error("주소 검색을 불러오지 못했습니다. 네트워크 연결을 확인한 뒤 다시 시도해 주세요.")
    } finally {
      setBusy(false)
    }
  }

  // 팝업(window.open) 대신 다이얼로그에 embed — 데스크탑 웹뷰·팝업 차단 환경에서도 동작.
  // 다이얼로그 팝업 DOM은 open 직후가 아니라 늦게 마운트되므로 effect가 아닌 ref 콜백에서 embed.
  function embedInto(el: HTMLDivElement | null) {
    if (!el || el.hasChildNodes() || !window.daum?.Postcode) return
    new window.daum.Postcode({
      oncomplete: (data) => {
        onCompleteRef.current({ zonecode: data.zonecode, address: data.roadAddress || data.jibunAddress })
        setOpen(false)
      },
      width: "100%",
      height: "100%",
    }).embed(el)
  }

  return (
    <>
      <Button type="button" variant="outline" onClick={openSearch} disabled={busy} className={className}>
        <Search className="size-3.5" />주소 검색
      </Button>
      <Dialog open={open} onOpenChange={setOpen}>
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>주소 검색</DialogTitle>
          </DialogHeader>
          <div ref={embedInto} className="h-112" />
        </DialogContent>
      </Dialog>
    </>
  )
}
