# Foundation specifications

## Phase gate

These specifications were **approved for implementation by the user on 2026-07-30**. Documentation milestone D0 is closed and application scaffolding starts with `FND-BST-001`.

Implementation follows the stable tasks in [the task ledger](tasks.md). A task cannot be marked complete until its tests, documentation update, and stated completion condition pass.

The [foundation expected-behavior specification](expected-behavior.md) translates all 71 acceptance criteria into Given/When/Then test oracles. Implementation tests and verification evidence cite the existing acceptance IDs; the behavior specification does not create a second requirements source or authorize implementation.

## Specification set

| ID | Specification | Architectural proof |
| --- | --- | --- |
| FND-01 | [Bootstrap and typed boundary](01-bootstrap-and-contracts.md) | Reproducible Rust/pnpm/Tauri/React toolchain, thin transport, stable errors, generated contracts, CI |
| FND-02 | [Runtime paths and storage](02-runtime-paths-and-storage.md) | XDG/profile/repository isolation, configuration, SQLite, migrations, backups, initial schema |
| FND-03 | [Application shell and module registry](03-shell-and-modules.md) | Accessible shell/themes/states, centralized registries, lazy routes, module preferences |
| FND-04 | [Read-only systemd](04-systemd-read.md) | Explicit user/system D-Bus reads, services/timers/details, bounded journal, events and health |
| FND-05 | [Minimal launcher](05-launcher.md) | Persistent URL/folder/executable CRUD/favorite and structured execution |
| FND-06 | [Diagnostics and observability](06-diagnostics.md) | Structured bounded tracing, subsystem/path health, safe export, partial-failure behavior |
| FND-07 | [Integration and release verification](07-integration-verification.md) | Security/data/performance/target-system evidence and documentation agreement |

Normative cross-cutting behavior lives in [Architecture](../architecture.md), [Security](../security.md), [Data and contracts](../data-and-contracts.md), and [Verification](../verification.md). Feature specifications reference rather than weaken those rules.

## Dependency order

```text
FND-01 Bootstrap/contracts
   |\
   | +--------> FND-03 Shell/registry --------+
   v                                      |   |
FND-02 Paths/storage ----------------+     |   v
   |                                 |     | FND-04 systemd reads
   +----------------------------+    |     |   |
                                v    v     |   |
                              FND-05 launcher  |
                                   \          /
                                    v        v
                                  FND-06 diagnostics
                                           |
                                           v
                                  FND-07 integration proof
```

FND-01 establishes build and boundary contracts. The path resolver in FND-02 must exist before any runtime component can safely open files. The shell can progress after the typed boundary, while registry preference persistence waits for storage. systemd and launcher can then proceed independently. Diagnostics integrates all health providers; final verification follows all workstreams.

## Cross-cutting foundation acceptance

The milestone is complete only when all specification criteria and the [verification definition-of-done matrix](../verification.md#foundation-definition-of-done-matrix) pass. In particular:

- target Arch/GNOME build and ordinary desktop lifecycle work;
- Rust application logic is usable/tested without Tauri;
- no live data appears under the source repository;
- normal source builds use `argos-dev`, packaged builds use `argos`, and tests use fresh temporary roots;
- SQLite initialization/migration/backup/recovery behavior is proven;
- generated frontend bindings match Rust;
- module registries are centralized and frontend routes lazy;
- systemd scope is explicit and all foundation operations are read-only;
- launcher records persist and execution never uses a shell;
- diagnostics is redacted, bounded, and useful under partial failure;
- disabled modules vanish from primary navigation while unhealthy enabled modules explain their state;
- closing the main window leaves no Argos process, daemon, or tray;
- no high-frequency global polling or continual idle write exists;
- automated, manual, security, accessibility, and performance evidence is recorded;
- implementation and documentation agree.

## Foundation-wide exclusions

No specification authorizes systemd mutations/timer creation, managed unit writes, tasks/CRM, workspaces, agents/CLI/MCP, Docker/n8n management, remote access, runtime plugins, embedded privileged remote content, tray/autostart/daemon behavior, system-unit editing, a dashboard builder, or a terminal emulator.

## Assumptions

- The target Arch/GNOME machine can install a stable Tauri 2-compatible GTK/WebKit toolchain.
- A user D-Bus session normally exists; system-manager and journal access may be unavailable and must degrade honestly.
- A standard desktop external opener exists; its absence is a typed launcher health/error state.
- Foundation user-entered/persisted paths must be representable as UTF-8 contract strings. Non-UTF-8 launcher targets are excluded rather than converted ambiguously.
- The local OS user controls their XDG directories; SQLite audit is application-append-only, not tamper-proof against that user.
- Multiple simultaneous Argos processes are not coordinated in the foundation. SQLite locking, short transactions, and launcher revisions protect integrity; cross-process live UI synchronization is not promised.
- Launcher arguments and targets can contain personal/sensitive values, so logs, audits, and safe exports redact them by default.
- systemd D-Bus additions/unknown enum strings are forward-compatible inputs, not fatal assumptions.

## Open decisions

There are no unresolved decisions that materially block or alter foundation implementation. Exact stable dependency versions, the compatible Rust-to-TypeScript generator crate, target-tested Arch package names, and eventual distribution/signing channel are implementation-time compatibility or post-foundation operational selections constrained by the accepted architecture.

## Documentation-phase consistency record

Reviewed on 2026-07-30 before implementation:

- all 18 Markdown files under `docs/` and their heading anchors resolve;
- all documents use **Argos**, production namespace `argos`, development namespace `argos-dev`, temporary test roots, and the same `ARGOS_HOME` model;
- XDG category ownership, repository exclusion, missing-runtime behavior, and development-to-production acknowledgement agree;
- systemd foundation behavior is everywhere explicitly scoped and read-only, with no mutation task/capability;
- module enablement is separate from health, disabled modules leave primary navigation, and unhealthy enabled modules remain explainable;
- Rust owns trusted operations/contracts and React has one typed API/Tauri transport boundary;
- launcher side effects use saved IDs, literal argument arrays, no shell, and correlated attempt/outcome audit records;
- foundation exclusions match the product definition and every feature specification;
- the ledger contains 41 unique owned tasks and the seven specifications contain 71 unique acceptance criteria with traceability ranges;
- the definition of done maps to automated, target-system, security, accessibility, performance, and documentation evidence;
- no application scaffold, migration, manifest, lockfile, generated binding, or runtime data was created during D0.

## Current implementation task

`FND-BST-001` through `FND-BST-006`, `FND-DAT-001` through `FND-DAT-002`, and `FND-SHL-001` through `FND-SHL-004` are complete. They provide the build/lifecycle profile boundary, safe runtime/configuration path, persisted themes, shared states/dialog, and matching lazy backend/frontend registries needed by the first feature. No later foundation task is selected by this status update.
