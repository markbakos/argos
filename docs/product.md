# Product definition

## Vision

Argos is a personal, local-first control center for a systemd-based Linux desktop. It gives one user a coherent way to inspect system state, launch tools and resources, manage personal automation, and eventually coordinate projects and coding agents without surrendering data ownership or broad operating-system access to a WebView.

The product should feel like a normal GNOME desktop application: available when opened, absent when closed, useful offline, and understandable by one maintainer. Growth comes from compiled, bounded modules and shared application services rather than a monolith or unrestricted plugin system.

## Target user and environment

The initial user is the owner of one local Arch Linux workstation running GNOME and systemd. Argos runs with that user's normal privileges. A reachable user systemd manager is expected for the systemd module; read access to the system manager and journal may vary with local policy.

There is no initial Windows support, multi-user collaboration, remote client, hosted backend, account, telemetry, mandatory daemon, tray process, or cloud dependency.

## Binding product principles

### Local-first ownership

Core behavior works without a network connection. The user owns configuration, records, audit history, and exported data. Argos has no account requirement, remote API dependency, hosted database, or telemetry.

### Repository and live-data separation

The source repository contains project assets only: source, documentation, migrations, build configuration, development scripts, static assets, and test fixtures. Live databases, preferences, launcher records, logs, audit history, icons, backups, runtime files, development data, and future agent state reside in XDG or explicit isolated roots outside the repository.

Rebuilding, switching branches, moving, deleting, or recloning the repository must not mutate or remove user data.

### Modular by construction

Modules integrate through centralized backend and frontend registries, contracts, and shared services. A module does not edit unrelated navigation, inspect another module's private state, bypass authorization, access storage directly from React, or put business logic in the Tauri entry point.

### Least authority

Argos never runs wholly as root. Rust owns trusted I/O and policy. React receives only typed, narrow operations. Privileged and destructive work is explicit, target-specific, authorized, and auditable. Future agents are separate identities and never gain generic command, file, SQL, or database access.

### Lightweight lifecycle

The main window is the application lifecycle in the foundation: closing it exits Argos. There is no tray or required daemon. Modules and their data are loaded on demand; event subscriptions are preferred to polling; logs and memory are bounded; idle operation causes no continual database writes.

## Foundation milestone

### Outcome

The foundation proves the architecture with a usable shell, a read-only systemd inspector, a persistent minimal launcher, settings, and diagnostics. It is a vertical architectural proof rather than the full product.

The foundation includes:

- a normal Tauri 2 desktop window with a React, strict TypeScript, Vite, and Tailwind CSS v4 interface;
- a reusable Rust core isolated from Tauri;
- generated TypeScript contracts and a single frontend API boundary;
- XDG-compliant production and development profiles plus temporary test roots;
- Rust-owned SQLite migrations and repositories;
- centralized compiled-in module registries and lazy frontend routes;
- a lightweight host-aware dashboard, sidebar navigation, settings, diagnostics, and theme preferences;
- read-only user- and system-scope service and timer inspection, bounded recent logs, and honest health states;
- launcher CRUD, favorites, URL/folder opening, and structured executable spawning;
- structured logging, safe diagnostics, automated quality gates, and target-machine verification.

### Explicit exclusions

The foundation does not implement systemd mutations, timer creation, unit-file management, full workspace execution, tasks or CRM, agents, MCP, Docker or n8n management, remote access, runtime third-party plugins, embedded privileged remote content, tray behavior, autostart, a daemon, a dashboard builder, or a terminal emulator.

Interfaces may reserve stable concepts for these areas, but foundation implementations must not simulate or partially expose excluded behavior.

## Long-term direction

After the foundation, Argos may add personal automations, Argos-managed user services and timers, workspaces, developer-tool and self-hosted-service modules, generic projects/tasks/resources, and controlled CLI, MCP, Unix-socket, or local API surfaces. These interfaces must reuse application use cases and identify actors independently from the desktop user.

A future Codex profile is configuration for a generic tool; it is not a hard-coded account slot. Future self-hosted dashboards open externally by default. A future embedded remote-content window has no system-control capabilities.

## Non-goals

Argos is not:

- a general-purpose terminal, shell wrapper, SQL console, or file manager;
- a replacement for systemd, journald, polkit, or Linux ownership and permission models;
- a full observability platform or continuous system monitor;
- a cloud collaboration service or remote administration panel;
- a web-app container with privileged access;
- an unrestricted third-party plugin host;
- a Notion clone or a product centered on one agent vendor;
- a process that must remain resident after its window closes.

## Terminology

| Term | Meaning |
| --- | --- |
| Argos | The product and its production storage namespace. |
| Foundation | The first implementation milestone described above. |
| Core | Rust domain and application layers that do not depend on Tauri. |
| Adapter | A boundary implementation for Tauri, SQLite, D-Bus, filesystem, journald, or processes. |
| Module | A compiled-in functional slice registered through defined backend and frontend extension points. |
| Manifest | Code-owned module identity, presentation metadata, requirements, and defaults. |
| Module preference | A user-owned database override for enablement, order, or bounded module settings. |
| Scope | The explicitly selected `user` or `system` systemd manager. |
| Runtime profile | `production`, `development`, or `test`, controlling storage namespace and safety rules. |
| Explicit root | A development/test-only absolute `ARGOS_HOME` root containing isolated path categories. |
| Contract | A Rust-owned cross-boundary request, response, event, identifier, or error type exported to TypeScript. |
| Actor/initiator | The human, CLI, agent, or automation identity responsible for an action. |
| Managed unit | A future user unit whose file contains an Argos ownership marker and stable resource ID. Filename alone never establishes ownership. |
| Health | A structured subsystem or module condition: available, unavailable, degraded, or error; disabled is an enablement state. |

## Milestones

| Milestone | Purpose | Exit condition |
| --- | --- | --- |
| D0 — Documentation | Lock the product, boundaries, specifications, tasks, and verification before code. | Documentation is reviewed, internally consistent, and explicitly approved. |
| F1 — Foundation | Prove the full local architecture with shell, systemd reads, launcher persistence/execution, and diagnostics. | Every item in the foundation definition of done and verification matrix passes. |
| F2 — Safe automation | Design and add Argos-managed user units and reversible user-scope actions. | A new approved specification covers ownership, validation, backups, confirmations, audit, and recovery. |
| F3 — Extensible workflows | Add selected workspace, tool, service, project, or agent-facing capabilities through the same core. | Each capability has its own approved specification and does not weaken foundation boundaries. |

F2 and F3 are directional, not commitments to scope or sequence. They cannot be used to expand F1.
