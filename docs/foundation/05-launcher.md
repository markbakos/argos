# FND-05 — Minimal launcher module

**Status:** Approved for foundation implementation  
**Depends on:** FND-02 schema/repositories, FND-03 module shell/registry  
**Integrates with:** FND-06 diagnostics/audit visibility

## Problem and user value

Argos needs one persistent write-and-side-effect slice to prove Rust-owned storage and safe process boundaries. A launcher is useful immediately, but a generic command runner would make the WebView dangerously powerful.

The user gains a Lutris-like tile/list collection of saved URLs, folders, and executables that survives restarts/rebuilds while preserving literal arguments and explicit targets.

## Workflows

1. The user views favorites first in a bounded tile/list view and pages/searches within saved items using supported sort/filter requests.
2. The user creates a URL, folder, or executable item with kind-appropriate fields and optional subtitle/icon reference.
3. The user edits an item; a stale revision produces a conflict prompt/refetch instead of overwriting newer data.
4. The user toggles favorite state and sees deterministic persisted ordering.
5. The user deletes an item only after a dialog names the exact title; the deletion is audited.
6. The user opens a saved HTTPS URL or absolute folder using the desktop handler, or directly spawns a saved executable with its arguments as literal separate values.
7. A target changed since saving produces a precise error without deleting the launcher record.

## Functional requirements

### CRUD and presentation

- Implement list/get/create/update/favorite/delete use cases over `LauncherRepository` with the schema, bounds, pagination, timestamps, UUIDs, and revisions in [Data and contracts](../data-and-contracts.md#launcher_items).
- Trim/normalize titles for validation and sort while preserving intended display text; reject blank or over-limit fields.
- Enforce kind-specific fields: URL/folder have no arguments/working directory; executable accepts argument array and optional working directory.
- Return field-specific validation details without exposing SQL or platform debug data.
- Default list order places favorites first and remains stable by normalized title/ID; support bounded page size. An explicit sort request may use update time.
- Provide tile and list presentation with favorite control, accessible action menu, empty/loading/error states, and kind labels not conveyed by icon/color alone.
- Require ID plus expected revision for update/favorite/delete. Conflict offers refetch/reapply; it never silently retries a stale destructive operation.
- Confirmation for delete displays exact item title and kind. Delete does not delete shared/icon assets in foundation.

### Target validation

- URL targets parse as absolute `http` or `https` URLs; reject credentials in URLs for foundation storage and warn/reject disallowed schemes.
- Folder targets and working directories are UTF-8 absolute Linux paths after lexical validation. At launch, folder must exist and be a directory; working directory must exist and be a directory.
- Executable target is either an absolute path or a basename containing no path separator. Absolute target must be a regular executable accessible to the user at launch.
- Basename resolution searches only the ordered explicit/default validated executable search paths; never current directory, shell aliases/functions, or arbitrary request-provided paths.
- Reject NULs and enforce argument count/item/total bounds both on request and loaded data.

### Launch/open

- Load the saved record by ID in Rust, revalidate it, derive a structured execution plan, and call a narrow `LauncherExecutor` port. The frontend cannot submit a different target to the launch operation.
- URL/folder open uses a fixed external desktop opener adapter with one target argument and no embedded WebView.
- Executable launch passes executable and argument vector separately, optional working directory separately, and an explicit empty environment-override map. No shell/terminal or concatenated command line is involved.
- Use a documented inherited-environment policy suitable for desktop processes; never log/audit it. Foundation has no environment editor.
- Do not retain unbounded stdout/stderr. The launch result means the opener/process was accepted/spawned, not that the child later succeeded.
- Launched applications may outlive Argos without creating an Argos daemon. Closing Argos leaves no Argos process; normal launched external applications are not Argos processes.

### Audit

- Classify create/update/favorite as `write`, delete as `destructive`, and launch/open as `write` external side effects.
- Database mutations and audit events are atomic. Before launch/open, commit an `attempted` audit row; if that fails, do not execute. After the operating-system attempt, append `succeeded` or `failed` with the same correlation ID. A post-action audit failure is returned as a typed partial-failure stating the side effect may have occurred and is also written to bounded structured diagnostics.
- Audit metadata contains item kind and safe outcome only; it omits target, URL query, arguments, working directory, environment, and helper output.

## Non-functional requirements

- Launcher data lives only in the profile data database and survives application restart, rebuild, dependency reinstall, branch switch, repository relocation, and repository deletion/reclone.
- Input and query sizes are bounded; list rendering remains responsive with a realistic personal collection.
- React uses TanStack Query invalidation and does not keep an independent persistent store.
- No execution occurs from list preview, hover, startup, or background refresh—only an explicit control.

## Failure and recovery states

| Failure | Required behavior |
| --- | --- |
| Validation failure | field-specific `VALIDATION_`/`LAUNCHER_` error; draft retained; no row/audit mutation |
| Stale revision | `LAUNCHER_CONFLICT`; refetch/reapply option; delete remains unperformed |
| Record missing | `LAUNCHER_NOT_FOUND`; invalidate list and return safely |
| Target missing/wrong type | precise process error; item retained for editing |
| Permission/not executable | distinguish permission/not-executable; no elevation |
| Desktop opener absent/fails | `PROCESS_OPEN_FAILED`/targeted details; item retained |
| Spawn fails | `PROCESS_SPAWN_FAILED`; failed attempt audited without sensitive fields |
| Audit insert fails for DB mutation | whole database mutation rolls back |
| Pre-action audit fails | do not open/spawn; return storage/audit failure |
| Outcome audit fails after side effect | report partial failure with `side_effect_may_have_occurred`; retain pre-action audit/correlation and structured diagnostic |
| Database unavailable | read/write unavailable state; shell/diagnostics remain usable |
| Corrupt loaded JSON/record | safe internal/storage error with correlation ID; no execution |

## Explicit exclusions

No shell launcher, terminal, environment editor, tags, collections, workspace/multi-step actions, process tracking, child lifecycle dashboard, icon import/editor, local-service/Docker status, network metadata fetching, agents, or full-text search is included.

## Architecture impact

Launcher domain values and use cases live in domain/application modules; SQLite implements persistence; Linux platform implements the opener/process port; Tauri translates narrow CRUD/launch commands; React uses only the semantic launcher API.

## Contracts

Contracts include `LauncherItemId`, `LauncherKind`, item summary/detail, create/update/favorite/delete requests with revision, bounded list filter/sort/page, field validation details, and `LaunchOutcome` (`opened` or `spawned`, with no process-control handle). `launch_saved_item` accepts only the item ID, not executable/arguments.

## Persistence and migrations

Uses initial `launcher_items` and `audit_events`; no extra migration. Arguments are the only foundation launcher JSON structure. Optional `icon_reference` accepts a validated theme/Argos-managed reference but foundation provides no import workflow. All mutations increment revision/update UTC and use optimistic checks.

## Security implications

Revalidate persisted data before side effects. Restrict schemes/search paths, pass literal argv, avoid current-directory lookup, assign action classes in Rust, and redact sensitive values. Tauri separates launcher read/write/execute permissions. Remote content receives none.

## Performance implications

Use bounded cursor pages and indexed ordering. Do not check every filesystem target while listing; validate existence/executability when opening detail on demand or launching. Do not capture child output or poll processes.

## Acceptance criteria

- **FND-05-AC01:** CRUD/favorite persists exact valid structured fields with UUID/UTC/revision behavior and bounded stable pagination.
- **FND-05-AC02:** Validation tests cover each kind, disallowed URL schemes/credentials, absolute paths, executable basenames, NULs, argument bounds, and incompatible fields.
- **FND-05-AC03:** Stale updates/favorites/deletes conflict without overwriting; deletion confirmation identifies the exact record.
- **FND-05-AC04:** Database mutations and redacted audit events commit/roll back atomically; external actions have correlated pre-attempt/outcome records, never run without the pre-record, and report post-action audit partial failure.
- **FND-05-AC05:** Process-plan tests prove no shell/terminal, no concatenation, literal metacharacter/space arguments, fixed search paths, and empty environment overrides.
- **FND-05-AC06:** URL/folder/executable target failures remain typed and retain the record; successful outcomes claim only opened/spawned.
- **FND-05-AC07:** Tile/list UI covers loading/empty/error/conflict/confirmation and is keyboard accessible.
- **FND-05-AC08:** Target smoke opens HTTPS/folder and starts a harmless executable, while safe logs/audits/exports contain no target/arguments.
- **FND-05-AC09:** A saved item survives close/reopen, rebuild, branch/source relocation, and cache deletion with data outside the repository.
- **FND-05-AC10:** Static capability/API scans find no arbitrary target launch, generic process/file/shell operation, or remote-content privilege.

## Testing strategy

Use domain table/property tests, fake-repository/use-case transaction tests, SQLite integration/revision/page tests, fake and real process-plan adapter tests with harmless fixtures, mocked frontend API behavior, accessibility interaction tests, redaction scans, and target-machine external-handler smoke checks.

## Implementation order and tasks

1. `FND-LCH-001` — launcher domain, validation, contracts, fake use cases.
2. `FND-LCH-002` — SQLite CRUD/pagination/revision and atomic audit.
3. `FND-LCH-003` — safe opener and structured executable adapter.
4. `FND-LCH-004` — narrow Tauri/API launcher boundary and capabilities.
5. `FND-LCH-005` — launcher tile/list/forms/dialogs and Query flows.
6. `FND-LCH-006` — persistence, redaction, execution, and target smoke proof.

## Verification and documentation update

Run all FND-05 criteria and data-location/security scans. Document the actual inherited environment/search-path/opener behavior after target testing. Any new scheme, shell behavior, environment persistence, or arbitrary target operation requires a preceding specification/security update and likely a decision record.
