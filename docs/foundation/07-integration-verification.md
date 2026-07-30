# FND-07 — Integration and release verification

**Status:** Approved for foundation implementation  
**Depends on:** FND-01 through FND-06  
**Completes:** Foundation milestone F1

## Problem and user value

Argos foundation crosses build tooling, a WebView, Rust trust boundaries, XDG storage, SQLite, D-Bus, journald, and desktop process behavior. Unit tests cannot prove that these pieces remain safe and lightweight on the actual target machine.

The user gains evidence that the entire foundation works as documented, does not touch production data during development, remains read-only toward systemd, persists launcher data outside source, and exits cleanly without hidden resource use.

## Workflows

1. CI reproduces all quality gates from a clean checkout and detects stale contracts/migrations/boundary violations.
2. A target-machine verifier runs a documented development-profile smoke checklist without mutating systemd or exposing personal data.
3. The verifier proves launcher persistence across restart/rebuild/source movement and proves repository data-location invariants.
4. A security reviewer inventories actual Tauri commands/capabilities/process plans and confirms no generic or systemd-write surface.
5. A performance reviewer records startup, process-group idle CPU/memory/database writes, active/inactive subscriptions, and close termination.
6. A documentation reviewer maps evidence back to acceptance IDs and resolves every contradiction before F1 is declared complete.

## Functional requirements

- Provide one `pnpm check` quality gate covering formatting, lint, strict typecheck, Rust/frontend tests, builds, contracts drift, migrations, capability/boundary checks, and repository live-data scan.
- Run GitHub Actions from committed lockfiles on a supported Linux environment without requiring live systemd for the main gate.
- Provide optional/manual target scripts or checklists that are read-only except for clearly identified Argos development data, harmless launcher actions, and safe diagnostics export.
- Record the exact target Arch/GNOME/native dependency setup and confirm Argos opens as a normal desktop application.
- Execute the target-system procedure in [Verification](../verification.md#target-system-verification), redacting machine-specific units/paths from committed evidence.
- Capture pre/post repository snapshots around runtime actions and test production/development/test/root isolation.
- Inventory every Tauri command, event, plugin, window label, and capability grant against [Security](../security.md#tauri-capability-model).
- Inspect compiled/source systemd surfaces to prove no mutation method/capability and inspect launcher execution to prove literal argv/no shell.
- Measure the complete process group for startup, idle CPU, extended-idle memory, idle database writes, route subscription cleanup, bounded logs, and exit.
- Perform a keyboard/focus/theme/contrast/status accessibility pass on core and feature workflows.
- Validate install/build/package artifacts contain no live data and that ordinary local optimized builds remain development-profile; explicit packages embed production.
- Produce a versioned foundation verification report mapping every specification acceptance criterion and definition-of-done row to evidence.
- Run the documentation consistency procedure and update specifications/decisions before accepting any implementation divergence.

## Non-functional requirements

- Verification scripts are non-destructive by default, print exact Argos development/temp targets, and refuse production/repository paths.
- Committed evidence contains no personal launcher target, journal text, environment, token, user-unit name, home path, database, or production record.
- Performance measurements record context and raw summaries rather than unsupported pass/fail claims.
- A flaky live-system check is not folded into deterministic unit CI; it is separately labeled and actionable.

## Failure and recovery states

- Quality gate failure blocks F1; it is not waived by manual success without a documented decision/update.
- Target system lacks system-manager/journal permission: verify the required explicit unavailable/permission UX; only user-manager connectivity is expected when available.
- Native packaging differs from CI: record the target incompatibility, update bootstrap/setup specification, and fix before target build acceptance.
- Repository snapshot detects live data: stop testing, preserve evidence, remove only clearly test-created external/temp data through safe procedure, fix path design, and rerun from clean state.
- Performance target missed: retain measurement, investigate and update implementation/spec assumptions; do not redefine measurement after the fact.
- Security inventory finds an unowned command/capability: remove it or introduce an approved specification; no undocumented exception.
- Documentation conflict: the affected acceptance result remains blocked until documents and implementation align.

## Explicit exclusions

This workstream does not add product features, systemd writes, benchmark telemetry, cloud CI requiring a personal desktop, release signing/distribution, automatic deletion of user data, or broad cleanup/reset tooling.

## Architecture impact

Verification may add repository-owned read-only analysis scripts, isolated test fixtures, CI workflows, and report templates. It must not introduce runtime services or privileged test helpers. Any measurement hook exposed in Diagnostics remains on-demand and narrow.

## Contracts

No new product contract is expected. A versioned verification report schema may use task/acceptance IDs, environment metadata, redacted evidence references, result, and notes. It never becomes a general command/result API.

## Persistence and migrations

Verification uses test temporary roots and deliberate `argos-dev` data. It does not write production. Report artifacts contain no database and live outside runtime data unless the user explicitly retains a redacted copy in project documentation. No migration is added.

## Security implications

Test scripts cannot accept broad deletion targets or use a generic shell executor through Argos. Manual launcher targets are harmless and selected by the human. Systemd checks are reads. Capability/process/static inventories are reviewed against generated/compiled artifacts, not only source intent.

## Performance implications

Use a production-optimized code build with development runtime data for baseline measurement, then separately confirm packaged profile metadata. Include host and WebView children. Avoid profiling instrumentation that changes idle behavior for final numbers; record measurement tools and sampling duration.

## Acceptance criteria

- **FND-07-AC01:** Clean checkout/install/build/test/check passes with committed lockfiles on CI and target Arch.
- **FND-07-AC02:** GNOME desktop smoke proves window/shell/themes/navigation/settings/diagnostics and clean process-group exit without tray/daemon.
- **FND-07-AC03:** Path snapshot proves no live file in repository and strict production/development/test isolation, including incomplete acknowledgement rejection.
- **FND-07-AC04:** Fresh/upgrade/failure database evidence proves migrations, backups, recovery, module preference and launcher persistence after rebuild/source movement/cache deletion.
- **FND-07-AC05:** Generated binding drift, backend/frontend registry parity, lazy modules, and thin Tauri/core-independent tests pass.
- **FND-07-AC06:** Real user/system systemd service/timer/detail/trigger/log checks pass where authorized and show honest unavailable/permission states otherwise; no mutation occurs.
- **FND-07-AC07:** URL/folder/executable smoke uses saved targets and literal arguments; command/audit/export inspection reveals no shell or sensitive leakage.
- **FND-07-AC08:** Command/event/plugin/window capability inventory contains only approved narrow foundation operations and no generic SQL/file/process/shell/systemd-write surface.
- **FND-07-AC09:** Loading/empty/error/unavailable/degraded/conflict/confirmation paths and keyboard/focus/contrast/reduced-motion checks pass.
- **FND-07-AC10:** Cold startup target, effectively zero idle CPU, no one-second global polling, no continual idle database writes, bounded logs/results, and close termination meet documented measurements.
- **FND-07-AC11:** Extended idle process-group memory shows no unexplained sustained growth; investigation is recorded for any growth.
- **FND-07-AC12:** Safe diagnostics/report/log inspection confirms redaction and partial-health utility.
- **FND-07-AC13:** Packaged artifacts contain no user data, package profile is production, and ordinary source optimized build profile remains development.
- **FND-07-AC14:** A final traceability report maps all acceptance/task/definition-of-done items and the consistency review finds no unresolved contradiction.

## Testing strategy

Combine all automated strategies in [Verification](../verification.md) with reproducible repository scripts, GitHub Actions, target smoke, static/capability inventory, filesystem snapshots, privacy corpus scans, accessibility review, and measured performance/idle sessions.

## Implementation order and tasks

1. `FND-VER-001` — complete CI/root quality and traceability gate.
2. `FND-VER-002` — security/capability/data-location review.
3. `FND-VER-003` — target Arch/GNOME/systemd/launcher smoke.
4. `FND-VER-004` — startup/idle/memory/write/process baseline.
5. `FND-VER-005` — accessibility and partial-failure review.
6. `FND-VER-006` — documentation consistency and F1 verification report.

## Verification and documentation update

This workstream is itself the final verification. Store a redacted report in the documentation or release evidence location chosen during implementation, update tested setup/packaging facts, and mark F1 complete only after every criterion is evidenced. Future feature planning starts after—not as part of—this sign-off.
