import { StrictMode } from "react"
import { createRoot } from "react-dom/client"
import { QueryClientProvider } from "@tanstack/react-query"
import { RouterProvider } from "react-router"
import "./globals.css"
import { ThemeProvider } from "@/components/layout/theme-provider"
import { Toaster } from "@/components/ui/sonner"
import { setOnUnauthorized } from "@/lib/api"
import { resetSession } from "@/lib/auth"
import { queryClient } from "@/lib/query-client"
import { router } from "@/router"

setOnUnauthorized(() => {
  resetSession(null)
  void router.navigate("/login", { replace: true })
})

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <QueryClientProvider client={queryClient}>
        <RouterProvider router={router} />
      </QueryClientProvider>
      <Toaster richColors closeButton />
    </ThemeProvider>
  </StrictMode>,
)
