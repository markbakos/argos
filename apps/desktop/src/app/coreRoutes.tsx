import {
  ActivityIcon,
  LayoutDashboardIcon,
  SettingsIcon,
  type LucideIcon,
} from "lucide-react";
import type { ReactNode } from "react";

import { DashboardPage, DiagnosticsPage, SettingsPage } from "./pages";

export interface CoreRoute {
  path: "/" | "/settings" | "/diagnostics";
  label: string;
  icon: LucideIcon;
  element: ReactNode;
}

export const CORE_ROUTES: readonly CoreRoute[] = [
  {
    path: "/",
    label: "Dashboard",
    icon: LayoutDashboardIcon,
    element: <DashboardPage />,
  },
  {
    path: "/settings",
    label: "Settings",
    icon: SettingsIcon,
    element: <SettingsPage />,
  },
  {
    path: "/diagnostics",
    label: "Diagnostics",
    icon: ActivityIcon,
    element: <DiagnosticsPage />,
  },
];
