import { describe, expect, it } from "vitest";

import { createApi } from ".";
import type { Transport } from "./transport/tauri";

const snapshot = {
  is_baseline: true,
  is_partial: false,
  observed_process_count: 0,
  matched_process_count: 0,
  cpu: { logical: [] },
  memory: {
    total_bytes: 1,
    available_bytes: 1,
    used_bytes: 0,
    cached_bytes: 0,
    buffers_bytes: 0,
    swap_total_bytes: 0,
    swap_used_bytes: 0,
  },
  load: {
    one_minute: 0,
    five_minutes: 0,
    fifteen_minutes: 0,
    runnable_tasks: 0,
    total_tasks: 0,
    uptime_seconds: 1,
  },
  block_devices: [],
  network_interfaces: [],
  processes: [],
};

describe("Task Manager API", () => {
  it("uses only the two narrow commands and validates their responses", async () => {
    const calls: { command: string; arguments_?: Record<string, unknown> }[] =
      [];
    const transport: Transport = {
      invoke<T>(
        command: string,
        decode: (value: unknown) => T,
        arguments_?: Record<string, unknown>,
      ) {
        calls.push({ command, ...(arguments_ ? { arguments_ } : {}) });
        const value =
          command === "task_manager_snapshot"
            ? snapshot
            : {
                identity: { pid: 42, start_time_ticks: 7 },
                parent_pid: 1,
                name: "synthetic",
                state: { kind: "sleeping" },
                nice: 0,
                thread_count: 1,
                cgroups: [],
                memory: {},
                restricted_fields: [],
              };
        return Promise.resolve(value).then(decode);
      },
      listen() {
        return Promise.resolve(() => undefined);
      },
    };
    const api = createApi(transport).taskManager;
    const request = {
      fresh_baseline: true,
      sort: "cpu" as const,
      direction: "descending" as const,
      limit: 200,
    };

    await expect(api.snapshot(request)).resolves.toEqual(snapshot);
    await expect(
      api.processDetails({ pid: 42, start_time_ticks: 7 }),
    ).resolves.toMatchObject({ name: "synthetic" });
    expect(calls).toEqual([
      { command: "task_manager_snapshot", arguments_: { request } },
      {
        command: "task_manager_process_details",
        arguments_: { identity: { pid: 42, start_time_ticks: 7 } },
      },
    ]);
  });

  it("rejects malformed snapshot values at the frontend trust boundary", async () => {
    const transport: Transport = {
      invoke<T>(_command: string, decode: (value: unknown) => T) {
        return Promise.resolve({
          ...snapshot,
          memory: { total_bytes: "private" },
        }).then(decode);
      },
      listen() {
        return Promise.resolve(() => undefined);
      },
    };

    await expect(
      createApi(transport).taskManager.snapshot({
        fresh_baseline: true,
        sort: "cpu",
        direction: "descending",
        limit: 200,
      }),
    ).rejects.toMatchObject({ code: "CORE_INTERNAL" });
  });
});
