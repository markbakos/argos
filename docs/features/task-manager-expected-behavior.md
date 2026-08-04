# TM-01 Task Manager expected behavior

## Purpose and authority

These Given/When/Then scenarios are the test-first oracle for [TM-01](01-task-manager.md). They refine its acceptance criteria without adding scope. If a scenario conflicts with TM-01 or a higher-authority document, update and approve the normative document before implementation.

TM-01 was approved on 2026-08-05. Implementation still follows the selected dependency-ordered task in the feature ledger.

## TM-01-AC01 — Lazy module integration

### Scenario: Task Manager occupies one registry entry

- **Given** the backend and frontend module registries contain Task Manager
- **When** effective module navigation is built
- **Then** stable ID `task-manager` maps to `/task-manager` exactly once
- **And** it is default-enabled and ordered immediately after Dashboard
- **And** no core-route or sidebar file contains a duplicate Task Manager entry.

### Scenario: Another route performs no Task Manager work

- **Given** the application opens directly on Dashboard, Settings, Diagnostics, or another feature
- **When** that route settles
- **Then** Task Manager route code is not loaded
- **And** no Task Manager API call, timer, `/proc` read, `/sys` read, or cache entry exists.

## TM-01-AC02 — Active-only lifecycle

### Scenario: Opening starts with a baseline

- **Given** no Task Manager sample exists
- **When** the visible Task Manager route mounts
- **Then** exactly one snapshot starts immediately
- **And** cumulative/current values may render
- **But** CPU, disk, network, and process rates render `Collecting…` until a valid delta exists.

### Scenario: Visible route samples at the bounded cadence

- **Given** a baseline completed at monotonic time zero
- **When** the document remains visible for six seconds
- **Then** no more than one snapshot starts at each two-second boundary
- **And** no application-wide sampler or subscription exists.

### Scenario: Slow work never overlaps

- **Given** one snapshot remains in progress past the next scheduled tick
- **When** that tick occurs
- **Then** no second snapshot starts
- **And** sampling resumes only after the prior request finishes and the next cadence is due.

### Scenario: Inactive route stops work

- **Given** Task Manager is sampling
- **When** the user navigates away, disables the module, hides/minimizes the document, or closes the window
- **Then** future ticks are removed immediately
- **And** cancellation is requested for obsolete work
- **And** any late result cannot update UI state
- **And** no Task Manager read/write continues while inactive.

### Scenario: Visibility restoration resets rates

- **Given** the route was hidden or the prior sample is more than five seconds old
- **When** Task Manager becomes visible again
- **Then** the next snapshot is a fresh baseline
- **And** old deltas are not divided across the inactive gap.

## TM-01-AC03 — System metric correctness

### Scenario: Aggregate and per-CPU percentages use deltas

- **Given** two `/proc/stat` fixtures with known user, nice, system, idle, I/O-wait, IRQ, softirq, steal, guest, and guest-nice counters
- **When** the second snapshot is derived
- **Then** total and each logical CPU percentage match the non-double-counted deltas
- **And** guest fields are not counted twice
- **And** I/O wait is labeled separately from productive CPU work.

### Scenario: CPU counter regression becomes unavailable

- **Given** any required CPU counter decreases or total delta is zero
- **When** usage is derived
- **Then** the affected usage is unavailable for that sample
- **And** it is neither negative nor clamped into a believable stale value.

### Scenario: Memory uses available memory

- **Given** a `/proc/meminfo` fixture with total, available, cache, buffers, and swap values
- **When** memory usage is derived
- **Then** used memory equals total minus available within safe bounds
- **And** overlapping categories are not summed into a false total
- **And** absent swap is displayed as unsupported/not configured rather than an error.

### Scenario: Optional aggregate sources fail independently

- **Given** CPU and memory are readable but PSI, one block source, or network statistics are missing/malformed
- **When** a snapshot is built
- **Then** CPU, memory, and valid sections remain available
- **And** each missing source has its own unsupported/partial state.

### Scenario: Block and network counters produce honest rates

- **Given** two disk/network fixtures with known elapsed time and cumulative counters
- **When** rates are derived
- **Then** each device/interface has the expected byte rate
- **And** reset counters produce no rate
- **And** the summary identifies the busiest eligible item instead of claiming a de-duplicated global total.

### Scenario: Static metadata is not repeatedly read

- **Given** Task Manager remains visible through several snapshots
- **When** the adapter records reads
- **Then** static CPU/block metadata is read at most once for that visible route session
- **And** recurring samples use only the required dynamic sources.

## TM-01-AC04 — Process identity and rates

### Scenario: A complex process name parses safely

- **Given** a `/proc/<pid>/stat` fixture whose parenthesized command contains spaces and closing-parenthesis characters
- **When** the row is parsed
- **Then** PID, name, state, parent, times, start identity, nice, threads, virtual size, and RSS remain correctly aligned.

### Scenario: Matching identity receives a CPU and I/O rate

- **Given** two samples contain the same PID and start time with increasing CPU and I/O counters
- **When** the second process summary is derived
- **Then** its CPU share uses aggregate machine ticks
- **And** physical read/write rates use elapsed monotonic time
- **And** its result remains within numeric bounds.

### Scenario: PID reuse never inherits counters

- **Given** two samples contain the same PID with different start times
- **When** the second summary is derived
- **Then** CPU and I/O rates are baseline/unavailable
- **And** no value from the first process appears on the replacement process.

### Scenario: I/O permission is field-level

- **Given** process stat is readable but `/proc/<pid>/io` returns permission denied
- **When** the row is built
- **Then** name, PID, CPU, RSS, state, and threads remain visible
- **And** disk fields say restricted/unavailable.

### Scenario: Exit races are normal partial results

- **Given** a process exits between directory enumeration and file reads
- **When** collection continues
- **Then** the process is skipped or incomplete
- **And** other rows remain available
- **And** no retry storm or error-level log is emitted.

## TM-01-AC05 — Search, sort, and bounds

### Scenario: Requests are validated in Rust

- **Given** empty/whitespace search, overlong search, unknown sort, invalid direction, zero limit, or limit above 200
- **When** a snapshot request crosses the application boundary
- **Then** valid values are normalized
- **And** invalid/out-of-range values return a stable validation error before procfs scanning.

### Scenario: Every sort is deterministic

- **Given** summaries tied on CPU, RSS, disk read, disk write, name, PID, or threads
- **When** each supported ascending/descending sort runs
- **Then** the requested order is correct
- **And** PID is the final deterministic tie-breaker.

### Scenario: Search does not widen collection

- **Given** a name/PID search and more matching/nonmatching processes than the response limit
- **When** the snapshot completes
- **Then** only matching rows are returned up to the validated limit
- **And** observed, matched, and returned counts remain explicit.

### Scenario: Scan limits return partial truth

- **Given** more than 4,096 candidates or the 250 ms soft budget is reached between candidates
- **When** collection stops
- **Then** at most 200 rows cross the contract
- **And** the snapshot is marked partial/truncated with a reason
- **And** the next sample stays on the normal cadence rather than immediately retrying.

## TM-01-AC06 — On-demand details

### Scenario: Details read only the selected identity

- **Given** a displayed process identity
- **When** the user opens its detail panel
- **Then** only that PID's bounded status, command line, executable link, cgroup, and I/O sources may be read
- **And** no environment, maps, file-descriptor directory, stack, namespace, capability, `smaps`, or thread directory is read.

### Scenario: Sensitive strings are bounded

- **Given** command line, executable path, or cgroup content exceeds its limit or contains invalid text
- **When** details are decoded
- **Then** the field is safely truncated/omitted with an availability state
- **And** no unbounded allocation or raw bytes cross the contract.

### Scenario: Selected process disappears or is reused

- **Given** the selected PID is gone or its current start time differs
- **When** detail is requested
- **Then** `TASK_MANAGER_PROCESS_GONE` is returned
- **And** data from the replacement process is never rendered.

### Scenario: Closing details releases personal data

- **Given** details contain synthetic command/executable/cgroup values
- **When** the panel or Task Manager route closes
- **Then** the detail Query/cache is removed
- **And** those values are absent from logs, audit, diagnostics, export, browser storage, and persisted files.

## TM-01-AC07 — Partial and fatal failure

### Scenario: Procfs root is unavailable

- **Given** the trusted procfs root cannot be opened
- **When** Task Manager requests a snapshot
- **Then** `TASK_MANAGER_UNAVAILABLE` is returned with a safe message and retryability
- **And** Dashboard, Settings, Diagnostics, and navigation remain usable.

### Scenario: Malformed optional data stays local

- **Given** one process or optional aggregate file is malformed
- **When** the snapshot completes
- **Then** the affected row/field/section is unavailable or partial
- **And** valid system/process data still renders.

### Scenario: Unknown future fields are ignored

- **Given** a procfs fixture contains supported prefixes plus extra trailing fields
- **When** it is parsed
- **Then** known fields remain correct
- **And** the whole snapshot does not fail because the kernel added data.

## TM-01-AC08 — Clean accessible UI

### Scenario: Processes is the useful default

- **Given** Task Manager has a valid snapshot
- **When** the route opens
- **Then** CPU, memory, disk, and network summaries are visible
- **And** Processes is the selected view
- **And** the table reports capture/baseline/partial state and showing counts without relying on color.

### Scenario: Keyboard-only process inspection works

- **Given** the process table has rows
- **When** a keyboard user searches, changes sort, selects a row, and closes details with Escape
- **Then** every control has an accessible name and visible focus
- **And** detail initial focus and focus restoration are correct.

### Scenario: Performance charts have text equivalents

- **Given** aggregate history exists
- **When** the Performance view renders in normal or reduced-motion mode
- **Then** current numeric values and accessible labels communicate every chart's meaning
- **And** animation is absent/reduced as requested
- **And** no charting package is required.

### Scenario: All view states remain understandable

- **Given** loading, first baseline, empty search, unsupported PSI, permission-limited I/O, partial snapshot, stale/late response, or fatal module failure
- **When** the corresponding state renders
- **Then** it has concise text, appropriate status semantics, and a safe next action where one exists.

## TM-01-AC09 — Read-only security and privacy

### Scenario: Capability inventory is read-only

- **Given** the built application and capability files
- **When** static inventory runs
- **Then** Task Manager exposes only narrow snapshot/detail reads to the main window
- **And** no signal, kill, suspend, priority, shell, generic file, arbitrary operation, or elevation surface exists.

### Scenario: React cannot select kernel paths

- **Given** any frontend request payload
- **When** the Rust adapter resolves sources
- **Then** runtime procfs/sysfs paths are fixed by trusted code
- **And** the request cannot supply or traverse a path.

### Scenario: Personal process data has no secondary sink

- **Given** a privacy corpus in process names, commands, paths, and cgroups
- **When** snapshots/details encounter it
- **Then** it may appear only in the active local Task Manager UI response
- **And** it does not appear in logs, audit rows, diagnostics snapshots/exports, XDG/repository files, frontend console output, or test evidence.

## TM-01-AC10 — Performance evidence

### Scenario: Inactive measurement is zero

- **Given** the full Argos process group has stabilized on another route
- **When** it is observed for ten minutes
- **Then** Task Manager contributes zero reads, timers, IPC calls, subscriptions, and writes.

### Scenario: Active measurement stays within budget

- **Given** target hardware, build mode, process count, and visible Task Manager view are recorded
- **When** Task Manager samples for ten minutes
- **Then** incremental full-process-group CPU averages below 1% of one logical CPU
- **And** snapshot wall time p95 is below 250 ms
- **And** no snapshots overlap or produce continual filesystem/database writes.

### Scenario: Extended active use does not grow

- **Given** Task Manager alternates Processes/Performance and process details during a 30-minute run
- **When** host/WebView memory and cache sizes are observed
- **Then** raw/history/result bounds hold
- **And** there is no sustained memory growth attributable to Task Manager.

## TM-01-AC11 — Cache and rate honesty

### Scenario: Cache limits remain fixed

- **Given** more than 30 successful samples and many processes
- **When** another sample arrives
- **Then** aggregate history contains at most 30 points
- **And** Rust retains at most one prior bounded raw snapshot
- **And** no per-process history accumulates.

### Scenario: Route teardown releases route data

- **Given** process rows, details, and chart history are present
- **When** the last Task Manager route observer unmounts
- **Then** route-owned data becomes immediately/near-immediately collectible
- **And** no cache policy keeps sampling alive.

### Scenario: Rates never masquerade as zero

- **Given** no valid prior identity/counter snapshot exists
- **When** a rate field renders
- **Then** it is explicitly collecting/unavailable
- **And** zero is shown only when two valid counters prove zero activity.

## TM-01-AC12 — Target smoke

### Scenario: Real normal-user inspection

- **Given** a development-profile Argos build on the target Arch/GNOME workstation running as the normal user
- **When** the user opens Task Manager
- **Then** real CPU, memory, process, available block, and available network data render
- **And** PSI/permission-limited sources report supported or honest unavailable states
- **And** no elevation prompt occurs.

### Scenario: Real lifecycle teardown

- **Given** target sampling is active
- **When** the window is minimized/hidden, another route opens, and finally Argos closes
- **Then** sampling stops in each inactive state
- **And** closing leaves no Argos process, sampler, daemon, or written Task Manager history.

## Traceability

| Acceptance criterion | Primary scenarios                                                     |
| -------------------- | --------------------------------------------------------------------- |
| TM-01-AC01           | Lazy registry and inactive-route scenarios                            |
| TM-01-AC02           | Baseline, cadence, overlap, teardown, restoration scenarios           |
| TM-01-AC03           | CPU, memory, optional source, disk/network, static metadata scenarios |
| TM-01-AC04           | Process parser, delta, PID reuse, permission, exit-race scenarios     |
| TM-01-AC05           | Validation, deterministic sort, search, scan-bound scenarios          |
| TM-01-AC06           | Selected reads, string bounds, gone/reused, release scenarios         |
| TM-01-AC07           | Procfs unavailable, malformed optional, future-field scenarios        |
| TM-01-AC08           | Default view, keyboard, chart, state scenarios                        |
| TM-01-AC09           | Capability, fixed path, privacy scenarios                             |
| TM-01-AC10           | Inactive, active, extended-use measurements                           |
| TM-01-AC11           | Cache, teardown, rate-honesty scenarios                               |
| TM-01-AC12           | Normal-user target and teardown smoke                                 |
