---
id: CODEX-R
title: Client connection event stream
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-R agenda entry. Goal: expose connection lifecycle events from the actor-backed client handle.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added public `ConnectionEvent` enum with `Connected`, `Disconnected`, and `WorkerStopped` variants.
- Added `Client::events()` returning a `tokio::sync::broadcast::Receiver<ConnectionEvent>`.
- Actor worker emits `Connected` on startup and terminal events when the final handle drops and the worker exits.
- Added simulator coverage in `tests/client_actor_tests.rs`.
- Verification:
  - `cargo test --test client_actor_tests --locked`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: workspace tests 236/0, `cargo test --test client_actor_tests` 5/5 (includes event-stream tests). Additive surface — new `ConnectionEvent` enum + `Client::events()` returning `tokio::sync::broadcast::Receiver<ConnectionEvent>`.

The event stream sources from the actor worker (CODEX-P) so the events reflect actual worker state transitions, not best-effort observation from caller code. `Connected` emits on actor startup; `WorkerStopped` on terminal shutdown. `Disconnected` is reserved for the eventual reconnect path (CODEX-P's actor is one-shot today, not reconnect-aware). HMI consumers can now learn about connection loss via push notification instead of pulling `ConnectionLost` errors out of next-operation results.

Zero defects. Future enhancement: emit `Reconnecting` / `SessionRecycled` once a real reconnect path lands. Brief explicitly notes the today-only subset — fine for v0.8.0.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Clean additive push-notification surface for connection state. Builds on CODEX-P's actor without coupling to its internal channels.
