# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Open

*(no open tasks — protocol just bootstrapped)*

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|

## Done

*(no merged tasks tracked through this protocol yet — work prior to bootstrap is in git history)*

| Id | Title | Owner | Merge commit |
|---|---|---|---|

## Project context

- **Last released version:** `v0.8.0` (per recent commits — release metadata at `844079e` / `972b10b`).
- **Current development focus:** the .NET stack — C# wrappers and examples (per `CLAUDE.md` Project Overview).
- **Hardware validation gate:** integration tests against real CompactLogix / ControlLogix PLCs are the maintainer's responsibility; CI runs `SKIP_PLC_TESTS=1` plus simulator-backed `plc_sim_tests`.

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
