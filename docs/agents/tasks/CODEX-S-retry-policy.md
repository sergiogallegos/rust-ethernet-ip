---
id: CODEX-S
title: RetryPolicy primitive
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-S agenda entry. Goal: add a retry-policy primitive and decorator-style client wrapper.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added `RetryPolicy`, `Backoff`, and `RetryClient` for the actor-backed `Client`.
- Added constant and exponential backoff constructors.
- Default policy retries reads only; writes require explicit `retry_writes(true)` opt-in.
- Retry predicate uses the existing `EtherNetIpError::is_retriable()`.
- Added simulator coverage for successful retry-client reads and opt-in writes.
- Verification:
  - `cargo test --test client_actor_tests --locked`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy clean; workspace tests 236/0. Additive — `RetryPolicy`, `Backoff`, `RetryClient` are new public types on the actor surface.

Builder/decorator combinator pattern (`client.with_retry(policy).read_tag(...).await`) is the right shape — non-invasive, composable. Reuses the existing `EtherNetIpError::is_retriable()` predicate (`src/error.rs:104`) so error-class semantics stay centralized. Default policy retries reads only — writes require explicit opt-in via `retry_writes(true)`. That's the conservative default; idempotent write semantics aren't guaranteed across all PLCs.

Constant + exponential backoff constructors cover the common cases. Decorrelated-jitter from the brief's wish-list is deferred — not blocking for v0.8.0.

Zero defects.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Centralizes retry policy so the C# and Python wrappers don't each maintain their own loop. Future enhancement: thread `RetryPolicy` config through to the wrappers (separate brief).
