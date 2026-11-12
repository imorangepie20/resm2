import { createBrowserRouter, Navigate } from "react-router"
import { RedirectIfAuthed, RequireAuth } from "@/lib/auth"
import { ShellLayout } from "@/components/layout/shell-layout"
import { Placeholder } from "@/routes/placeholder"
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
          { path: "/properties", element: <Placeholder name="/properties" /> },
          { path: "/properties/favorites", element: <Placeholder name="/properties/favorites" /> },
          { path: "/properties/progress", element: <Placeholder name="/properties/progress" /> },
          { path: "/properties/contracted", element: <Placeholder name="/properties/contracted" /> },
          { path: "/properties/new", element: <Placeholder name="/properties/new" /> },
          { path: "/properties/:id/edit", element: <Placeholder name="/properties/:id/edit" /> },
        ],
      },
    ],
  },
  { path: "*", element: <NotFound /> },
])
