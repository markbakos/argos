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
  it("opens a sparse host-aware Dashboard with the required shell landmarks", async () => {
    const identity = vi
      .spyOn(api.core, "getSystemIdentity")
      .mockResolvedValue({ hostname: "argos-workstation" });
    renderRoutes(appRoutes);

    const navigation = screen.getByRole("navigation", {
      name: "Primary navigation",
    });

    expect(screen.getByRole("main")).toBeTruthy();
    expect(
      await screen.findByRole("heading", {
        level: 1,
        name: "argos-workstation",
      }),
    ).toBeTruthy();
    expect(
      within(screen.getByRole("main")).getByText("Dashboard"),
    ).toBeTruthy();
    expect(screen.getByText("Your local control center.")).toBeTruthy();
    expect(
      within(screen.getByRole("main")).queryAllByRole("link"),
    ).toHaveLength(0);
    expect(identity).toHaveBeenCalledTimes(1);
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
    const identity = vi
      .spyOn(api.core, "getSystemIdentity")
      .mockResolvedValue({ hostname: "argos-workstation" });
    const { user } = renderRoutes(appRoutes);

    expect(
      await screen.findByRole("heading", {
        level: 1,
        name: "argos-workstation",
      }),
    ).toBeTruthy();

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
      screen.getByRole("heading", { level: 1, name: "argos-workstation" }),
    ).toBeTruthy();
    expect(identity).toHaveBeenCalledTimes(1);
  });

  it("stays usable when hostname is unavailable without exposing the failure", async () => {
    const boundaryProof = vi
      .spyOn(api.core, "proveBoundary")
      .mockRejectedValue(new Error("backend unavailable"));
    const identity = vi
      .spyOn(api.core, "getSystemIdentity")
      .mockRejectedValue(new Error("private hostname source failure"));
    const { queryClient, user } = renderRoutes(appRoutes);

    expect(await screen.findByText("Hostname unavailable")).toBeTruthy();
    expect(
      screen.getByRole("heading", { level: 1, name: "This machine" }),
    ).toBeTruthy();
    expect(screen.queryByText("private hostname source failure")).toBeNull();

    await user.click(screen.getByRole("link", { name: "Settings" }));
    await user.click(screen.getByRole("link", { name: "Diagnostics" }));
    await user.click(screen.getByRole("link", { name: "Dashboard" }));

    expect(screen.getByRole("main")).toBeTruthy();
    expect(screen.getByRole("navigation")).toBeTruthy();
    expect(screen.getByText("Hostname unavailable")).toBeTruthy();
    expect(boundaryProof).not.toHaveBeenCalled();
    expect(identity).toHaveBeenCalledTimes(1);
    expect(queryClient.getQueryCache().getAll()).toHaveLength(1);
  });

  it("preserves the Dashboard composition while hostname is loading", () => {
    vi.spyOn(api.core, "getSystemIdentity").mockReturnValue(
      new Promise<never>(() => undefined),
    );

    renderRoutes(appRoutes);

    expect(
      screen.getByRole("heading", { level: 1, name: "This machine" }),
    ).toBeTruthy();
    expect(screen.getByRole("status").textContent).toBe("Reading hostname…");
    expect(screen.getByText("Your local control center.")).toBeTruthy();
  });

  it("does not request hostname when another core route opens directly", async () => {
    const identity = vi.spyOn(api.core, "getSystemIdentity");
    const { queryClient, user } = renderRoutes(appRoutes, "/settings");

    expect(
      screen.getByRole("heading", { level: 1, name: "Settings" }),
    ).toBeTruthy();
    await user.click(screen.getByRole("link", { name: "Diagnostics" }));

    expect(identity).not.toHaveBeenCalled();
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
