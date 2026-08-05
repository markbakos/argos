import type {
  TaskManagerBlockDeviceUsage,
  TaskManagerCpuUsage,
  TaskManagerMemoryUsage,
  TaskManagerNetworkInterfaceUsage,
  TaskManagerPressureUsage,
  TaskManagerProcessDetails,
  TaskManagerProcessIdentity,
  TaskManagerProcessMemoryDetails,
  TaskManagerProcessState,
  TaskManagerProcessSummary,
  TaskManagerSnapshot,
  TaskManagerSnapshotRequest,
} from "../generated";
import { normalizeAppError } from "./errors";
import type { Transport } from "./transport/tauri";

const SNAPSHOT_COMMAND = "task_manager_snapshot";
const PROCESS_DETAILS_COMMAND = "task_manager_process_details";

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isNumber(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value);
}

function isInteger(value: unknown): value is number {
  return isNumber(value) && Number.isSafeInteger(value);
}

function isOptionalNumber(value: unknown): boolean {
  return value == null || isNumber(value);
}

function isOptionalInteger(value: unknown): boolean {
  return value == null || isInteger(value);
}

function isIdentity(value: unknown): value is TaskManagerProcessIdentity {
  return (
    isRecord(value) &&
    isInteger(value["pid"]) &&
    isInteger(value["start_time_ticks"])
  );
}

function isProcessState(value: unknown): value is TaskManagerProcessState {
  if (!isRecord(value) || typeof value["kind"] !== "string") return false;
  return [
    "running",
    "sleeping",
    "disk_sleep",
    "stopped",
    "tracing_stop",
    "zombie",
    "dead",
    "idle",
  ].includes(value["kind"])
    ? true
    : value["kind"] === "unknown" && typeof value["value"] === "string";
}

function isPressure(value: unknown): value is TaskManagerPressureUsage {
  if (!isRecord(value)) return false;
  return [value["some"], value["full"]].every(
    (window) =>
      window == null ||
      (isRecord(window) &&
        isNumber(window["average_10"]) &&
        isNumber(window["average_60"]) &&
        isNumber(window["average_300"]) &&
        isInteger(window["total_microseconds"])),
  );
}

function isCpu(value: unknown): value is TaskManagerCpuUsage {
  return (
    isRecord(value) &&
    (value["model"] == null || typeof value["model"] === "string") &&
    [
      value["total_percent"],
      value["user_percent"],
      value["system_percent"],
      value["idle_percent"],
      value["io_wait_percent"],
    ].every(isOptionalNumber) &&
    Array.isArray(value["logical"]) &&
    value["logical"].every(
      (core) =>
        isRecord(core) &&
        isInteger(core["logical_index"]) &&
        isOptionalNumber(core["usage_percent"]),
    )
  );
}

function isMemory(value: unknown): value is TaskManagerMemoryUsage {
  return (
    isRecord(value) &&
    [
      value["total_bytes"],
      value["available_bytes"],
      value["used_bytes"],
      value["cached_bytes"],
      value["buffers_bytes"],
      value["swap_total_bytes"],
      value["swap_used_bytes"],
    ].every(isInteger)
  );
}

function isProcess(value: unknown): value is TaskManagerProcessSummary {
  return (
    isRecord(value) &&
    isIdentity(value["identity"]) &&
    isInteger(value["parent_pid"]) &&
    typeof value["name"] === "string" &&
    isProcessState(value["state"]) &&
    isOptionalNumber(value["cpu_percent"]) &&
    isInteger(value["resident_memory_bytes"]) &&
    isNumber(value["resident_memory_percent"]) &&
    isInteger(value["virtual_memory_bytes"]) &&
    isOptionalNumber(value["disk_read_bytes_per_second"]) &&
    isOptionalNumber(value["disk_write_bytes_per_second"]) &&
    typeof value["io_permission_denied"] === "boolean" &&
    isInteger(value["thread_count"]) &&
    isInteger(value["nice"])
  );
}

function isDevice(value: unknown): value is TaskManagerBlockDeviceUsage {
  return (
    isRecord(value) &&
    typeof value["name"] === "string" &&
    isOptionalNumber(value["read_bytes_per_second"]) &&
    isOptionalNumber(value["write_bytes_per_second"]) &&
    isOptionalNumber(value["busy_percent"]) &&
    isInteger(value["io_in_progress"]) &&
    isOptionalInteger(value["capacity_bytes"])
  );
}

function isInterface(
  value: unknown,
): value is TaskManagerNetworkInterfaceUsage {
  return (
    isRecord(value) &&
    typeof value["name"] === "string" &&
    isOptionalNumber(value["received_bytes_per_second"]) &&
    isOptionalNumber(value["transmitted_bytes_per_second"]) &&
    typeof value["is_loopback"] === "boolean"
  );
}

function decodeSnapshot(value: unknown): TaskManagerSnapshot {
  if (
    !isRecord(value) ||
    typeof value["is_baseline"] !== "boolean" ||
    typeof value["is_partial"] !== "boolean" ||
    ![
      undefined,
      null,
      "candidate_limit",
      "time_budget",
      "source_unavailable",
    ].includes(value["partial_reason"] as string | null | undefined) ||
    !isInteger(value["observed_process_count"]) ||
    !isInteger(value["matched_process_count"]) ||
    !isCpu(value["cpu"]) ||
    !isMemory(value["memory"]) ||
    !isRecord(value["load"]) ||
    ![
      value["load"]["one_minute"],
      value["load"]["five_minutes"],
      value["load"]["fifteen_minutes"],
    ].every(isNumber) ||
    ![
      value["load"]["runnable_tasks"],
      value["load"]["total_tasks"],
      value["load"]["uptime_seconds"],
    ].every(isInteger) ||
    ![
      value["cpu_pressure"],
      value["memory_pressure"],
      value["io_pressure"],
    ].every((pressure) => pressure == null || isPressure(pressure)) ||
    !Array.isArray(value["block_devices"]) ||
    !value["block_devices"].every(isDevice) ||
    !Array.isArray(value["network_interfaces"]) ||
    !value["network_interfaces"].every(isInterface) ||
    !Array.isArray(value["processes"]) ||
    !value["processes"].every(isProcess)
  ) {
    throw new Error("The Task Manager snapshot response is invalid.");
  }
  return value as TaskManagerSnapshot;
}

function isProcessMemory(
  value: unknown,
): value is TaskManagerProcessMemoryDetails {
  return (
    isRecord(value) &&
    [
      value["peak_virtual_bytes"],
      value["virtual_bytes"],
      value["peak_resident_bytes"],
      value["resident_bytes"],
      value["resident_anonymous_bytes"],
      value["resident_file_bytes"],
      value["resident_shared_bytes"],
      value["swap_bytes"],
    ].every(isOptionalInteger)
  );
}

function decodeProcessDetails(value: unknown): TaskManagerProcessDetails {
  const io = isRecord(value) ? value["io"] : undefined;
  if (
    !isRecord(value) ||
    !isIdentity(value["identity"]) ||
    !isInteger(value["parent_pid"]) ||
    typeof value["name"] !== "string" ||
    !isProcessState(value["state"]) ||
    !isOptionalInteger(value["uid"]) ||
    !isInteger(value["nice"]) ||
    !isInteger(value["thread_count"]) ||
    (value["command_line"] != null &&
      typeof value["command_line"] !== "string") ||
    (value["executable"] != null && typeof value["executable"] !== "string") ||
    !Array.isArray(value["cgroups"]) ||
    !value["cgroups"].every((item) => typeof item === "string") ||
    !isProcessMemory(value["memory"]) ||
    (io != null &&
      (!isRecord(io) ||
        ![
          io["characters_read"],
          io["characters_written"],
          io["read_syscalls"],
          io["write_syscalls"],
          io["read_bytes"],
          io["write_bytes"],
        ].every(isInteger))) ||
    !isOptionalInteger(value["voluntary_context_switches"]) ||
    !isOptionalInteger(value["involuntary_context_switches"]) ||
    !Array.isArray(value["restricted_fields"]) ||
    !value["restricted_fields"].every((field) =>
      ["command_line", "executable", "cgroup", "io"].includes(String(field)),
    )
  ) {
    throw new Error("The Task Manager process details response is invalid.");
  }
  return value as TaskManagerProcessDetails;
}

function normalizeFailure<T>(promise: Promise<T>): Promise<T> {
  return promise.catch((error: unknown) =>
    Promise.reject(normalizeAppError(error)),
  );
}

export interface TaskManagerApi {
  snapshot(request: TaskManagerSnapshotRequest): Promise<TaskManagerSnapshot>;
  processDetails(
    identity: TaskManagerProcessIdentity,
  ): Promise<TaskManagerProcessDetails>;
}

export function createTaskManagerApi(transport: Transport): TaskManagerApi {
  return {
    snapshot(request) {
      return normalizeFailure(
        transport.invoke(SNAPSHOT_COMMAND, decodeSnapshot, { request }),
      );
    },
    processDetails(identity) {
      return normalizeFailure(
        transport.invoke(PROCESS_DETAILS_COMMAND, decodeProcessDetails, {
          identity,
        }),
      );
    },
  };
}
