# Argos agent instructions

## Purpose

This file defines how agents work in this repository. Product behavior and architecture remain in the normative documents under `docs/`; do not duplicate or weaken them here.

## Start every task

1. Read `docs/README.md` for document authority and reading order.
2. Read `docs/architecture.md` and the normative documents relevant to the task.
3. Check `README.md` and `docs/foundation/README.md` for the current phase gate.
4. Inspect the current files and git state before proposing or making changes. Preserve unrelated work.
5. Identify the owning specification, acceptance criteria, task ID, and dependencies before implementation.
6. Check the skills available in the current environment and use any skill whose description matches the task.

If the task description, target, expected behavior, or acceptance criteria are unclear, investigate read-only context first, then ask the user for confirmation before editing files or running mutating commands. Do not guess the user's intent.

## Instruction and document authority

Explicit user instructions take precedence. Within the repository, follow this order:

1. accepted decisions in `docs/decisions.md`;
2. `docs/architecture.md`, `docs/security.md`, and `docs/data-and-contracts.md`;
3. approved feature or foundation specifications;
4. `docs/foundation/tasks.md` and task-specific plans;
5. implementation and supporting documentation.

Stop and surface a conflict before changing a lower-authority artifact. Planning or documenting work does not authorize implementation.

## Specification-driven development

All behavior changes use this sequence:

```text
Problem -> User value -> Requirements -> Scope -> Acceptance criteria
        -> Architecture impact -> Plan -> Tasks -> Implementation
        -> Verification -> Documentation update
```

- Do not implement behavior without an approved owning specification and authorized task.
- A specification must define in-scope and excluded behavior, workflows and failure states, acceptance criteria, affected boundaries, security/performance impact, task order, and verification.
- If a request changes behavior not covered by an approved specification, update the owning specification, decisions, task ledger, and verification plan as needed before code. Obtain approval before implementation.
- Execute only the approved task, in dependency order. Do not silently include adjacent tasks.
- Keep specifications, contracts, tests, implementation, task status, and documentation synchronized in the same change when they are affected.
- If implementation evidence contradicts a specification, stop. Update and approve the normative documents before resuming.
- A docs-only request remains docs-only. The documentation phase gate forbids application scaffolding, dependency installation, migrations, generated bindings, and runtime implementation until explicitly approved.

When compatible SDD skills are available, use the skill for the current phase—clarification, specification, planning, task breakdown, consistency analysis, or implementation—without skipping gates or combining phases that require user approval.

## Skills

- Treat the active environment's skill list and repository `.agents/skills/` as the skill catalog.
- When the user names a skill or the task clearly matches one, read its complete `SKILL.md` before acting and follow it for that task.
- Use the smallest set of skills that fully covers the request. State which skills are being used and why.
- Prefer a focused existing skill over recreating its workflow in prompts or scripts.
- If no skill matches, follow the repository documents directly. Do not invent, install, or modify a skill unless requested.
- Keep future repository skills focused on one repeatable workflow. Put deterministic processing in scripts only when instructions and existing tools are insufficient.

## Scope and implementation discipline

- Do only what the user requested and what is obviously necessary to make it correct.
- Do not speculate about future needs or add features, refactors, optimizations, cleanup, or broad hardening without a request.
- Prefer deletion, existing repository patterns, standard-library or native platform behavior, and already-installed dependencies, in that order.
- Do not add an abstraction, utility, wrapper, configuration option, or indirection used only once.
- Do not add fallback logic or error handling for implausible scenarios. Preserve validation at trust boundaries, data-integrity and data-loss protections, security controls, and accessibility requirements.
- Fix root causes at the narrowest shared boundary after checking all callers. Do not patch only one symptom when sibling paths share the defect.
- Delete code confirmed to be useless. Do not leave commented-out code, tombstones, or explanatory comments about its removal.
- Avoid new dependencies unless the approved task requires one and `docs/development.md#dependency-policy` is satisfied.
- Keep the diff as small as possible while fully meeting the approved acceptance criteria.

## Architecture guardrails

`docs/architecture.md` is the canonical architecture and directory specification. The required dependency direction is:

```text
React -> typed frontend API -> thin Tauri boundary -> application -> domain/ports -> adapters
```

- Dependencies point inward; domain and application code remain independent of Tauri and concrete adapters.
- Rust owns trusted database, filesystem, process, systemd, validation, permission, and audit operations.
- React uses the semantic typed API only; raw Tauri calls stay in the sole transport boundary.
- Keep SQLite, D-Bus, process, and filesystem details inside their owning adapters.
- Never use a generic shell/filesystem API or concatenate command lines. Use narrow operations and literal argument arrays.
- Keep production, development, and test data outside the repository using the documented XDG/profile rules.
- Foundation systemd behavior is read-only until a separately approved specification authorizes mutations.
- Add crates, modules, registries, or directory boundaries only through an approved architecture/documentation change.

## Verification

- Derive checks from the owning acceptance criteria and `docs/verification.md`.
- Use the narrowest relevant check during development, then run every required task gate before completion.
- The commands in `docs/development.md` are planned contracts until the scaffold implements them. Never claim an unimplemented command was run.
- After bootstrap, prefer repository-defined commands and run `pnpm check` when the task's completion condition requires the full gate.
- Add the smallest test that would fail for non-trivial changed logic. Do not build unrelated test infrastructure.
- Report exactly what was run, what passed, and what could not be run. Never mark a task complete without its required evidence and documentation update.

## Project memory graph

Use the existing documents as the durable memory graph; update the smallest owning node instead of creating parallel notes.

| Question | Canonical memory |
| --- | --- |
| What and why are we building? | `docs/product.md` |
| Which decisions are fixed, and why? | `docs/decisions.md` |
| How is the system structured? | `docs/architecture.md` |
| What are the trust and action boundaries? | `docs/security.md` |
| Where does data live and who owns contracts? | `docs/data-and-contracts.md` |
| How must contributors and agents work? | `docs/development.md` and this file |
| What evidence proves completion? | `docs/verification.md` |
| What behavior is specified? | `docs/foundation/*.md` and future approved feature specs |
| What runs next and what does it depend on? | `docs/foundation/tasks.md` and future task ledgers |

When a user correction, failed implementation, or review exposes a durable lesson:

1. fix the root cause;
2. update the owning specification or decision if intent changed;
3. add or strengthen the smallest test, lint rule, or acceptance check that prevents recurrence;
4. add one concise anti-pattern below only when the lesson is likely to recur and is not already enforced mechanically.

Do not record guesses, one-off typos, chat history, or duplicate lessons. Consolidate entries as the project grows. Add nested `AGENTS.md` files only when a subtree has genuinely different commands or rules; do not copy this file into subdirectories.

## Confirmed anti-patterns

- Implementing before the specification and phase gate are approved -> complete and approve the owning SDD artifacts first.
- Letting code silently diverge from normative documents -> stop, update the higher-authority document, then resume.
- Putting trusted logic or raw Tauri access in React -> route it through typed contracts and application use cases.
- Resolving live data from the repository or current directory -> use the documented XDG/profile resolver and repository guard.
- Exposing a generic shell, filesystem, SQL, or invoke-by-name surface -> expose one narrow typed operation instead.
- Adding scattered module navigation or service wiring -> use the centralized backend and frontend registries.
- Adding speculative abstraction, fallback behavior, or future-facing scaffolding -> remove it until an approved requirement needs it.
