# Architecture

## Architectural goals

Argos must keep trusted Linux operations out of the WebView, keep core behavior reusable outside Tauri, isolate live data from source, make modules independently understandable, and remain nearly idle when the user is not interacting with it.

The foundation is one normal desktop process group: a Tauri host, its WebView children, and short-lived explicitly launched helper processes such as bounded `journalctl` calls. There is no resident Argos process after the main window closes.

## Layering and dependency rule

```text
React interface
    -> typed frontend API
    -> thin Tauri commands/events
    -> application use cases
    -> domain types and ports
    -> SQLite, systemd, Linux process/filesystem adapters
    -> database, D-Bus, journalctl, desktop opener, executable process
```

Dependencies point inward. Domain types have no Tauri, SQLite, D-Bus, WebView, or process dependency. Application use cases depend on domain types and port traits. Adapters implement those ports. Contracts translate at the Tauri boundary and must not leak adapter types.

### Layer responsibilities

| Layer | Owns | Must not own |
| --- | --- | --- |
| Domain | IDs, entities, value objects, invariants, action classes, module/scope/health concepts, port traits where domain vocabulary is required | Tauri, SQL, D-Bus proxies, process commands, presentation state |
| Application | Use-case orchestration, authorization policy, transaction intent, validation flow, audit context, module coordination | Concrete SQLite, `zbus`, `journalctl`, Tauri handlers |
| Contracts | Serializable requests/responses/events/errors and Rust-to-TypeScript export annotations | Persistence entities, secrets, adapter handles, business orchestration |
| Adapters | SQLite repositories, path/config I/O, D-Bus mapping, journal reads, open/spawn mechanics, tracing sinks | Cross-module business policy or UI decisions |
| Tauri | Composition root, state handles, narrow command/event translation, window lifecycle, capability application | SQL, business rules, shell composition, direct systemd mapping |
| Frontend API | The only `invoke`/event-subscription transport, typed method surface, error normalization | View rendering or backend authorization |
| React | Routing, accessible interaction, rendering, transient state, query orchestration | Raw Tauri calls, filesystem/process/SQL/systemd access, persistent authority |

## Planned repository boundaries

The first implementation should use the following shape. Names may change only through a documentation update that preserves the dependency boundaries.

```text
apps/
  desktop/
    src/                       React application
      api/                     typed API facade and sole Tauri transport
      app/                     shell, router, providers, core routes
      modules/                 centralized frontend registry and lazy modules
      components/              shared accessible primitives/patterns
      styles/                  Tailwind entry and semantic tokens
      generated/               generated, never hand-edited contracts
    src-tauri/                 thin Tauri adapter and capabilities
crates/
  argos-domain/                domain types, invariants, ports
  argos-application/           use cases and backend module registry
  argos-contracts/             cross-boundary contract source of truth
  argos-storage-sqlite/        SQLite repositories and embedded migrations
  argos-platform-linux/        XDG/config, filesystem, opener, controlled process adapter
  argos-systemd/               zbus systemd and bounded journald adapters
tools/
  xtask/                       deterministic repository development commands
docs/                          normative product and implementation design
```

The launcher domain and use cases begin as cohesive modules inside `argos-domain` and `argos-application`; their execution port is implemented by `argos-platform-linux`. They should become a separate crate only if independent compilation or dependency pressure emerges. This avoids both a monolithic adapter and speculative micro-crates.

Expected Rust dependency direction:

```text
argos-domain
  ^       ^
  |       +-- argos-contracts (domain vocabulary only)
  +-- argos-application
          ^       ^       ^
          |       |       +-- argos-systemd
          |       +---------- argos-platform-linux
          +------------------ argos-storage-sqlite

desktop Tauri adapter -> all composition-time crates
```

If contracts need application result shapes, both depend on a small domain type rather than creating a cycle. Adapter crates never depend on Tauri.

## Application composition and lifecycle

At startup the Tauri composition root:

1. determines embedded build information and selects a runtime profile;
2. resolves and validates all paths without deriving them from the current directory;
3. initializes bounded tracing to the state path;
4. loads small bootstrap configuration;
5. opens SQLite, verifies connection settings, backs up when required, and migrates;
6. constructs repositories and Linux/systemd adapters;
7. builds application services and validates the backend module registry;
8. exposes immutable service handles through managed Tauri state;
9. creates the main window with only its declared capabilities.

Failure before full startup produces a minimal safe error/diagnostics view when possible. A migration failure does not open the database for ordinary writes. Closing the only main window terminates Argos and drops subscriptions; there is no hide-on-close behavior.

## Application use cases and transactions

Each public operation is a named use case, for example `ListLauncherItems`, `CreateLauncherItem`, `LaunchSavedItem`, `ListSystemdUnits`, or `GetDiagnostics`. Requests contain an `ActorContext` behind the adapter boundary. Foundation desktop mutations use the human actor; read requests still carry request correlation data but are not persistently audited.

The application layer chooses the transaction boundary for a business mutation. Storage supplies a transaction/session abstraction and repositories that can participate in it. A launcher mutation and its successful/failed audit record are committed atomically when the outcome is database-local. For an external side effect, the use case first commits an `attempted` audit record, performs the operating-system action, then appends its `succeeded` or `failed` outcome with the same correlation ID. If the attempt record cannot be stored, the side effect does not run; this two-record protocol acknowledges that SQLite cannot commit atomically with the operating system.

Cancellation is propagated where the adapter permits it. Frontend query cancellation or route changes must prevent obsolete results from updating views; expensive backend operations should observe a cancellation token when practical.

## Module architecture

### Backend registry

Every compiled backend module contributes one manifest and a service registration function to one application registry. A manifest contains:

- stable module ID;
- display name, description, and module version;
- frontend route key and navigation order default;
- default-enabled flag;
- declared capabilities;
- dependencies and platform requirements;
- current health provider.

The registry merges code-owned defaults with database preferences. It rejects duplicate IDs/routes at startup. Missing dependencies, dependency cycles, or invalid ordering are explicit registry errors; an unavailable platform requirement changes module health rather than silently removing the module.

The effective state distinguishes enablement from health:

```text
enablement: enabled | disabled
health: available | unavailable | degraded | error
```

The UI may present the combined user vocabulary `enabled`, `disabled`, `unavailable`, `degraded`, and `error`. Disabled modules disappear from ordinary navigation but remain configurable in Settings. Enabled unavailable/degraded modules remain navigable and explain their state. A module dependency on a disabled module makes the dependent module unavailable with a structured reason; it does not automatically change the stored preference.

### Frontend registry

One TypeScript registry maps each user-facing module ID to a lazy route loader, icon/presentation adapter, and optional route-level error boundary. Sidebar entries are derived from the backend's effective module list joined to this registry. Core dashboard, Settings, and Diagnostics routes belong to the shell and cannot be disabled in the foundation.

The frontend registry is the only place where a new module route is wired. A CI test compares its module IDs with exported backend user-facing manifests. This unavoidable code-to-component mapping is not duplicated in navigation or routing files.

Modules may use shared application services and contracts. They must not invoke arbitrary command names, import the raw Tauri transport, import another module's private directory, or read persistent data directly. Cross-module behavior requires an application service/contract promoted to a shared boundary.

### Foundation modules

- `systemd`: enabled by default, read-only, with separately reported user/system health.
- `launcher`: enabled by default, persistent CRUD and narrow launch/open behavior.

Dashboard is a lightweight host-aware core route rather than a data-hungry module. It may read the current Linux kernel hostname through one narrow cached core query; it does not aggregate feature data, poll, or become a customizable dashboard. Diagnostics and Settings are core routes.

## Frontend architecture

### State and data flow

React Router owns URL-to-view state. TanStack Query owns asynchronous backend snapshots, invalidation, retries, cancellation, and cache lifetime. Local React state owns dialogs, selections, disclosure, and drafts; narrow context owns theme and shell concerns only. Persistent state is never sourced solely from a component.

React components call semantic methods such as `api.launcher.create(request)` or `api.systemd.listUnits(request)`. Only `api/transport/tauri.ts` imports Tauri invocation/event packages. ESLint enforces that boundary. Backend errors are normalized into the generated `AppError` contract before reaching features.

Queries are enabled when their route is active unless shared shell behavior requires them. Systemd events invalidate scoped queries through the API/event layer with debounce/coalescing; they do not push unlimited unit snapshots. There is no application-wide interval.

### Shell and navigation

The shell provides a persistent sidebar, main landmark/content outlet, page headers, health badges with text/icon labels, Settings and Diagnostics entries, and a reserved future search/command-palette trigger that performs no foundation work. On narrow windows the sidebar may become an accessible disclosure without changing module registration.

Module pages may use internal tabs. Shared patterns cover tiles, lists, tables, forms, dialogs, detail panels, confirmations, empty/loading/error states, and unavailable/degraded states.

### Design tokens and themes

Tailwind CSS v4 consumes CSS custom properties with semantic names rather than raw component colors:

```text
background, surface, surface-raised, surface-hover, border,
text, text-muted, primary, danger, warning, success, focus-ring
```

Each token has foreground pairings where needed. `system` is the default theme preference; `light` and `dark` are explicit. System mode follows `prefers-color-scheme` changes while the application runs. The preference is small bootstrap configuration, not module data.

Accessibility is a release gate: keyboard-operable controls, semantic elements, programmatic labels, visible focus, sufficient contrast, reduced-motion behavior, text/icon status cues, initial focus and focus trapping in modal dialogs, Escape where safe, and focus restoration to the invoking control.

## systemd architecture

`argos-systemd` implements application ports using separate explicit connections/proxies for `user` and `system` scope. No operation retries against the other scope. Connection health distinguishes manager absence, bus absence, permission denial, timeout, malformed response, and internal adapter failure.

The foundation reads manager/unit data through D-Bus:

- list service and timer units;
- loaded, active, sub, and enablement-related file state when exposed;
- descriptions and stable unit names/object references;
- failed states;
- timer last/next trigger information with the source clock semantics mapped into UTC/optional monotonic context;
- bounded unit details and dependency names;
- current jobs when relevant to a detail view.

Raw D-Bus types and object paths stay inside the adapter. The application receives normalized domain models. Unknown future systemd enum strings are represented as `unknown(raw)` or raw supplementary data rather than crashing deserialization.

While a systemd view is active, the adapter may subscribe to manager/unit/job and property-change signals and emit coalesced invalidation events. It unsubscribes when no consumer remains. If a reliable signal is unavailable, the UI provides explicit refresh; any fallback polling must be view-local, justified, no faster than necessary, and disabled when inactive. No foundation design requires polling.

Recent unit logs go through a `JournalReader` port. The initial adapter directly starts `journalctl` with fixed argument construction, JSON output, `--no-pager`, a unit selector appropriate to the explicit scope, and a caller limit clamped to a documented maximum. It never uses a shell and never streams when the log view is inactive.

## Launcher architecture

Launcher manifests and records are separate: the compiled launcher module describes functionality; user items are SQLite data. Use cases list, create, edit, delete, favorite, and launch/open items. Kind-specific domain validation occurs before persistence and again before execution.

Foundation kinds are:

- `url`: an allowed external URL opened by the desktop's external handler;
- `folder`: an absolute directory path opened by the desktop's external handler;
- `executable`: an absolute executable or a simple executable name resolved only through configured search paths, started directly with an argument vector and optional absolute working directory.

No kind accepts a combined command line. Foundation records contain no per-item environment overrides; the structured process request therefore carries an empty override map and uses a centrally defined inherited-environment policy. A future environment editor requires secret/redaction analysis and a schema migration.

Delete is destructive and confirms the exact title/target identity. Launch is an external side effect classified as `write`. The adapter reports accepted/spawned/opened or a typed failure; it does not claim the launched application completed successfully. Argos does not retain unbounded child output or turn launches into a background daemon.

## Diagnostics and logging

Rust structured tracing fields include timestamp, level, component, module, operation, duration, result, correlation ID, and error code where applicable. Production defaults to concise bounded files under the state directory; development may enable more detail. A size-and-file-count rotation policy prevents unbounded retention. In-memory recent failures use a fixed-capacity buffer.

Logs exclude secrets, tokens, full environment maps, arbitrary file contents, complete records, raw database rows, and full helper-process output. Journal content viewed by the user is not re-logged.

Diagnostics report:

- application/version/build and runtime profile;
- resolved config/data/state/cache/runtime paths and runtime availability;
- database connection, journal mode, migration version, and last backup/migration failure;
- user/system systemd connectivity independently;
- effective enabled modules and their health;
- bounded recent Argos failures;
- process and relevant WebView child memory snapshots when supported.

A safe export creates a redacted timestamped JSON report in the state diagnostics directory and returns a narrow result. It contains no launcher targets, logs, environment, tokens, home-directory contents, or database dump. The user may open the containing folder through the same narrow opener.

## Performance model

Foundation engineering targets are cold startup below two seconds on the target machine, effectively zero idle CPU, no process after the window closes, no one-second global polling, no constant idle database writes, and no unbounded memory retention.

Rules:

- open only bootstrap resources at startup; lazy-load route code and module data;
- keep database transactions short and do not refresh merely because time passed;
- subscribe to system events only for active consumers and debounce bursts;
- bound queries, logs, diagnostic buffers, output capture, and Query cache times;
- cancel obsolete requests and release D-Bus/event listeners on route exit;
- measure the host and WebView child process group, not just Rust memory;
- establish cold-start, idle CPU, and extended-idle memory baselines before optimization claims.

## Future-compatible boundaries

Future Argos-managed user units live in the standard user unit directory `${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user/`, which is intentionally outside the Argos XDG namespace because systemd must load it. The Argos database stores metadata/ownership references and persistent backups belong under the Argos data backup area, but marked unit files remain authoritative. Ownership requires a stable in-file marker plus Argos resource UUID; a matching filename or database row alone is insufficient. Creation/replacement must later constrain paths beneath that directory, reject traversal/symlink escape, validate generated units, compare expected content for manual drift, back up the managed file, atomically replace it, and refuse to overwrite unmanaged or manually changed content. System-unit file editing remains separately privileged and out of scope.

Generic future concepts—workspace, project, task, resource, tool, profile, run, and artifact—must enter through new domain/application modules and migrations. Agent, CLI, MCP, socket, or local API adapters supply distinct actors and the same narrow use cases; none receives storage internals or a generic executor.
