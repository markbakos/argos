# Foundation expected-behavior specification

- **Status:** Approved for foundation implementation
- **Covers:** FND-01 through FND-07
- **Purpose:** Test-first behavior contract for foundation implementation

## Use of this specification

This document translates every foundation acceptance criterion into an observable Given/When/Then scenario. It defines the expected result before implementation and is the test oracle for automated checks, static checks, and target-machine verification.

The owning foundation specification remains authoritative. If a scenario here conflicts with its source criterion or a higher-authority document, stop and update the documents before implementation. A scenario is not implementation evidence and does not authorize application work while the [documentation phase gate](README.md#phase-gate) remains closed.

Verification modes used below:

- **Automated** — deterministic Rust, frontend, integration, fixture, or repository test.
- **Static** — dependency, source, generated artifact, capability, or package inspection.
- **Target** — redacted check on the supported Arch Linux/GNOME machine.
- **Measured** — recorded performance or resource observation with environment and duration.

All test execution must use temporary or development data, never production data. systemd verification is read-only. Committed evidence must follow the redaction rules in [Verification](../verification.md).

## FND-01 — Bootstrap and typed boundary

Source: [FND-01 specification](01-bootstrap-and-contracts.md)

### FND-01-AC01 — Reproducible locked build

**Mode:** Automated and Target

- **Given** a clean checkout on the documented target machine with required native prerequisites,
- **When** the committed dependency installation and root build commands run from locked inputs,
- **Then** installation and builds complete without manual file changes or access to production data.

### FND-01-AC02 — Window lifecycle

**Mode:** Automated and Target

- **Given** the foundation desktop is built with its placeholder React/Tailwind interface,
- **When** Argos opens and the user closes its only main window,
- **Then** the placeholder renders in a standard Tauri 2 window and no Argos host, WebView, tray, daemon, or hidden process remains.

### FND-01-AC03 — Reusable headless core

**Mode:** Automated and Static

- **Given** the Cargo workspace and an application use case backed by fake ports,
- **When** dependency-boundary checks and the use-case test run without Tauri or a WebView,
- **Then** domain, application, and contracts have no forbidden Tauri dependency and the use case passes headlessly.

### FND-01-AC04 — Baseline quality gate

**Mode:** Automated

- **Given** a clean working tree with locked toolchains and dependencies,
- **When** the repository quality gate runs strict TypeScript, formatting, ESLint, Clippy, Rust tests, frontend tests, and builds,
- **Then** every required gate passes with no ignored warning or type escape.

### FND-01-AC05 — Deterministic generated contracts

**Mode:** Automated

- **Given** unchanged Rust-owned contract definitions,
- **When** contract generation runs twice,
- **Then** both generated trees are byte-identical;
- **And when** a generated fixture is deliberately made stale,
- **Then** the CI drift check fails with the changed artifact identified.

### FND-01-AC06 — Sole frontend transport

**Mode:** Automated and Static

- **Given** a React component that calls a semantic frontend API method,
- **When** it reaches the narrow typed proof command and receives its result,
- **Then** the round trip succeeds through the API facade and sole Tauri transport;
- **And** static checks find no Tauri invocation or event import outside that transport.

### FND-01-AC07 — Stable safe contract round trips

**Mode:** Automated

- **Given** representative success values and every public error shape, including an unknown internal failure,
- **When** each fixture crosses Rust serialization, JSON, generated TypeScript, and back where applicable,
- **Then** tags, values, details, retryability, and correlation IDs remain stable;
- **And** no Rust debug string or sensitive internal detail is disclosed.

### FND-01-AC08 — Least-privilege capabilities

**Mode:** Static

- **Given** the committed Tauri configuration and capability files,
- **When** the capability inventory is scanned,
- **Then** it contains only approved narrow foundation grants and no broad shell, broad filesystem, or systemd-write permission.

### FND-01-AC09 — Build profile separation

**Mode:** Automated and Static

- **Given** source debug, source optimized, and explicit packaging builds,
- **When** each build's typed build information is inspected without opening runtime data,
- **Then** both source builds report `development` and only the explicit package reports `production`.

## FND-02 — Runtime paths and storage

Source: [FND-02 specification](02-runtime-paths-and-storage.md)

### FND-02-AC01 — Exact runtime path categories

**Mode:** Automated

- **Given** table-driven production, development, test, XDG, and explicit-root inputs,
- **When** the pure path resolver evaluates each case,
- **Then** it returns the exact documented `argos`, `argos-dev`, temporary, or explicit-root category directories;
- **And** no default is derived from the repository, executable path, or current working directory.

### FND-02-AC02 — Non-destructive unsafe-path handling

**Mode:** Automated

- **Given** unsafe or relative roots, missing required HOME, incomplete production acknowledgement, category collisions, or missing runtime state,
- **When** path resolution runs before I/O,
- **Then** each case returns its specified typed error or runtime-unavailable health;
- **And** no directory, database, fallback under `/tmp`, or production path is opened or created.

### FND-02-AC03 — Repository remains free of live data

**Mode:** Automated and Static

- **Given** a repository filesystem snapshot and isolated development/test roots,
- **When** initialization, CRUD, logging, backup, migration, and diagnostics operations run,
- **Then** a second snapshot contains no database, sidecar, log, backup, config, cache, state, diagnostic, or other live-data pattern under the repository.

### FND-02-AC04 — Profile and test isolation

**Mode:** Automated

- **Given** resolved production, development, and fresh test roots,
- **When** path overlap and access probes run for normal source and test builds,
- **Then** production and development never overlap, tests read and write neither, and a normal source build cannot open production.

### FND-02-AC05 — Fresh SQLite initialization

**Mode:** Automated

- **Given** an empty isolated data directory,
- **When** storage initializes for the first time,
- **Then** the expected migration metadata, tables, constraints, indexes, and foreign keys exist;
- **And** the effective busy timeout is 5000 ms and normal journal mode is WAL.

### FND-02-AC06 — Recoverable migration

**Mode:** Automated

- **Given** a valid prior-version database,
- **When** an upgrade succeeds,
- **Then** a consistent verified backup is created before the transactional migration and the upgraded database opens;
- **But when** migration failure is injected,
- **Then** the transaction rolls back, original and backup data remain, and ordinary writes stay blocked.

### FND-02-AC07 — Verified automatic backup retention

**Mode:** Automated

- **Given** automatic backup retention of five files plus manual or unrecognized files,
- **When** six successful verified automatic backups are created,
- **Then** only the oldest recognized automatic backup is removed after the newest verifies;
- **And** manual, unrecognized, original, and newest backup files remain.

### FND-02-AC08 — Repository and transaction behavior

**Mode:** Automated

- **Given** isolated SQLite repositories and valid, invalid, conflicting, and bounded inputs,
- **When** CRUD, pagination, JSON bounds, foreign-key, integrity, revision-conflict, and audited mutation cases run,
- **Then** valid domain values round-trip, invalid/conflicting work fails without partial state, and each database mutation and audit event commits or rolls back atomically.

### FND-02-AC09 — Disposable cache

**Mode:** Automated and Target

- **Given** persisted launcher, module preference, and audit records plus disposable cache data,
- **When** the cache is deleted and Argos restarts,
- **Then** all source-of-truth records remain unchanged and required cache state is recreated safely.

### FND-02-AC10 — No generic storage authority

**Mode:** Static

- **Given** frontend contracts, Tauri commands, capabilities, and source boundaries,
- **When** their exposed storage surface is inspected,
- **Then** no frontend or Tauri operation returns a database path or permits generic SQL, filesystem, backup, or migration work.

## FND-03 — Application shell and module registry

Source: [FND-03 specification](03-shell-and-modules.md)

### FND-03-AC01 — Resilient shell

**Mode:** Automated and Target

- **Given** module health and data providers that are loading, unavailable, or failing,
- **When** the application shell renders,
- **Then** its sidebar, main landmark, Dashboard, Settings, Diagnostics, and routing remain present and usable without waiting for feature data.

### FND-03-AC02 — Theme behavior and accessibility

**Mode:** Automated and Target

- **Given** system, light, and dark theme choices and a mutable OS color preference,
- **When** the user changes theme or the OS preference changes while system mode is active,
- **Then** the effective theme updates and persists across restart;
- **And** both visual themes pass contrast and reduced-motion checks.

### FND-03-AC03 — Validated deterministic backend registry

**Mode:** Automated

- **Given** registries containing valid modules, duplicate IDs/routes, missing dependencies, cycles, and preference orders,
- **When** registry validation and effective ordering run,
- **Then** invalid graphs fail explicitly and valid graphs produce one deterministic effective order.

### FND-03-AC04 — Single lazy frontend registry

**Mode:** Automated and Static

- **Given** matching backend manifests and frontend route entries,
- **When** parity, routing, and lazy-load instrumentation tests run,
- **Then** navigation and routes derive from the single frontend registry, IDs match the backend, and inactive feature bundles are not eagerly loaded.

### FND-03-AC05 — Persistent enablement without data deletion

**Mode:** Automated and Target

- **Given** an enabled module with persisted feature data,
- **When** the user disables it, restarts Argos, and later re-enables it,
- **Then** it leaves ordinary navigation while disabled, remains configurable, returns after re-enable, and its feature data is unchanged.

### FND-03-AC06 — Honest unhealthy-module states

**Mode:** Automated and Target

- **Given** enabled modules in unavailable, degraded, and error health states,
- **When** navigation and each module route render,
- **Then** every module remains reachable and shows a text/icon label, reason, and actionable state that does not rely on color alone.

### FND-03-AC07 — Atomic preference updates

**Mode:** Automated

- **Given** a valid module enablement or ordering change,
- **When** storage and audit succeed,
- **Then** the preference and audit event persist atomically and the UI confirms the result;
- **But when** storage fails,
- **Then** neither persists and the UI remains read-only/degraded without showing false success.

### FND-03-AC08 — Keyboard-complete core interactions

**Mode:** Automated and Target

- **Given** a keyboard-only user operating sidebar links, forms, dialogs, exact confirmations, and error recovery,
- **When** the user completes and dismisses those interactions,
- **Then** all controls are reachable and operable, focus is visible and trapped where required, and closing a dialog restores focus to its invoker.

### FND-03-AC09 — Inactive modules release resources

**Mode:** Automated and Measured

- **Given** a lazy feature route with query, timer, and event-subscription instrumentation,
- **When** the route has never opened or the user navigates away,
- **Then** it performs no feature fetch or interval and retains no feature subscription.

### FND-03-AC10 — Two registry extension points

**Mode:** Static and Automated

- **Given** a test module with its own feature files,
- **When** it is registered and parity/navigation tests run,
- **Then** integration requires changes only to the centralized backend registry, centralized frontend registry, and that module's own files.

### FND-03-AC11 — Host-aware Dashboard

**Mode:** Automated and Target

- **Given** a valid current Linux kernel hostname,
- **When** the user opens Dashboard,
- **Then** the hostname is read through the narrow typed core boundary and presented as the page's primary identity without shortcuts, onboarding, fake metrics, feature aggregation, polling, persistence, logging, audit, events, or export;
- **And** opening Settings or Diagnostics directly does not request hostname.
- **Given** hostname loading, invalid data, or a platform read failure,
- **When** Dashboard renders,
- **Then** the shell remains usable and the page shows a quiet loading or `Hostname unavailable` state without exposing raw errors or guessing an identity.

## FND-04 — Read-only systemd module

Source: [FND-04 specification](04-systemd-read.md)

### FND-04-AC01 — Normalized unit reads

**Mode:** Automated

- **Given** service and timer fixtures containing normal, failed, missing, and unknown future values, details, dependencies, and jobs,
- **When** adapter mapping and application read use cases run,
- **Then** they return bounded normalized domain values without leaking protocol types or failing the whole response on unknown values.

### FND-04-AC02 — Explicit independent scope

**Mode:** Automated and Target

- **Given** independent user and system manager readers,
- **When** an operation is requested without scope or one scope fails,
- **Then** the request without scope is rejected and the failed scope is never retried against the other;
- **And** the UI defaults visibly to user scope.

### FND-04-AC03 — Timer trigger semantics

**Mode:** Automated

- **Given** scheduled, inactive, unknown, and missing previous/next timer-trigger fixtures with realtime or monotonic context,
- **When** timer details are mapped,
- **Then** each state remains distinguishable and every realtime contract value is normalized to UTC without invented precision.

### FND-04-AC04 — Independent scoped health

**Mode:** Automated and Target

- **Given** combinations of healthy, missing bus, missing manager, permission-denied, timeout, disconnected, and malformed responses for user and system scopes,
- **When** module health is aggregated,
- **Then** each cause and scope remains distinct and a partial scope failure degrades rather than erases the working scope.

### FND-04-AC05 — Bounded direct journal reads

**Mode:** Automated and Static

- **Given** valid and invalid units, requested limits below/within/above bounds, JSON fixtures, cancellation, timeout, oversized output, and nonzero exits,
- **When** the journal adapter builds and runs its command plan,
- **Then** it uses direct literal argv with the explicit scope selector, clamps results to 1–500, bounds time/output, parses safe fields, and maps each failure without a shell.

### FND-04-AC06 — Active-only invalidation

**Mode:** Automated and Measured

- **Given** an active systemd page and a burst of scoped change signals,
- **When** signals arrive and the page later tears down,
- **Then** authoritative queries receive bounded debounced invalidation and all subscribers are released;
- **But when** subscription fails,
- **Then** manual refresh remains available and no global polling begins.

### FND-04-AC07 — No systemd mutation authority

**Mode:** Static

- **Given** compiled commands, contracts, capability files, adapters, and process-call sites,
- **When** systemd authority is scanned,
- **Then** no mutation method, unit-file write, shell command, polkit flow, or user/system write permission exists.

### FND-04-AC08 — Honest target-system read smoke

**Mode:** Target

- **Given** a target Arch/GNOME session and a human-selected known user timer,
- **When** Argos connects to the user manager, reads its timer/details and bounded recent logs, and probes system scope,
- **Then** authorized reads succeed and unavailable or permission-limited system/journal access is reported explicitly without elevation or mutation.

### FND-04-AC09 — Accessible scoped states

**Mode:** Automated and Target

- **Given** loading, empty, error, unavailable, and degraded systemd responses in both scopes,
- **When** a keyboard-only user navigates list, detail, log, refresh, and scope controls,
- **Then** every state and active scope is labeled, operable, and understandable without color alone.

## FND-05 — Minimal launcher module

Source: [FND-05 specification](05-launcher.md)

### FND-05-AC01 — Durable bounded launcher CRUD

**Mode:** Automated

- **Given** valid URL, folder, and executable launcher requests across multiple pages,
- **When** create, get, update, favorite, list, and delete use cases run,
- **Then** exact structured fields persist with UUID, normalized UTC timestamps, increasing revisions, favorites-first stable order, and bounded pagination.

### FND-05-AC02 — Kind-specific validation

**Mode:** Automated

- **Given** valid and invalid fields for every launcher kind, including URL schemes/credentials, paths, executable basenames, NULs, arguments, and incompatible fields,
- **When** requests and loaded records are validated,
- **Then** only values satisfying the documented kind and size rules proceed and failures return field-specific typed details before persistence or execution.

### FND-05-AC03 — Optimistic conflict and exact deletion

**Mode:** Automated and Target

- **Given** a launcher item whose persisted revision is newer than the caller's revision,
- **When** update, favorite, or delete is attempted,
- **Then** the operation returns a conflict and does not overwrite or delete the record;
- **And** a valid delete can proceed only after confirmation identifies the exact title and kind.

### FND-05-AC04 — Atomic and correlated audit

**Mode:** Automated

- **Given** database mutations and external launcher actions with injected audit failures,
- **When** each operation runs,
- **Then** database mutations and redacted audits commit or roll back atomically;
- **And** an external action never runs without a committed attempted record, appends a correlated outcome, and reports a typed partial failure if only the post-action audit fails.

### FND-05-AC05 — Literal process plans

**Mode:** Automated and Static

- **Given** executable targets and arguments containing spaces and shell metacharacters,
- **When** a structured execution plan is derived from the saved record,
- **Then** executable, arguments, and optional working directory remain separate literal values, basename resolution uses only fixed validated search paths, environment overrides are empty, and no shell or terminal is involved.

### FND-05-AC06 — Typed target outcomes

**Mode:** Automated and Target

- **Given** missing, wrong-type, permission-denied, not-executable, opener-failure, and spawn-failure URL/folder/executable targets,
- **When** the saved item is launched,
- **Then** the exact typed failure is returned and the item remains editable;
- **But when** the opener or process accepts the request,
- **Then** the outcome claims only `opened` or `spawned`, not later child success.

### FND-05-AC07 — Accessible launcher states

**Mode:** Automated and Target

- **Given** launcher tile/list loading, empty, error, conflict, and delete-confirmation states,
- **When** a keyboard-only user navigates and operates the view,
- **Then** actions, kind labels, states, confirmation, focus movement, and recovery are accessible without icon or color-only meaning.

### FND-05-AC08 — Safe target smoke and redaction

**Mode:** Target and Static

- **Given** human-selected harmless saved HTTPS, temporary folder, and executable targets with literal test arguments,
- **When** each target is opened or spawned and resulting logs, audits, and safe exports are inspected,
- **Then** the external action is accepted as expected and no target, URL query, arguments, working directory, environment, or helper output is retained in those artifacts.

### FND-05-AC09 — Persistence outside source

**Mode:** Target

- **Given** a saved launcher item in development data outside the repository,
- **When** Argos closes and reopens after rebuild, harmless branch/source relocation, and cache deletion,
- **Then** the item remains intact and no launcher database or sidecar appears under the source tree.

### FND-05-AC10 — No arbitrary execution surface

**Mode:** Static

- **Given** launcher contracts, Tauri commands, capabilities, frontend calls, and remote-window grants,
- **When** their authority is inventoried,
- **Then** launch accepts only a saved item ID and no arbitrary target, generic process/file/shell operation, or remote-content launcher privilege exists.

## FND-06 — Diagnostics and observability

Source: [FND-06 specification](06-diagnostics.md)

### FND-06-AC01 — Bounded structured traces

**Mode:** Automated

- **Given** tracing events and forced small rotation thresholds equivalent to the production policy,
- **When** enough safe events are emitted to rotate repeatedly,
- **Then** each record contains the applicable required fields and retention never exceeds five files of 5 MiB under production-equivalent limits.

### FND-06-AC02 — Bounded recent failures

**Mode:** Automated

- **Given** more than 50 safe Argos error summaries plus a sink/rotation failure,
- **When** the recent-failure buffer records them,
- **Then** it retains only the newest 50, evicts oldest entries deterministically, and does not recursively amplify logging failures.

### FND-06-AC03 — Complete safe diagnostic snapshot

**Mode:** Automated and Target

- **Given** available and unavailable build, path, storage, migration, systemd, module, journal, recent-failure, and process-memory providers,
- **When** Diagnostics is refreshed on demand,
- **Then** every required section renders its independent safe state, including explicit unsupported memory measurement where applicable, without exposing database content or ordinary feature data.

### FND-06-AC04 — Bounded partial-provider failure

**Mode:** Automated

- **Given** each diagnostics provider failing or timing out one at a time,
- **When** the aggregate snapshot is requested,
- **Then** the affected section reports a bounded error with correlation while every other section completes and remains usable within its deadline.

### FND-06-AC05 — Safe application-owned export

**Mode:** Automated and Target

- **Given** healthy storage and separately unavailable database audit storage,
- **When** the user requests export,
- **Then** a versioned private report is atomically created under the state diagnostics directory and its containing folder opens only through the narrow action;
- **And** database failure still permits export while explicitly marking persistent audit unavailable.

### FND-06-AC06 — Redaction boundary

**Mode:** Automated and Static

- **Given** a forbidden-value corpus containing launcher targets/arguments, journal text, environments, tokens, database details, arbitrary content, and command output,
- **When** errors, tracing, UI messages, recent failures, audits where applicable, and safe exports are produced,
- **Then** none of the forbidden values appear and only allowlisted safe fields remain.

### FND-06-AC07 — On-demand resource use

**Mode:** Automated and Measured

- **Given** Diagnostics opened, refreshed, and then left inactive,
- **When** timers, writes, queries, and measurement handles are observed,
- **Then** refresh work is bounded, every measurement resource is released, and no global polling, heartbeat, background sampler, or continual write remains.

### FND-06-AC08 — Visible profile and category truth

**Mode:** Automated and Target

- **Given** normal development, packaged production, and explicitly acknowledged development-to-production override cases,
- **When** Settings and Diagnostics display build/profile/path state,
- **Then** each case shows the correct resolved category directories and the override case carries a persistent prominent warning.

### FND-06-AC09 — Accessible diagnostic interactions

**Mode:** Automated and Target

- **Given** healthy, loading, partial, unsupported, and failed diagnostic sections plus export results,
- **When** a keyboard-only user refreshes, inspects, exports, and opens the report folder,
- **Then** controls and states are labeled, focus-visible, operable, and understandable without color alone.

## FND-07 — Integration and release verification

Source: [FND-07 specification](07-integration-verification.md)

### FND-07-AC01 — Clean full gate

**Mode:** Automated and Target

- **Given** a clean checkout with committed lockfiles on CI and the target Arch machine,
- **When** documented install, build, test, and full check commands run,
- **Then** every deterministic foundation gate passes reproducibly.

### FND-07-AC02 — Complete GNOME lifecycle smoke

**Mode:** Target

- **Given** the development-profile app on the target GNOME desktop,
- **When** the verifier opens the window, uses shell, themes, navigation, Settings, and Diagnostics, then closes it,
- **Then** the workflows operate and the complete Argos process group exits with no tray or daemon.

### FND-07-AC03 — End-to-end data isolation

**Mode:** Automated, Static, and Target

- **Given** repository snapshots and production, development, test, explicit-root, and incomplete-acknowledgement cases,
- **When** foundation runtime workflows execute against permitted isolated roots,
- **Then** no live file appears in the repository, categories never overlap, tests avoid user data, and incomplete production acknowledgement is rejected before I/O.

### FND-07-AC04 — Storage lifecycle evidence

**Mode:** Automated and Target

- **Given** fresh, prior-version, and injected-failure databases plus persisted module and launcher records,
- **When** initialization, migration, backup, recovery, restart, rebuild, source movement, and cache deletion cases run,
- **Then** schema and recovery invariants hold and user records persist without production or repository access.

### FND-07-AC05 — Generated and architectural boundaries

**Mode:** Automated and Static

- **Given** contract generation, both module registries, lazy routes, Cargo boundaries, and the Tauri transport,
- **When** drift, parity, lazy-load, dependency, and headless-core tests run,
- **Then** generated output matches Rust, registries agree, inactive modules stay unloaded, and core behavior remains independent of Tauri.

### FND-07-AC06 — Real read-only systemd behavior

**Mode:** Target and Static

- **Given** human-selected user/system services and timers on the target machine,
- **When** list, detail, trigger, and bounded-log checks run in each authorized scope,
- **Then** reads return normalized results or honest permission/unavailable states and the inventory proves no systemd mutation occurred or was exposed.

### FND-07-AC07 — Real launcher safety

**Mode:** Target and Static

- **Given** harmless saved URL, folder, and executable items with arguments containing spaces and metacharacters,
- **When** the verifier opens or spawns them and inspects command plans, audits, logs, and exports,
- **Then** only saved targets execute with literal arguments, no shell is used, and sensitive target data is absent from retained artifacts.

### FND-07-AC08 — Complete authority inventory

**Mode:** Static

- **Given** built commands, events, plugins, windows, capability grants, contracts, and frontend API methods,
- **When** the foundation authority inventory is compared with Security and the approved specifications,
- **Then** every surface has an owner and no generic SQL, file, process, shell, systemd-write, or undeclared remote-window authority exists.

### FND-07-AC09 — Complete interaction-state accessibility

**Mode:** Automated and Target

- **Given** loading, empty, error, unavailable, degraded, conflict, and confirmation states across core and feature workflows,
- **When** keyboard, focus, contrast, status-label, theme, and reduced-motion checks run,
- **Then** every core workflow remains operable without pointer or color-only understanding and dialog focus restores correctly.

### FND-07-AC10 — Startup, idle, bounds, and termination

**Mode:** Measured and Target

- **Given** a documented optimized-development build, target hardware, enabled modules, and representative data,
- **When** cold startup, ten-minute idle CPU/write, bounded-result/log, active/inactive subscription, and close measurements run,
- **Then** cold startup is below two seconds, idle CPU is effectively zero, no one-second global polling or continual database write occurs, bounds hold, and closing terminates the process group.

### FND-07-AC11 — Extended-idle memory stability

**Mode:** Measured and Target

- **Given** the full Argos host and WebView process group after startup stabilization,
- **When** resident/proportional memory is sampled at start and for at least two hours idle,
- **Then** there is no unexplained sustained growth;
- **And** any observed growth retains its measurements and investigation rather than being silently accepted.

### FND-07-AC12 — Safe useful diagnostics

**Mode:** Target and Static

- **Given** healthy and partially failed subsystems plus generated local logs and a safe diagnostics report,
- **When** the verifier inspects their allowlisted content, bounds, and redaction,
- **Then** the artifacts remain useful for correlation and partial-health diagnosis without forbidden personal or system data.

### FND-07-AC13 — Artifact and profile separation

**Mode:** Automated, Static, and Target

- **Given** an ordinary source optimized build and an explicit packaged artifact,
- **When** their embedded profile metadata and package contents are inspected,
- **Then** the source build reports `development`, the package reports `production`, and neither package nor build artifact contains user data.

### FND-07-AC14 — Final traceability and consistency

**Mode:** Static and Target

- **Given** all 71 foundation acceptance criteria, 41 tasks, definition-of-done rows, and collected redacted evidence,
- **When** the final traceability and documentation consistency review runs,
- **Then** every item maps to passing evidence and no unresolved requirement, implementation, security, path, contract, or documentation contradiction remains before F1 approval.

## Coverage summary

| Specification | Scenarios |
| --- | ---: |
| FND-01 | 9 |
| FND-02 | 10 |
| FND-03 | 10 |
| FND-04 | 9 |
| FND-05 | 10 |
| FND-06 | 9 |
| FND-07 | 14 |
| **Total** | **71** |

Implementation tests and verification records must cite these existing acceptance IDs. Do not create a second ID namespace for the same behavior.
