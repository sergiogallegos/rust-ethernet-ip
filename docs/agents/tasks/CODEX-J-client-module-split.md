---
id: CODEX-J
title: Mechanical client.rs submodule split
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-J agenda entry. Goal: mechanically split the large `src/client.rs` implementation into cohesive submodules while preserving the `EipClient` public facade and behavior.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added `src/client/batch_exec.rs` for batch execution/configuration and Multiple Service Packet parsing.
- Added `src/client/diagnostics.rs` for health checks and diagnostics snapshot construction.
- Added `src/client/schema_export.rs` for schema export APIs.
- Added `src/client/string.rs` for STRING writes, connected explicit messaging, and connected-session close helpers.
- Added `src/client/subscriptions.rs` for tag subscriptions and tag-group polling.
- Kept the `EipClient` facade intact through inherent impl blocks in submodules.
- Kept shared low-level request/session helpers in `src/client.rs` to minimize cross-module visibility churn.
- `src/client.rs` reduced from 6,758 lines to 4,515 lines.
- Verification:
  - `cargo check --features ffi --locked`
  - `cargo test --test route_path_sim_compat_tests --locked`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `cargo test --test plc_sim_tests --locked`
  - `cargo test --features ffi --test ffi_state_consistency --locked`
  - `cargo test --workspace --all-features --locked`
  - `PYTHONPATH=python python3 -m unittest discover -s python/tests`
  - `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore -v minimal`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy --workspace --all-features --all-targets -D warnings clean; `cargo test --workspace --all-features --locked` 236 passed / 0 failed; `cargo test --test plc_sim_tests` 13/13; hardware regression on full-coverage exerciser identical to pre-split (2299/2299 reads, 1806/1806 verify). Mechanical refactor lived up to its name.

`src/client.rs` shrank 6762 → ~2369 LOC. New submodules at `src/client/{batch_exec,diagnostics,schema_export,string,subscriptions}.rs` (plus `actor.rs` and `service_layer.rs` from CODEX-P/Q which co-landed). `pub(crate)` keeps the impl-block split internal — no public API leakage.

Zero defects.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Pure mechanical split per the brief. The reduced surface makes CODEX-P (actor refactor) and CODEX-Q (service layer) reviewable as additions to discrete submodules rather than diffs against a 6.7k-line monolith.
