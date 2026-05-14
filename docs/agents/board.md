# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Open

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|

> All briefs from the original second-pass review have merged. Future polish candidates are listed in CODEX-D's verdict, and the SemVer-major release-window items are tracked in CODEX-F's verdict.

## Done

| Id | Title | Owner | Merge commit |
|---|---|---|---|
| CODEX-A | FFI safety, runtime hardening, and lint baseline | codex | `3d98abf` |
| CODEX-B | Contained API cleanup — thiserror, dead deps, dead state, must_use | codex | `9aca8d2` |
| CODEX-E | Small polish — runtime-init log dedupe, regex caching, re-export merge, dev-dep audit | codex | `fc63735` |
| CODEX-C | Decompose lib.rs into route, batch, types, and client modules | codex | `476f21c` |
| CODEX-D | Extract Encoder/Decoder boundary for the wire protocol | codex | `c58a905` |
| CODEX-F | RoutePath ordered hops + ASCII ethernet link-address encoding | codex | `9a3d192` |

## Project context

- **Last released version:** `v0.8.0` (per recent commits — release metadata at `844079e` / `972b10b`).
- **Current development focus:** the .NET stack — C# wrappers and examples (per `CLAUDE.md` Project Overview).
- **Hardware validation gate:** integration tests against real CompactLogix / ControlLogix PLCs are the maintainer's responsibility; CI runs `SKIP_PLC_TESTS=1` plus simulator-backed `plc_sim_tests`.

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
