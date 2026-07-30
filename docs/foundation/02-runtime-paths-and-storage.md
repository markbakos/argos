# FND-02 — Runtime paths and storage

**Status:** Approved for foundation implementation  
**Depends on:** FND-01 typed errors/core boundaries  
**Enables:** persisted module preferences, launcher, audit, diagnostics

## Problem and user value

Argos cannot be trusted if a development build opens production data or if rebuilding the repository can delete personal records. It also needs durable schema evolution without letting React or Tauri handlers become database owners.

The user gains predictable XDG storage, hard development/test isolation, recoverable migrations, and data that survives source-tree changes.

## Workflows

1. A normal source build resolves `argos-dev`; a packaged build resolves `argos`; diagnostics can later display both profile and category paths.
2. A test creates a fresh temporary root and database without reading any user XDG directory.
3. A developer deliberately supplies an external `ARGOS_HOME` and receives isolated `config/data/state/cache/runtime` children.
4. First launch creates a migrated database. A later schema upgrade creates a verified backup, migrates transactionally, and opens only on success.
5. A migration failure preserves the original/backup and allows restricted diagnostic recovery instead of resetting data.

## Functional requirements

- Implement a pure/injectable path resolver for production, development, test, and explicit development root exactly as specified in [Data and contracts](../data-and-contracts.md#runtime-profiles-and-path-resolution), including invalid relative XDG values and repository checks for every source-build category.
- Reject relative/empty/unsafe roots, repository containment, category collisions, missing required HOME, and incomplete production acknowledgement. Report missing runtime without `/tmp` fallback.
- Create category directories lazily with private permissions and never use the repository/current executable/current working directory as storage.
- Load and atomically write versioned small `config.toml` preferences; validate theme and executable search paths.
- Initialize one Rust-owned SQLite database under the data directory with foreign keys, 5000 ms busy timeout, WAL and verified settings.
- Embed immutable ordered migrations, create/verify a consistent pre-migration backup for existing databases, apply transactionally, run integrity/version checks, and fail closed.
- Implement the foundation tables/constraints/indexes for module preferences, launcher items, and append-only audit events from [Data and contracts](../data-and-contracts.md#foundation-schema).
- Implement domain-specific repositories, a transaction manager, cursor pagination, optimistic launcher revisions, and audit append.
- Keep automatic backup retention at five verified files without touching future manual backups.
- Expose storage/profile/config through application ports and diagnostics models, never a generic database/path command.
- Provide restricted storage health/migration status even when ordinary repositories cannot open.

## Non-functional requirements

- No initialization, test, logging, migration, backup, sidecar, or recovery path writes under the repository.
- Tests are deterministic and parallel-safe through injected environment/path inputs rather than process-global mutation where possible.
- Transactions are short; pool maximum begins at four and changes only with measurement.
- Cache is fully disposable and contains no source of truth.
- Config/database/backup files are private to the user where supported.

## Failure and recovery states

| Failure | Required behavior |
| --- | --- |
| Missing HOME for required fallback | `CONFIG_HOME_UNAVAILABLE`; no directory created |
| Missing `XDG_RUNTIME_DIR` | runtime unavailable health; foundation otherwise continues |
| Unsafe/repository `ARGOS_HOME` | `CONFIG_PATH_UNSAFE`; no category/database created |
| Partial production acknowledgement | `CONFIG_PRODUCTION_ACK_REQUIRED`; no production path opened |
| Invalid non-security config preference | report diagnostic, use documented safe default, preserve invalid file for user correction |
| Invalid executable search path | fail that setting closed and report field error; do not search it |
| Database busy | wait at most configured timeout, return retryable `STORAGE_BUSY` |
| Corruption/integrity failure | restricted recovery health; no destructive recreation |
| Backup creation/verification failure | do not start pending migration |
| Migration failure | rollback, preserve database/backup, record state diagnostic, ordinary writes unavailable |
| Newer schema/downgrade request | refuse open for writes; no automatic downgrade |

## Explicit exclusions

No UI feature CRUD is implemented here, no automatic restore/reset/downgrade, no cloud sync, encryption/key management, multi-process coordination, database console, generic export, or production-data cleanup command.

## Architecture impact

`argos-platform-linux` owns path/config implementations; `argos-storage-sqlite` owns SQL/migrations/repositories. `argos-application` chooses transaction boundaries. Tauri only constructs and holds the resulting services.

## Contracts

Ordinary frontend contracts receive profile names and configuration views only through narrow settings/app-info use cases. Resolved filesystem paths and storage/migration details are diagnostics-only. Database paths never appear in launcher/module contracts. Error codes follow `CONFIG_`, `STORAGE_`, and `VALIDATION_` namespaces.

## Persistence and migrations

The initial migration creates `module_preferences`, `launcher_items`, `audit_events`, their checks/indexes, and the migration tool's metadata. Fixture databases cover empty and each future prior version. JSON columns enforce valid JSON/size defensively while Rust enforces semantic limits. Timestamps/UUIDs follow the shared normalization.

## Security implications

Validate repository containment before creating directories. Treat backups as sensitive database copies. Do not follow unsafe symlinks during atomic config/diagnostic writes. Never reveal arbitrary paths to ordinary feature APIs or log full config/database data. Development-to-production access is two-part and visibly reported.

## Performance implications

Resolve paths/config and migrate once at startup. Do not poll storage health or continually write access timestamps. WAL/checkpoint behavior and migration duration are measured; heavy future backfills require their own plan.

## Acceptance criteria

- **FND-02-AC01:** Path table cases produce exact `argos`, `argos-dev`, temporary, and explicit-root categories with no repository-derived default.
- **FND-02-AC02:** Unsafe roots, missing HOME, incomplete acknowledgement, collisions, and runtime absence produce the specified non-destructive behavior.
- **FND-02-AC03:** A full initialization/CRUD/logging/backup/diagnostics test produces no live-data pattern anywhere under the repository.
- **FND-02-AC04:** Development and production paths never overlap; tests neither read nor write either; normal source builds cannot open production.
- **FND-02-AC05:** Fresh initialization has the expected schema, indexes, foreign keys, busy timeout, and normal WAL mode.
- **FND-02-AC06:** Upgrade creates a verified backup then migrates; injected failure rolls back and blocks ordinary writes without deleting data.
- **FND-02-AC07:** Six successful automatic backups rotate only the oldest after the newest verifies; manual/unrecognized files remain.
- **FND-02-AC08:** Repository tests prove CRUD, bounded pages/JSON, revision conflicts, foreign-key/integrity behavior, and atomic database mutation plus audit.
- **FND-02-AC09:** Cache deletion loses no launcher/module/audit source of truth and reinitializes safely.
- **FND-02-AC10:** No frontend/Tauri command can obtain a database path or execute generic SQL/file work.

## Testing strategy

Use table-driven pure path tests, temporary filesystem permission/symlink cases, repository snapshots, SQLite integration fixtures, migration fault injection, backup open/integrity checks, pool contention tests, property tests for bounded JSON/value validation, and static SQL-boundary checks. Avoid user XDG paths entirely in automated tests.

## Implementation order and tasks

1. `FND-DAT-001` — runtime-profile selection and pure path resolution.
2. `FND-DAT-002` — safe directory/config service.
3. `FND-DAT-003` — SQLite connection and health bootstrap.
4. `FND-DAT-004` — migrations, backup, retention, and failure recovery.
5. `FND-DAT-005` — initial schema and repositories.
6. `FND-DAT-006` — isolation/repository-write prevention suite.

## Verification and documentation update

Run FND-02 criteria and the data-location proof in [Verification](../verification.md#data-location-and-isolation-proof). Update exact config schema/migration IDs and target-tested permission behavior; any storage deviation requires updating ADR-005/006 before code.
