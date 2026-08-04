# Feature implementation task ledger

## Use of this ledger

Feature tasks are stable, dependency-ordered implementation units. A task includes implementation, the expected-behavior tests it owns, documentation synchronization, and its completion condition.

**Implementation status:** `TMG-001` selected and in progress; `TMG-002` through `TMG-005` pending. TM-01 was approved by the user on 2026-08-05. This ledger does not change or select work in the [foundation ledger](../foundation/tasks.md).

## TM-01 — Task Manager

| ID        | Depends on                                              | Affected parts and deliverable                                                                                                                                                                                      | Tests, docs, and completion condition                                                                                                                                                                                                                           |
| --------- | ------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `TMG-001` | Explicit TM-01 approval; FND-BST-004                    | Task Manager domain values/read port, application snapshot/delta policy, typed contracts/errors, fixture-root fake use cases, deterministic search/sort/bounds. No Linux/Tauri/UI behavior.                         | Pure baseline/delta/reset/PID-reuse/memory/bounds/serialization tests for TM-01-AC03–05/07/11; update exact public contract semantics. **Done when all metric/rate policy runs without Tauri or live procfs and invalid requests cannot reach a reader.**       |
| `TMG-002` | TMG-001                                                 | Bounded read-only procfs/sysfs implementation in `argos-platform-linux`, aggregate/process parsers, on-demand details, partial/permission/exit behavior, and static-source reuse. No new crate or generic file API. | Synthetic fixture-root parser/source/read-budget/privacy tests for TM-01-AC03–07/09/11; document exact supported kernel fields. **Done when one bounded adapter returns correct partial snapshots/details and never reads excluded sources.**                   |
| `TMG-003` | TMG-001, TMG-002, FND-BST-006, FND-SHL-003, FND-SHL-004 | Backend manifest/service composition, narrow Tauri read permissions/handlers, generated bindings, semantic frontend API/query keys, lazy frontend module registration.                                              | Registry parity/lazy-load, Tauri translation/IPC, API decoder/error/cancellation, capability/import tests for TM-01-AC01/02/07/09. **Done when the main window can request typed reads only and every non-Task-Manager route performs zero feature work.**      |
| `TMG-004` | TMG-003, FND-SHL-002                                    | Processes and Performance UI, fixed active-only cadence/visibility teardown, summary/table/search/sort/detail, 30-point lightweight charts, all accessible states.                                                  | Mocked timers/visibility/API, non-overlap/cache teardown, interaction/accessibility/reduced-motion tests for TM-01-AC02/05/06/08/11. **Done when the clean UI exposes every in-scope metric and fake timers prove sampling exists only for the visible route.** |
| `TMG-005` | TMG-004                                                 | Target Arch/GNOME correctness, privacy/capability review, inactive/active CPU and snapshot timing, 30-minute memory/cache/write observation, clean-close evidence and final docs sync.                              | Execute TM-01-AC09/10/12 plus full required repository gate; record redacted environment/process-count/results. **Done when every TM-01 criterion has passing evidence, budgets pass honestly, and the specification matches the implementation.**              |

## Acceptance traceability

| Specification criteria | Primary task evidence                          |
| ---------------------- | ---------------------------------------------- |
| TM-01-AC01–AC02        | TMG-003–TMG-004, finalized by TMG-005          |
| TM-01-AC03–AC05        | TMG-001–TMG-002, surfaced by TMG-004           |
| TM-01-AC06–AC07        | TMG-002–TMG-004                                |
| TM-01-AC08             | TMG-004, finalized by TMG-005                  |
| TM-01-AC09             | TMG-002–TMG-003, finalized by TMG-005          |
| TM-01-AC10             | TMG-005                                        |
| TM-01-AC11             | TMG-001, TMG-003–TMG-004, finalized by TMG-005 |
| TM-01-AC12             | TMG-005                                        |
