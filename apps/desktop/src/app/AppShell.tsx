import { SearchIcon } from "lucide-react";
import { NavLink, Outlet } from "react-router-dom";

import { CORE_ROUTES } from "./coreRoutes";
import { useModules } from "./modules";
import { getModuleRegistration } from "../modules/registry";

function navigationClassName(isActive: boolean) {
  const base =
    "group flex min-h-11 items-center gap-3 rounded-lg px-3 py-2 text-sm font-medium outline-none transition-colors focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)]";

  return isActive
    ? `${base} bg-[var(--nav-active)] text-[var(--text)]`
    : `${base} text-[var(--text-muted)] hover:bg-[var(--surface-hover)] hover:text-[var(--text)]`;
}

export function AppShell() {
  const modules = useModules();
  const dashboard = CORE_ROUTES.filter(({ path }) => path === "/");
  const utilities = CORE_ROUTES.filter(({ path }) => path !== "/");

  return (
    <div className="grid min-h-dvh grid-cols-[15rem_minmax(0,1fr)] bg-[var(--background)] text-[var(--text)] max-[760px]:grid-cols-1 max-[760px]:grid-rows-[auto_1fr]">
      <a
        href="#main-content"
        className="fixed top-3 left-3 z-50 -translate-y-20 rounded-md bg-[var(--text)] px-3 py-2 text-sm font-semibold text-[var(--background)] focus:translate-y-0"
      >
        Skip to content
      </a>

      <aside className="flex min-h-0 flex-col border-r border-[var(--border)] bg-[var(--sidebar)] p-4 max-[760px]:border-r-0 max-[760px]:border-b">
        <div className="flex items-center gap-3 px-2 py-2">
          <div
            aria-hidden="true"
            className="grid size-9 place-items-center rounded-xl bg-[var(--accent)] text-sm font-bold text-[var(--accent-contrast)] shadow-sm"
          >
            A
          </div>
          <div>
            <p className="font-semibold tracking-tight">Argos</p>
            <p className="text-xs text-[var(--text-muted)]">Control center</p>
          </div>
        </div>

        <button
          type="button"
          disabled
          aria-label="Search (coming later)"
          className="mt-5 flex min-h-10 cursor-not-allowed items-center gap-2 rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 text-left text-sm text-[var(--text-muted)] opacity-75"
        >
          <SearchIcon aria-hidden="true" className="size-4" />
          <span>Search</span>
          <span className="ml-auto text-xs">Later</span>
        </button>

        <nav
          aria-label="Primary navigation"
          className="mt-6 flex flex-1 flex-col gap-1 max-[760px]:mt-4 max-[760px]:flex-row max-[760px]:overflow-x-auto"
        >
          {dashboard.map(({ path, label, icon: Icon }) => (
            <NavLink
              key={path}
              to={path}
              end={path === "/"}
              className={({ isActive }) => navigationClassName(isActive)}
            >
              <Icon aria-hidden="true" className="size-[1.125rem] shrink-0" />
              <span>{label}</span>
            </NavLink>
          ))}
          {modules.data?.modules
            .filter((module) => module.enablement === "enabled")
            .map((module) => {
              const registration = getModuleRegistration(module.manifest.id);
              if (!registration) return null;
              const Icon = registration.icon;
              return (
                <NavLink
                  key={module.manifest.id}
                  to={module.manifest.route}
                  className={({ isActive }) => navigationClassName(isActive)}
                >
                  <Icon
                    aria-hidden="true"
                    className="size-[1.125rem] shrink-0"
                  />
                  <span>{module.manifest.display_name}</span>
                  {module.health !== "available" ? (
                    <span className="ml-auto text-[0.625rem] font-semibold tracking-wide uppercase">
                      {module.health}
                    </span>
                  ) : null}
                </NavLink>
              );
            })}
          <div
            aria-hidden="true"
            className="my-2 border-t border-[var(--border)]"
          />
          {utilities.map(({ path, label, icon: Icon }) => (
            <NavLink
              key={path}
              to={path}
              className={({ isActive }) => navigationClassName(isActive)}
            >
              <Icon aria-hidden="true" className="size-[1.125rem] shrink-0" />
              <span>{label}</span>
            </NavLink>
          ))}
        </nav>

        <p className="px-3 pt-4 text-xs text-[var(--text-muted)] max-[760px]:hidden">
          Foundation build
        </p>
      </aside>

      <main
        id="main-content"
        tabIndex={-1}
        className="min-w-0 overflow-auto p-8 outline-none sm:p-10"
      >
        <Outlet />
      </main>
    </div>
  );
}
