// 다음 작업에서 실제 화면으로 교체되는 자리 표시 페이지.
export function Placeholder({ name }: { name: string }) {
  return <div className="flex flex-1 items-center justify-center p-4 text-muted-foreground">{name}</div>
}
