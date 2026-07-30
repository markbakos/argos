---
name: build-react-typescript-apps
description: "Build, refactor, and review React and TypeScript applications using feature-oriented architecture, strict boundary typing, small composable components, context-driven compound UI systems, predictable list rendering, deliberate state ownership, and maintainable interaction patterns. Use for React/TypeScript components, feature organization, lists and tables, filters, editors, routing, state APIs, or frontend code-quality reviews."
---

# Build React and TypeScript Applications

## Working stance

Treat these practices as strong defaults, not exhaustive laws.

- Inspect the existing architecture before changing it.
- Preserve established conventions unless they cause a concrete correctness, usability, security, or maintenance problem.
- Introduce patterns incrementally; avoid rewriting working code solely for stylistic uniformity.
- Prefer framework- and library-independent concepts.
- Adapt the amount of abstraction to the feature's real complexity.
- Optimize first for clarity, predictable behavior, and safe change.

## Organize around features and stable boundaries

Prefer vertical feature ownership over grouping the entire application by technical file type.

Use a structure similar to:

```text
src/
├── app/
│   ├── routing/
│   ├── providers/
│   └── configuration/
├── features/
│   └── feature-name/
│       ├── pages/
│       ├── components/
│       ├── model/
│       ├── api/
│       └── index.ts
└── shared/
    ├── ui/
    ├── hooks/
    ├── lib/
    └── types/
```

Adjust this structure to project size. Do not create empty layers.

- Keep routing composition, application providers, and bootstrapping in `app`.
- Keep pages, domain components, feature state, and feature-specific transformations together.
- Move code into `shared` only when it is genuinely domain-independent or has multiple real consumers.
- Keep domain knowledge out of shared UI primitives.
- Allow feature code to import shared code.
- Prevent shared code from importing feature modules.
- Avoid feature-to-feature deep imports; expose deliberate public contracts.
- Use barrel files only to define a module's public API, not to re-export everything recursively.
- Use `.tsx` only for files containing JSX and `.ts` otherwise.
- Prefer stable path aliases for architectural boundaries when the project supports them.

## Name by responsibility

Use names that communicate role without opening the file.

- Name components and component files in PascalCase: `CustomerRow`, `DateRangeEditor`.
- Name hooks with `use`: `useDebouncedValue`, `useOverlayNavigation`.
- Name internal event handlers `handleSave`, `handleSelect`, or `handlePointerEnter`.
- Name callback props `onSave`, `onSelect`, or `onOpen`.
- Prefix booleans with `is`, `has`, `can`, or `should`.
- Use singular names for entities and plural names for collections.
- Name transformations with verbs such as `filterCustomers`, `groupTasksByOwner`, or `formatDuration`.
- Name providers and contexts by their domain: `CollectionProvider`, `CollectionContext`.
- Prefer descriptive page names such as `CustomerListPage` over generic names such as `Main`.
- Avoid broad containers named `utils`, `common`, or `helpers` when a narrower name is possible.
- Keep abbreviations and identifier terminology consistent across types, props, routes, and data access.
- Follow local casing conventions consistently when changing an established project.

## Design component boundaries deliberately

Separate orchestration, reusable behavior, and domain presentation.

A typical feature should contain:

- A page or feature shell that obtains data and coordinates major actions.
- Domain components that render domain-specific rows, cards, headers, and details.
- Shared primitives that implement reusable interaction mechanics.
- Pure functions that transform, compare, sort, validate, or format data.
- Adapters or services that perform external mutations.

Prioritize small components when they provide at least one of:

- A reusable behavior.
- A meaningful visual or semantic unit.
- A state or effect boundary.
- A domain boundary.
- A simpler parent component.
- An independently testable interaction.

Do not split every small piece of markup into a component. Avoid "component confetti" that adds navigation without adding meaning.

Keep props narrow and explicit. Pass the information a child needs rather than an entire page model by default.

Prefer composition over boolean-heavy components. When a component accumulates many switches such as `searchable`, `selectable`, `sortable`, and `showCount`, consider compound components instead.

## Build compound component systems

Use a provider and focused sibling components when multiple descendants coordinate around one interaction model.

Suitable examples include:

- Lists and tables.
- Tabs.
- Menus and selects.
- Filters.
- Editors with popovers.
- Multi-step forms.

The provider should own shared mechanics. Child components should consume only the behavior they need.

Prefer an API shaped like:

```tsx
<Collection.Root items={items} getId={(item) => item.id}>
  <Collection.Toolbar>
    <Collection.Count />
    <Collection.Search filter={filterItems} />
    <Collection.BulkActions>
      <ArchiveSelection />
    </Collection.BulkActions>
  </Collection.Toolbar>

  <CustomerHeader />

  <Collection.Items>
    {(item) => <CustomerRow item={item} />}
  </Collection.Items>
</Collection.Root>
```

Keep row and header content domain-specific. Keep searching, sorting, selection, counting, and interaction state generic.

Use context when it:

- Coordinates several siblings or deeply nested compound components.
- Removes plumbing for a coherent subsystem.
- Provides a stable domain-specific interface.

Do not introduce context solely to avoid passing one prop through one level.

Create contexts with `undefined` as the default and expose a guarded hook:

```tsx
function useCollection() {
  const value = useContext(CollectionContext);

  if (!value) {
    throw new Error("Collection components must be used inside Collection.Root");
  }

  return value;
}
```

Do not publish optional context actions that are required for correct operation.

Split state and actions into separate contexts when consumers update at very different frequencies. Use a reducer when transitions such as range selection, select-all, filtering, and data replacement become difficult to reason about independently.

Prefer render props or explicitly typed component props over cloning an arbitrary child and injecting undocumented props.

## Model list state explicitly

Treat a reusable list as a small interaction system.

Keep these concepts distinct:

- `items`: immutable source data.
- `query`: current search input.
- `sort`: active sort key and direction.
- `selectedIds`: selected stable identifiers.
- `visibleItems`: items derived from filtering and sorting.
- `focusedId` or `activeId`: keyboard or pointer focus when required.
- Async states such as loading, empty, error, and fetching-more.

Derive `visibleItems` during render or with memoization when the computation is expensive. Do not mirror derived lists into state unless an external process truly mutates them independently.

Apply transformations in an explicit pipeline:

```text
source items
  → domain filter
  → text search
  → non-mutating sort
  → pagination or windowing
  → render
```

### Rendering rules

- Require a stable `getId(item)` contract.
- Use domain identifiers as React keys.
- Never use array indexes as keys for reorderable, filterable, selectable, or editable rows.
- Render through a typed function such as `(item) => ReactNode`.
- Keep generic renderers unaware of domain field names.
- Provide explicit loading, empty, error, and no-results states.
- Keep the header and row column definitions aligned through shared configuration when drift becomes likely.
- Preserve scroll ownership at one deliberate container.

### Sorting rules

- Never mutate source props with in-place sorting.
- Use `toSorted`, or sort a copied array.
- Represent direction with a union such as `"asc" | "desc"`.
- Support an optional typed value extractor or comparator for computed columns.
- Define how null and missing values are ordered.
- Use locale-aware comparison when sorting human-readable text.
- Keep the sort indicator and accessible sort state synchronized.

### Searching and filtering rules

- Keep filtering functions pure.
- Pass domain search logic into the generic list.
- Normalize the query once rather than once per field.
- Debounce the input or request, not the rendering itself.
- Distinguish an empty query from a query with no matches.
- Make minimum-query-length behavior explicit.
- Consider URL-controlled search when it should survive navigation or sharing.

### Selection rules

- Store selected identifiers in a `Set<Id>`.
- Create a new `Set` for every state update.
- Do not mix sentinel strings such as `"ALL"` into domain identifiers.
- Derive `isAllSelected` from the selection and the intended selection scope.
- Define whether "select all" means all source items, all filtered items, or the current page.
- Define what happens when selected items disappear after filtering or data refresh.
- Build bulk-action payloads from the latest source data at action time.
- Keep single-selection and multi-selection behavior explicit.
- Avoid firing a callback with stale state; compute the next state once, store it, and emit the same value.
- Model drag or range selection as explicit interaction state rather than scattered pointer flags.

### List performance rules

- Build identifier maps for repeated lookups instead of repeatedly scanning arrays.
- Memoize expensive indexing or transformation, not trivial calculations.
- Keep transient hover state local when the rest of the collection does not need it.
- Split context or use context selectors if one changing value rerenders the entire list.
- Consider virtualization only when measured list size or rendering cost warrants it.
- Move CPU-heavy parsing or export work away from the UI thread.
- Define cancellation and stale-result behavior for asynchronous searching.

## Own state at the narrowest useful level

Classify state before choosing where it lives:

1. Temporary component interaction state.
2. Shared state for one component subtree.
3. Feature state shared across feature screens.
4. URL or navigation state.
5. Remote or server state.
6. Application-wide client state.

Keep state local until multiple consumers genuinely require shared ownership.

Use context for cohesive subtree state. Use an application store for cross-feature state, externally synchronized data, or derived indexes with many consumers.

Prefer selectors and hooks over reading a global store singleton inside general utilities. Explicit dependencies are easier to test and reuse.

Create derived maps or indexes for frequent identifier lookups:

```ts
const byId = new Map(items.map((item) => [item.id, item]));
```

Treat these maps as derived data. Rebuild them when their source changes instead of maintaining two competing sources of truth.

## Use effects only for synchronization

Use effects to synchronize React with something external:

- Network or subscription lifecycles.
- Timers.
- Global browser events.
- Imperative third-party widgets.
- Focus, measurement, or scrolling.
- Persistent browser storage.

Do not use an effect merely to calculate a value from props or state.

- Derive values during render when possible.
- Put user-triggered behavior in event handlers.
- Always clean up timers, subscriptions, and global listeners.
- Include every reactive dependency or restructure the code so the dependency is unnecessary.
- Cancel or ignore obsolete asynchronous work.
- Avoid copying props into state except for deliberate drafts or controlled/uncontrolled APIs.
- Make controlled and uncontrolled behavior explicit rather than inferring it accidentally.

## Build editing flows around drafts

For edit screens and inline editors, distinguish:

- The persisted source value.
- The local editable draft.
- Whether the draft is dirty.
- Validation state.
- Save state and errors.

A reliable flow is:

1. Initialize a draft from the source.
2. Update the draft immutably.
3. Derive whether it differs from the source.
4. Save through a feature service or callback.
5. Replace the source only after confirmed success.
6. Reset the draft on cancel.
7. Handle external source changes intentionally.

For inline editing:

- Enter edit mode explicitly.
- Focus the input when editing begins.
- Let Enter commit when appropriate.
- Let Escape cancel.
- Define blur behavior deliberately.
- Parse numeric, date, and time input at the boundary.
- Keep display formatting separate from stored representation.
- Preserve invalid input long enough to communicate the error instead of silently coercing it.

Prefer typed update functions over arbitrary string paths when the model is stable. If dynamic paths are required, type or validate them.

## Use TypeScript to model behavior

Enable the strictest practical compiler settings for new code.

- Avoid `any`; use generics, concrete models, or `unknown` at untrusted boundaries.
- Validate and narrow `unknown` before use.
- Keep types close to the feature or module that owns them.
- Export types only when another module needs them.
- Use `ReactNode` for renderable children.
- Type refs and browser events precisely.
- Use unions for finite states and variants.
- Use discriminated unions for state with mutually exclusive cases.
- Use `readonly` inputs when mutation is not part of the contract.
- Use `satisfies` for configuration objects that should retain narrow inference.
- Avoid optional properties for operations required at runtime.
- Avoid assertions that hide nullability problems.
- Prefer explicit result types for parsing and fallible operations.
- Model identifiers consistently; introduce branded IDs only when accidental cross-domain mixing is a real risk.

Do not require one component declaration style. Direct functions and `React.FC` can both work; follow the project convention unless one causes a typing problem.

## Separate behavior from integrations

Keep generic UI components unaware of databases, routers, analytics, file formats, or remote clients.

Prefer:

```tsx
<DeleteButton
  disabled={!selection.size}
  onConfirm={() => deleteSelected(selectedIds)}
/>
```

over a supposedly generic button that knows collection names, database calls, or navigation behavior.

- Put remote operations in feature services or callbacks.
- Convert external models into UI models at a boundary.
- Return explicit success or error results.
- Represent loading and failure states in the UI.
- Keep browser-only operations behind focused adapters.
- Type worker messages as request and response unions.
- Never embed secrets or privileged credentials in client code.

## Build a coherent UI layer

Create small primitives for recurring visual and interaction rules:

- Buttons and icon buttons.
- Fields and labels.
- Cards and panels.
- Menus and popovers.
- Modals.
- Badges.
- Tables or collection shells.
- Loading, empty, and error states.

Support a `className` or equivalent escape hatch, but keep essential structure and accessibility inside the primitive.

Use semantic design tokens for:

- Brand, neutral, success, warning, and error colors.
- Typography roles.
- Spacing.
- Radius.
- Shadows.
- Layering.
- Focus indicators.
- Motion duration.

Keep domain meaning out of raw color names where possible.

Plan dense layouts around:

- A single intentional scroll container.
- Sticky headers where useful.
- Truncation with an accessible way to reveal full content.
- Stable column sizing.
- Responsive behavior.
- Overlay placement near viewport edges.
- Portals when an overlay would be clipped by overflow.

## Preserve accessibility

- Use `button`, `a`, `input`, `table`, and other semantic elements for their intended roles.
- Do not implement clickable controls as plain `div` elements.
- Give icon-only controls accessible names.
- Preserve keyboard activation and visible focus.
- Associate fields with labels and error descriptions.
- Expose sorting state through accessible attributes.
- Manage focus when opening and closing dialogs, menus, and popovers.
- Support Escape where users expect dismissal.
- Avoid hover-only access to important information.
- Respect reduced-motion preferences.
- Use pointer events in addition to mouse-specific events when touch or pen input matters.

## Keep utilities pure and narrow

- Keep formatting, parsing, sorting, and color calculations free of React state.
- Avoid hidden reads from mutable global stores in general-purpose utilities.
- Do not mutate function inputs unless mutation is the explicit contract.
- Return new values from transformations.
- Prefer domain-named modules over catch-all utility files.
- Document units in names or types: milliseconds, seconds, minutes, meters, or currency minor units.
- Handle dates and time zones deliberately.
- Use the platform's internationalization APIs for user-facing dates, numbers, currency, and relative time.

## Review before completing work

Check that:

- Component and file names expose responsibility.
- Feature-specific code remains inside its feature.
- Shared code does not depend on a feature.
- Props and contexts have concrete types.
- Context consumers fail clearly outside their provider.
- Lists use stable domain keys.
- Sorting does not mutate inputs.
- Derived data is not unnecessarily duplicated in state.
- Selection semantics are explicit.
- Effects synchronize external systems and clean up correctly.
- Async work handles failure and obsolete results.
- Browser listeners, timers, object URLs, workers, and subscriptions are released.
- Editing supports save, cancel, validation, and failure.
- Controls are semantic and keyboard accessible.
- Loading, empty, no-results, and error states exist.
- Debug logging, dead imports, stale comments, and abandoned code are removed.
- Configuration and secrets are outside client source.
- Tests cover important transitions and user-visible behavior.

Do not turn this checklist into mandatory abstraction. Apply only the sections relevant to the task.

## Research and hardening candidates

Treat these as separate investigation topics rather than automatic requirements:

- Generic context factories that preserve item and identifier types.
- Reducers or state machines for drag, range, and cross-page selection.
- Context selectors or external stores for very large compound component trees.
- Virtualized rendering and variable-height rows.
- Server-controlled sorting, filtering, pagination, and infinite loading.
- URL-persisted list state and shareable views.
- Locale-aware collation, fuzzy search, and indexed client-side search.
- Accessible grid, listbox, combobox, dialog, and menu interaction models.
- Focus trapping, focus restoration, and collision-aware popover positioning.
- Runtime schema validation at API, storage, import, and worker boundaries.
- Optimistic mutations, rollback, retries, and conflict resolution.
- Error boundaries and recoverable rendering failures.
- Worker cancellation, typed worker protocols, and transferables.
- Property-based tests for sort, selection, parsing, and immutable updates.
- Automated accessibility and keyboard-navigation tests.
- Design-token generation and prevention of arbitrary styling drift.
