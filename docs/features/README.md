# Argos feature specifications

## Purpose and phase gate

This directory owns behavior proposed beyond the approved foundation scope. A feature specification, expected-behavior test, and task ledger may be prepared while foundation work continues, but documentation does not authorize implementation.

Implementation requires all of the following:

1. the feature specification is explicitly approved by the user;
2. the selected task is dependency-ready;
3. the user authorizes that task;
4. its implementation, tests, documentation, and completion evidence remain synchronized.

The foundation ledger remains authoritative for foundation tasks. Feature tasks do not silently reorder, replace, or expand the F1 milestone.

## Specification set

| ID    | Specification                      | Status                       | Expected behavior                                    | Task ledger               |
| ----- | ---------------------------------- | ---------------------------- | ---------------------------------------------------- | ------------------------- |
| TM-01 | [Task Manager](01-task-manager.md) | Implemented; target verification pending | [TM-01 scenarios](task-manager-expected-behavior.md) | [Feature tasks](tasks.md) |

## Current feature task

`TMG-001` through `TMG-004` are complete. `TMG-005` remains pending for the full ten-minute active/inactive CPU observation, 30-minute memory/write observation, and interactive target accessibility matrix; the optimized build, real GNOME launch/clean exit, 305-process snapshot smoke, and read-only static gates already pass.
