import { createBrowserRouter, Navigate } from "react-router"
import { RedirectIfAuthed, RequireAuth } from "@/lib/auth"
import { ShellLayout } from "@/components/layout/shell-layout"
import { PropertyListPage } from "@/routes/properties/list-page"
import { NewPropertyPage } from "@/routes/properties/new-page"
import { EditPropertyPage } from "@/routes/properties/edit-page"
import { NotFound } from "@/routes/not-found"
import { AuthLayout } from "@/routes/auth-layout"
import { LoginPage } from "@/routes/login"
import { RegisterPage } from "@/routes/register"
import { TermsPage } from "@/routes/terms"
import { PrivacyPage } from "@/routes/privacy"

export const router = createBrowserRouter([
  { path: "/", element: <Navigate to="/properties" replace /> },
  {
    element: <RedirectIfAuthed />,
    children: [
      {
        element: <AuthLayout />,
        children: [
          { path: "/login", element: <LoginPage /> },
          { path: "/register", element: <RegisterPage /> },
        ],
      },
    ],
  },
  { path: "/terms", element: <TermsPage /> },
  { path: "/privacy", element: <PrivacyPage /> },
  {
    element: <RequireAuth />,
    children: [
      {
        element: <ShellLayout />,
        children: [
          // key: 같은 컴포넌트를 쓰는 4개 경로 사이를 오갈 때 원본(경로별 페이지)처럼 검색·필터 상태를 새로 시작한다.
          { path: "/properties", element: <PropertyListPage key="all" view="all" /> },
          { path: "/properties/favorites", element: <PropertyListPage key="favorites" view="favorites" /> },
          { path: "/properties/progress", element: <PropertyListPage key="in-progress" view="in-progress" /> },
          { path: "/properties/contracted", element: <PropertyListPage key="contracted" view="contracted" /> },
          { path: "/properties/new", element: <NewPropertyPage /> },
          { path: "/properties/:id/edit", element: <EditPropertyPage /> },
        ],
      },
    ],
  },
  { path: "*", element: <NotFound /> },
])
