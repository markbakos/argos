# Development guide

## Phase gate

The foundation documentation was approved for implementation on 2026-07-30. The Cargo/pnpm/Tauri scaffold began with `FND-BST-001`; commands below become available when their owning bootstrap task implements and verifies them.

Implementation follows the approved specifications and [dependency-ordered task ledger](foundation/tasks.md). A later task must not begin before its dependencies pass.

## Local prerequisites

The target development environment is Arch Linux with GNOME/systemd. Contributors will need:

- a stable Rust toolchain with `rustfmt` and `clippy`;
- a supported Node.js LTS and pnpm through Corepack;
- Cargo and the native GTK/WebKit/system libraries required by the selected stable Tauri 2 release;
- SQLite development/runtime support used by the selected Rust driver;
- a user D-Bus session and systemd user manager for optional real-system checks;
- Git and standard build tools.

The bootstrap task must record exact Arch package prerequisites after testing them on the target machine. Dependency versions belong in committed Cargo and pnpm lockfiles; architecture documents name compatibility ranges/concepts only.

### Tested bootstrap baseline

Validated on Arch Linux on 2026-07-30:

- Rust 1.97.1 through `rustup`, with `rustfmt` and Clippy pinned by `rust-toolchain.toml`;
- Node.js 24.15 or newer and pnpm 11.18.0, pinned by the root `packageManager` field; Node.js 24 LTS is the CI target and local bootstrap was checked with Node.js 26.5.0;
- Arch packages `webkit2gtk-4.1`, `base-devel`, `corepack`, `curl`, `wget`, `file`, `openssl`, `nodejs`, `npm`, and `sqlite`;
- Tauri 2.11, React 19, Vite 8, Tailwind CSS 4 through its first-party Vite plugin, and Lucide 1;
- `rusqlite` with bundled SQLite for deterministic application storage, `zbus` with the Tokio backend for systemd D-Bus, `ts-rs` for Rust-owned TypeScript generation, and Lucide for the lightweight icon set.

The clean-checkout bootstrap proof used `pnpm install --frozen-lockfile`, the desktop frontend build, and `cargo check --workspace --locked --offline`. The empty domain crate has no dependency, and Tauri appears only under `argos-desktop` in the Cargo graph. Generator, database, and D-Bus behavior remain owned by their later tasks.

## Command surface

The implemented root pnpm scripts delegate to workspace tools so contributors do not memorize crate paths:

| Command                                          | Current behavior                                                                                       |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------ |
| `pnpm install --frozen-lockfile`                 | Reproduce committed frontend/tool dependencies                                                         |
| `pnpm dev`                                       | Run the Tauri development app with embedded `development` profile                                      |
| `pnpm build`                                     | Type-check and build frontend plus Rust desktop artifacts without packaging                            |
| `pnpm tauri:build`                               | Produce a local development-profile optimized desktop build                                            |
| `pnpm format` / `pnpm format:check`              | Apply/check Prettier and rustfmt                                                                       |
| `pnpm lint`                                      | Run ESLint and Clippy with warnings denied                                                             |
| `pnpm test`                                      | Run Rust and frontend automated tests with isolated roots                                              |
| `pnpm test:rust`                                 | Workspace Rust unit/integration/doc tests                                                              |
| `pnpm test:web`                                  | Vitest and React Testing Library                                                                       |
| `pnpm boundaries:check` / `pnpm boundaries:test` | Enforce and test Rust dependency and frontend Tauri-import boundaries                                  |
| `pnpm check`                                     | Run formatting, lint, typecheck, Rust/frontend/boundary tests, boundary scan, and frontend/Rust builds |

`pnpm package` and `pnpm contracts:generate` remain reserved contracts until `FND-BST-006` and `FND-BST-004` implement them. Migration, contract-drift, and repository-live-data gates join `pnpm check` in their owning tasks. Commands fail with actionable prerequisite errors and do not access production data at runtime.

## Quality gates

Every change must pass:

1. rustfmt check;
2. Clippy for all targets/features with warnings denied;
3. strict TypeScript build with no `any` escape;
4. ESLint, including restricted Tauri imports and module-boundary rules;
5. Prettier check;
6. Rust and frontend tests;
7. migration and contract-generation drift checks when applicable;
8. a repository live-data scan;
9. relevant acceptance criteria and documentation links.

GitHub Actions runs the implemented baseline through `pnpm check` on Ubuntu 24.04, Node.js 24, pnpm 11.18.0, and Rust 1.97.1 using committed lockfiles and pnpm's frozen mode. Later task-owned gates become mandatory when implemented. Most CI does not require a live systemd manager. Optional/target-machine systemd checks are clearly separate and do not make unit tests flaky.

## Coding conventions

### Rust

- Avoid `unwrap()` and `expect()` in runtime paths; tests may use them only when failure makes the test itself impossible and the message is clear.
- Use typed errors across crate boundaries and translate to public errors once at the adapter boundary.
- Keep domain/application crates free of Tauri, concrete SQL, D-Bus, and process types.
- Keep transactions short and selected by use cases.
- Document public interfaces and non-obvious safety invariants.
- Treat Clippy warnings as failures and format with rustfmt.
- Use structured tracing fields; do not interpolate sensitive values into messages.
- Keep async cancellation and resource teardown explicit.

### TypeScript and React

- Enable all practical strict TypeScript checks; do not use `any` or hand-edit generated types.
- Only the transport module imports Tauri invoke/event APIs.
- Use TanStack Query for backend snapshots and mutations; do not mirror persistent records into global frontend state.
- Lazy-load route modules and stop inactive subscriptions/requests.
- Use semantic design tokens for meaningful colors and lightweight accessible primitives.
- Prefer small composed components with user-observable tests.
- Do not parse backend error messages; branch on generated codes/details.
- Format with Prettier and lint with ESLint.

### General

- Use explicit, domain-oriented names and one reason for each dependency.
- Build abstractions at I/O, trust, ownership, or independently changing domain boundaries—not merely to reduce line count.
- Classify every side effect and add a migration for every schema change.
- Never commit databases, sidecars, logs, state, backups, runtime files, or personal fixtures.
- Keep modules independently readable and synchronize documentation in the same change.

## Specification-driven development workflow

Every implementation unit begins in an approved specification with:

1. problem and user value;
2. functional and non-functional requirements;
3. in-scope and excluded behavior;
4. workflows and failure/recovery states;
5. acceptance criteria;
6. affected components/contracts/persistence/migrations;
7. security and performance impact;
8. implementation order and stable task IDs;
9. automated and manual verification;
10. required documentation update.

Tasks use the IDs in [the foundation ledger](foundation/tasks.md). A task is small enough to produce one independently reviewable result, names its dependencies and affected boundaries, includes tests and docs, and has a binary completion condition. Implementation pull requests cite the specification and task IDs.

If evidence contradicts a specification, stop and update documents/decisions/tasks first. A code comment or issue does not supersede the normative docs.

## Adding a compiled module

After the foundation registry exists, adding a module requires:

1. approve a feature specification, capability/action inventory, and data/security/performance plan;
2. define domain vocabulary, application ports, use cases, and contracts without Tauri types;
3. add any forward-only migrations and repository adapters;
4. implement platform adapters behind the ports;
5. register one backend manifest/service factory in the centralized registry;
6. register one frontend lazy route/presentation entry in the centralized registry;
7. expose only semantic methods through the frontend API facade;
8. add module health, unavailable/degraded/error UI, accessibility, and route cleanup;
9. add registry parity, domain/use-case/adapter/frontend/security tests;
10. update product/module/development/verification documentation.

No other navigation edit should be needed. Cross-module access is introduced as a shared application service, not an import of another module's private store. A module generator may automate these edits later but is not foundation scope.

## Generated contracts

Rust definitions in `argos-contracts` are authoritative. `pnpm contracts:generate` invokes an `xtask`, writes deterministic committed files beneath the frontend `generated` directory, and never modifies handwritten API wrappers or components. Generation runs locally after a Rust contract change and in CI to detect drift.

A contract change includes:

- Rust serialization and export tests;
- regenerated TypeScript;
- frontend API and consumer updates;
- compatibility/error/event considerations;
- documentation of semantic changes.

Direct edits to generated files are discarded. The chosen generator must prove correct tagged enums, optionals, maps, branded IDs (or safe aliases), and deterministic ordering before adoption.

## Database migrations

Each persisted change adds an immutable numbered migration under the SQLite adapter. The change must document forward behavior, indexes/constraints, backfill, size/locking risk, backup need, recovery, and tests from every supported prior schema state.

The workflow is:

1. update the relevant specification/data model;
2. add a forward migration; never edit a released migration;
3. update domain/repository code and contract views;
4. test empty initialization and upgrade from a fixture of the prior schema;
5. test rollback/failure and foreign-key/integrity checks;
6. regenerate contracts if public views changed;
7. update migration documentation and diagnostics expectations.

There is no automatic destructive downgrade. A development reset command, if later provided, must resolve and print the exact `argos-dev` target, refuse production/repository paths, and require explicit confirmation; it is not part of foundation bootstrap.

## Profile isolation in development

Normal source commands embed `development`, regardless of debug/optimized compilation. Tests inject temporary roots. Production access from a source build requires the two-part acknowledgement documented in [Data and contracts](data-and-contracts.md#runtime-profiles-and-path-resolution) and remains visibly marked.

Developers use `ARGOS_HOME` only for deliberately isolated development scenarios. It must be an absolute path outside the repository. Neither scripts nor IDE settings should set it to `.`. Test helpers expose resolved paths so assertions can prove non-overlap.

Target-machine production verification uses a documented disposable production test account or an explicit preserved backup/approval procedure; routine smoke testing uses development data.

## Dependency policy

Select compatible stable versions during implementation and commit both lockfiles. A dependency needs a boundary-specific reason, active maintenance, acceptable licensing, and a smaller custom-code risk than avoiding it. Default features are reviewed, especially Tauri plugins, process/shell utilities, database drivers, UI frameworks, and serialization generators.

Exact versions are intentionally absent from architecture docs unless a compatibility constraint becomes an architectural fact. Automated dependency updates cannot bypass tests or capability review.

## Release and packaging direction

Foundation must build and run on the target Arch/GNOME machine before claiming release readiness. GitHub Actions initially proves builds/tests; packaging work records the Tauri-supported Linux artifact(s), desktop entry, icons, metadata, checksums, and native dependencies after target testing.

Only the explicit packaging workflow embeds `production`. Packages contain application assets and embedded migrations, never a database or generated user defaults copied from a developer machine. Install, upgrade, reinstall, branch/source deletion, and uninstall do not remove XDG user data. A future data-removal flow must be separate, exact, and confirmed.

Release notes name schema migrations and backup implications. Signing/distribution channels are an open post-foundation operational decision, not a blocker for architectural implementation.
