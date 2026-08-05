import {
  AppWindowIcon,
  GaugeIcon,
  ServerCogIcon,
  type LucideIcon,
} from "lucide-react";
import { lazy, type ComponentType, type LazyExoticComponent } from "react";

import type { EffectiveModule } from "../generated";

export interface ModulePageProps {
  module: EffectiveModule;
}

interface FrontendModuleRegistration {
  icon: LucideIcon;
  route: string;
  page: LazyExoticComponent<ComponentType<ModulePageProps>>;
}

export const FRONTEND_MODULE_IDS = [
  "task-manager",
  "systemd",
  "launcher",
] as const;

export const MODULE_REGISTRY = {
  "task-manager": {
    icon: GaugeIcon,
    route: "/task-manager",
    page: lazy(() => import("./task-manager/TaskManagerPage")),
  },
  systemd: {
    icon: ServerCogIcon,
    route: "/systemd",
    page: lazy(() => import("./unavailable/UnavailableModulePage")),
  },
  launcher: {
    icon: AppWindowIcon,
    route: "/launcher",
    page: lazy(() => import("./unavailable/UnavailableModulePage")),
  },
} satisfies Record<
  (typeof FRONTEND_MODULE_IDS)[number],
  FrontendModuleRegistration
>;

export type RegisteredModuleId = (typeof FRONTEND_MODULE_IDS)[number];

export const FRONTEND_MODULE_ROUTES = Object.values(MODULE_REGISTRY).map(
  ({ route }) => route,
);

export function getModuleRegistration(
  moduleId: string,
): FrontendModuleRegistration | undefined {
  return Object.hasOwn(MODULE_REGISTRY, moduleId)
    ? MODULE_REGISTRY[moduleId as RegisteredModuleId]
    : undefined;
}
