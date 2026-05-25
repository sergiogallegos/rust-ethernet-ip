---
id: CODEX-Q
title: Service layer for restricted writes
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-Q agenda entry. Goal: add concrete service-layer helpers for documented STRING and UDT member write restrictions.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added `src/client/service_layer.rs`.
- Added `EipClient::write_string_tag()`.
- Added `EipClient::write_udt_member()` and `EipClient::write_udt_array_member()` using read-modify-write of the full UDT value through `UdtData::parse()` and `UdtData::from_hash_map()`.
- Added actor `Client` wrappers for the same service-layer methods.
- Verification:
  - `cargo check --features ffi --locked`
  - `cargo test --test client_actor_tests --locked`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy clean; workspace tests 236/0; hardware regression OK. New methods are additive on both `EipClient` and `Client` actor handle.

`write_string_tag` / `write_udt_member` / `write_udt_array_member` package the read-modify-write dance for the firmware-restricted paths documented in `docs/agents/notes/ab-firmware-quirks.md`. Removes the workaround ritual from consumer code. Stays scoped to the documented Logix STRING / UDT-array-member-write quirks — no generalization into a broader "service pattern framework" (per the brief's "stay concrete" guidance). Good restraint.

Zero defects. Hardware re-run targeted at these specific paths is a maintainer-owned follow-up — the existing 60 firmware-blocked tags in `test_plc_full_coverage.rs` are candidates to verify the new service methods unblock them.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Additive surface, narrow scope, zero defects. Future enhancement: thread these service methods through the C# and Python wrappers so the firmware quirk becomes invisible to those consumers too (separate brief).
