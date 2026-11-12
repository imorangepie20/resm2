import { expect, test } from "@playwright/test"

// 첫 가입자는 superadmin, 이후는 member — 어느 쪽이든 통과해야 한다. 실행마다 새 이메일을 쓴다.
const email = `e2e-${Date.now()}-${Math.floor(Math.random() * 1e6)}@example.com`

test("가입부터 로그아웃까지", async ({ page }) => {
  await page.goto("/register")
  await page.getByLabel("상호명").fill("E2E 공인중개사")
  await page.getByLabel("이름").fill("E2E 사용자")
  await page.getByLabel("이메일").fill(email)
  await page.getByLabel("비밀번호", { exact: true }).fill("e2e-password-1")
  await page.getByLabel("비밀번호 확인").fill("e2e-password-1")
  await page.getByRole("checkbox", { name: /동의합니다/ }).click()
  await page.getByRole("button", { name: "가입하기" }).click()
  await expect(page).toHaveURL(/\/properties$/)

  // 등록 (가격 입력은 만원 단위: 12,000 → 12,000만원)
  await page.getByRole("button", { name: "새 매물" }).click()
  await page.getByLabel("매물명").fill("E2E 매물")
  await page.getByLabel("가격").fill("12,000")
  await page.getByRole("button", { name: "등록", exact: true }).click()
  await expect(page).toHaveURL(/\/properties$/)
  const table = page.locator("table")
  await expect(table.getByText("E2E 매물", { exact: true })).toBeVisible()
  await expect(table.getByRole("cell", { name: "12,000만원", exact: true })).toBeVisible()

  // 수정
  await table.getByRole("link", { name: "수정" }).click()
  await expect(page).toHaveURL(/\/properties\/[^/]+\/edit$/)
  await page.getByLabel("매물명").fill("E2E 매물 수정")
  await page.getByRole("button", { name: "수정", exact: true }).click()
  await expect(page).toHaveURL(/\/properties$/)
  await expect(table.getByText("E2E 매물 수정", { exact: true })).toBeVisible()

  // 딥링크 새로고침
  await table.getByRole("link", { name: "수정" }).click()
  await expect(page).toHaveURL(/\/properties\/[^/]+\/edit$/)
  await page.reload()
  await expect(page.getByLabel("매물명")).toHaveValue("E2E 매물 수정")

  // 모바일: 카드 표시, 표 숨김
  await page.goto("/properties")
  await page.setViewportSize({ width: 390, height: 844 })
  await expect(table).toBeHidden()
  await expect(page.getByText("E2E 매물 수정", { exact: true }).locator("visible=true")).toHaveCount(1)
  await page.setViewportSize({ width: 1280, height: 800 })

  // 선택 삭제 → 빈 상태
  await table.getByRole("checkbox", { name: "선택", exact: true }).click()
  await page.getByRole("button", { name: /^삭제 1/ }).click()
  await page.getByRole("alertdialog").getByRole("button", { name: "삭제", exact: true }).click()
  await expect(page.getByText("매물이 없습니다")).toBeVisible()

  // 로그아웃 → /login, 이후 보호 경로 접근 차단
  await page.getByRole("button", { name: "로그아웃" }).click()
  await expect(page).toHaveURL(/\/login$/)
  await page.goto("/properties")
  await expect(page).toHaveURL(/\/login$/)
})
