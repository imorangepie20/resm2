import { QueryClientProvider } from "@tanstack/react-query"
import { cleanup, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { MemoryRouter, useLocation } from "react-router"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"
import { queryClient } from "@/lib/query-client"
import { setOnUnauthorized } from "@/lib/api"
import { LoginPage } from "./login"
import { RegisterPage } from "./register"

const fetchMock = vi.fn()
const me = { id: "u1", email: "a@b.kr", name: "홍", role: "owner", agencyId: "g1" }

function json(status: number, body: unknown) {
  return new Response(JSON.stringify(body), { status, headers: { "Content-Type": "application/json" } })
}

function Where() {
  return <span data-testid="where">{useLocation().pathname}</span>
}

function mount(ui: React.ReactElement, path: string) {
  return render(
    <QueryClientProvider client={queryClient}>
      <MemoryRouter initialEntries={[path]}>
        {ui}
        <Where />
      </MemoryRouter>
    </QueryClientProvider>,
  )
}

beforeEach(() => {
  vi.stubGlobal("fetch", fetchMock)
  fetchMock.mockReset()
})
afterEach(() => {
  cleanup()
  vi.unstubAllGlobals()
  queryClient.clear()
  setOnUnauthorized(() => {})
})

describe("LoginPage", () => {
  it("shows the 401 message inline, stays on /login, no global redirect", async () => {
    const onUnauthorized = vi.fn()
    setOnUnauthorized(onUnauthorized)
    fetchMock.mockResolvedValue(json(401, { error: "이메일 또는 비밀번호가 올바르지 않습니다" }))
    mount(<LoginPage />, "/login")
    await userEvent.type(screen.getByLabelText("이메일"), "a@b.kr")
    await userEvent.type(screen.getByLabelText("비밀번호"), "wrong-pass")
    await userEvent.click(screen.getByRole("button", { name: "로그인" }))
    expect(await screen.findAllByText("이메일 또는 비밀번호가 올바르지 않습니다")).toHaveLength(1)
    expect(screen.getByTestId("where").textContent).toBe("/login")
    expect(onUnauthorized).not.toHaveBeenCalled()
    expect(screen.queryByText("비밀번호 찾기")).toBeNull()
  })
})

describe("RegisterPage", () => {
  it("disables submit until terms are agreed", async () => {
    mount(<RegisterPage />, "/register")
    const submit = screen.getByRole("button", { name: "가입하기" })
    expect((submit as HTMLButtonElement).disabled).toBe(true)
    await userEvent.click(screen.getByRole("checkbox"))
    expect((submit as HTMLButtonElement).disabled).toBe(false)
  })

  it("builds the signup payload: formatted phones, trimmed optionals, agree boolean", async () => {
    fetchMock.mockResolvedValue(json(200, { token: "t", user: me }))
    mount(<RegisterPage />, "/register")
    await userEvent.type(screen.getByLabelText("상호명"), "테스트공인")
    await userEvent.type(screen.getByLabelText("중개사무소 전화번호"), "0311234567")
    await userEvent.type(screen.getByLabelText("이름"), "홍길동")
    await userEvent.type(screen.getByLabelText("개인 전화번호"), "01012345678")
    await userEvent.type(screen.getByLabelText("이메일"), "a@b.kr")
    await userEvent.type(screen.getByLabelText("비밀번호"), "password1")
    await userEvent.type(screen.getByLabelText("비밀번호 확인"), "password1")
    await userEvent.type(screen.getByLabelText("중개사무소 주소"), "  ")
    await userEvent.click(screen.getByRole("checkbox"))
    await userEvent.click(screen.getByRole("button", { name: "가입하기" }))
    await waitFor(() => expect(fetchMock).toHaveBeenCalled())
    const [url, init] = fetchMock.mock.calls[0]
    expect(String(url)).toMatch(/\/auth\/signup$/)
    expect(JSON.parse(init.body)).toEqual({
      agencyName: "테스트공인",
      agencyPhone: "031-123-4567",
      name: "홍길동",
      phone: "010-1234-5678",
      email: "a@b.kr",
      password: "password1",
      passwordConfirm: "password1",
      agree: true,
    })
  })

  it("shows the server error inline exactly once", async () => {
    fetchMock.mockResolvedValue(json(400, { error: "비밀번호가 일치하지 않습니다" }))
    mount(<RegisterPage />, "/register")
    await userEvent.type(screen.getByLabelText("상호명"), "x")
    await userEvent.type(screen.getByLabelText("이름"), "x")
    await userEvent.type(screen.getByLabelText("이메일"), "a@b.kr")
    await userEvent.type(screen.getByLabelText("비밀번호"), "password1")
    await userEvent.type(screen.getByLabelText("비밀번호 확인"), "password2")
    await userEvent.click(screen.getByRole("checkbox"))
    await userEvent.click(screen.getByRole("button", { name: "가입하기" }))
    expect(await screen.findAllByText("비밀번호가 일치하지 않습니다")).toHaveLength(1)
    expect(screen.getByTestId("where").textContent).toBe("/register")
  })
})
