import {
  act,
  cleanup,
  fireEvent,
  render,
  screen,
  waitFor,
} from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { MemoryRouter } from "react-router-dom";
import { afterEach, describe, expect, it, vi } from "vitest";

import { api } from "../../api";
import type {
  EffectiveModule,
  TaskManagerProcessDetails,
  TaskManagerSnapshot,
} from "../../generated";
import TaskManagerPage from "./TaskManagerPage";

const module: EffectiveModule = {
  manifest: {
    id: "task-manager",
    display_name: "Task Manager",
    description: "Current local resource use.",
    version: "1",
    route: "/task-manager",
    default_order: 100,
    default_enabled: true,
    capabilities: ["task_manager_read"],
    dependencies: [],
    linux_required: true,
  },
  enablement: "enabled",
  order: 100,
  health: "available",
};

const snapshot: TaskManagerSnapshot = {
  is_baseline: false,
  is_partial: false,
  observed_process_count: 1,
  matched_process_count: 1,
  cpu: {
    model: "Synthetic CPU",
    total_percent: 12.5,
    user_percent: 8,
    system_percent: 4,
    idle_percent: 87.5,
    io_wait_percent: 0.5,
    logical: [{ logical_index: 0, usage_percent: 12.5 }],
  },
  memory: {
    total_bytes: 16 * 1024 ** 3,
    available_bytes: 10 * 1024 ** 3,
    used_bytes: 6 * 1024 ** 3,
    cached_bytes: 2 * 1024 ** 3,
    buffers_bytes: 128 * 1024 ** 2,
    swap_total_bytes: 4 * 1024 ** 3,
    swap_used_bytes: 0,
  },
  load: {
    one_minute: 0.4,
    five_minutes: 0.3,
    fifteen_minutes: 0.2,
    runnable_tasks: 1,
    total_tasks: 200,
    uptime_seconds: 7_200,
  },
  cpu_pressure: {
    some: {
      average_10: 0.1,
      average_60: 0.2,
      average_300: 0.3,
      total_microseconds: 100,
    },
  },
  block_devices: [
    {
      name: "test-disk",
      read_bytes_per_second: 1024,
      write_bytes_per_second: 2048,
      busy_percent: 2,
      io_in_progress: 0,
      capacity_bytes: 512 * 1024 ** 3,
    },
  ],
  network_interfaces: [
    {
      name: "test-net",
      received_bytes_per_second: 4096,
      transmitted_bytes_per_second: 1024,
      is_loopback: false,
    },
  ],
  processes: [
    {
      identity: { pid: 4242, start_time_ticks: 99 },
      parent_pid: 1,
      name: "synthetic-worker",
      state: { kind: "sleeping" },
      cpu_percent: 4.2,
      resident_memory_bytes: 64 * 1024 ** 2,
      resident_memory_percent: 0.4,
      virtual_memory_bytes: 256 * 1024 ** 2,
      disk_read_bytes_per_second: 512,
      disk_write_bytes_per_second: 0,
      io_permission_denied: false,
      thread_count: 4,
      nice: 0,
    },
  ],
};

const details: TaskManagerProcessDetails = {
  identity: { pid: 4242, start_time_ticks: 99 },
  parent_pid: 1,
  name: "synthetic-worker",
  state: { kind: "sleeping" },
  uid: 1000,
  nice: 0,
  thread_count: 4,
  command_line: "synthetic-worker --test",
  executable: "/test/synthetic-worker",
  cgroups: ["user.slice/test.scope"],
  memory: { resident_bytes: 64 * 1024 ** 2 },
  io: {
    characters_read: 100,
    characters_written: 200,
    read_syscalls: 3,
    write_syscalls: 4,
    read_bytes: 512,
    write_bytes: 0,
  },
  voluntary_context_switches: 5,
  involuntary_context_switches: 1,
  restricted_fields: [],
};

function renderPage() {
  return render(
    <MemoryRouter initialEntries={["/task-manager"]}>
      <TaskManagerPage module={module} />
    </MemoryRouter>,
  );
}

afterEach(() => {
  cleanup();
  Reflect.deleteProperty(HTMLDialogElement.prototype, "showModal");
  Reflect.deleteProperty(HTMLDialogElement.prototype, "close");
  vi.useRealTimers();
  vi.restoreAllMocks();
});

describe("Task Manager page", () => {
  it("samples only while visible, never overlaps, and resumes with a baseline", async () => {
    vi.useFakeTimers();
    let visibility: DocumentVisibilityState = "visible";
    vi.spyOn(document, "visibilityState", "get").mockImplementation(
      () => visibility,
    );
    const snapshotRequest = vi
      .spyOn(api.taskManager, "snapshot")
      .mockResolvedValue(snapshot);
    const rendered = renderPage();

    await act(() => Promise.resolve());
    expect(snapshotRequest).toHaveBeenCalledTimes(1);
    expect(snapshotRequest.mock.calls[0]?.[0].fresh_baseline).toBe(true);

    await act(async () => {
      await vi.advanceTimersByTimeAsync(2_000);
    });
    expect(snapshotRequest).toHaveBeenCalledTimes(2);
    expect(snapshotRequest.mock.calls[1]?.[0].fresh_baseline).toBe(false);

    visibility = "hidden";
    document.dispatchEvent(new Event("visibilitychange"));
    await vi.advanceTimersByTimeAsync(10_000);
    expect(snapshotRequest).toHaveBeenCalledTimes(2);

    visibility = "visible";
    document.dispatchEvent(new Event("visibilitychange"));
    await act(() => Promise.resolve());
    expect(snapshotRequest).toHaveBeenCalledTimes(3);
    expect(snapshotRequest.mock.calls[2]?.[0].fresh_baseline).toBe(true);

    rendered.unmount();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(snapshotRequest).toHaveBeenCalledTimes(3);
  });

  it("does not start another snapshot while the current one is unresolved", async () => {
    vi.useFakeTimers();
    let resolveSnapshot: ((value: TaskManagerSnapshot) => void) | undefined;
    const snapshotRequest = vi
      .spyOn(api.taskManager, "snapshot")
      .mockReturnValue(
        new Promise((resolve) => {
          resolveSnapshot = resolve;
        }),
      );
    const rendered = renderPage();

    await act(() => Promise.resolve());
    await vi.advanceTimersByTimeAsync(10_000);
    expect(snapshotRequest).toHaveBeenCalledTimes(1);

    const refresh = screen.getByRole("button", { name: "Refresh" });
    fireEvent.click(refresh);
    fireEvent.click(refresh);
    fireEvent.click(refresh);
    expect(snapshotRequest).toHaveBeenCalledTimes(1);

    act(() => {
      resolveSnapshot?.(snapshot);
    });
    await act(() => Promise.resolve());
    expect(snapshotRequest).toHaveBeenCalledTimes(2);

    rendered.unmount();
    await vi.advanceTimersByTimeAsync(10_000);
    expect(snapshotRequest).toHaveBeenCalledTimes(2);
  });

  it("renders both views and reads sensitive details only after selection", async () => {
    const snapshotRequest = vi
      .spyOn(api.taskManager, "snapshot")
      .mockResolvedValue(snapshot);
    const detailsRequest = vi
      .spyOn(api.taskManager, "processDetails")
      .mockResolvedValue(details);
    Object.defineProperties(HTMLDialogElement.prototype, {
      showModal: {
        configurable: true,
        value(this: HTMLDialogElement) {
          this.setAttribute("open", "");
        },
      },
      close: {
        configurable: true,
        value(this: HTMLDialogElement) {
          this.removeAttribute("open");
        },
      },
    });
    const user = userEvent.setup();
    renderPage();

    expect(await screen.findByText("synthetic-worker")).toBeTruthy();
    expect(screen.getByText("12.5%")).toBeTruthy();
    expect(detailsRequest).not.toHaveBeenCalled();

    await user.click(screen.getByRole("button", { name: "CPU" }));
    await waitFor(() => {
      expect(snapshotRequest).toHaveBeenLastCalledWith(
        expect.objectContaining({
          sort: "cpu",
          direction: "ascending",
          fresh_baseline: false,
        }),
      );
    });

    const processButton = screen.getByRole("button", {
      name: "synthetic-worker",
    });
    await user.click(processButton);
    expect(await screen.findByText("synthetic-worker --test")).toBeTruthy();
    expect(detailsRequest).toHaveBeenCalledWith({
      pid: 4242,
      start_time_ticks: 99,
    });
    const closeButton = screen.getByRole("button", { name: "Close dialog" });
    expect(document.activeElement).toBe(closeButton);
    await user.click(closeButton);
    expect(screen.queryByRole("dialog")).toBeNull();
    expect(document.activeElement).toBe(processButton);

    await user.click(screen.getByRole("tab", { name: "performance" }));
    expect(screen.getByRole("heading", { name: "Performance" })).toBeTruthy();
    expect(screen.getByRole("img", { name: /CPU history/ })).toBeTruthy();
    expect(screen.getByRole("heading", { name: "Block devices" })).toBeTruthy();
  });
});
