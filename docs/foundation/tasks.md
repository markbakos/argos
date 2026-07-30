# Foundation implementation task ledger

## Use of this ledger

Tasks are ordered by dependency and have stable IDs. A task includes implementation, tests, and documentation in one reviewable change; it is not complete when only code exists. Dependencies may be developed in parallel only after their listed prerequisites are complete and their files do not conflict.

The [documentation phase gate](README.md#phase-gate) was approved by the user on 2026-07-30. Tasks are authorized only in the dependency order below.

**Implementation status:** `FND-BST-001` and `FND-BST-002` complete; `FND-BST-003` is next; all later tasks pending.

## Stage 1 — Reproducible boundary

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-BST-001` | Documentation approval | Root Cargo/pnpm workspace; domain, application, contracts, SQLite, Linux, systemd, Tauri, React, and `xtask` boundaries; select compatible stable dependencies and commit lockfiles. No feature behavior. | Prove clean metadata/build resolution and no forbidden dependency direction. Update exact prerequisites/dependency rationale in Development. **Done when both workspaces build an empty boundary skeleton from a clean checkout without opening runtime data.** |
| `FND-BST-002` | BST-001 | Root format/lint/typecheck/test/build command contracts, rustfmt/Clippy/ESLint/Prettier/Vitest/RTL, GitHub Actions, and structural check harness. | Add deliberate lint/failure fixtures and clean CI run; document actual commands. **Done when `pnpm check` executes every baseline gate with locked dependencies and produces actionable failures.** |
| `FND-BST-003` | BST-001 | Domain/application typed error hierarchy, correlation IDs, safe error translator, and action/actor base types. | Unit-test namespace/code/retry/detail/redaction and unknown internal failures; update error code table if implementation evidence requires. **Done when no runtime debug string crosses a public test boundary.** |
| `FND-BST-004` | BST-002, BST-003 | Rust contract crate/export metadata, generator selection, `xtask`, committed generated directory, deterministic formatting/drift check, base IDs/health/profile/page/event/error contracts. | Round-trip every enum/error, run generation twice, deliberately stale a fixture in CI; document selected generator. **Done when deterministic regeneration and stale-binding rejection pass.** |
| `FND-BST-005` | BST-004 | Sole frontend Tauri transport, semantic API facade, narrow typed proof command/event, Tauri translation/state pattern, restricted imports. | Component/API transport mock tests and forbidden-import lint fixture; document extension pattern. **Done when a typed round trip works and no component can import/invoke Tauri directly.** |
| `FND-BST-006` | BST-005 | Standard main window lifecycle, minimal capability groups/permissions, no tray/autostart/hide-on-close, build-profile reporting. | Capability scan and target process-close smoke; debug/local optimized/package profile metadata tests. **Done when close exits all Argos processes and only approved baseline permissions exist.** |

## Stage 2 — Safe runtime and persistence

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-DAT-001` | BST-003, BST-004 | Pure runtime-profile selector and XDG/explicit/test path resolver with source-build defaults, two-part production acknowledgement, runtime-unavailable state, repository guard. | Table-driven set/unset HOME/XDG/root/profile tests without filesystem writes; update resolved rules only through spec. **Done when every FND-02 path/isolation case resolves or fails exactly before I/O.** |
| `FND-DAT-002` | DAT-001 | Private lazy directory creation; atomic, versioned `config.toml`; theme and executable-search-path validation; safe symlink/ownership behavior. | Temporary filesystem/permission/config corruption/atomic-failure tests; document actual config keys and platform permission limitations. **Done when valid config round-trips and unsafe/security-sensitive config fails without partial replacement or repository writes.** |
| `FND-DAT-003` | DAT-002, BST-002 | SQLite pool/bootstrap and storage health with foreign keys, 5000 ms busy timeout, WAL, bounded connections, quick verification. | Temp-file pragma, busy, close/cleanup, and health mapping integration tests. **Done when a validated external-profile database opens with exact effective settings and no SQL leaks outside storage.** |
| `FND-DAT-004` | DAT-003 | Embedded immutable migration runner, consistent verified pre-migration backup, five-file automatic retention, integrity/version check, restricted failure recovery. | Fresh/prior fixture upgrades, injected rollback, checksum/newer-schema refusal, backup open/retention tests; document migration workflow. **Done when failed upgrade never exposes ordinary repositories or destroys source/backup.** |
| `FND-DAT-005` | DAT-004 | Initial tables/checks/indexes and transaction-aware module preference, launcher, audit repositories with UUID/UTC/JSON/page/revision behavior. | SQLite CRUD/constraint/page/conflict/audit atomicity tests and schema snapshot; record migration ID. **Done when repositories expose domain values only and every initial schema invariant passes.** |
| `FND-DAT-006` | DAT-005, BST-002 | Automated production/development/test collision suite and pre/post repository live-data scanner covering DB sidecars, logs, backup, cache, state/runtime patterns. | Run representative init/migration/CRUD/log/export fixtures in isolated roots and a deliberate offending fixture. **Done when CI fails on any repository live data and proves normal tests never inspect XDG data.** |

## Stage 3 — Shell and modules

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-SHL-001` | BST-005 | React root/providers, router, Query defaults/cancellation, shell landmarks/sidebar, Dashboard placeholder, Settings/Diagnostics routes, route error boundary. | Shell/render/navigation/loading/fatal recovery tests; update actual frontend layout. **Done when the shell is usable with backend feature APIs mocked unavailable and performs no feature fetch.** |
| `FND-SHL-002` | SHL-001, DAT-002 | Tailwind v4 semantic tokens, system/light/dark persistence/live media behavior, shared focus/status/loading/empty/error/dialog/form primitives with reduced motion. | Theme/config/media, keyboard/focus restoration, accessible-name and contrast/manual checks. **Done when core patterns work in light/dark/system without raw semantic colors or color-only status.** |
| `FND-SHL-003` | BST-004 | Backend manifest/effective registry, capability/dependency/platform/health model, duplicate/route/cycle/order validation, systemd/launcher built-in manifests. | Pure registry validation/merge/unknown override/health tests; document actual public registry API. **Done when deterministic effective modules require no Tauri and invalid graphs fail explicitly.** |
| `FND-SHL-004` | SHL-001, SHL-003 | One frontend module registry, lazy route loaders/presentation entries, derived sidebar/router join, disabled/unhealthy route behavior, backend/frontend parity gate. | Lazy-load instrumentation, nav/order/route/parity/error tests including a test module. **Done when adding the test module touches only the two registries and module files.** |
| `FND-SHL-005` | DAT-005, SHL-004, BST-006 | Application/Tauri/API/Settings flows for module enable/order/reset overrides and audit; disabled navigation persistence and storage degradation. | Use-case transaction, Tauri translation, Query mutation/restart, conflict/failure UI tests; update module-add guide. **Done when preference+audit is atomic and disable/re-enable survives restart without deleting module data.** |
| `FND-SHL-006` | SHL-002, SHL-005 | Shell/module accessibility review and instrumentation proving inactive lazy modules release queries/listeners and perform no intervals. | Keyboard/focus/status/manual pass plus fake timer/subscription teardown tests; record results. **Done when FND-03 accessibility and inactive-resource criteria have evidence.** |

## Stage 4A — Read-only systemd (parallel with launcher after prerequisites)

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-SYS-001` | BST-004, SHL-003 | Explicit scope/unit/timer/journal domain values, contracts, read ports, use cases, limits, error/health combination using fakes. | Scope-required, no-fallback, partial-health, limit/cancellation and serialization tests. **Done when all read behavior runs headlessly and no mutation type exists.** |
| `FND-SYS-002` | SYS-001 | Independent user/system `zbus` connection providers, manager health, reconnect/backoff on demand, normalized unit/state mapping with unknown fallback. | Captured/fake D-Bus mapping, permission/bus/manager/timeout/disconnect tests. **Done when each scope fails independently without raw D-Bus types crossing the adapter.** |
| `FND-SYS-003` | SYS-002 | Service/timer lists; detail core properties, failed state, dependency names/jobs; timer previous/next trigger/clock mapping. | Fixture lists/details and timestamp/unknown/missing/unit-disappeared cases; update actual method/property mapping. **Done when normalized outputs meet FND-04 read contracts across target-supported variations.** |
| `FND-SYS-004` | SYS-001 | Replaceable `journalctl` adapter using direct argv, scope selector, JSON/no-pager/quiet, 1–500 clamp, time/output bounds, cancellation, safe parsing. | Command-plan, literal malicious-looking unit rejection, JSON/missing field/nonzero/permission/timeout/oversize tests. **Done when no shell/concatenation is possible and every result is bounded.** |
| `FND-SYS-005` | SYS-003, SYS-004, SHL-002, SHL-004, BST-006 | Narrow user/system read Tauri permissions/API plus lazy React service/timer/detail/log views and scoped health/loading/empty/error states. | Capability/error translation, mocked Query/view/scope/keyboard tests and mutation-surface static scan. **Done when the UI reads each explicit scope and compiled capabilities contain no write.** |
| `FND-SYS-006` | SYS-005 | Active-consumer D-Bus signal subscription, bounded/coalesced invalidation, manual refresh, teardown/cancellation, redacted target-system smoke. | Fake signal burst/refcount/failure tests and real user/system/timer/log checklist. **Done when active changes refresh, inactive views leave no listener/poll, and target behavior is recorded honestly.** |

## Stage 4B — Launcher (parallel with systemd after prerequisites)

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-LCH-001` | BST-004, SHL-003 | Launcher values/contracts/validators, action classifications, execution-plan port, CRUD/launch use cases against fakes. | Kind/field/scheme/path/UTF-8/NUL/argument/revision/classification/redaction tests. **Done when invalid data cannot reach a repository/executor and launch accepts saved ID only.** |
| `FND-LCH-002` | LCH-001, DAT-005 | SQLite launcher CRUD/favorite/delete/pagination/revision integration, atomic mutation audits, and correlated external-action attempt/outcome audit protocol. | Fresh/persisted ordering/page/conflict/rollback plus pre/outcome audit failure tests. **Done when valid items round-trip exactly, stale/destructive operations cannot overwrite, and side effects cannot start without an attempt record.** |
| `FND-LCH-003` | LCH-001, DAT-002 | Linux external URL/folder opener and direct executable adapter with validated search paths, literal argv, optional working directory, empty overrides, bounded/no output retention. | Fake command-plan plus harmless process fixtures for spaces/metacharacters/missing/permission/type/opener errors; document inherited environment/opener. **Done when no code path uses a shell/current-directory search/arbitrary request target.** |
| `FND-LCH-004` | LCH-002, LCH-003, BST-006 | Narrow launcher read/write/execute Tauri commands/capabilities and typed frontend API/Query keys/error translation. | Translation/capability/import tests; ensure launch command accepts only ID. **Done when the WebView can perform saved-item use cases but cannot submit arbitrary process/file operations.** |
| `FND-LCH-005` | LCH-004, SHL-002, SHL-004 | Lazy launcher tile/list, bounded paging, forms, favorite, revision conflict, exact delete confirmation, explicit launch/open outcome states. | Mocked CRUD/launch/loading/empty/error/conflict and keyboard/dialog focus tests. **Done when every foundation workflow is accessible and backed only by semantic API calls.** |
| `FND-LCH-006` | LCH-005, DAT-006 | Restart/rebuild/source-move/cache-deletion persistence proof; external URL/folder/executable target smoke; audit/log/export privacy scan. | Redacted target checklist and filesystem snapshots; update tested opener/search/env notes. **Done when records survive all required lifecycle cases and no sensitive execution data/repository live file appears.** |

## Stage 5 — Diagnostics and observability

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-DIA-001` | DAT-002, BST-003 | Structured tracing initialization, private five-by-5 MiB rotation, field/redaction policy, correlation linkage, 50-entry recent-error buffer, early sink failure behavior. | Forced small rotation/buffer/redaction/recursive-failure tests; document tracing implementation. **Done when logs are event-driven, bounded, and forbidden corpus values never appear.** |
| `FND-DIA-002` | DAT-004, SHL-003, SYS-005, LCH-004 | Safe typed path/build/storage/migration/module/systemd/journal/launcher provider health and deadline-bounded partial diagnostics aggregator. | Each provider success/timeout/failure permutation with contract snapshots. **Done when one failing provider cannot fail/hang other sections and no database file/content leaks.** |
| `FND-DIA-003` | DIA-002, SHL-002, SHL-004 | Diagnostics React page, manual refresh, section states, recent failures, resolved category/profile display, persistent production-override warning. | Mocked partial/unsupported/loading/error and accessibility tests. **Done when required facts are readable without polling and warnings cannot be mistaken for normal development.** |
| `FND-DIA-004` | DIA-001, DIA-002, DAT-005, LCH-003 | Versioned redacted atomic report under state diagnostics, narrow open-folder action, DB-audit when available and explicit audit-unavailable behavior. | Safe schema/privacy/collision/partial write/storage-down/permission tests and capability scan. **Done when export cannot choose arbitrary paths or contain forbidden data and remains useful during DB failure.** |
| `FND-DIA-005` | DIA-003, DIA-004, SYS-006, LCH-006 | On-demand host/WebView child memory snapshot where supported; complete privacy, partial-failure, idle-write/resource teardown, and target report inspection. | Supported/unsupported/permission metric tests, idle observation, export/log manual inspection. **Done when FND-06 evidence proves no background sampler/heartbeat or unbounded retention.** |

## Stage 6 — Integrated foundation proof

| ID | Depends on | Affected parts and deliverable | Tests, docs, and completion condition |
| --- | --- | --- | --- |
| `FND-VER-001` | All implementation tasks | Final clean-checkout `pnpm check`, CI matrix, contract/migration/registry/boundary traceability, tested Arch prerequisite record. | Run green twice from clean states and capture revision/toolchains. **Done when deterministic automated gates map to every automatable acceptance criterion.** |
| `FND-VER-002` | VER-001 | Actual command/event/plugin/window/capability inventory; generic surface/systemd write/process argv/privacy audit; full data-location/isolation snapshot. | Independent checklist with exact artifacts and deliberate guard failures; update security/data docs before code if mismatch. **Done when no unowned authority or repository/production data access remains.** |
| `FND-VER-003` | VER-001, VER-002 | Target Arch/GNOME read-only smoke for window, shell, profiles, SQLite persistence, module prefs, user/system services/timers/logs, launcher opens/spawn, diagnostics, close. | Redacted date/revision/environment/result report. **Done when every target procedure passes or allowed permission/unavailable outcome is demonstrated.** |
| `FND-VER-004` | VER-003 | Optimized-development full process-group cold/warm startup, ten-minute idle CPU/write, two-hour idle memory, active/inactive subscription, bounded log, clean-close baseline. | Record tools/context/raw summaries and investigate misses/growth. **Done when targets have honest passing evidence and no high-frequency poll/idle writer/process remnant exists.** |
| `FND-VER-005` | VER-003 | Manual plus automated core/feature keyboard, focus, labels, contrast, reduced motion, and all loading/empty/error/unavailable/degraded/conflict/confirmation/partial-failure states. | Accessibility/state matrix with fixes and reruns. **Done when no core workflow requires pointer/color-only understanding and dialog focus restores correctly.** |
| `FND-VER-006` | VER-002, VER-003, VER-004, VER-005 | Final acceptance/task/definition-of-done traceability report, link/term/path/scope/module/security/exclusion consistency review, package profile/data inspection, documentation synchronization. | Run Markdown/link/terminology scans and human review; resolve contradictions before sign-off. **Done when all criteria have evidence, docs match implementation, and F1 is explicitly approved complete.** |

## Acceptance traceability

| Specification criteria | Primary task evidence |
| --- | --- |
| FND-01-AC01–AC09 | BST-001–BST-006, finalized by VER-001/003 |
| FND-02-AC01–AC10 | DAT-001–DAT-006, finalized by VER-002/003 |
| FND-03-AC01–AC10 | SHL-001–SHL-006, finalized by VER-005 |
| FND-04-AC01–AC09 | SYS-001–SYS-006, finalized by VER-002/003/004 |
| FND-05-AC01–AC10 | LCH-001–LCH-006, finalized by VER-002/003 |
| FND-06-AC01–AC09 | DIA-001–DIA-005, finalized by VER-002/004/005 |
| FND-07-AC01–AC14 | VER-001–VER-006 |

The final report must expand ranges into individual evidence references. A range here is scheduling traceability, not permission to omit a criterion.
