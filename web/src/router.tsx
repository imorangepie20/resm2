import { createBrowserRouter, Navigate } from "react-router"
import { RedirectIfAuthed, RequireAuth } from "@/lib/auth"
import { ShellLayout } from "@/components/layout/shell-layout"
import { Placeholder } from "@/routes/placeholder"
import { NotFound } from "@/routes/not-found"

export const router = createBrowserRouter([
  { path: "/", element: <Navigate to="/properties" replace /> },
  {
    element: <RedirectIfAuthed />,
    children: [
      { path: "/login", element: <Placeholder name="/login" /> },
      { path: "/register", element: <Placeholder name="/register" /> },
    ],
  },
  { path: "/terms", element: <Placeholder name="/terms" /> },
  { path: "/privacy", element: <Placeholder name="/privacy" /> },
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
