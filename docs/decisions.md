# Architectural decision register

## Record format

Each record has a stable ID, status, decision, and consequences. All entries below are **Accepted** for the foundation. A change requires the entry to be marked superseded, a replacement decision to be added, and dependent specifications to be updated before implementation diverges.

## ADR-001 — Tauri 2 desktop lifecycle

**Decision.** Argos is a Linux-first Tauri 2 desktop application with one standard GNOME-compatible main window. Closing that window exits the foundation application. There is no tray, autostart component, required daemon, or background lifecycle.

**Consequences.** Tauri supplies window/WebView integration and capability enforcement, not business logic. Work that requires persistence after exit is outside the foundation.

## ADR-002 — React, strict TypeScript, Vite, and Tailwind CSS v4

**Decision.** The frontend uses React, strict TypeScript, Vite, React Router, TanStack Query, Tailwind CSS v4 through Vite, semantic design tokens, and lightweight icons/headless primitives. Redux and large component frameworks are excluded without a demonstrated later need.

**Consequences.** Server/backend state belongs in Query; transient view state remains local or narrowly contextual. Route modules are lazy, themes include system/light/dark, and raw palette colors do not become component-level semantics.

## ADR-003 — Reusable Rust core and thin Tauri adapter

**Decision.** Domain and application crates do not depend on Tauri. Tauri initializes services, owns shared state, registers narrow commands/events, applies capabilities, and translates contracts/errors only.

**Consequences.** Business rules are testable without a WebView and can later serve a CLI, MCP server, Unix socket, controlled local service, or integration tests. Tauri handlers remain short and contain no SQL, D-Bus mapping, process policy, or domain decisions.

## ADR-004 — Rust owns trusted operations

**Decision.** Rust exclusively owns database, filesystem, process, systemd, validation, permission checks, and mutation audit operations. React never receives generic SQL, filesystem, shell, or backend invocation access.

**Consequences.** All WebView-to-core interactions use narrow typed application operations. Authorization is enforced behind the frontend boundary even when the UI has already validated or confirmed an action.

## ADR-005 — One Rust-owned SQLite database

**Decision.** Foundation records use one primary SQLite database accessed only through Rust repositories and application use cases. Migrations are embedded, ordered, and transactional; foreign keys, a busy timeout, and WAL are enabled in normal profiles.

**Consequences.** Use cases own business transaction boundaries. There is no database path or raw query contract. Public persisted identities are UUIDs and timestamps are normalized UTC. Destructive automatic downgrades are forbidden.

## ADR-006 — XDG storage and repository separation

**Decision.** Production uses the `argos` XDG namespace, development uses `argos-dev`, and tests use temporary isolated directories. Live data never belongs to or defaults into the source repository. An absolute `ARGOS_HOME` may override development/test roots only under explicit safety rules.

**Consequences.** A repository rebuild, branch change, relocation, or deletion cannot affect live data. Path resolution is a core service tested independently of Tauri. Missing `XDG_RUNTIME_DIR` never silently falls back to `/tmp`.

## ADR-007 — Compiled-in modules with centralized registries

**Decision.** Foundation modules are compiled into the application. Backend manifests and services use one backend registry; frontend lazy route/presentation adapters use one frontend registry. Runtime libraries, downloaded JavaScript, marketplaces, and code injection are excluded.

**Consequences.** Navigation derives from effective registered modules rather than scattered edits. Startup validation detects duplicate IDs and invalid dependencies. Modules communicate through contracts/application services, not private state or direct I/O.

## ADR-008 — systemd D-Bus is primary and scope is explicit

**Decision.** Rust uses `zbus` and systemd's D-Bus API for manager, unit, service, timer, job, property, and change-event integration. Every operation carries `user` or `system` scope; the UI defaults to user and never falls back silently.

**Consequences.** Normal human-readable `systemctl` output is not an architectural dependency. Scope and permission/unavailability errors remain distinct. The foundation exposes reads only.

## ADR-009 — Bounded journald adapter

**Decision.** Initial recent logs may use a replaceable Rust journald adapter that invokes `journalctl` directly with argument arrays, machine-readable output, an explicit limit, and no shell. Streaming exists only while an active future view needs it; foundation retrieval is bounded snapshots.

**Consequences.** Parsing and command failures are isolated from the systemd domain. Log access can honestly report permissions and availability. Command output and personal log contents are not copied into normal Argos logs.

## ADR-010 — No generic shell or filesystem operation

**Decision.** Argos exposes purpose-specific operations. Executables use structured executable, argument array, working directory, and explicit environment-override data; URL and folder openers are narrow adapters. Foundation has no shell launcher.

**Consequences.** Fields are never concatenated into a command line. Executable resolution, target validation, audit redaction, and errors are centrally enforced. Any future shell type needs a separate security decision and visual/authorization model.

## ADR-011 — Rust-generated TypeScript contracts

**Decision.** Rust contract definitions are the source of truth for cross-boundary IDs, requests, responses, events, errors, classifications, pagination, availability, and health. A deterministic repository command regenerates committed TypeScript output, and CI rejects drift.

**Consequences.** Generated files are read-only. React components call a typed frontend API, never Tauri directly. The exact compatible generator crate is selected during bootstrap without changing this contract architecture.

## ADR-012 — Explicit action classification and append-only audit

**Decision.** Operations are classified `read`, `write`, `privileged`, or `destructive`. Privileged flows are explicit; destructive flows confirm the exact target. Normal mutation paths append structured audit events with separate actor identity and redacted, bounded metadata.

**Consequences.** Read operations do not create persistent audit noise. Audit insertion shares a transaction with database mutations when possible. Future agents cannot inherit the human identity or bypass approvals.

## ADR-013 — Tauri capabilities separated by concern

**Decision.** Tauri capabilities and custom-command permissions are narrow and grouped by core window, read-only core, launcher read/write/execute, user-systemd read/write, and system-systemd read/write concerns. Only foundation-required read and launcher permissions exist initially.

**Consequences.** A future secondary or remote-content window receives only its explicit subset; embedded remote content receives no system-control permissions. Registered Rust functions alone do not imply exposure to every window.

## ADR-014 — Local-first diagnostics and structured tracing

**Decision.** Rust structured tracing is the primary diagnostic source. Bounded state-directory logs, subsystem health, resolved paths, migration state, enabled modules, versions, and recent failures feed diagnostics. Diagnostic export is narrow and redacted.

**Consequences.** Frontend console messages are not operational records. Secrets, environment maps, arbitrary content, and full command output are excluded. Retention and in-memory buffers are bounded.

## ADR-015 — Future interfaces reuse application use cases

**Decision.** Future CLI, MCP, Unix-socket, local API, automation, and agent actors call the same Rust application services as the desktop adapter. They do not edit SQLite or acquire general filesystem/process access.

**Consequences.** Actor identity is part of mutation context. Tool profiles are generic; Codex is configuration of a tool rather than a fixed account model. Adding an interface requires an authentication/authorization specification without relocating domain logic.

## ADR-016 — Foundation systemd is read-only

**Decision.** Foundation systemd capabilities cover connectivity, services, timers, details, failed state, trigger information, bounded logs, and practical event invalidation only. No enable, start, stop, restart, mask, timer creation, or unit-file write operation exists.

**Consequences.** Write permission definitions may be named in the conceptual capability model but are not granted or implemented. Mutations begin only after a separate approved safety design.
