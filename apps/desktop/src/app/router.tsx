import { createBrowserRouter, type RouteObject } from "react-router-dom";

import { AppShell } from "./AppShell";
import { ModuleRoute } from "./ModuleRoute";
import { FRONTEND_MODULE_ROUTES } from "../modules/registry";
import { CORE_ROUTES } from "./coreRoutes";
import { NotFoundPage, RouteErrorPage } from "./RouteErrorPage";

const coreRouteObjects: RouteObject[] = CORE_ROUTES.map(({ path, element }) =>
  path === "/" ? { index: true, element } : { path: path.slice(1), element },
);

export const appRoutes: RouteObject[] = [
  {
    path: "/",
    element: <AppShell />,
    children: [
      {
        errorElement: <RouteErrorPage />,
        children: [
          ...coreRouteObjects,
          ...FRONTEND_MODULE_ROUTES.map((path) => ({
            path: path.slice(1),
            element: <ModuleRoute />,
          })),
          { path: "*", element: <NotFoundPage /> },
        ],
      },
    ],
  },
];

export function createAppRouter() {
  return createBrowserRouter(appRoutes);
}
