# FND-01 — Bootstrap and typed boundary

**Status:** Approved for foundation implementation  
**Depends on:** Accepted architecture decisions  
**Enables:** Every other foundation specification

## Problem and user value

Argos needs a reproducible desktop/tooling baseline whose structure makes unsafe shortcuts difficult. Without it, UI code can accrete Tauri calls, Rust logic can collapse into handlers, generated contracts can drift, and quality gates can disagree.

The user gains a foundation that builds consistently, fails with stable errors, and can grow without making the WebView trusted or Tauri inseparable from application behavior.

## Workflows

1. A developer installs locked dependencies, runs one root development command, and gets a development-profile desktop window.
2. A Rust contract author changes a request/response/error, regenerates TypeScript deterministically, and frontend type checking identifies all consumers.
3. CI reproduces formatting, lint, typecheck, tests, generation, and architecture-boundary checks from committed lockfiles.
4. A use-case test constructs the Rust application core without starting Tauri or a WebView.
5. Closing the main window terminates the application rather than hiding it.

## Functional requirements

- Create a Cargo workspace with the crate boundaries in [Architecture](../architecture.md#planned-repository-boundaries) and no cyclic dependencies.
- Create one pnpm workspace containing the Tauri desktop's strict React/TypeScript/Vite frontend.
- Integrate Tauri 2, React, Vite, Tailwind CSS v4 through Vite, React Router, TanStack Query, the selected lightweight icon set, and test tooling with committed lockfiles.
- Make ordinary source development builds embed the `development` runtime profile; only the explicit packaging command embeds `production`.
- Keep Tauri as a composition/transport crate. Prove an application use case can run through a fake port with no Tauri dependency.
- Define typed internal errors and the public `AppError` contract with stable namespaced code, safe message, optional structured details, retryability, and correlation ID.
- Define Rust-owned baseline contracts for build/profile info, health/availability, module IDs/states, action classes, paging, and events.
- Provide `pnpm contracts:generate` backed by a Rust `xtask`, write committed generated TypeScript to one read-only directory, and make drift fail CI.
- Provide one frontend API facade and one raw Tauri transport module. Components and feature modules cannot import `invoke` or event APIs directly.
- Register only narrow placeholder/core commands needed to prove typed round trips; do not expose generic invocation.
- Create root command contracts and GitHub Actions gates documented in [Development](../development.md#planned-command-surface).
- Configure rustfmt, Clippy warnings-as-errors, ESLint boundary rules, Prettier, Vitest, React Testing Library, and strict TypeScript.
- Ensure the main window is standard GNOME-compatible, has no tray/hide-on-close/autostart behavior, and exits on close.

## Non-functional requirements

- A clean checkout produces byte-stable generated bindings with the locked toolchain/dependencies.
- No build step needs production data, a live systemd manager, or internet access after dependencies are present.
- Startup placeholder work is bounded and introduces no polling/daemon.
- Dependency features and Tauri plugins follow least privilege.
- Public Rust interfaces and generated-file ownership are documented.

## Failure and recovery states

- Missing native prerequisites fail during setup/build with a documented actionable error.
- Contract generation failures leave the last committed generated tree intact or replace it atomically; partial output is not accepted.
- An unknown backend error becomes `CORE_INTERNAL` with a correlation ID, never a Rust debug string.
- A Tauri command serialization mismatch fails a contract test/CI before release.
- Window initialization failure writes a redacted diagnostic when state paths later become available; it does not start a hidden process.

## Explicit exclusions

This workstream does not implement persistence, feature modules, systemd access, launcher behavior, a tray, autostart, a daemon, remote content, or release distribution/signing.

## Architecture impact

This workstream creates the dependency graph, Tauri composition edge, frontend transport/API boundary, generated contract flow, and repository quality command surface. It must implement ADR-001 through ADR-004 and ADR-011 without placing feature logic in adapters.

## Contracts

Initial generated contracts include `AppError`, `HealthState`, `HealthReason`, `ActionClassification`, `RuntimeProfile`, `BuildInfo`, `PageRequest`, `Page<T>` (or generator-compatible concrete aliases), `ModuleId`, `CorrelationId`, and event envelopes. Contract serialization uses explicit stable tagging and lowercase wire values where documented. No database path or arbitrary operation name is present.

## Persistence and migrations

None. The bootstrap must not create a database. Tests write only to build outputs or injected temporary directories.

## Security implications

Review all Tauri plugins/default features. Do not grant shell, broad filesystem, systemd write, or remote-window capabilities. Assign the human actor behind the Tauri boundary rather than accepting it from React. Enforce restricted frontend imports and crate dependency checks in CI.

## Performance implications

Keep the placeholder startup free of eager module data, global intervals, and background tasks. Capture an early empty-shell startup measurement as a comparison point, not the final baseline.

## Acceptance criteria

- **FND-01-AC01:** Clean locked installs/builds succeed on the target machine using the documented root commands.
- **FND-01-AC02:** The React/Tailwind placeholder renders in a Tauri 2 window and close leaves no Argos process.
- **FND-01-AC03:** Cargo dependency checks prove domain/application/contracts do not depend on Tauri and application tests run headlessly.
- **FND-01-AC04:** Strict TypeScript, formatting, ESLint, Clippy, Rust tests, frontend tests, and build gates pass.
- **FND-01-AC05:** Two contract-generation runs are identical; CI detects a deliberately stale generated file.
- **FND-01-AC06:** Only the allowed transport imports Tauri frontend invocation/events, and a component reaches a typed test command through the API facade.
- **FND-01-AC07:** Representative success and every error shape round-trip Rust/JSON/TypeScript fixtures without debug disclosure.
- **FND-01-AC08:** Committed Tauri capabilities contain no broad shell/filesystem or systemd-write grant.
- **FND-01-AC09:** Normal source debug and optimized builds report `development`; the explicit packaging build reports `production` in a build-info test without opening data.

## Testing strategy

Use Cargo dependency/layer tests, fake-port application tests, contract snapshot/round-trip/determinism tests, Vitest transport/API mocks, lint fixtures for forbidden imports, Tauri capability scans, and a target-window lifecycle smoke test. CI uses no live systemd.

## Implementation order and tasks

1. `FND-BST-001` — workspace/toolchain/boundary skeleton.
2. `FND-BST-002` — root commands, formatting, lint, tests, and CI.
3. `FND-BST-003` — typed internal/public error foundation.
4. `FND-BST-004` — Rust contract export and deterministic drift gate.
5. `FND-BST-005` — frontend transport/API facade and narrow Tauri round trip.
6. `FND-BST-006` — normal window lifecycle and capability baseline.

Detailed dependencies and completion conditions are in [the task ledger](tasks.md).

## Verification and documentation update

Run all FND-01 acceptance tests plus the target close-process check. Replace planned-command wording in [Development](../development.md) with tested prerequisite details, record chosen dependency/generator rationale in the decision register only if it changes consequences, and keep generated-file instructions current.
