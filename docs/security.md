# Security architecture

## Security objectives

Argos is a privileged-looking control center that normally runs without privilege. Its security model minimizes authority, makes scope explicit, validates again in Rust, prevents WebView compromise from becoming general host compromise, and preserves an attributable audit trail for side effects.

The foundation threat model includes malformed or hostile user input, accidental target confusion, a compromised frontend/WebView, unsafe path or command construction, local permission differences, corrupted persisted data, and future secondary content windows. It does not claim to defend against an attacker who already controls the user's OS account or can replace the Argos binary.

## Trust boundaries and actors

```text
Untrusted/presentation: React components, rendered strings, user-entered fields
Constrained transport: generated contracts, frontend API, Tauri IPC and events
Trusted policy: Rust application use cases and domain validation
Boundary implementations: SQLite, filesystem, process, D-Bus, journal adapters
External authority: Linux permissions, systemd, D-Bus policy, polkit (future)
```

The initial actor is `human:local-user`. Future `cli`, `agent:<uuid>`, and `automation:<uuid>` actors are distinct authenticated/authorized contexts. The desktop adapter does not accept an actor identity supplied by React; it assigns the human context. Future adapters assign their own verified contexts.

## Operation surface

The frontend never receives operations equivalent to:

- run a shell string or arbitrary command;
- read/write/delete an arbitrary file;
- execute SQL or reveal the database path;
- invoke a backend operation by arbitrary name;
- call systemd methods directly;
- choose an unaudited actor or action classification.

Foundation operations are narrow groups:

| Concern | Read operations | Side-effect operations |
| --- | --- | --- |
| Core | application info, effective modules, settings, diagnostics | update theme/bootstrap settings; export safe diagnostics |
| Modules | effective registry and health | set module enablement/order/bounded settings |
| Launcher | list/get items | create/update/favorite, delete, launch/open saved item |
| systemd user | connectivity, services, timers, details, bounded logs | none |
| systemd system | connectivity, services, timers, details, bounded logs | none |

There is no foundation systemd write command hidden behind the UI. The Rust command is absent, not merely disabled.

## Action classification and confirmation

| Classification | Meaning | Foundation examples | Confirmation |
| --- | --- | --- | --- |
| `read` | No intended persistent or external state change | list units, get timer details, list launcher items | none |
| `write` | Reversible record change or deliberate external launch/open | create/edit/favorite item, set module preference, launch item | normally none; launch control names target |
| `privileged` | OS policy may elevate authority | none in foundation | explicit target-specific flow in future |
| `destructive` | Deletes, disables, masks, or risks difficult recovery | delete launcher item | confirmation naming exact item; future agents need human approval |

Classification is declared by the application use case, not trusted from the request. A confirmation token, when introduced for privileged/destructive remote actors, must be action- and target-bound, short-lived, single-use, and verified in Rust. The local foundation delete dialog is a UX guard; Rust still validates identity and authorization.

## Structured launcher execution

The execution port receives a typed structure:

```text
executable
arguments[]
working_directory?
environment_overrides{}
```

Foundation policies:

- never pass these fields to `sh -c`, a terminal, `eval`, or a concatenated string;
- reject embedded NULs and apply documented length/count bounds;
- require a folder and working directory to be absolute after lexical validation;
- accept an executable as an absolute path or a single basename with no separators, resolved against explicit configured search paths;
- do not search the current working directory and do not interpret shell aliases, expansions, redirections, pipes, globbing, or substitutions;
- provide an empty environment override map for persisted foundation launcher items and never audit the inherited environment;
- route URLs/folders through a fixed desktop opener adapter with one validated target argument;
- detach/capture output only according to a bounded platform policy; never retain unbounded stdout/stderr.

URL validation initially permits `http` and `https`. Other schemes require a documented allowlist change because desktop handlers can produce side effects. Folder creation is not implied by opening. Missing, non-directory, non-executable, or permission-denied targets produce distinct typed errors.

The launcher audit target uses item ID and display title. Metadata may include kind and a redacted outcome, never complete arguments, environment, or sensitive target/query values.

## Path and file safety

All application path categories come from the trusted path resolver, not React. Development/test `ARGOS_HOME` is absolute, explicitly provided, and rejected when it is the repository or a descendant. Relative paths, empty roots, repository-derived defaults, and insecure runtime fallbacks fail closed.

Application-owned directories and files use user-only permissions where practical (`0700` directories and `0600` data/config/state files, subject to platform APIs and existing ownership). Symlink-sensitive writes use safe create/replace primitives and verify ownership/expected location. Foundation has no generic managed-unit write operation.

A future managed-unit adapter must constrain targets beneath the standard user unit directory, reject traversal and symlink escapes, require an ownership marker plus stable ID, refuse unmanaged content, atomically replace only validated managed files, and preserve backups/drift evidence. These requirements are tested before such an adapter is enabled.

## Tauri capability model

Capabilities are separate files/permission groups by concern even if the foundation has one main window:

| Capability group | Intended commands | Foundation grant |
| --- | --- | --- |
| core-window | minimal window lifecycle and native behavior | main window only |
| core-read | app info, effective modules, settings, diagnostics | main window |
| core-write | settings/module preference and safe diagnostic export | main window |
| launcher-read | list/get launcher | main window |
| launcher-write | create/update/favorite/delete | main window |
| launcher-execute | launch/open a saved item | main window |
| systemd-user-read | user manager/unit/timer/log reads | main window |
| systemd-user-write | future mutations | absent |
| systemd-system-read | system manager/unit/timer/log reads | main window |
| systemd-system-write | future privileged mutations | absent |

Tauri built-in/plugin permissions are minimized; shell and broad filesystem plugins are not granted. The API transport exports only known wrappers, and command permission generation/allowlists are reviewed in CI. A future remote/self-hosted content window uses a different label and receives no system-control, launcher execution, file, or process capability. Initial n8n/Docker-like resources open in the external browser.

## systemd and privilege safety

Every request contains a required `SystemdScope`; frontend defaults are never interpreted as backend fallback. User and system connection states are reported independently. Permission denied, manager unavailable, bus unavailable, timeout, and adapter failure have different error codes/health reasons.

The foundation calls read D-Bus methods/properties only. It does not construct or register write methods. `journalctl` is invoked directly with a unit selector, scope option, JSON output, no pager, quiet behavior, and a numeric result limit clamped in Rust. Unit names are passed as individual arguments and validated as unit identifiers, not parsed as options.

Argos never starts as root or asks to relaunch itself as root. Future system-scope mutations use systemd D-Bus authorization and polkit for each narrow action. Editing `/etc/systemd` or other system unit locations requires a separate privileged-helper design and is not implied.

## Audit integrity and privacy

Normal application mutation paths append an audit event with:

```text
UUID, UTC time, actor, module, action, classification,
target type/ID/display name, result, optional error code, bounded metadata
```

Database-local mutations and their audit rows share a transaction. Failed validation may be traced but is not required to create persistent audit noise. External actions commit an `attempted` record before execution and append the outcome afterward using one correlation ID; an unavailable attempt audit blocks the side effect. Audit rows are append-only through application repositories: no update/delete use case exists in the foundation. Retention/export policy changes require a later decision.

Metadata is key-allowlisted, serialized to a fixed maximum size, and excludes secrets, tokens, full environments, command argument arrays, arbitrary file contents, full process output, and complete personal records. Human display names are convenience fields; stable target IDs establish identity.

SQLite alone cannot make audit history tamper-proof from the owning OS user. The foundation promises application-level attribution and append-only behavior, not cryptographic non-repudiation.

## Error and diagnostic disclosure

User-visible errors use stable generated codes and safe messages. Rust debug chains, SQL, filesystem internals beyond resolved diagnostic paths, D-Bus implementation details, and command output are retained only in appropriately redacted structured diagnostics. The UI branches on code/structured fields, never string parsing.

Safe diagnostic export contains system facts needed for support but no launcher records/targets, unit logs, environment variables, database contents, tokens, or arbitrary home-directory data.

## Security verification gates

Foundation approval requires static and behavioral evidence that:

- only the API transport imports Tauri invocation/event APIs;
- no shell string, generic file, generic SQL, or arbitrary operation command exists;
- launcher fields remain separate arguments at the process boundary;
- systemd exposes no write operation or capability;
- scope does not fall back;
- production/development/test roots do not collide or default to the repository;
- journal results and diagnostic/log buffers are bounded;
- remote content cannot inhabit the privileged main WebView;
- destructive confirmation identifies the exact target;
- audit metadata redaction and limits are tested;
- future managed-file path tests exist before managed writes are introduced.
