---
id: CODEX-H
title: Dead-code purge (partial — TagCache + client.rs allow-list deferred)
owner: codex
status: merged
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
---

## Brief

Backlog-tracked polish item (no standalone brief was authored — board "Post-1.0.0 polish" item #2 served as the contract). Six concrete dead-code items:

- `TagCache` struct in `src/tag_manager.rs:73-113` — entirely `#[allow(dead_code)]`; never wired into `TagManager`. Either build the feature or delete the type.
- `PlcManager::health_check_interval` field at `src/plc_manager.rs:95` — initialized to default, never read.
- Nine `#[allow(dead_code)]` annotations in `src/client.rs` (lines 1617, 2112, 2163, 3326, 3837, 6486, 6597, 6607, 6628). Per-method audit needed; most are unused FFI helpers or half-finished features.
- `BOOL_ARRAY_DWORD` dead `else if` branch at `src/protocol/values.rs:158-176` — `len >= 4` always matches before `len >= 8`.
- Leftover `#[allow(dead_code)] fn serialize_value` at `src/client.rs:3326`.

Note: removing `TagCache` from the public re-export at `src/lib.rs:150` is SemVer-major; defer to the 1.0.0 release-window bundle (CODEX-K) if not done now.

## Codex log

### 2026-05-26  codex [gpt-5]

Removed `PlcManager::health_check_interval` (field initialized to default, never read) and the dead `BOOL_ARRAY_DWORD` `else if value_data.len() >= 8` branch in `crates/protocol/src/values.rs`. Updated the residual error message to mention DWORD only. Deferred `TagCache` because it's publicly re-exported at `src/lib.rs:150` (`pub use tag_manager::{TagCache, ...}`) — removal is SemVer-major and belongs in CODEX-K, not a patch. Deferred the `src/client.rs` `#[allow(dead_code)]` audit (9 sites) — each needs per-method review. Bundled with CODEX-G, CODEX-I, and CODEX-O at commit `2690669`.

## Claude review

### 2026-05-26  claude [Opus 4.7]

Reviewed as part of the consolidated `2690669` bundle. Full structured review entry in [`docs/agents/log.md`](../log.md) (2026-05-26 entry). Two items merged; three deferred. Deferrals are correct: `TagCache` removal is provably SemVer-major (verified via `grep "pub use tag_manager" src/lib.rs`), and the `src/client.rs` audit needs careful per-site review to distinguish "unused FFI helper kept for ABI stability" from "actually dead". Independent verification: workspace builds clean, no clippy regressions.

## Verdict

### 2026-05-26  claude [Opus 4.7]

**Merged at `2690669`** as partial. Remaining deferred items (`TagCache`, `client.rs` allow-list audit, `serialize_value`) tracked under "CODEX-H residual" in the polish backlog. Zero defects on the merged portion. Patch-eligible.
