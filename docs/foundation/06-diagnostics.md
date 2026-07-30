# FND-06 — Diagnostics and observability

**Status:** Approved for foundation implementation  
**Depends on:** FND-02 health/paths/storage, FND-03 module registry; integrates FND-04 and FND-05 health  
**Enables:** supportable verification and recovery

## Problem and user value

A local application touching XDG storage, SQLite, systemd, journals, and desktop processes needs useful failure evidence without telemetry or privacy leakage. Frontend console output and raw debug strings are insufficient.

The user gains an offline Diagnostics page that explains the active profile, safe resolved paths, database/migration state, module and systemd health, application build, and recent Argos failures, plus a redacted report they control.

## Workflows

1. The user opens Diagnostics even when a feature module is unavailable and sees independent subsystem states.
2. The user verifies `development` versus `production` and category directories without receiving a raw SQL interface or database file contract.
3. A system-manager permission failure appears separately from user-manager health and storage health.
4. A support scenario creates a timestamped redacted JSON report in the Argos state diagnostics directory and opens that folder through a narrow action.
5. A developer correlates a safe UI error ID with structured local logs that rotate and never grow without bound.
6. A verification run captures a point-in-time host/WebView memory snapshot without turning Diagnostics into a continuous monitor.

## Functional requirements

### Structured tracing

- Initialize Rust tracing after state-path resolution and before other I/O where possible; buffer only a small fixed number of early failures until a sink is available.
- Emit timestamp, level, component, module, operation, duration, result, correlation ID, and error code as applicable.
- Production defaults to concise rolling files under `state/logs`: at most five files of 5 MiB each. Development may use more verbose levels but retains the same bounded default unless explicitly changed outside production.
- Maintain an in-memory fixed-capacity view of the 50 most recent Argos error summaries, containing safe messages/codes/times/correlation IDs only.
- Never treat frontend console output as primary diagnostics or copy journal content, full helper output, launcher targets/arguments, environments, tokens, arbitrary file contents, SQL rows, or complete records into traces.

### Diagnostics aggregation

- Implement a use case that gathers each provider independently with an overall timestamp and reports partial failure rather than failing the whole page.
- Report application version, build identity, target/build mode, runtime profile, and whether a development-to-production override is active.
- Report resolved **category directories** for config/data/state/cache/runtime, runtime availability, and explicit-root status. Do not expose a database-file path to ordinary frontend state.
- Report database reachability, effective foreign keys/journal mode, current/target migration version, last migration/backup outcome, and integrity summary without table contents.
- Report user and system systemd connection health independently and journal adapter availability without loading unit logs.
- Report effective enabled modules, version, enablement, health/reason, and registry mismatches.
- Report the bounded recent Argos failures and an on-demand point-in-time memory snapshot for the Argos host and identifiable WebView child processes when supported. Unsupported measurement is an explicit state.
- Settings links to Diagnostics and displays persistent production-override warning when applicable.

### Safe export

- `ExportDiagnostics` creates a new timestamped JSON document under `state/diagnostics/` using a safe application-owned atomic create; it does not accept an arbitrary output path.
- Include schema version, generated time, build/profile, category paths, subsystem/module health, migration summary, safe recent failures, and optional memory snapshot.
- Exclude launcher items/targets/arguments, unit names/logs, database path/content, environment variables, tokens, home listings, command output, audit metadata, and arbitrary configuration values.
- Return export ID, generated time, and a safe display/location result sufficient to offer `OpenDiagnosticsFolder`; do not expose generic write/open operations.
- Classify export as `write` and audit it when storage is healthy. If database failure is the reason for diagnostics, permit export, record structured state tracing, and mark persistent audit unavailable in the report.

## Non-functional requirements

- Opening Diagnostics performs bounded on-demand reads and no continual polling. Manual refresh is available.
- Logging has no constant heartbeat and writes only when events occur.
- Aggregation deadlines prevent one unavailable provider from hanging the page.
- Export is deterministic in schema, human-inspectable, privately permissioned, and useful offline.
- Path display supports user verification while redaction treats the home prefix consistently in exported/support contexts according to the safe schema.

## Failure and recovery states

- State directory/log sink unavailable: retain bounded in-memory errors, use safe stderr in development when available, and report logging unavailable; do not fall back to repository or generic `/tmp`.
- One health provider times out/fails: its section is error/unavailable with correlation ID; other sections render.
- Database/migration unavailable: Diagnostics and safe export operate from non-database providers; database audit is marked unavailable.
- Runtime missing: explicit unavailable, not application failure.
- Memory inspection unsupported/permission denied: explicit unsupported/permission state, no repeated attempts.
- Export filename collision or write failure: safe typed error, no overwrite/partial final file.
- Log rotation failure: preserve current usable log, report bounded recent failure without recursive logging storms.

## Explicit exclusions

No telemetry/upload, crash-report service, log viewer for arbitrary files, database browser/dump, journal aggregation, continuous process monitor, system-wide metrics dashboard, user-selectable arbitrary export path, log tail, or automatic support submission.

## Architecture impact

Diagnostics is an application aggregator over build/path/storage/module/systemd/log/process-metric ports. Adapters return safe typed health, not raw handles. Tauri exposes narrow read/export/open-diagnostics-folder commands; React renders sections through TanStack Query.

## Contracts

Contracts include versioned `DiagnosticsSnapshot`, `SubsystemHealth`, `ResolvedCategoryPaths`, `StorageHealth`, `SystemdScopeHealth`, `ModuleDiagnostic`, `RecentFailure`, optional `ProcessMemorySnapshot`, `DiagnosticsExportResult`, and typed refresh/export errors. Diagnostic path views contain category directories only; safe export has its own redacted schema version.

## Persistence and migrations

Structured logs/reports live in the state directory, not SQLite. Audit uses the existing table when available. No new migration. Log/report retention for diagnostic exports is user-controlled in foundation—reports are not auto-deleted—but the application does not create them unless requested. Cache contains no diagnostic source of truth.

## Security implications

Diagnostics is a potential aggregation leak, so fields are allowlisted and snapshots/export have redaction tests. Export location is application-owned and not caller-selected. Opening uses the narrow folder opener. Correlation IDs are opaque. Paths are informative only in the privileged main UI and are home-redacted in shareable export when sufficient.

## Performance implications

No background sampler. Memory and integrity checks run on demand with timeouts; full SQLite integrity may be a separate explicit check if too expensive, while routine health uses a bounded quick check. Logs rotate by size, recent errors use fixed capacity, and diagnostics Query caching prevents duplicate concurrent gathers.

## Acceptance criteria

- **FND-06-AC01:** Structured traces contain required safe fields and rotation never exceeds five 5 MiB files under tested thresholds.
- **FND-06-AC02:** Recent error memory holds at most 50 entries and evicts oldest without recursive error growth.
- **FND-06-AC03:** Diagnostics renders build/profile/category paths, storage/migration, independent systemd scopes, modules, journal, recent failures, and point-in-time memory/unsupported state.
- **FND-06-AC04:** Failure/timeout of every provider independently leaves other diagnostic sections usable and bounded.
- **FND-06-AC05:** Safe export is versioned, private, atomically created under state diagnostics, opens through a narrow folder action, and handles storage failure without pretending persistent audit.
- **FND-06-AC06:** Redaction fixtures prove forbidden launcher, journal, environment, token, database, arbitrary content, and command-output data never enter logs/UI messages/exports.
- **FND-06-AC07:** Diagnostics performs no global polling/heartbeat/continual write and releases any measurement resource after refresh.
- **FND-06-AC08:** Production override is prominently visible; normal development and target production show correct resolved categories.
- **FND-06-AC09:** Diagnostics page and export interactions are keyboard accessible with labeled non-color health states.

## Testing strategy

Use fake provider aggregation/deadline tests, forced tiny-threshold rotation tests, fixed-buffer tests, redaction corpus/snapshot tests, safe atomic export filesystem tests, storage-unavailable tests, mocked frontend section tests, accessibility checks, and target inspection of a real report/log directory.

## Implementation order and tasks

1. `FND-DIA-001` — structured tracing, rotation, redaction, and recent-error buffer.
2. `FND-DIA-002` — typed health providers and partial diagnostics aggregator.
3. `FND-DIA-003` — Diagnostics UI, manual refresh, and profile warning.
4. `FND-DIA-004` — safe export and narrow folder-open flow.
5. `FND-DIA-005` — memory snapshot, privacy, idle, and failure verification.

## Verification and documentation update

Run FND-06 criteria plus logging/privacy/idle portions of [Verification](../verification.md). Document target-supported child-process measurement and actual tracing backend. Any new diagnostic field must receive an explicit privacy classification and export decision before implementation.
