# Argos documentation

## Purpose and status

This directory is the normative design for Argos. It explains why the product exists, fixes the architectural boundaries, and turns the foundation milestone into verifiable implementation units.

The foundation documentation set was **approved for implementation on 2026-07-30**. Implementation must remain traceable to these documents and update them before any behavior diverges.

Normative words such as **must**, **must not**, **should**, and **may** are intentional. If implementation evidence invalidates a requirement, update the affected specification and decision record before changing code.

## Reading order

1. [Product definition](product.md) — vision, users, scope, non-goals, terms, and milestones.
2. [Architecture](architecture.md) — layers, code boundaries, module model, frontend, Linux adapters, performance, and diagnostics.
3. [Security](security.md) — trust boundaries, action classification, process execution, Tauri capabilities, and agent constraints.
4. [Data and contracts](data-and-contracts.md) — XDG paths, profiles, SQLite, schema, migrations, backups, and Rust/TypeScript contracts.
5. [Decision register](decisions.md) — accepted architectural decisions and consequences.
6. [Development guide](development.md) — setup intent, conventions, SDD workflow, module addition, generation, migrations, and release direction.
7. [Verification strategy](verification.md) — automated, target-machine, security, performance, and consistency gates.
8. [Foundation specifications](foundation/README.md) — workstreams, dependency order, phase gate, and detailed specifications.
9. [Foundation expected behavior](foundation/expected-behavior.md) — test-first Given/When/Then scenarios for every foundation acceptance criterion.
10. [Foundation task ledger](foundation/tasks.md) — stable, dependency-ordered implementation tasks.

## Document map

| Concern | Normative source |
| --- | --- |
| Product outcomes and exclusions | [Product definition](product.md) |
| Layering, crates, frontend layout, modules, systemd, diagnostics | [Architecture](architecture.md) |
| Privilege, commands, capabilities, audit, remote content | [Security](security.md) |
| Runtime paths, profile isolation, database, schemas, contracts | [Data and contracts](data-and-contracts.md) |
| Why architectural choices are fixed | [Decision register](decisions.md) |
| Contributor and specification workflow | [Development guide](development.md) |
| Quality and acceptance evidence | [Verification strategy](verification.md) |
| Foundation feature requirements | [Foundation specifications](foundation/README.md) |
| Testable foundation behavior | [Foundation expected behavior](foundation/expected-behavior.md) |
| Executable implementation sequence | [Task ledger](foundation/tasks.md) |

## Authority and change control

The accepted [decision register](decisions.md) has highest authority. Architecture, security, and data documents refine those decisions. Foundation specifications define milestone behavior, and the task ledger schedules it. A lower-level document must not contradict a higher-level one.

For every implementation unit, follow this lifecycle:

```text
Problem -> User value -> Requirements -> Scope -> Acceptance criteria
        -> Architecture impact -> Plan -> Tasks -> Implementation
        -> Verification -> Documentation update
```

When implementation proves the design wrong:

1. stop the affected implementation;
2. update the relevant specification and explain the evidence;
3. update or supersede affected decisions;
4. adjust dependent tasks and verification criteria;
5. only then resume implementation.

Silent divergence is a defect. Each material documentation change should name the affected specification or decision IDs in its commit or pull-request description.

## Documentation design

The set deliberately uses one decision register instead of one file per decision, and one task ledger instead of duplicating tasks in several plans. Separate foundation specification files are justified because they have different trust boundaries, failure modes, and acceptance evidence. No file is an empty placeholder.
