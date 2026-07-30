# Data, runtime paths, and contracts

## Ownership rule

Rust owns path resolution, configuration I/O, SQLite access, migrations, backups, repositories, and cross-boundary validation. React never receives the SQLite path, a raw SQL interface, generic file access, or a way to select a storage root.

No profile derives live paths from the process working directory, executable directory, Cargo manifest directory, or repository. The repository contains migration source files and fixtures, never migrated databases or runtime artifacts.

## Runtime profiles and path resolution

The application has exactly three foundation profiles:

| Profile | Namespace/source | Intended use |
| --- | --- | --- |
| `production` | XDG namespace `argos` | Distributed desktop application and deliberate production verification |
| `development` | XDG namespace `argos-dev` | Normal builds and `tauri dev` from source |
| `test` | Injected temporary root | Automated tests only |

Ordinary source builds, including local optimized builds, embed `development` as the default. Only the release packaging task explicitly embeds `production`. A development binary may select production only when both `ARGOS_PROFILE=production` and the exact acknowledgement `ARGOS_ACKNOWLEDGE_PRODUCTION_DATA=argos-production` are present. That mode emits a prominent startup warning, displays a persistent diagnostics/settings warning, and is covered by tests. One variable without the other fails closed. A packaged production binary does not need the acknowledgement for its embedded default.

Tests construct a test profile with a fresh temporary-directory handle; they do not rely on the developer's XDG environment. The temporary handle lives for the test duration and cleanup occurs only after all connections close.

### XDG resolution

When no explicit development root is active, paths resolve as follows:

| Category | Production | Development | Content |
| --- | --- | --- | --- |
| Configuration | `${XDG_CONFIG_HOME:-$HOME/.config}/argos/` | `${XDG_CONFIG_HOME:-$HOME/.config}/argos-dev/` | `config.toml`, small bootstrap preferences |
| Persistent data | `${XDG_DATA_HOME:-$HOME/.local/share}/argos/` | `${XDG_DATA_HOME:-$HOME/.local/share}/argos-dev/` | SQLite, backups, imported icons, exports, future managed-resource references |
| Persistent state | `${XDG_STATE_HOME:-$HOME/.local/state}/argos/` | `${XDG_STATE_HOME:-$HOME/.local/state}/argos-dev/` | logs, diagnostics, window/recovery state |
| Disposable cache | `${XDG_CACHE_HOME:-$HOME/.cache}/argos/` | `${XDG_CACHE_HOME:-$HOME/.cache}/argos-dev/` | rebuildable cached data only |
| Runtime | `$XDG_RUNTIME_DIR/argos/` | `$XDG_RUNTIME_DIR/argos-dev/` | future sockets/coordination; not required by foundation |

If an XDG category variable is unset, only the standard HOME fallback shown above is used. Per the XDG specification, a relative category value is invalid: config/data/state/cache use the HOME fallback with a diagnostic, while a relative/missing runtime value makes runtime unavailable. An unset `HOME` when a fallback is needed is `CONFIG_HOME_UNAVAILABLE`, not a current-directory fallback. Argos never substitutes `/tmp`.

For source builds, every resolved category—including explicit XDG values—is checked against the known repository root before directory creation and rejected if equal to or contained by it (including resolved symlink containment where applicable). Packaged builds do not embed a source-repository path and never derive one; explicit XDG environment choices remain visible inputs, not defaults.

### Explicit `ARGOS_HOME`

`ARGOS_HOME=/absolute/path` is a development/testing override. In development it resolves:

```text
/absolute/path/config/
/absolute/path/data/
/absolute/path/state/
/absolute/path/cache/
/absolute/path/runtime/
```

It takes precedence over XDG only after validation. It must be explicit, absolute, non-empty, and not the repository root or any descendant of it. Relative paths, lexical traversal, unsafe symlink resolution, wrong ownership, and collision among categories fail before database open. The source-build composition root supplies its known repository root to validation. The override never silently defaults, is rejected by an embedded production profile, and cannot be combined with the development-to-production acknowledgement mode.

Tests normally inject a temporary root directly rather than mutating process-global `ARGOS_HOME`, but exercise equivalent validation behavior.

### Directory and file policy

Required config/data/state/cache directories are created lazily by the owning service with user-only permissions where supported. Runtime is created only when a feature needs it. Cache deletion cannot lose a record or prevent recovery. Imported icons are copied into the data directory and addressed by an Argos icon reference, not a repository path.

Planned production layout:

```text
config/config.toml
data/argos.sqlite3
data/backups/automatic/
data/icons/
data/exports/
state/logs/
state/diagnostics/
state/recovery/
cache/<rebuildable entries>
runtime/<future only>
```

SQLite `-wal` and `-shm` sidecars live beside the database and are live data. They receive the same repository-exclusion checks.

## Bootstrap configuration

`config.toml` contains only small application-level preferences:

- theme preference: `system` (default), `light`, or `dark`;
- optional explicit executable search paths;
- small startup/window behavior preferences added through a migration-like config version;
- visibly named development overrides that do not weaken profile checks.

User-created launcher/module/task records, audit events, imported content, and large settings do not belong in this file. Writes use an atomic temporary-file, sync/replace policy in the same config directory and preserve a last-known valid file on validation failure. Invalid configuration yields structured diagnostics and safe defaults only for non-security-sensitive preferences; unsafe paths fail closed.

Executable search paths are absolute directories, ordered, de-duplicated, and validated. Defaults may be derived from a documented desktop environment policy at bootstrap, but the current directory is never searched. React cannot add a one-off path during launch.

## SQLite lifecycle

### Initialization

The storage adapter creates or opens `data/argos.sqlite3` only after profile/path validation. It uses a small bounded pool (initial maximum four connections) and applies per-connection settings:

```text
PRAGMA foreign_keys = ON
PRAGMA busy_timeout = 5000
PRAGMA journal_mode = WAL       (normal file-backed profiles)
PRAGMA synchronous = NORMAL     (with WAL)
```

Initialization verifies the effective values and reports them through storage health. Tests may use another journal mode only when a platform limitation is demonstrated and the test explicitly records that choice. A busy timeout reduces transient contention; it does not authorize long transactions or indefinite retry.

### Transaction ownership

Application use cases define business transaction boundaries. The storage crate controls connection/transaction mechanics and repository implementations. A repository never commits a transaction it did not start. Read-only use cases use short snapshots; database mutations and their application-level audit record share one transaction when no external side effect prevents it.

### Migrations

Migrations are ordered immutable SQL assets embedded in `argos-storage-sqlite`. Their source lives in the repository; applied state lives only in each database. On open, the adapter:

1. opens and verifies the existing database readably;
2. identifies current and target schema versions/checksums;
3. creates a consistent automatic backup when an existing database has pending migrations;
4. applies pending migrations in a transaction (or the migration tool's strongest verified transactional unit);
5. runs foreign-key/integrity and expected-version checks;
6. exposes the database to application repositories only after success.

Migration failure rolls back, records a redacted state diagnostic outside the database, preserves the original database and backup, and starts only a restricted recovery/diagnostics path. Argos does not delete/recreate a failed database, mark a failed migration as applied, or automatically downgrade. Changing an already released migration is forbidden; add a new forward migration.

### Backups and recovery

Automatic pre-migration backups use SQLite's consistent backup API rather than copying an active database file. A name includes UTC timestamp, source schema version, and database identity. Backups live in `data/backups/automatic/`, use private permissions, and are verified openable before migration begins. Retention keeps the five most recent verified automatic backups; rotation occurs only after a new verified backup exists. User exports/manual backups, when added, are never deleted by automatic rotation.

Restoration is not automatic. The diagnostics/recovery flow reports the database and backup paths and an actionable error without exposing them to ordinary frontend feature contracts. A future restore command requires its own destructive confirmation specification. Foundation recovery documentation may instruct a human to preserve files, but the application does not overwrite the live database.

## Foundation schema

All public IDs are lowercase canonical UUID strings generated in Rust (UUID v4 in the foundation). UTC timestamps are normalized RFC 3339 strings with millisecond precision and a `Z` suffix. Frontends localize only for display. Database row numbers, D-Bus object paths, and filenames are never public resource identities.

Domain validation is authoritative. SQL checks defend against corruption but do not replace it. JSON fields are permitted only for bounded structures that are read/written as a whole.

### `module_preferences`

| Column | Constraint/purpose |
| --- | --- |
| `module_id` | Text primary key matching a compiled manifest ID |
| `enabled_override` | Nullable boolean; null means manifest default |
| `display_order_override` | Nullable bounded integer; null means manifest order |
| `settings_json` | Nullable JSON object, maximum 64 KiB, validated by the owning module |
| `updated_at_utc` | Normalized UTC timestamp |

This table stores overrides only. Display names, versions, routes, capabilities, dependencies, health, and defaults remain code-owned. Unknown preferences are retained but ignored/reported, allowing a temporarily removed module to return without data loss. Effective-order tie-breaking is manifest order then module ID, making navigation deterministic.

### `launcher_items`

| Column | Constraint/purpose |
| --- | --- |
| `id` | UUID text primary key |
| `kind` | `url`, `folder`, or `executable` |
| `title` | Trimmed non-empty text, maximum 200 Unicode scalar values |
| `subtitle` | Nullable text, maximum 500 values |
| `target` | Kind-specific text, maximum 4096 bytes |
| `arguments_json` | JSON string array, default `[]`, at most 128 entries, 4096 bytes each, 64 KiB total |
| `working_directory` | Nullable absolute path, maximum 4096 bytes; executable kind only |
| `icon_reference` | Nullable Argos-managed or theme icon reference, not arbitrary file access |
| `is_favorite` | Boolean |
| `revision` | Positive integer incremented on update for optimistic concurrency |
| `created_at_utc` | Normalized UTC timestamp |
| `updated_at_utc` | Normalized UTC timestamp |

Indexes support favorite/title ordering and updated-time pagination. URL/folder kinds require an empty argument list and no working directory. URL target scheme is `http` or `https`; folder and absolute executable targets are absolute. An executable basename contains no separator and is resolved through configured search paths. Targets are revalidated at launch because the filesystem can change.

List uses bounded cursor pagination (default 100, maximum 200) with a stable `(is_favorite, normalized title, id)` or updated-time cursor selected by the request's documented sort. Create returns the new resource. Update/delete require ID and expected `revision`; conflicts return `LAUNCHER_CONFLICT` rather than overwriting another edit.

### `audit_events`

| Column | Constraint/purpose |
| --- | --- |
| `id` | UUID text primary key |
| `occurred_at_utc` | Normalized UTC timestamp |
| `initiator_kind` | `human`, future `cli`, `agent`, or `automation` |
| `initiator_id` | Stable actor identifier; foundation local-human constant |
| `module_id` | Stable module/core identifier |
| `action` | Stable action name |
| `classification` | `write`, `privileged`, or `destructive` for persisted mutation/side-effect audits |
| `target_type` | Stable resource type |
| `target_id` | Stable resource identity where available |
| `target_display_name` | Bounded human aid, not identity |
| `result` | `attempted`, `succeeded`, `failed`, or `rejected` |
| `error_code` | Nullable stable application error code |
| `metadata_json` | Key-allowlisted JSON object, maximum 16 KiB |
| `correlation_id` | Request/trace correlation identifier |

Indexes support time, module/time, actor/time, and target/time queries. Foundation application repositories expose insert and bounded diagnostics reads only—no update or delete. Read operations are not persistently audited. Database administration by the owning OS user remains outside application guarantees.

## Repository interfaces

Application ports are domain-specific, for example:

- `ModulePreferenceRepository` — read overrides and write one effective override;
- `LauncherRepository` — bounded list/get/insert/update/delete with revision checks;
- `AuditRepository` — append and bounded recent diagnostic reads;
- `UnitReader` and `JournalReader` — scoped read models, not storage abstractions;
- `LauncherExecutor` — open/spawn a validated saved-item execution plan;
- `TransactionManager` — run a closure/use-case unit against repositories sharing one transaction.

Repositories accept domain values, not SQL fragments, table names, JSON blobs from React, or database paths.

## Cross-boundary contracts

### Source and generation

Serializable boundary types live in `argos-contracts` and derive Serde plus the selected TypeScript export metadata. The deterministic planned command is:

```text
pnpm contracts:generate
```

It delegates to the Rust `xtask`, writes only `apps/desktop/src/generated/`, formats deterministically, and includes a generated-file warning. CI runs generation and fails if `git diff --exit-code -- apps/desktop/src/generated` is non-empty. Contract tests also round-trip representative JSON, including every enum variant and error shape. Generated files are committed so frontend type checking does not depend on generator execution, but they are never manually edited.

The compatible generator library is selected in the bootstrap task after verifying Tauri 2, Serde tagging, map/optional semantics, and deterministic output. That library choice is an implementation dependency, not permission to change Rust source-of-truth or the API facade.

### Contract families

Contracts cover:

- branded/string resource identifiers (`LauncherItemId`, module and actor IDs);
- `PageRequest`, opaque cursor, bounded limits, and `Page<T>` where lists need pagination;
- application/build/profile information and resolved path diagnostics (diagnostics only);
- module manifests/effective states, requirements, capabilities, and health reasons;
- launcher requests, item views, kinds, validation results, and launch outcomes;
- explicit `SystemdScope`, unit kind/state summaries, timer trigger data, details, journal entries, filters, and health;
- actions and audit classifications;
- typed events used for query invalidation;
- the stable application error contract.

Contracts are purpose-specific views, not serialized database entities. Sensitive fields are omitted rather than annotated as hidden in React.

### Events

Foundation events are hints, not authoritative state dumps:

- `ModuleHealthChanged { module_id }`;
- `SystemdChanged { scope, affected_kinds, unit_names? }` with a strict bounded name list;
- `SettingsChanged { category }` when a backend-side change must invalidate UI state.

The frontend API subscribes, validates, coalesces, and invalidates relevant Query keys. Consumers refetch authoritative snapshots. Subscriptions are reference-counted and released on route/window teardown.

## Stable error contract

Every frontend-visible failure has:

```text
code: stable namespaced identifier
message: safe human-readable summary
details?: bounded structured object
retryable: boolean
correlation_id: opaque diagnostic reference
```

The root namespaces are `CORE_`, `CONFIG_`, `STORAGE_`, `MODULE_`, `SYSTEMD_`, `PROCESS_`, `LAUNCHER_`, `PERMISSION_`, and `VALIDATION_`. Initial codes include, at minimum:

```text
CORE_INTERNAL, CORE_CANCELLED
CONFIG_HOME_UNAVAILABLE, CONFIG_INVALID, CONFIG_PATH_UNSAFE,
CONFIG_PRODUCTION_ACK_REQUIRED
STORAGE_UNAVAILABLE, STORAGE_BUSY, STORAGE_INTEGRITY_FAILED,
STORAGE_MIGRATION_FAILED, STORAGE_AUDIT_FAILED
MODULE_DUPLICATE, MODULE_DEPENDENCY_INVALID, MODULE_DISABLED,
MODULE_UNAVAILABLE
SYSTEMD_BUS_UNAVAILABLE, SYSTEMD_MANAGER_UNAVAILABLE,
SYSTEMD_PERMISSION_DENIED, SYSTEMD_TIMEOUT, SYSTEMD_UNIT_NOT_FOUND,
SYSTEMD_JOURNAL_UNAVAILABLE, SYSTEMD_JOURNAL_PARSE_FAILED
PROCESS_TARGET_NOT_FOUND, PROCESS_NOT_EXECUTABLE, PROCESS_OPEN_FAILED,
PROCESS_SPAWN_FAILED
LAUNCHER_NOT_FOUND, LAUNCHER_CONFLICT, LAUNCHER_KIND_UNSUPPORTED
PERMISSION_DENIED
VALIDATION_REQUIRED, VALIDATION_INVALID_FORMAT, VALIDATION_OUT_OF_RANGE
```

`retryable` is assigned by the backend based on cause; the UI does not infer it from namespace. Details use code-specific allowlisted fields such as `field`, `scope`, `module_id`, safe target display value, or `side_effect_may_have_occurred` for a failed post-action audit. Rust source/debug strings, SQL, command output, and arbitrary paths are never the user message. Internal tracing retains a redacted cause chain keyed by `correlation_id`.

The frontend switches on codes or typed detail discriminants and has a generic fallback for unknown future codes. It never parses message text.
