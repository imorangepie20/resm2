import { fileURLToPath, URL } from "node:url"
import { loadEnv } from "vite"
import { defineConfig } from "vitest/config"
import react from "@vitejs/plugin-react"
import tailwindcss from "@tailwindcss/vite"

export default defineConfig(({ mode }) => {
  // 앱(tauri-*) 빌드는 API 주소가 없으면 `undefined/...` 로 요청하게 되므로 빌드 단계에서 멈춘다.
  if (mode.startsWith("tauri") && !loadEnv(mode, fileURLToPath(new URL(".", import.meta.url)), "VITE_").VITE_API_BASE) {
    throw new Error(`VITE_API_BASE 가 없습니다: web/.env.${mode} 를 확인하세요`)
  }
  return {
    plugins: [react(), tailwindcss()],
    resolve: {
      alias: { "@": fileURLToPath(new URL("./src", import.meta.url)) },
    },
    server: {
      port: 5173,
      strictPort: true,
      proxy: { "/api": "http://127.0.0.1:3104" },
    },
    build: {
      outDir: "dist",
      rolldownOptions: {
        output: {
          codeSplitting: {
            groups: [
              { name: "react", test: /node_modules[\/](react|react-dom|react-router|scheduler)[\/]/ },
              { name: "vendor", test: /node_modules/ },
            ],
          },
        },
      },
    },
    test: {
      environment: "jsdom",
      include: ["src/**/*.test.{ts,tsx}"],
    },
  }
})
