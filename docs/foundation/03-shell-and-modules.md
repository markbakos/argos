# FND-03 — Application shell and module registry

**Status:** Approved for foundation implementation  
**Depends on:** FND-01; preference persistence depends on FND-02  
**Enables:** systemd/launcher presentation and diagnostics integration

## Problem and user value

A control center needs one predictable, accessible shell without turning navigation, module state, or styling into scattered special cases. Modules also need an explicit compiled extension point so growth does not couple unrelated features.

The user gains persistent navigation, honest module health, theme choice, consistent interaction states, and settings that can disable/reorder modules without losing code-owned metadata.

## Workflows

1. The user opens Argos to a lightweight dashboard placeholder and navigates enabled modules from a persistent sidebar.
2. The user disables or reorders a module in Settings; navigation updates and the override persists across restart.
3. An enabled module whose dependency/platform is unavailable remains reachable, shows a labeled health badge, and explains recovery.
4. The user selects system, light, or dark theme; system follows OS changes and the selection persists.
5. Keyboard users navigate the sidebar, operate forms/dialogs, dismiss a safe dialog, and regain focus at the invoking control.
6. A developer registers a module once in each centralized backend/frontend registry; routing/navigation parity tests catch omissions and routes load lazily.

## Functional requirements

### Shell and presentation

- Implement one React root with providers for router, TanStack Query, theme, error boundary, and typed event invalidation.
- Provide a persistent sidebar, main landmark/outlet, page headers, Dashboard placeholder, Settings, Diagnostics entry, and a non-functional reserved global-search trigger.
- Support system/light/dark themes from semantic tokens defined in [Architecture](../architecture.md#design-tokens-and-themes), with system default and live OS preference changes.
- Provide shared accessible patterns for tiles, lists, tables, forms, dialogs, detail panels, exact-target confirmations, empty/loading/error/unavailable/degraded states, and health indicators.
- Restore focus after dialogs, support keyboard navigation and reduced motion, and never encode status by color alone.
- Use route-level lazy imports and error boundaries; an inactive module must not load data or maintain refresh subscriptions.

### Backend registry

- Define a compiled module manifest with ID, display/description/version, route key, default order/enabled, capabilities, dependencies, platform requirements, and health provider.
- Centralize registration and validate IDs/routes, duplicates, missing dependencies, cycles, capability declarations, and deterministic ordering at startup.
- Merge code-owned manifests with `module_preferences`; retain but ignore/report overrides for unknown modules.
- Represent enablement separately from `available`, `unavailable`, `degraded`, and `error` health/reasons.
- Make dependency on a disabled/unhealthy required module an explicit unavailable reason without rewriting the user's override.
- Expose effective modules through a narrow application use case/contract; do not expose service objects.

### Frontend registry and settings

- Map each user-facing module ID once to a lazy component/route and presentation icon; derive sidebar/router entries from effective backend modules joined to this mapping.
- Treat Dashboard, Settings, and Diagnostics as non-disableable core shell routes.
- Hide disabled modules from ordinary navigation and reject/direct their route to a clear disabled state; keep them visible in Settings.
- Keep enabled unhealthy modules in navigation with text/icon status and a full unavailable/degraded/error page state.
- Allow enablement and display-order overrides, validate bounds/dependencies in Rust, persist in SQLite, and audit successful writes.
- Permit optional per-module settings only through a module-owned bounded typed validator; foundation systemd/launcher need no generic JSON editor.
- Test backend/frontend manifest parity in CI.

## Non-functional requirements

- No component outside the API transport calls Tauri.
- Navigation has a single derived source and adding a module does not require edits in multiple shell files.
- Initial shell render does not wait for systemd or launcher data; module health may settle asynchronously.
- Query retries are bounded and appropriate to `retryable`; error strings are never parsed.
- Token contrast and keyboard behavior meet accessibility verification on both themes.

## Failure and recovery states

- Duplicate backend IDs/routes or dependency cycles are startup registry errors with programmer-focused diagnostics; unsafe ambiguous registration is not accepted.
- A backend module unknown to the frontend registry is visible in Diagnostics as a registry error and omitted from routable navigation; parity CI prevents release.
- A frontend entry unknown to the backend is not displayed; parity CI prevents release.
- Preference database failure leaves manifest defaults visible read-only and marks Settings/storage degraded; it does not pretend a change persisted.
- A lazy bundle failure shows a route error with retry navigation; the shell stays usable.
- A disabled route reached by bookmark redirects to Settings or renders a stable disabled explanation without loading feature data.
- Invalid/corrupt theme uses `system`, reports a safe configuration diagnostic, and preserves correction ability.

## Explicit exclusions

No runtime/downloader plugin system, module marketplace/generator, customizable dashboard, drag-and-drop layout, global search implementation, Redux store, tray, embedded remote app, or module-specific business behavior is included.

## Architecture impact

This workstream establishes the two centralized compiled registries, effective manifest merge, query/event providers, shell route boundaries, and design-system primitives. It implements ADR-002, ADR-007, and the UI consequences of ADR-013.

## Contracts

Contracts include `ModuleManifestView`, `EffectiveModule`, `ModuleEnablement`, `ModuleHealth`, tagged `HealthReason`, capability/platform/dependency views, `ListModulesResponse`, and typed preference update requests/results. Frontend route loaders are handwritten and keyed by generated `ModuleId`; they are not serialized service registrations.

## Persistence and migrations

Uses the initial `module_preferences` table; no new migration beyond FND-02. Theme remains `config.toml`. Enable/order mutation and its audit event share a transaction. Null override restores the manifest default rather than copying the default into user data.

## Security implications

Module enablement does not grant undeclared Tauri capability or backend authority. Manifests declare capabilities for diagnostics/review; Tauri capabilities remain code-owned. Settings input is bounded and revalidated in Rust. React health/disable state is not an authorization check.

## Performance implications

Only shell/core queries run initially. Lazy module bundles and queries begin at navigation. Event listeners are reference-counted. Module health checks are cached, event-driven or explicit; no global one-second poll exists.

## Acceptance criteria

- **FND-03-AC01:** Shell renders required landmarks/routes and remains usable while module health/data is loading or failing.
- **FND-03-AC02:** System/light/dark behavior persists, follows live OS changes in system mode, and passes contrast/reduced-motion checks.
- **FND-03-AC03:** Duplicate/missing/cyclic registry cases and deterministic effective order are covered by backend tests.
- **FND-03-AC04:** Frontend parity and route tests prove one registry, derived navigation, and lazy loading.
- **FND-03-AC05:** Disable removes a module from navigation without deleting its data; re-enable restores it after restart.
- **FND-03-AC06:** Enabled unavailable/degraded/error modules remain labeled and display actionable, non-color-only states.
- **FND-03-AC07:** Preference changes persist and audit atomically; storage failure does not show false success.
- **FND-03-AC08:** Core sidebar, forms, dialogs, confirmation, and error recovery are keyboard operable with visible focus and correct focus restoration.
- **FND-03-AC09:** Inactive lazy modules perform no data fetch, interval, or lingering feature subscription.
- **FND-03-AC10:** Adding a test module requires only the backend and frontend registry extension points, plus its own feature files.

## Testing strategy

Use Rust registry/preference use-case tests, SQLite preference integration tests, frontend route/query/provider tests, lazy import instrumentation, theme media-query mocks, accessibility interaction tests, lint boundary checks, and manual light/dark keyboard/contrast passes.

## Implementation order and tasks

1. `FND-SHL-001` — React providers, router, shell, and core routes.
2. `FND-SHL-002` — semantic tokens, themes, accessible shared states/primitives.
3. `FND-SHL-003` — backend manifest/effective registry and validation.
4. `FND-SHL-004` — frontend registry, lazy routes, derived navigation, parity.
5. `FND-SHL-005` — persisted module settings/ordering and audit.
6. `FND-SHL-006` — shell/module accessibility and inactive-resource verification.

## Verification and documentation update

Run FND-03 criteria and the shell/frontend portions of [Verification](../verification.md). Update [Development: adding a module](../development.md#adding-a-compiled-module) with the actual extension APIs and document any token primitive added; do not duplicate module metadata in docs.
