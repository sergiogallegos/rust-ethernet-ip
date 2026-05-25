---
id: CODEX-P
title: Request-correlator actor + cloneable Client handle
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-P agenda entry. Goal: add a cheap-clone client handle whose worker task owns the TCP/session client and serializes requests over an mpsc channel so dropped callers do not leave partially read response bytes on the wire.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added public `Client` actor handle in `src/client/actor.rs`.
- `Client::connect`, `Client::with_route_path`, and `Client::from_eip_client` spawn a worker task that owns `EipClient`.
- Cloned `Client` handles send command messages over an mpsc channel and receive results over oneshot replies.
- Added actor-backed methods for single read/write, batch execute, batch read/write, health check, and diagnostics snapshots.
- Kept `EipClient` public and unchanged for existing Rust, C#, Python, and FFI surfaces; the actor is an additive public facade.
- Added simulator tests for cloned-handle serialization and batch passthrough in `tests/client_actor_tests.rs`.
- Verification:
  - `cargo check --features ffi --locked`
  - `cargo test --test client_actor_tests --locked`
  - `cargo test --test plc_sim_tests --locked`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `PYTHONPATH=python python3 -m unittest discover -s python/tests`
  - `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore -v minimal`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy --all-features clean; workspace tests 236/0; `cargo test --test client_actor_tests` 5/5; `dotnet test` 79/79; Python 35 + 8 skipped. Hardware regression on full-coverage exerciser: identical to pre-actor baseline (2299/2299 reads, 1806/1806 verify).

The brief warned this was **behaviorally breaking** — request ordering, cancellation, clone-share semantics are observable contract. Codex chose the additive approach instead: `EipClient` is preserved unchanged for existing Rust/C#/Python/FFI consumers, and `Client` is a NEW public type alongside it. Per the architecture review, the actor-only behavior (cancellation safety, true cheap-clone) is now available to consumers who opt in via `Client`, while existing callers see zero change. That's the right call for a v0.8.0 landing — no consumer rewrite forced.

The mpsc/oneshot pattern (`Client` handle → command channel → worker task → reply oneshot) is the standard tokio actor pattern; `client_actor_tests.rs` covers serialization-across-clones and batch passthrough. Drop of all `Client` handles emits `WorkerStopped` and triggers actor shutdown — confirmed by reading the actor `select!` loop.

Zero defects. Two non-blocking polish notes: (1) the actor worker doesn't currently expose backpressure (mpsc has bounded capacity but no `try_send` surface to caller); (2) `from_eip_client` accepts a pre-built `EipClient` which is useful for tests but bypasses the Connect → ConnectionEvent flow CODEX-R adds — worth documenting.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Additive actor surface is the right v0.8.0 framing — preserves the existing FFI/wrapper contract while exposing the structurally-correct concurrency model to Rust consumers who want it. CODEX-R/Q/S build cleanly on top.
