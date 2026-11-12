import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react"
import userEvent from "@testing-library/user-event"
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest"

const toastError = vi.hoisted(() => vi.fn())
vi.mock("sonner", () => ({ toast: { error: toastError } }))

import { PostcodeSearch } from "./postcode-search"

const SRC = "//t1.kakaocdn.net/mapjsapi/bundle/postcode/prod/postcode.v2.js"
const MESSAGE = "주소 검색을 불러오지 못했습니다. 네트워크 연결을 확인한 뒤 다시 시도해 주세요."
const scripts = () => document.querySelectorAll(`script[src*="t1.kakaocdn.net/mapjsapi/bundle/postcode"]`)

beforeEach(() => {
  toastError.mockReset()
})
afterEach(() => {
  cleanup()
  scripts().forEach((s) => s.remove())
  delete window.daum
})

describe("PostcodeSearch script loading", () => {
  it("failure: toast once, button enabled again, failed script tag removed", async () => {
    render(<PostcodeSearch onComplete={() => {}} />)
    const btn = screen.getByRole("button", { name: /주소 검색/ }) as HTMLButtonElement
    await userEvent.click(btn)
    expect(scripts()).toHaveLength(1)
    fireEvent.error(scripts()[0])
    await waitFor(() => expect(toastError).toHaveBeenCalledTimes(1))
    expect(toastError).toHaveBeenCalledWith(MESSAGE)
    await waitFor(() => expect(btn.disabled).toBe(false))
    expect(scripts()).toHaveLength(0)
  })

  it("retry after failure appends a fresh script tag", async () => {
    render(<PostcodeSearch onComplete={() => {}} />)
    const btn = screen.getByRole("button", { name: /주소 검색/ }) as HTMLButtonElement
    await userEvent.click(btn)
    fireEvent.error(scripts()[0])
    await waitFor(() => expect(btn.disabled).toBe(false))
    await userEvent.click(btn)
    expect(scripts()).toHaveLength(1)
    fireEvent.error(scripts()[0])
    await waitFor(() => expect(toastError).toHaveBeenCalledTimes(2))
  })

  it("success: dialog opens, embed called with container, oncomplete reports and closes", async () => {
    const embed = vi.fn()
    let oncomplete: (d: { zonecode: string; roadAddress: string; jibunAddress: string }) => void = () => {}
    window.daum = {
      Postcode: class {
        constructor(opts: { oncomplete: typeof oncomplete }) {
          oncomplete = opts.oncomplete
        }
        embed = embed
      },
    } as unknown as typeof window.daum
    const onComplete = vi.fn()
    render(<PostcodeSearch onComplete={onComplete} />)
    await userEvent.click(screen.getByRole("button", { name: /주소 검색/ }))
    await waitFor(() => expect(embed).toHaveBeenCalledTimes(1))
    const dialog = screen.getByRole("dialog")
    expect(dialog.contains(embed.mock.calls[0][0])).toBe(true)
    oncomplete({ zonecode: "13494", roadAddress: "경기 성남시 분당구 판교역로 235", jibunAddress: "지번" })
    expect(onComplete).toHaveBeenCalledWith({ zonecode: "13494", address: "경기 성남시 분당구 판교역로 235" })
    await waitFor(() => expect(screen.queryByRole("dialog")).toBeNull())
  })
})
