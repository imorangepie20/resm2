import { resolve } from "node:path"
import { defineConfig, devices } from "@playwright/test"

// E2E 는 반드시 테스트 DB 로만 돈다. DATABASE_URL 은 무시하고 TEST_DATABASE_URL 을 서버에 넘긴다.
try { process.loadEnvFile(resolve(import.meta.dirname, "../.env")) } catch { /* .env 없음 */ }
const testDb = process.env.TEST_DATABASE_URL
if (!testDb) throw new Error("TEST_DATABASE_URL 이 필요합니다 (E2E 는 테스트 DB 로만 실행됩니다)")

export default defineConfig({
  testDir: "./e2e",
  workers: 1,
  reporter: "list",
  use: { baseURL: "http://127.0.0.1:3104", trace: "retain-on-failure" },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
  webServer: {
    command: "cargo run -p resm-server",
    cwd: resolve(import.meta.dirname, ".."),
    url: "http://127.0.0.1:3104/api/v1/health",
    reuseExistingServer: false, // 개발 DB 에 붙은 서버를 재사용하지 않는다
    timeout: 180_000,
    env: {
      DATABASE_URL: testDb,
      RESM_BIND: "127.0.0.1:3104",
      RESM_STATIC_DIR: resolve(import.meta.dirname, "dist"),
      RESM_COOKIE_SECURE: "false",
    },
  },
})
