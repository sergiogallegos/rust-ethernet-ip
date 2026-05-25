---
id: CODEX-T
title: Fleet multi-PLC actor pool
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-T agenda entry. Goal: add an explicit multi-PLC pool built on the actor-backed `Client` API, with per-PLC handles, fleet-level health checks, and fleet-level connection events.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added `src/fleet.rs`.
- Added `Fleet<PlcId>` with:
  - `connect()`
  - `connect_with_route()`
  - `insert_client()`
  - `client()`
  - `events()`
  - `check_health()`
  - `len()` / `is_empty()`
- Added `FleetEvent<PlcId>` carrying the PLC id and forwarded `ConnectionEvent`.
- Re-exported `Fleet` and `FleetEvent` from the crate root.
- Added simulator coverage for fleet connect/client lookup, event forwarding, and health checks.
- Verification:
  - `cargo test --test fleet_tests --locked`
  - `cargo check --features ffi --locked`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy clean; workspace tests 236/0; `cargo test --test fleet_tests` 3/3.

**Code: clean.** `Fleet<PlcId>` is a thin pool over the actor-backed `Client` from CODEX-P. Per-PLC handles via `client(&id)`, fleet-level health checks, fleet-level event forwarding via `FleetEvent<PlcId>`. Pure additive — no public-API change to anything else. Builds correctly on CODEX-P/R.

**🟠 Process concern (maintainer must resolve):**

This brief was explicitly listed in the board agenda as a **v0.9.0 deferral**: "CODEX-T (fleet) and CODEX-U (sibling crates) are the largest items and the natural v0.9.0 deferrals if scope tightens." The 2026-05-24 directive lifted activation gates on L/M/N/O/V (post-books reviews), not on T/U. Codex went outside the v0.8.0 scope.

Code is correct and well-tested — but landing in this release expands scope:
- Adds new public type family (`Fleet`, `FleetEvent`) that consumers may build on
- Once shipped, any change to `Fleet` shape is a SemVer-major break

Two resolutions:

1. **Accept into this release (paired with CODEX-K as 1.0.0)** — code is done and tested; deferral re-adds it later for no benefit
2. **Hold for v0.9.0** — `git reset` `src/fleet.rs` + `tests/fleet_tests.rs` + the brief, re-land later

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged — maintainer accepted into 1.0.0

**Merged.** Maintainer selected option 1 (accept, paired with CODEX-K as 1.0.0). `Fleet` and `FleetEvent` are now public 1.0.0 surface; future changes to their shape are SemVer-major.
