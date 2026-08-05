# Verification strategy

## Evidence model

Foundation completion requires automated test output, static boundary checks, a target Arch/GNOME smoke record, performance measurements, and a documentation consistency review. A green unit-test suite alone is insufficient; manual checks are reserved for behavior that depends on a real desktop/systemd/journal environment.

The [foundation expected-behavior specification](foundation/expected-behavior.md) is the test-first oracle for FND-01 through FND-07. Automated tests, static checks, target procedures, and final evidence cite its existing acceptance IDs.

Every acceptance result records the build revision, build/runtime profile, target environment, date, command or procedure, outcome, and diagnostic reference with personal values redacted.

## Automated Rust tests

### Domain and application

- value-object validation for UUIDs, timestamps, module IDs, unit scope/names, URLs, folders, executables, arguments, limits, and revisions;
- module registry duplicate, route, order, missing dependency, cycle, enablement, and health behavior;
- launcher CRUD/favorite/delete/launch use cases using fake repositories/executors, including transaction/audit outcomes and concurrency conflicts;
- action classification, actor assignment, redaction, error mapping, and retryability;
- systemd use cases using fake readers for available, unavailable, degraded, permission, timeout, unknown-state, and cancellation paths;
- diagnostics aggregation with partial subsystem failure.

### Paths, storage, and adapters

- XDG resolution for set/unset category variables and missing HOME/runtime;
- production/development namespaces, explicit-root validation, repository-containment rejection, and two-part production acknowledgement;
- test temporary-root isolation and cleanup after connection close;
- directory permissions where reliably testable and absence of repository writes;
- fresh database initialization, required pragmas, schema constraints/indexes, foreign keys, busy behavior, and WAL sidecars outside the repository;
- migrations from each committed schema fixture, checksum drift detection, transactional failure rollback, pre-migration backup verification, and retention;
- SQLite repository CRUD/pagination/revision conflict/audit atomicity and bounded JSON checks;
- process plan construction proving separate executable/arguments and rejecting shell/path tricks;
- systemd D-Bus response mapping and unknown enum/property handling from fixtures;
- journal JSON parsing, bounded results, direct argument-vector construction, invalid unit rejection, nonzero exit, timeout, permission, and malformed output;
- contract generation determinism, representative JSON round trips, public error translation, and generated-tree drift.

Most tests use fakes or captured protocol fixtures and must not require a live systemd manager, journal permissions, desktop opener, network, or production paths.

## Automated frontend tests

Vitest and React Testing Library cover:

- shell landmarks, sidebar, host-aware Dashboard hostname loading/success/unavailable and direct-route inactivity, Settings, Diagnostics, responsive navigation, and route error boundary;
- backend/frontend registry parity, derived navigation, module ordering, disabled disappearance, and unavailable/degraded/error presentation;
- lazy route loading and failure recovery;
- system/light/dark theme persistence, live system preference changes, and reduced motion;
- typed API query loading, empty, retryable/non-retryable error, cancellation, and unknown-error fallback states;
- launcher form validation for each kind, CRUD/favorite/delete-confirmation flows, revision conflict, and launch outcomes with mocked API methods;
- systemd service/timer/detail/log rendering with mocked scoped responses, permission/unavailable states, event-driven invalidation, and explicit refresh;
- diagnostics partial health and safe-export results;
- keyboard operation, visible focus class application, dialog focus trap/Escape/restoration, accessible names, and non-color status cues for core controls.

Components mock the semantic frontend API, not Tauri. A lint/static test ensures no component or module imports the raw transport.

## Static boundary and security checks

CI fails when:

- Tauri imports occur outside the allowed transport/composition files;
- generated TypeScript differs after regeneration or contains manual-source markers incorrectly;
- SQL appears outside the storage/migration boundary;
- `zbus`/systemd protocol types appear outside the systemd adapter;
- general process primitives appear outside the Linux process adapter or a reviewed tool;
- shell plugins, `sh -c`, generic invoke-by-name, raw SQL/file commands, or foundation systemd mutation names are exposed;
- capability files grant shell/broad filesystem/systemd-write permissions;
- launcher execution code concatenates executable and arguments;
- persisted/runtime filename patterns appear in tracked files or a test run writes beneath the repository;
- Rust runtime paths introduce `unwrap`/`expect` contrary to the lint policy;
- unbounded journal/query/metadata inputs bypass clamp validation.

Some checks use lint/compiler boundaries; small repository scripts may enforce structural rules that type systems cannot. Scripts report exact offending files and avoid fragile scans of documentation/fixtures.

## Target-system verification

Run on the target Arch Linux, GNOME, systemd environment with development data unless the case explicitly concerns production:

1. build and open Argos as a normal non-root application;
2. confirm production and development resolved paths differ and neither is within the repository;
3. create a launcher item, close Argos, rebuild or switch a harmless source revision, reopen, and confirm persistence;
4. confirm disabled modules leave primary navigation and can be re-enabled in Settings;
5. connect explicitly to the user manager; list services/timers and identify a known existing user timer;
6. connect independently to the system manager when policy permits; record a clear permission/unavailable state otherwise;
7. inspect service/timer core properties, failed units, and next/previous timer triggers;
8. read a bounded recent-log result and confirm permission/empty states are honest;
9. open a harmless HTTPS URL in the external browser;
10. open a temporary folder in the external file handler;
11. start a harmless direct executable with arguments containing spaces/metacharacters and confirm they remain literal arguments;
12. export safe diagnostics and inspect it for forbidden personal data;
13. close the main window and verify no Argos/tray/daemon process remains.

No verification step mutates systemd. Tests use targets selected by the human and do not expose personal unit names in committed artifacts.

## Data-location and isolation proof

The verification harness records a repository filesystem snapshot before and after fresh start, launcher CRUD, diagnostics export, database migration, logs, and shutdown. Only expected build/test artifacts in ignored build directories may change; no SQLite database/sidecar, config, state log, backup, imported icon, cache, diagnostic export, or runtime file may appear anywhere under the repository.

Isolation cases prove:

- development actions change only `argos-dev` or an explicit external root;
- tests change only their own temporary roots and do not inspect XDG production/development data;
- a normal source build cannot resolve `argos`;
- incomplete production acknowledgement is rejected;
- an acknowledged production selection is visible and resolves exactly `argos` without touching it during resolution-only tests;
- deleting cache preserves all launcher/module/audit data and the app recovers;
- repository relocation/deletion does not alter the resolved XDG data location.

## Performance baseline

Measure the full process group, including Tauri host and WebView children, on the target machine. Record hardware, desktop/session, build mode, database size, enabled modules, and sample duration.

Minimum foundation evidence:

- cold start from process invocation to interactive shell, targeting under two seconds;
- warm start recorded separately;
- idle CPU sampled after startup stabilization for at least ten minutes, with no periodic one-second spikes attributable to Argos;
- database write/file timestamp observation while idle, showing no continual writes;
- resident/proportional memory for host and child processes at start and after at least two hours idle, investigating sustained growth;
- systemd page active/inactive comparison demonstrating subscriptions stop and no global polling continues;
- bounded journal request at the maximum allowed result count;
- clean process-group termination after window close.

Targets are engineering goals, not permission to hide measurements. If cold start exceeds two seconds or idle behavior is not effectively zero, file an evidence-backed issue/spec update before declaring foundation complete.

### Task Manager implementation checkpoint — 2026-08-05

The `TMG-001` through `TMG-004` implementation checkpoint passed deterministic parser/rate/search/sort/bound tests, Tauri/API contract translation, frontend fake-timer visibility/non-overlap/teardown tests, registry parity, read-only capability/privacy scans, contract drift, and optimized development plus explicit production-profile builds. The optimized frontend emitted Task Manager as a separate 28.46 kB lazy chunk (7.67 kB gzip), and a development-profile GNOME launch using an isolated `/tmp` root exited cleanly with no remaining Argos/Vite process.

The ignored target-only Linux reader check sampled the host's bounded snapshot 50 times with 305 observed processes: average wall time 11 ms and p95 11 ms, below the 250 ms criterion. This checkpoint contains aggregate values only. It does not replace `TMG-005`: the full ten-minute inactive/active process-group CPU/write observation, 30-minute alternating-view memory/cache observation, and interactive real-data/accessibility matrix remain required before final TM-01 sign-off.

## Logging and diagnostics verification

Tests force rotation thresholds with small configured test limits and verify file count/size bounds. A fixed-capacity recent-error buffer evicts oldest entries. Structured events contain component/operation/result/duration/error code without secrets.

Redaction fixtures include tokens, URL query secrets, environment maps, launcher arguments, file contents, SQL/debug errors, and journal text; none may appear in user messages, normal logs, audit metadata, or safe exports. Diagnostics remains usable when database, user systemd, system systemd, runtime path, or a module is independently unavailable.

## Foundation definition-of-done matrix

| Outcome | Required evidence |
| --- | --- |
| Tauri/React/Vite/Tailwind and GNOME window work | build gate plus target smoke |
| Tauri is thin and Rust core reusable | dependency/static checks and use-case tests without Tauri |
| XDG/profile/repository isolation | path tests and data-location proof |
| SQLite init/migrate/backup/recovery | storage integration and migration-failure tests |
| Generated Rust-to-TypeScript contracts | deterministic generation and CI drift test |
| Central backend/frontend registries and lazy modules | registry tests, route tests, bundle/lazy-load observation |
| Read-only scoped systemd services/timers/logs | fake tests, capability scan, target smoke |
| Launcher persistence and safe execution | repository/use-case/UI tests plus restart/rebuild smoke |
| Diagnostics/settings/themes/states | frontend and aggregation tests plus export inspection |
| Window close exits; no daemon/tray/poll loop | process and idle measurements |
| Accessibility and stable errors | interaction tests, manual keyboard pass, error mapping tests |
| Documentation agrees with implementation | final link/term/scope/decision/task review |

## Documentation consistency review

Before implementation approval and again before foundation completion:

1. validate every relative Markdown link;
2. search for product/profile/namespace variants and resolve them;
3. compare path tables, schema names, health/action enums, and error namespaces;
4. confirm all systemd foundation operations are read-only and explicitly scoped;
5. compare module registry/nav rules and disabled/unavailable behavior;
6. compare security capability groups with actual Tauri permissions;
7. compare exclusions across product and every specification;
8. map every acceptance criterion to a task and evidence source;
9. search for implementation behavior without an owning specification/decision;
10. record assumptions and only materially blocking open decisions.
