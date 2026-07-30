# FND-04 — Read-only systemd module

**Status:** Approved for foundation implementation  
**Depends on:** FND-01 contracts, FND-03 registries/shell  
**Integrates with:** FND-06 diagnostics

## Problem and user value

The first system-control domain must prove that Argos can inspect real user and system managers safely without parsing presentation-oriented CLI output or accidentally widening into mutation authority.

The user gains a scoped view of services, timers, failed states, core details, trigger times, jobs/dependencies, and bounded recent logs, with honest permission and availability behavior.

## Workflows

1. The systemd page opens in `user` scope and reports user-manager connection health.
2. The user lists services or timers, filters the current snapshot, refreshes explicitly, and opens details.
3. A timer detail displays next and previous trigger information with clear missing/unknown semantics and localizes UTC for display.
4. The user switches explicitly to `system`; Argos connects independently and never substitutes one scope for the other.
5. The user views at most a bounded number of recent journal entries for the selected unit; empty, permission denied, missing journal, and parse failure are distinct.
6. While the page is active, relevant D-Bus changes coalesce into query invalidation; leaving the page releases subscriptions.

## Functional requirements

### D-Bus reads

- Implement `UnitReader` through `zbus` with explicit `SystemdScope::User` or `SystemdScope::System` on every use case.
- Maintain independent connection/health state for the session/user manager and system manager; never fall back between them.
- List loaded service and timer units using systemd manager D-Bus methods/properties, not human-readable `systemctl` output.
- Normalize unit name, description, load state, active state, sub-state, following/related identity, unit-file/enablement state when obtainable, and failed indication.
- Read a selected service/timer's bounded core properties, dependency unit names, and relevant current job summary without leaking D-Bus object paths as public identity.
- Read timer next and previous trigger data. Distinguish unavailable/not scheduled/unknown values and map realtime values to UTC while retaining enough clock-source context to avoid false precision.
- Preserve unknown future state strings safely rather than failing an entire response.
- Provide filters/kind selection and a conservative response cap; target measurement decides whether additional paging is needed without exposing unbounded data.

### Health and errors

- Map bus missing, manager missing, permission denial, timeout, unit gone/not found, malformed protocol data, and adapter error to distinct public codes/reasons.
- Compute module health from both scopes: user failure makes the default function unavailable/error; system unavailability/permission may degrade the module while user reads remain usable.
- Display scope prominently in list/detail/log page headers and retry/refresh actions.

### Journal reads

- Implement `JournalReader` as a replaceable adapter that directly invokes `journalctl` with an argument vector, explicit scope-specific unit selector, JSON/machine-readable output, no pager, quiet behavior, and numeric result limit.
- Default to 200 entries and clamp requests to 1–500. Apply a bounded runtime/timeout and bounded captured output.
- Validate the unit identifier before argument construction; use no shell and no concatenated command line.
- Map structured journal fields into timestamp, priority/severity, message, boot ID/cursor only where useful, and safe optional source metadata. Missing fields do not crash the result.
- Do not automatically load logs with every list row or stream when the log panel is inactive.

### Change invalidation

- While there is an active consumer, subscribe where practical to manager unit/job signals and property changes, then emit bounded/coalesced `SystemdChanged` hints by explicit scope.
- Refetch authoritative Query data after invalidation. Provide manual refresh even when signals work.
- Unsubscribe on page/window teardown. Do not add a foundation global interval; document any measured view-local fallback before implementing it.

## Non-functional requirements

- Most automated tests use fake readers and captured D-Bus/journal fixtures, not live systemd.
- No systemd write method, unit-file operation, polkit request, or write capability exists in compiled foundation code.
- List/detail/log requests are cancellable or obsolete UI results are discarded.
- Journal/unit output, unit names from personal systems, and D-Bus debug payloads are not copied to normal tracing or safe exports.
- The adapter is resilient to systemd version/property variation on supported target systems.

## Failure and recovery states

| State | UI/application behavior |
| --- | --- |
| User bus/manager unavailable | module unavailable with reason and retry; shell/settings remain usable |
| System manager unavailable | system scope unavailable; user scope remains intact; overall module may be degraded |
| Permission denied | explicit permission state, not `unavailable` or empty list |
| Unit disappears between list/detail | `SYSTEMD_UNIT_NOT_FOUND`, return to/refetch list |
| Unknown state/property | show safe raw/unknown label; do not fail whole list |
| D-Bus timeout/disconnect | retryable scoped error and reconnect on explicit/event demand with backoff |
| Journal missing | log panel unavailable; D-Bus unit details still work |
| Journal permission denied | exact permission explanation; no elevation attempt |
| Malformed/oversized journal output | bounded parse error; process killed at limits; other views survive |
| Signal subscription fails | degraded live-refresh indicator plus explicit refresh; no hidden rapid polling |

## Explicit exclusions

No start/stop/restart/reload/enable/disable/mask, timer creation, daemon reload, unit-file write, systemd mutation command, polkit interaction, continuous log tail, full dependency graph visualization, or long-term monitoring/history.

## Architecture impact

`argos-systemd` implements read ports and maps D-Bus/journal details. Application use cases own scope/limits and module-health combination. Tauri exposes narrow read commands/events under user/system read capabilities. React owns scoped presentation only.

## Contracts

Contracts include explicit `SystemdScope`, `UnitKind`, filter/request/summary, normalized state values with unknown fallback, `TimerTrigger`, bounded `UnitDetails`, `UnitJobSummary`, `JournalRequest/Entry/Page`, scope health, and `SystemdChanged`. No contract contains D-Bus proxies/object paths, journal command arguments, or mutation fields.

## Persistence and migrations

None. systemd and journal remain authoritative. Query caches are disposable. Scope/UI selection may be recent UI state later, but no foundation systemd snapshot is persisted in SQLite.

## Security implications

The compiled command/capability inventory is read-only. Scope is required in Rust requests, unit identifiers are validated, and journal invocation is direct. Argos never starts/elevates as root. Remote content has no systemd read capability. Personal log content is rendered safely and excluded from Argos logs/audit/export.

## Performance implications

Do not preload both scopes or logs at startup. Fetch active kind/scope only; fetch details/logs on selection. Clamp results, coalesce signals, cache briefly, cancel obsolete requests, and release subscriptions when inactive. Measure mapping of large real unit sets.

## Acceptance criteria

- **FND-04-AC01:** Fixture/fake tests list and map services/timers, core states, failed units, details, dependencies/jobs, and unknown future values.
- **FND-04-AC02:** Every operation requires scope; user is the UI default; tests prove no cross-scope fallback.
- **FND-04-AC03:** Timer previous/next trigger cases map accurately for scheduled, inactive, unknown, and missing values with UTC contracts.
- **FND-04-AC04:** User/system health and permission/manager/bus/timeout errors remain distinct and partial scope failure degrades rather than erases the working scope.
- **FND-04-AC05:** Journal adapter tests prove direct argv invocation, valid unit selection, JSON parsing, 1–500 clamp, output/time bounds, cancellation, and error mapping.
- **FND-04-AC06:** Active-page signals cause debounced scoped invalidation; teardown removes subscribers; subscription failure offers manual refresh without global polling.
- **FND-04-AC07:** Capability/static scans find no systemd mutation method, unit write, shell command, or user/system write permission.
- **FND-04-AC08:** Target smoke connects to the real user manager, finds a known timer, reads bounded logs, and reports system-scope access honestly.
- **FND-04-AC09:** Loading/empty/error/unavailable/degraded states and scope labels are keyboard accessible and do not rely on color.

## Testing strategy

Use application fakes, captured/constructed D-Bus property fixtures, journal JSON/exit fixtures, command-plan assertions, time conversion boundary tests, signal-stream fake tests, React mocked API/event tests, capability scans, and optional/manual target-system checks.

## Implementation order and tasks

1. `FND-SYS-001` — scoped domain/contracts and fake use cases.
2. `FND-SYS-002` — user/system zbus connections, health, and unit mapping.
3. `FND-SYS-003` — service/timer list and detail/trigger reads.
4. `FND-SYS-004` — bounded direct journald adapter.
5. `FND-SYS-005` — Tauri read capabilities/API and React module views.
6. `FND-SYS-006` — active-view events, cancellation, teardown, and real-system smoke.

## Verification and documentation update

Run automated FND-04 criteria, capability inventory, and the redacted target-system record. Update exact systemd property/method mappings and known target-version behavior in this specification if evidence differs; never substitute `systemctl` parsing or add writes without a preceding decision/specification update.
