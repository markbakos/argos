import {
  ArrowDownIcon,
  ArrowUpIcon,
  CpuIcon,
  HardDriveIcon,
  MemoryStickIcon,
  NetworkIcon,
  RefreshCwIcon,
  SearchIcon,
} from "lucide-react";
import { useMemo, useState, type ReactNode, type SyntheticEvent } from "react";
import { useSearchParams } from "react-router-dom";

import { StateMessage } from "../../components/StateMessage";
import type {
  TaskManagerProcessIdentity,
  TaskManagerProcessSummary,
  TaskManagerSnapshot,
  TaskManagerSnapshotRequest,
  TaskManagerSort,
  TaskManagerSortDirection,
} from "../../generated";
import type { ModulePageProps } from "../registry";
import {
  formatBytes,
  formatCount,
  formatDuration,
  formatPercent,
  formatRate,
  formatState,
} from "./format";
import { ProcessDetailsDialog } from "./ProcessDetailsDialog";
import { useTaskManager, type TaskManagerHistoryPoint } from "./useTaskManager";

type View = "processes" | "performance";

const SORTS: readonly TaskManagerSort[] = [
  "cpu",
  "memory",
  "disk_read",
  "disk_write",
  "name",
  "pid",
  "threads",
];

const controlClass =
  "min-h-10 rounded-lg border border-[var(--border)] bg-[var(--surface)] px-3 text-sm outline-none hover:bg-[var(--surface-hover)] focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)]";

function memoryPercent(snapshot: TaskManagerSnapshot) {
  return snapshot.memory.total_bytes
    ? (snapshot.memory.used_bytes / snapshot.memory.total_bytes) * 100
    : 0;
}

function SummaryCard({
  icon,
  label,
  value,
  detail,
}: {
  icon: ReactNode;
  label: string;
  value: string;
  detail: string;
}) {
  return (
    <article className="min-w-0 rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-5 shadow-sm">
      <div className="flex items-center gap-2 text-sm font-medium text-[var(--text-muted)]">
        {icon}
        <h2>{label}</h2>
      </div>
      <p className="mt-4 truncate text-2xl font-semibold tracking-tight">
        {value}
      </p>
      <p
        className="mt-1 truncate text-xs text-[var(--text-muted)]"
        title={detail}
      >
        {detail}
      </p>
    </article>
  );
}

function Summary({ snapshot }: { snapshot: TaskManagerSnapshot }) {
  const disk = snapshot.block_devices.reduce<
    (typeof snapshot.block_devices)[number] | undefined
  >((busiest, device) => {
    const total =
      (device.read_bytes_per_second ?? 0) +
      (device.write_bytes_per_second ?? 0);
    const busiestTotal = busiest
      ? (busiest.read_bytes_per_second ?? 0) +
        (busiest.write_bytes_per_second ?? 0)
      : -1;
    return total > busiestTotal ? device : busiest;
  }, undefined);
  const network = snapshot.network_interfaces
    .filter((item) => !item.is_loopback)
    .reduce<(typeof snapshot.network_interfaces)[number] | undefined>(
      (busiest, item) => {
        const total =
          (item.received_bytes_per_second ?? 0) +
          (item.transmitted_bytes_per_second ?? 0);
        const busiestTotal = busiest
          ? (busiest.received_bytes_per_second ?? 0) +
            (busiest.transmitted_bytes_per_second ?? 0)
          : -1;
        return total > busiestTotal ? item : busiest;
      },
      undefined,
    );
  const diskRate = disk
    ? disk.read_bytes_per_second == null && disk.write_bytes_per_second == null
      ? null
      : (disk.read_bytes_per_second ?? 0) + (disk.write_bytes_per_second ?? 0)
    : undefined;
  const networkRate = network
    ? network.received_bytes_per_second == null &&
      network.transmitted_bytes_per_second == null
      ? null
      : (network.received_bytes_per_second ?? 0) +
        (network.transmitted_bytes_per_second ?? 0)
    : undefined;

  return (
    <section
      aria-label="System summary"
      className="grid gap-3 sm:grid-cols-2 xl:grid-cols-4"
    >
      <SummaryCard
        icon={<CpuIcon aria-hidden="true" className="size-4" />}
        label="CPU"
        value={formatPercent(snapshot.cpu.total_percent)}
        detail={
          snapshot.cpu.model ??
          `${formatCount(snapshot.cpu.logical.length)} logical CPUs`
        }
      />
      <SummaryCard
        icon={<MemoryStickIcon aria-hidden="true" className="size-4" />}
        label="Memory"
        value={formatPercent(memoryPercent(snapshot))}
        detail={`${formatBytes(snapshot.memory.used_bytes)} of ${formatBytes(snapshot.memory.total_bytes)}`}
      />
      <SummaryCard
        icon={<HardDriveIcon aria-hidden="true" className="size-4" />}
        label="Busiest disk"
        value={disk ? formatRate(diskRate) : "Unavailable"}
        detail={
          disk
            ? `${disk.name} · ${formatPercent(disk.busy_percent)} busy`
            : "No block device data"
        }
      />
      <SummaryCard
        icon={<NetworkIcon aria-hidden="true" className="size-4" />}
        label="Busiest network"
        value={network ? formatRate(networkRate) : "Unavailable"}
        detail={network?.name ?? "No non-loopback interface data"}
      />
    </section>
  );
}

function SortButton({
  label,
  sort,
  current,
  direction,
  onSort,
}: {
  label: string;
  sort: TaskManagerSort;
  current: TaskManagerSort;
  direction: TaskManagerSortDirection;
  onSort: (sort: TaskManagerSort) => void;
}) {
  const active = current === sort;
  const Icon = direction === "ascending" ? ArrowUpIcon : ArrowDownIcon;
  return (
    <button
      type="button"
      onClick={() => {
        onSort(sort);
      }}
      className="inline-flex min-h-10 items-center gap-1 rounded-md px-2 text-left text-xs font-semibold hover:bg-[var(--surface-hover)] focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:outline-none"
    >
      {label}
      {active ? <Icon aria-hidden="true" className="size-3.5" /> : null}
    </button>
  );
}

function ProcessesView({
  snapshot,
  search,
  sort,
  direction,
  onSearch,
  onSort,
  onSelect,
}: {
  snapshot: TaskManagerSnapshot;
  search: string;
  sort: TaskManagerSort;
  direction: TaskManagerSortDirection;
  onSearch: (value: string) => void;
  onSort: (value: TaskManagerSort) => void;
  onSelect: (process: TaskManagerProcessSummary) => void;
}) {
  const [searchInput, setSearchInput] = useState(search);
  const columns: { label: string; sort: TaskManagerSort; align?: "right" }[] = [
    { label: "Process", sort: "name" },
    { label: "PID", sort: "pid", align: "right" },
    { label: "CPU", sort: "cpu", align: "right" },
    { label: "Memory", sort: "memory", align: "right" },
    { label: "Disk read", sort: "disk_read", align: "right" },
    { label: "Disk write", sort: "disk_write", align: "right" },
    { label: "Threads", sort: "threads", align: "right" },
  ];

  function submit(event: SyntheticEvent<HTMLFormElement>) {
    event.preventDefault();
    onSearch(searchInput.trim());
  }

  return (
    <section aria-labelledby="processes-heading" className="mt-6">
      <div className="flex flex-wrap items-end justify-between gap-4">
        <div>
          <h2 id="processes-heading" className="text-xl font-semibold">
            Processes
          </h2>
          <p className="mt-1 text-sm text-[var(--text-muted)]">
            Showing {formatCount(snapshot.processes.length)} of{" "}
            {formatCount(snapshot.matched_process_count)} matching ·{" "}
            {formatCount(snapshot.observed_process_count)} observed
          </p>
        </div>
        <form role="search" onSubmit={submit} className="flex gap-2">
          <label className="relative block">
            <span className="sr-only">Search processes by name or PID</span>
            <SearchIcon
              aria-hidden="true"
              className="pointer-events-none absolute top-1/2 left-3 size-4 -translate-y-1/2 text-[var(--text-muted)]"
            />
            <input
              type="search"
              value={searchInput}
              maxLength={128}
              onChange={(event) => {
                setSearchInput(event.target.value);
              }}
              placeholder="Name or PID"
              className={`${controlClass} w-56 pl-9`}
            />
          </label>
          <button type="submit" className={controlClass}>
            Search
          </button>
        </form>
      </div>

      {snapshot.is_partial ? (
        <div className="mt-4">
          <StateMessage tone="warning" title="Partial process snapshot">
            The bounded scan stopped at{" "}
            {snapshot.partial_reason?.replaceAll("_", " ") ??
              "an unavailable source"}
            . Displayed values remain valid.
          </StateMessage>
        </div>
      ) : null}

      <div className="mt-4 overflow-auto rounded-xl border border-[var(--border)] bg-[var(--surface)]">
        <table className="w-full min-w-[58rem] border-collapse text-sm">
          <thead className="sticky top-0 z-[1] bg-[var(--surface-raised)] text-[var(--text-muted)]">
            <tr>
              {columns.map((column) => (
                <th
                  key={column.sort}
                  scope="col"
                  aria-sort={sort === column.sort ? direction : "none"}
                  className={`border-b border-[var(--border)] px-2 ${column.align === "right" ? "text-right" : "text-left"}`}
                >
                  <SortButton
                    label={column.label}
                    sort={column.sort}
                    current={sort}
                    direction={direction}
                    onSort={onSort}
                  />
                </th>
              ))}
              <th
                scope="col"
                className="border-b border-[var(--border)] px-4 py-3 text-left text-xs font-semibold"
              >
                State
              </th>
            </tr>
          </thead>
          <tbody>
            {snapshot.processes.map((process) => (
              <tr
                key={`${String(process.identity.pid)}:${String(process.identity.start_time_ticks)}`}
                className="task-manager-process-row border-b border-[var(--border)] last:border-0 hover:bg-[var(--surface-hover)]"
              >
                <th
                  scope="row"
                  className="max-w-72 px-4 py-2 text-left font-medium"
                >
                  <button
                    type="button"
                    onClick={() => {
                      onSelect(process);
                    }}
                    className="min-h-10 max-w-full truncate rounded-md text-left text-[var(--accent)] hover:underline focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] focus-visible:outline-none"
                    title={`Open details for ${process.name}`}
                  >
                    {process.name}
                  </button>
                </th>
                <td className="px-4 py-2 text-right tabular-nums">
                  {formatCount(process.identity.pid)}
                </td>
                <td className="px-4 py-2 text-right tabular-nums">
                  {formatPercent(process.cpu_percent)}
                </td>
                <td
                  className="px-4 py-2 text-right tabular-nums"
                  title={formatPercent(process.resident_memory_percent)}
                >
                  {formatBytes(process.resident_memory_bytes)}
                </td>
                <td className="px-4 py-2 text-right tabular-nums">
                  {process.io_permission_denied
                    ? "Restricted"
                    : formatRate(process.disk_read_bytes_per_second)}
                </td>
                <td className="px-4 py-2 text-right tabular-nums">
                  {process.io_permission_denied
                    ? "Restricted"
                    : formatRate(process.disk_write_bytes_per_second)}
                </td>
                <td className="px-4 py-2 text-right tabular-nums">
                  {formatCount(process.thread_count)}
                </td>
                <td className="px-4 py-2 capitalize text-[var(--text-muted)]">
                  {formatState(process.state.kind)}
                </td>
              </tr>
            ))}
          </tbody>
        </table>
        {!snapshot.processes.length ? (
          <div className="p-6">
            <StateMessage tone="empty" title="No matching processes">
              Clear or change the search to see the current bounded process
              list.
            </StateMessage>
          </div>
        ) : null}
      </div>
    </section>
  );
}

function Sparkline({
  values,
  label,
}: {
  values: (number | null)[];
  label: string;
}) {
  const points = values
    .map((value, index) =>
      value == null
        ? null
        : `${String(values.length === 1 ? 150 : (index / (values.length - 1)) * 300)},${String(64 - (Math.max(0, Math.min(100, value)) / 100) * 60)}`,
    )
    .filter((point): point is string => point !== null)
    .join(" ");
  return (
    <svg
      viewBox="0 0 300 68"
      role="img"
      aria-label={label}
      className="mt-4 h-20 w-full overflow-visible"
    >
      <path
        d="M0 64H300 M0 34H300 M0 4H300"
        stroke="var(--border)"
        strokeWidth="1"
      />
      {points ? (
        <polyline
          points={points}
          fill="none"
          stroke="var(--accent)"
          strokeWidth="2.5"
          vectorEffect="non-scaling-stroke"
        />
      ) : null}
    </svg>
  );
}

function Pressure({
  label,
  value,
}: {
  label: string;
  value: TaskManagerSnapshot["cpu_pressure"];
}) {
  return (
    <div className="rounded-lg bg-[var(--background)] p-3">
      <h4 className="text-sm font-medium">{label}</h4>
      {value?.some || value?.full ? (
        <dl className="mt-2 space-y-1 text-xs text-[var(--text-muted)]">
          {value.some ? (
            <div className="flex justify-between gap-3">
              <dt>Some, 10s</dt>
              <dd>{formatPercent(value.some.average_10)}</dd>
            </div>
          ) : null}
          {value.full ? (
            <div className="flex justify-between gap-3">
              <dt>Full, 10s</dt>
              <dd>{formatPercent(value.full.average_10)}</dd>
            </div>
          ) : null}
        </dl>
      ) : (
        <p className="mt-2 text-xs text-[var(--text-muted)]">Unsupported</p>
      )}
    </div>
  );
}

function PerformanceView({
  snapshot,
  history,
}: {
  snapshot: TaskManagerSnapshot;
  history: TaskManagerHistoryPoint[];
}) {
  return (
    <section aria-labelledby="performance-heading" className="mt-6 space-y-4">
      <div>
        <h2 id="performance-heading" className="text-xl font-semibold">
          Performance
        </h2>
        <p className="mt-1 text-sm text-[var(--text-muted)]">
          Current values and up to one minute of route-local aggregate history.
        </p>
      </div>
      <div className="grid gap-4 xl:grid-cols-2">
        <article className="rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-5">
          <div className="flex items-end justify-between gap-4">
            <div>
              <h3 className="font-semibold">CPU</h3>
              <p className="mt-1 text-xs text-[var(--text-muted)]">
                {snapshot.cpu.model ??
                  `${formatCount(snapshot.cpu.logical.length)} logical CPUs`}
              </p>
            </div>
            <p className="text-2xl font-semibold tabular-nums">
              {formatPercent(snapshot.cpu.total_percent)}
            </p>
          </div>
          <Sparkline
            values={history.map((point) => point.cpu)}
            label={`CPU history. Current ${formatPercent(snapshot.cpu.total_percent)}.`}
          />
          <dl className="grid grid-cols-2 gap-2 text-sm sm:grid-cols-4">
            <div>
              <dt className="text-[var(--text-muted)]">User</dt>
              <dd>{formatPercent(snapshot.cpu.user_percent)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">System</dt>
              <dd>{formatPercent(snapshot.cpu.system_percent)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">I/O wait</dt>
              <dd>{formatPercent(snapshot.cpu.io_wait_percent)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Load 1m</dt>
              <dd>{snapshot.load.one_minute.toFixed(2)}</dd>
            </div>
          </dl>
          <div className="mt-5 grid grid-cols-2 gap-2 sm:grid-cols-4">
            {snapshot.cpu.logical.map((core) => (
              <div
                key={core.logical_index}
                className="rounded-lg bg-[var(--background)] p-2"
              >
                <div className="flex justify-between gap-2 text-xs">
                  <span>CPU {formatCount(core.logical_index)}</span>
                  <span>{formatPercent(core.usage_percent)}</span>
                </div>
                <div className="mt-2 h-1.5 overflow-hidden rounded-full bg-[var(--border)]">
                  <div
                    className="h-full bg-[var(--accent)]"
                    style={{
                      width: `${String(Math.max(0, Math.min(100, core.usage_percent ?? 0)))}%`,
                    }}
                  />
                </div>
              </div>
            ))}
          </div>
        </article>

        <article className="rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-5">
          <div className="flex items-end justify-between gap-4">
            <div>
              <h3 className="font-semibold">Memory</h3>
              <p className="mt-1 text-xs text-[var(--text-muted)]">
                {formatBytes(snapshot.memory.total_bytes)} installed
              </p>
            </div>
            <p className="text-2xl font-semibold tabular-nums">
              {formatPercent(memoryPercent(snapshot))}
            </p>
          </div>
          <Sparkline
            values={history.map((point) => point.memory)}
            label={`Memory history. Current ${formatPercent(memoryPercent(snapshot))}.`}
          />
          <dl className="grid grid-cols-2 gap-2 text-sm sm:grid-cols-4">
            <div>
              <dt className="text-[var(--text-muted)]">Used</dt>
              <dd>{formatBytes(snapshot.memory.used_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Available</dt>
              <dd>{formatBytes(snapshot.memory.available_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Cache</dt>
              <dd>{formatBytes(snapshot.memory.cached_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Buffers</dt>
              <dd>{formatBytes(snapshot.memory.buffers_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Swap used</dt>
              <dd>{formatBytes(snapshot.memory.swap_used_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Swap total</dt>
              <dd>{formatBytes(snapshot.memory.swap_total_bytes)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Uptime</dt>
              <dd>{formatDuration(snapshot.load.uptime_seconds)}</dd>
            </div>
            <div>
              <dt className="text-[var(--text-muted)]">Tasks</dt>
              <dd>
                {snapshot.load.runnable_tasks} / {snapshot.load.total_tasks}
              </dd>
            </div>
          </dl>
        </article>
      </div>

      <article className="rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-5">
        <h3 className="font-semibold">Resource pressure</h3>
        <p className="mt-1 text-xs text-[var(--text-muted)]">
          Time tasks were delayed by resource contention. Categories are
          independent.
        </p>
        <div className="mt-4 grid gap-3 sm:grid-cols-3">
          <Pressure label="CPU" value={snapshot.cpu_pressure} />
          <Pressure label="Memory" value={snapshot.memory_pressure} />
          <Pressure label="I/O" value={snapshot.io_pressure} />
        </div>
      </article>

      <div className="grid gap-4 xl:grid-cols-2">
        <MetricTable
          title="Block devices"
          headers={["Device", "Read", "Write", "Busy"]}
          rows={snapshot.block_devices.map((device) => [
            device.name,
            formatRate(device.read_bytes_per_second),
            formatRate(device.write_bytes_per_second),
            formatPercent(device.busy_percent),
          ])}
          empty="Block-device metrics are unavailable."
        />
        <MetricTable
          title="Network interfaces"
          headers={["Interface", "Receive", "Transmit", "Kind"]}
          rows={snapshot.network_interfaces.map((item) => [
            item.name,
            formatRate(item.received_bytes_per_second),
            formatRate(item.transmitted_bytes_per_second),
            item.is_loopback ? "Loopback" : "Network",
          ])}
          empty="Network-interface metrics are unavailable."
        />
      </div>
    </section>
  );
}

function MetricTable({
  title,
  headers,
  rows,
  empty,
}: {
  title: string;
  headers: string[];
  rows: string[][];
  empty: string;
}) {
  return (
    <article className="overflow-hidden rounded-2xl border border-[var(--border)] bg-[var(--surface)]">
      <h3 className="px-5 pt-5 font-semibold">{title}</h3>
      {rows.length ? (
        <div className="mt-3 overflow-auto">
          <table className="w-full min-w-[28rem] text-sm">
            <thead className="text-left text-xs text-[var(--text-muted)]">
              <tr>
                {headers.map((header) => (
                  <th
                    key={header}
                    scope="col"
                    className="border-b border-[var(--border)] px-5 py-2 font-semibold"
                  >
                    {header}
                  </th>
                ))}
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr
                  key={row[0]}
                  className="border-b border-[var(--border)] last:border-0"
                >
                  {row.map((value, index) => (
                    <td key={headers[index]} className="px-5 py-3 tabular-nums">
                      {value}
                    </td>
                  ))}
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      ) : (
        <p className="px-5 py-5 text-sm text-[var(--text-muted)]">{empty}</p>
      )}
    </article>
  );
}

export default function TaskManagerPage({ module }: ModulePageProps) {
  const [parameters, setParameters] = useSearchParams();
  const view: View =
    parameters.get("view") === "performance" ? "performance" : "processes";
  const search = parameters.get("search") ?? "";
  const requestedSort = parameters.get("sort") as TaskManagerSort | null;
  const sort =
    requestedSort && SORTS.includes(requestedSort) ? requestedSort : "cpu";
  const direction: TaskManagerSortDirection =
    parameters.get("direction") === "ascending" ? "ascending" : "descending";
  const request = useMemo<TaskManagerSnapshotRequest>(
    () => ({
      fresh_baseline: false,
      ...(search ? { search } : {}),
      sort,
      direction,
      limit: 200,
    }),
    [direction, search, sort],
  );
  const taskManager = useTaskManager(request);
  const [selected, setSelected] = useState<{
    identity: TaskManagerProcessIdentity;
    name: string;
  }>();

  function update(next: Record<string, string | undefined>) {
    setParameters((current) => {
      const result = new URLSearchParams(current);
      Object.entries(next).forEach(([key, value]) => {
        if (value) result.set(key, value);
        else result.delete(key);
      });
      return result;
    });
  }

  function changeSort(next: TaskManagerSort) {
    update({
      sort: next,
      direction:
        sort === next && direction === "descending"
          ? "ascending"
          : "descending",
    });
  }

  return (
    <div className="mx-auto max-w-[96rem]">
      <header className="flex flex-wrap items-start justify-between gap-4">
        <div>
          <p className="text-xs font-semibold tracking-[0.18em] text-[var(--text-muted)] uppercase">
            Live system view
          </p>
          <h1 className="mt-2 text-3xl font-semibold tracking-tight">
            {module.manifest.display_name}
          </h1>
          <p className="mt-2 max-w-2xl text-sm leading-6 text-[var(--text-muted)]">
            Current resource use from bounded Linux kernel reads. Sampling stops
            when this page is hidden or closed.
          </p>
        </div>
        <button
          type="button"
          onClick={taskManager.refresh}
          className={`${controlClass} inline-flex items-center gap-2`}
          title="Refresh now"
        >
          <RefreshCwIcon aria-hidden="true" className="size-4" /> Refresh
        </button>
      </header>

      <div
        className="mt-6 inline-flex rounded-xl border border-[var(--border)] bg-[var(--surface)] p-1"
        role="tablist"
        aria-label="Task Manager views"
      >
        {(["processes", "performance"] as const).map((item) => (
          <button
            key={item}
            type="button"
            role="tab"
            aria-selected={view === item}
            onClick={() => {
              update({ view: item === "processes" ? undefined : item });
            }}
            className={`min-h-10 rounded-lg px-4 text-sm font-medium capitalize outline-none focus-visible:ring-2 focus-visible:ring-[var(--focus-ring)] ${view === item ? "bg-[var(--nav-active)] text-[var(--text)]" : "text-[var(--text-muted)] hover:bg-[var(--surface-hover)]"}`}
          >
            {item}
          </button>
        ))}
      </div>

      {taskManager.isPending && !taskManager.snapshot ? (
        <div className="mt-6">
          <StateMessage
            tone="loading"
            title="Taking the first bounded snapshot…"
          >
            Rates will appear after the next visible sample.
          </StateMessage>
        </div>
      ) : null}
      {taskManager.error && !taskManager.snapshot ? (
        <div className="mt-6">
          <StateMessage tone="error" title="Task Manager is unavailable">
            <button
              type="button"
              onClick={taskManager.refresh}
              className="mt-2 font-medium text-[var(--accent)] underline"
            >
              Try again
            </button>
          </StateMessage>
        </div>
      ) : null}
      {taskManager.error && taskManager.snapshot ? (
        <div className="mt-6">
          <StateMessage tone="warning" title="The latest refresh failed">
            Showing the last successful in-memory snapshot. Argos will retry at
            the normal cadence.
          </StateMessage>
        </div>
      ) : null}

      {taskManager.snapshot ? (
        <>
          {taskManager.snapshot.is_baseline ? (
            <div className="mt-6">
              <StateMessage tone="loading" title="Collecting rates…">
                Cumulative memory and process values are ready; CPU and
                throughput need one more sample.
              </StateMessage>
            </div>
          ) : null}
          <div className="mt-6">
            <Summary snapshot={taskManager.snapshot} />
          </div>
          {view === "processes" ? (
            <ProcessesView
              snapshot={taskManager.snapshot}
              search={search}
              sort={sort}
              direction={direction}
              onSearch={(value) => {
                update({ search: value || undefined });
              }}
              onSort={changeSort}
              onSelect={(process) => {
                setSelected({ identity: process.identity, name: process.name });
              }}
            />
          ) : (
            <PerformanceView
              snapshot={taskManager.snapshot}
              history={taskManager.history}
            />
          )}
        </>
      ) : null}

      {selected ? (
        <ProcessDetailsDialog
          identity={selected.identity}
          name={selected.name}
          onClose={() => {
            setSelected(undefined);
          }}
        />
      ) : null}
    </div>
  );
}
