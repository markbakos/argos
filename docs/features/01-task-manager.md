# TM-01 — Task Manager

**Status:** Approved for implementation on 2026-08-05  
**Type:** Compiled read-only module  
**Depends on:** FND-BST-004 contracts; FND-BST-006 capability/lifecycle; FND-SHL-002 shared UI patterns; FND-SHL-003/004 backend and frontend registries  
**Persistence:** None

## Problem and user value

Argos needs a first new section that gives the local user a clean, Windows Task Manager-like explanation of current CPU, memory, storage, network, and process usage without becoming a resident monitor or making the machine noticeably busier.

The user can answer two questions quickly:

1. How busy is this machine right now?
2. Which process is using how much CPU, resident memory, and disk I/O?

The feature goes as deep as Linux exposes cheaply through bounded virtual kernel files. It does not add a daemon, persistent history, vendor-specific hardware probing, or expensive inspection merely to fill the screen.

## User-visible scope

Task Manager is a compiled module with stable ID `task-manager`, route `/task-manager`, and label `Task Manager`. When enabled, it is the first feature section after Dashboard in default navigation order. Settings and Diagnostics retain their core-route positions.

The page has two cleanly separated views:

- **Processes** — the default view, with current system summary cards, process search/sort, a bounded process table, and on-demand details for one selected process;
- **Performance** — current and short route-local history for CPU, memory, pressure, block devices, and network interfaces.

No metric is gathered at application startup or while another route is active.

## Workflows

1. The user opens Task Manager. The route lazy-loads, takes one baseline snapshot, and labels delta-based rates as `Collecting…` rather than showing a false zero.
2. After the next sample, summary cards show total CPU, used/available memory and swap, busiest block-device activity, and busiest non-loopback interface throughput.
3. The Processes view shows a sortable/searchable bounded table with process name, PID, state, CPU share, resident memory, disk read/write rate, and thread count.
4. The user selects one process to request deeper identity, scheduling, memory, I/O, and cgroup details. Details are read only for that selection and are discarded when closed.
5. The Performance view shows aggregate and per-logical-CPU usage, memory composition, load/uptime, supported pressure values, block-device rates, and network-interface rates.
6. Leaving Task Manager, hiding/minimizing the WebView, disabling the module, or closing Argos stops future sampling and cancels/discards obsolete work.
7. Missing kernel interfaces, process exit races, or permissions degrade only affected fields/sections and never trigger elevation or rapid retry.

## Functional requirements

### Module and lifecycle

- Reuse the centralized backend manifest and frontend lazy registry; do not hard-code Task Manager into core routes or sidebar markup.
- Register one read-only module capability. The main local window is the only initial consumer.
- Lazy-load route code and issue no Task Manager command, timer, subscription, `/proc` read, or `/sys` read until `/task-manager` is active and the document is visible.
- Take one immediate baseline and then at most one snapshot every two seconds. The interval is fixed for the first version; there is no settings/config surface.
- Never overlap snapshots. If a snapshot is still running, skip the next scheduled tick.
- On route unmount or document invisibility, cancel when practical, discard any late response, and schedule no further sample. Visibility restoration starts with a fresh baseline.
- Use no background thread, daemon, heartbeat, system event subscription, startup prefetch, or application-wide interval.

### Sampling model and bounded cache

- A snapshot is a point-in-time read. Rates come from monotonic counter deltas between two valid snapshots; the first sample and any sample after a gap greater than five seconds have no rate.
- Counter regression, reset, process identity change, zero elapsed time, or malformed input makes only that derived rate unavailable. Values are never allowed to underflow, overflow, become negative, or present stale rates as current.
- Identify a process sample by `(pid, start_time_ticks)` so PID reuse cannot inherit an earlier process's CPU or I/O rate.
- Retain at most one bounded previous raw snapshot in the Rust application service for delta calculation.
- Retain at most 30 aggregate display samples in the mounted React route, representing about one minute at the default cadence. Do not retain per-process history.
- Task Manager Query data uses route-local ownership and immediate/near-immediate garbage collection after the final observer leaves. No Task Manager data enters SQLite, `config.toml`, browser storage, XDG files, logs, diagnostics export, or audit.

### System metrics

Read the smallest Linux kernel interfaces that provide the required snapshot:

| Metric                         | Primary source                                              | Display/derivation                                                                  |
| ------------------------------ | ----------------------------------------------------------- | ----------------------------------------------------------------------------------- |
| Total and per-logical-CPU time | `/proc/stat`                                                | busy percentage from deltas; user, system, idle, and supported supplementary shares |
| Memory and swap                | `/proc/meminfo`                                             | total, available, used as `MemTotal - MemAvailable`, cache/buffers, swap used       |
| Load and uptime                | `/proc/loadavg`, `/proc/uptime`                             | 1/5/15-minute load, runnable/total tasks, uptime                                    |
| CPU/memory/I/O pressure        | `/proc/pressure/{cpu,memory,io}`                            | `some`/`full` 10/60/300-second percentages when supported                           |
| Block-device activity          | `/proc/diskstats` plus bounded static `/sys/block` metadata | per whole device read/write rates, in-flight work, and busy percentage              |
| Network activity               | `/proc/net/dev`                                             | per-interface receive/transmit rates and busiest non-loopback interface             |
| Static CPU identity            | bounded `/proc/cpuinfo` read once per visible route session | model label and logical processor count when present                                |

Rules:

- CPU total excludes guest counters already included in user/nice counters. Busy time is total delta minus idle and I/O-wait deltas; I/O wait may be displayed separately but is not presented as exact process attribution.
- Per-process CPU is the process tick delta divided by aggregate machine tick delta, expressed as a share of total logical machine capacity from 0–100%.
- Memory uses the kernel's `MemAvailable` estimate. Categories may overlap and must not be rendered as if every row sums exactly to total memory.
- Block devices are shown separately. Do not sum partitions, device-mapper layers, and underlying devices into a misleading machine total. The summary names the busiest whole device.
- Network interfaces are shown separately because bridges, virtual devices, VPNs, and physical links may count the same traffic. The summary names the busiest non-loopback interface rather than claiming unique machine-wide bytes.
- A missing PSI, block, network, or static metadata source is `unsupported`/`unavailable` for that section; it does not fail CPU, memory, or the process table.

### Process snapshot

- Enumerate numeric `/proc/<pid>` entries only.
- Read `/proc/<pid>/stat` for identity, state, parent PID, CPU ticks, start identity, nice value, thread count, virtual size, and resident pages.
- Read `/proc/<pid>/io` for physical `read_bytes`/`write_bytes` and cumulative I/O fields when permitted. Missing permission makes I/O unavailable for that process; it does not remove an otherwise valid row.
- Do not read every process's command line, environment, maps, file descriptors, stack, or precise `smaps` data during recurring sampling.
- Bound each pass to 4,096 candidate processes, a 250 ms soft processing budget checked between candidates, and at most 200 returned rows. If a bound is reached, return a structured partial/truncated state and the observed counts.
- Bound process names and all parsed text before contract construction. Ignore unknown trailing fields for forward compatibility.
- The request accepts an optional trimmed name/PID search, sort key, sort direction, and result limit from 1–200. Rust validates them and performs deterministic sorting with PID as the final tie-breaker.
- Supported sort keys are CPU, resident memory, disk read rate, disk write rate, process name, PID, and thread count.
- A process that exits or changes identity during collection is skipped or has the affected optional fields omitted. Expected `/proc` races do not create error logs.

### On-demand process details

Opening one row may read only that observed `(pid, start_time_ticks)` from bounded files:

- `/proc/<pid>/status` for UID, state, thread/context-switch counts, resident breakdown, virtual/peak/high-water memory, and process swap;
- `/proc/<pid>/cmdline` for a display command capped at 16 KiB and 256 arguments;
- `/proc/<pid>/exe` for a bounded executable-path display when permission permits;
- `/proc/<pid>/cgroup` for bounded membership display;
- `/proc/<pid>/io` for cumulative character/physical I/O and syscall counts.

The backend rechecks start identity before returning details. A gone/reused process returns `TASK_MANAGER_PROCESS_GONE`. Permission-limited fields are individually unavailable. Environment variables, open-file enumeration, memory maps, stacks, namespaces, capabilities, and thread-by-thread inspection are excluded.

### Presentation and interaction

- Summary cards show CPU, memory, busiest disk, and busiest network values with an explicit capture/refresh state.
- The Processes table uses semantic table markup, a sticky header where supported, keyboard-operable sort controls, a labeled search field, non-color state text, localized byte/rate formatting, and a clear `showing N of M`/partial message.
- Selecting a row opens an accessible detail panel/dialog with initial focus, Escape close, and focus restoration.
- The Performance view uses lightweight CSS and inline SVG only; no charting dependency. Charts have text equivalents/current values and respect reduced motion.
- Loading, baseline, empty, unsupported, partial, permission, stale, and fatal module states are visually distinct and readable without color.
- Sorting or switching views reuses the current snapshot when possible and must not create additional simultaneous samplers.

## Non-functional requirements

### Performance

- Inactive Task Manager overhead is zero Task Manager reads, timers, IPC calls, subscriptions, and writes.
- Active sampling uses the fixed two-second cadence and the bounded sources above. Static metadata is read once per visible route session, not every sample.
- On the target workstation, the incremental full Argos process-group CPU attributable to an open Task Manager averages below 1% of one logical CPU over ten minutes, with snapshot wall time p95 below 250 ms. Record process count and visible view with the result.
- Active sampling must not cause continual database/config/state writes, unbounded allocations, overlapping work, or sustained memory growth over a 30-minute route-open run.
- If a target misses a budget, reduce sampled depth/returned rows or slow the cadence through a specification update; do not hide the measurement or add caching infrastructure speculatively.

### Maintainability

- Reuse existing domain/application/contracts/Linux-adapter/Tauri/API/module boundaries. Do not add a Task Manager crate, generic procfs browser, generic file-read command, sampler framework, worker service, or new dependency without measured evidence that the direct implementation is insufficient.
- Keep procfs/sysfs parsing in `argos-platform-linux`, delta/rate policy in `argos-application`, public values in domain/contracts, Tauri translation thin, and presentation state in the lazy React module.
- Parsers accept fixture roots in tests; runtime paths remain fixed trusted kernel paths and are never supplied by React.

## Failure and recovery states

| State                              | Required behavior                                                                       |
| ---------------------------------- | --------------------------------------------------------------------------------------- |
| `/proc` unavailable/unreadable     | module unavailable with safe retry; shell remains usable                                |
| First/resumed sample               | cumulative values shown where useful; rates say `Collecting…`                           |
| One aggregate source missing       | affected section unsupported/partial; other sections update                             |
| Process exits during scan          | skip/partial field without noisy error or retry storm                                   |
| Process metadata permission denied | keep safe summary; mark restricted detail/I/O fields                                    |
| PID reused before detail           | reject with `TASK_MANAGER_PROCESS_GONE`; never show the new process as the selected one |
| Counter reset/regression           | suppress that rate for one baseline interval                                            |
| Snapshot exceeds bound             | return partial/truncated snapshot and remain at normal cadence                          |
| Snapshot exceeds interval          | do not overlap; next sample begins only after the prior one completes                   |
| Route hidden/closed                | stop scheduling immediately and discard obsolete results                                |

## Explicit exclusions

- Ending, killing, suspending, renicing, signaling, or changing any process.
- Root/polkit elevation or reading data unavailable to the normal user.
- Continuous/background monitoring, startup collection, tray behavior, notifications, alerts, telemetry, persistent history, recording, or export.
- GPU utilization or per-process GPU attribution; vendor/driver APIs are not uniform or cheap enough for the first version.
- Per-process network attribution; Linux does not provide it through one cheap procfs snapshot and packet/eBPF accounting is disproportionate.
- Temperatures, fans, voltages, power draw, SMART/NVMe health, battery health, and hardware benchmarking.
- Precise recurring `smaps`, environment, open files, memory maps, stacks, namespaces, security capabilities, or thread-by-thread inspection.
- Process grouping heuristics, application icons, dependency trees, cgroup mutation, containers, remote hosts, historical comparison, configurable refresh intervals, or a generic observability platform.

## Architecture impact

No accepted architecture decision changes. TM-01 reuses ADR-003/004/007/011/013: Rust owns fixed kernel reads, the application owns metric semantics, contracts remain typed, Tauri exposes narrow reads, and React uses the centralized lazy module registry.

The implementation adds cohesive Task Manager vocabulary/use cases inside the existing domain/application crates and a Linux reader inside `argos-platform-linux`. A separate crate or generic sampler abstraction is not justified.

The polling exception is route-local and required because procfs counters do not emit a complete change event. It does not weaken the ban on application-wide intervals or inactive work.

## Contracts and errors

Purpose-specific contracts include:

- `TaskManagerSnapshotRequest`, `TaskManagerSort`, and bounded result limit/search;
- `TaskManagerSnapshot`, capture/baseline/partial metadata, `SystemUsage`, `CpuUsage`, `MemoryUsage`, `PressureUsage`, `BlockDeviceUsage`, and `NetworkInterfaceUsage`;
- `ProcessIdentity`, `ProcessSummary`, field availability, and `ProcessDetails`;
- optional numeric rates represented explicitly rather than magic zeros.

Add stable public errors under a `TASK_MANAGER_` namespace: `TASK_MANAGER_UNAVAILABLE`, `TASK_MANAGER_SNAPSHOT_FAILED`, and `TASK_MANAGER_PROCESS_GONE`. Ordinary per-field permission limits are availability values; `PERMISSION_DENIED` is reserved for a request that cannot produce any safe result.

No contract exposes an arbitrary filesystem path, raw procfs text, file descriptor, environment, kernel handle, process mutation, or generic PID operation. Process detail accepts the observed PID plus start identity only.

## Persistence, audit, and privacy

There is no migration and no persistent Task Manager setting or record. All operations are classified `read` and create no audit event.

Process names, command lines, executable paths, cgroups, and resource values may be personal. They may be rendered only in the local main window and must not enter normal logs, recent failures, diagnostics, safe export, telemetry, browser storage, screenshots/fixtures, or committed evidence. Tests use synthetic names and values.

## Security implications

- The capability inventory adds only narrow Task Manager snapshot/detail reads to the local main window.
- Rust fixes all kernel source paths and validates search, sort, limit, PID, and start identity.
- No request accepts a path, shell text, command, signal, priority, arbitrary operation name, or actor.
- The module never elevates or works around `hidepid`, ptrace, ownership, or other kernel access policy.
- Base snapshots avoid command lines; potentially sensitive detail files are read only after explicit row selection.

## Acceptance criteria

- **TM-01-AC01:** Backend/frontend registries expose one lazy default-enabled `task-manager` module after Dashboard without duplicating navigation or loading code/data on other routes.
- **TM-01-AC02:** Tests prove one immediate baseline, a non-overlapping two-second visible-route cadence, fresh baseline after a gap, and zero further Task Manager work after hide/unmount/disable/close.
- **TM-01-AC03:** Fixture tests correctly derive total/per-CPU, memory/swap, load/uptime, supported PSI, per-device disk, and per-interface network values, including resets, missing fields, and unsupported sources.
- **TM-01-AC04:** Fixture tests parse process names/states/CPU/RSS/threads/I/O safely, match deltas by PID plus start identity, and never attribute a reused PID's counters to the prior process.
- **TM-01-AC05:** Search, every sort key, deterministic tie-breaking, 1–200 response limits, 4,096/250 ms scan bounds, and partial/truncated metadata are validated in Rust and tested.
- **TM-01-AC06:** Process details are explicit and bounded; process exit/reuse and permission-limited fields remain honest; environment/maps/fds/stacks are absent from reads and contracts.
- **TM-01-AC07:** Partial aggregate/process failures leave unaffected metrics usable, expected process races create no noisy logs, and fatal procfs failure leaves the shell usable.
- **TM-01-AC08:** Process and Performance views provide clean semantic summaries, accessible sorting/search/detail interaction, text equivalents for charts, and complete loading/baseline/empty/partial/unsupported/error states.
- **TM-01-AC09:** Static/capability/privacy checks find only narrow reads, no process mutation/elevation/generic file API, and no Task Manager data in persistence, logs, audit, diagnostics, or export.
- **TM-01-AC10:** Target measurements prove zero inactive work, active average incremental CPU below 1% of one logical CPU, snapshot p95 below 250 ms, no writes/overlap, and no sustained 30-minute memory growth.
- **TM-01-AC11:** Query/raw/history caches stay within specified bounds, first/reset samples never show false rates, and leaving the route releases route-owned process rows/details/history.
- **TM-01-AC12:** Target Arch/GNOME smoke displays real CPU, memory, process, disk, network, pressure-supported/unsupported, permission, visibility teardown, and clean-close behavior without root.

## Testing strategy

The normative scenarios are in [Task Manager expected behavior](task-manager-expected-behavior.md). Use pure delta/parser fixtures, fake monotonic time and Linux roots, application fakes, contract serialization, Tauri/API translation tests, mocked React timers/visibility/API, capability/privacy scans, and redacted target measurements.

Tests must not depend on the developer's live process names, command lines, XDG data, network state, or permissions. Only the final manual/measurement task reads the target system, and committed evidence is aggregate/redacted.

## Implementation order and tasks

The dependency-ordered `TMG-*` tasks live in [the feature task ledger](tasks.md). Approval selected `TMG-001` first; later tasks still require their listed dependencies.

## Source-interface rationale

The selected sources are kernel-owned snapshot interfaces: [procfs CPU, memory, and process fields](https://docs.kernel.org/filesystems/proc.html), [pressure stall information](https://docs.kernel.org/accounting/psi.html), [block I/O statistics](https://docs.kernel.org/admin-guide/iostats.html), and [network interface statistics](https://docs.kernel.org/networking/statistics.html). The design uses aggregate files when they avoid repeated sysfs opens and treats counter resets and unsupported fields explicitly.

## Verification and documentation update

Run all TM-01 expected-behavior cases, the full repository gate required by each task, boundary/capability/privacy scans, and the target lifecycle/performance record. Update this specification before implementation if real target evidence requires another source, cadence, bound, metric, dependency, capability, or privacy decision.
