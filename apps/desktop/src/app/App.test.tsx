import { cleanup, render, screen, within } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { createMemoryRouter, type RouteObject } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";

import { api } from "../api";
import { AppProviders } from "./providers";
import { createAppQueryClient } from "./query";
import { appRoutes } from "./router";
import { AppShell } from "./AppShell";
import { DiagnosticsPage, SettingsPage } from "./pages";
import { RouteErrorPage } from "./RouteErrorPage";

afterEach(() => {
  cleanup();
  vi.restoreAllMocks();
});

function renderRoutes(routes: RouteObject[], initialPath = "/") {
  const queryClient = createAppQueryClient();
  const router = createMemoryRouter(routes, {
    initialEntries: [initialPath],
  });

  const result = render(
    <AppProviders queryClient={queryClient} router={router} />,
  );

  return {
    ...result,
    queryClient,
    user: userEvent.setup(),
  };
}

describe("application shell", () => {
  it("opens the Dashboard with the required shell landmarks", () => {
    renderRoutes(appRoutes);

    const navigation = screen.getByRole("navigation", {
      name: "Primary navigation",
    });

    expect(screen.getByRole("main")).toBeTruthy();
    expect(
      screen.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeTruthy();
    expect(
      within(navigation)
        .getByRole("link", { name: "Dashboard" })
        .getAttribute("aria-current"),
    ).toBe("page");
    expect(
      within(navigation).getByRole("link", { name: "Settings" }),
    ).toBeTruthy();
    expect(
      within(navigation).getByRole("link", { name: "Diagnostics" }),
    ).toBeTruthy();
    expect(
      screen
        .getByRole("button", { name: "Search (coming later)" })
        .hasAttribute("disabled"),
    ).toBe(true);
  });

  it("navigates among every non-disableable core route", async () => {
    const { user } = renderRoutes(appRoutes);

    await user.click(screen.getByRole("link", { name: "Settings" }));
    expect(
      screen.getByRole("heading", { level: 1, name: "Settings" }),
    ).toBeTruthy();

    await user.click(screen.getByRole("link", { name: "Diagnostics" }));
    expect(
      screen.getByRole("heading", { level: 1, name: "Diagnostics" }),
    ).toBeTruthy();

    await user.click(screen.getByRole("link", { name: "Dashboard" }));
    expect(
      screen.getByRole("heading", { level: 1, name: "Dashboard" }),
    ).toBeTruthy();
  });

  it("stays usable with the backend unavailable and starts no data query", async () => {
    const boundaryProof = vi
      .spyOn(api.core, "proveBoundary")
      .mockRejectedValue(new Error("backend unavailable"));
    const { queryClient, user } = renderRoutes(appRoutes);

    await user.click(screen.getByRole("link", { name: "Settings" }));
    await user.click(screen.getByRole("link", { name: "Diagnostics" }));

    expect(screen.getByRole("main")).toBeTruthy();
    expect(screen.getByRole("navigation")).toBeTruthy();
    expect(boundaryProof).not.toHaveBeenCalled();
    expect(queryClient.getQueryCache().getAll()).toHaveLength(0);
  });

  it("keeps the shell available around loading route content", () => {
    const loadingRoutes: RouteObject[] = [
      {
        path: "/",
        element: <AppShell />,
        children: [
          {
            path: "loading",
            element: <p role="status">Loading module data</p>,
          },
        ],
      },
    ];

    renderRoutes(loadingRoutes, "/loading");

    expect(screen.getByRole("status").textContent).toBe("Loading module data");
    expect(screen.getByRole("main")).toBeTruthy();
    expect(
      screen.getByRole("navigation", { name: "Primary navigation" }),
    ).toBeTruthy();
  });

  it("contains a route failure and preserves navigation recovery", async () => {
    const rawFailure = "private loader detail must stay hidden";
    const failureRoutes: RouteObject[] = [
      {
        path: "/",
        element: <AppShell />,
        children: [
          {
            errorElement: <RouteErrorPage />,
            children: [
              {
                index: true,
                loader: () => {
                  throw new Error(rawFailure);
                },
                element: <p>Unreachable</p>,
              },
              { path: "settings", element: <SettingsPage /> },
              { path: "diagnostics", element: <DiagnosticsPage /> },
            ],
          },
        ],
      },
    ];
    const { user } = renderRoutes(failureRoutes);

    expect(
      await screen.findByRole("heading", {
        level: 1,
        name: "This page could not be opened",
      }),
    ).toBeTruthy();
    expect(screen.queryByText(rawFailure)).toBeNull();
    expect(screen.getByRole("button", { name: "Try again" })).toBeTruthy();

    await user.click(screen.getByRole("link", { name: "Settings" }));
    expect(
      screen.getByRole("heading", { level: 1, name: "Settings" }),
    ).toBeTruthy();
  });

  it("shows a recoverable page for an unknown route", () => {
    renderRoutes(appRoutes, "/not-a-route");

    expect(
      screen.getByRole("heading", {
        level: 1,
        name: "That page does not exist",
      }),
    ).toBeTruthy();
    expect(
      screen
        .getByRole("link", { name: "Go to Dashboard" })
        .getAttribute("href"),
    ).toBe("/");
    expect(screen.getByRole("navigation")).toBeTruthy();
  });
});
