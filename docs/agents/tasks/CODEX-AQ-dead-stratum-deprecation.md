---
id: CODEX-AQ
title: Dead-stratum deprecation — TagManager UDT pipeline, ProductionMonitor/Config, PlcManager, SubscriptionManager, TagCache; diagnostics honesty
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
---

## Brief

### Goal

Quarantine the pre-1.0 "enterprise checklist" stratum identified by the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §4): modules that are publicly exported and plausible-looking but wired to nothing — or worse, return fabricated data. This extends ROADMAP items 5–9 and the existing 2.0 removal list. Same SemVer posture as CODEX-AP: `#[deprecated]` + honest behavior on 1.x, deletion queued for 2.0.

1. **`TagManager` UDT pipeline — the dangerous one.** `EipClient::discover_udt_members` (`src/client.rs:497-505`, public) delegates to `TagManager::get_udt_definition`, whose request builder has an off-by-one path size (`src/tag_manager.rs:356-357`: `2 + div_ceil(len,2)` words vs the `1 + div_ceil` actually emitted — pinned *wrong* by its own tests at `tag_manager.rs:1046-1067`) and whose "parser" (`:373-442`) scans a read response for byte pairs resembling type codes, invents `Member_1…` names with sequential offsets, and falls back to a fabricated `Value: DINT` member. Real template-based parsing exists (`get_udt_definition` at `src/client.rs:2156`, `crates/udt`). Fix: `discover_udt_members` delegates to the real template path (breaking only in accuracy); delete the fabricating parser + its wrong pinned tests; deprecate any `TagManager` UDT surface that remains public.
2. **Discovery filter bugs in the live `TagManager` path** (used by `EipClient::discover_tags`): `validate_tag_name` (`tag_manager.rs:9-12, 471-491`) rejects `_`-prefixed, `Program:`-scoped, and bracketed names — all legal — silently dropping real tags from discovery; `is_structure()` (`:62-68`) checks `0x00A0..=0x00AF` while real structure handles surface as `0x02A0`-class values (its own `parse_tag_type` at `:723-734` derives structure-ness from bit 15 and then discards it) — fix both from the type-word semantics, plus: `parse_tag_list`'s bogus four-zero-bytes resync heuristic (`:610-634`) and unused `item_count` bound (`:573, 585`), fictional `dimensions = vec![0; dims]` array metadata (`:666-680`), the five `RwLock` `.unwrap()`s (`:319-507` — the file already has a `?`-based pattern at `:134`), and the never-called cache eviction (`:133-181`).
3. **`ProductionMonitor` / `config.rs`** — exported, consumed by nothing (grep-verified), `get_memory_usage`/`get_cpu_usage` return literal `10.0`/`5.0`, `start_monitoring` leaks an unstoppable task per call; `ProductionConfig` implies enforcement (rate limiting, memory limits, encryption) that exists nowhere, and its `Duration` fields don't round-trip human-written TOML. Deprecate both wholesale (module-level `#[deprecated]` on the re-exports), stop the task leak in whatever remains, and make `get_metrics()` carry the same `system_metrics_are_placeholders` honesty flag `DiagnosticsSnapshot` already has.
4. **FFI diagnostics honesty**: `build_diagnostics_snapshot` (`src/client/diagnostics.rs:86-110`) hardcodes every operation/performance counter to zero because nothing accumulates them — `eip_get_diagnostics_json` consumers can mistake "0 failed reads" for health. Either wire minimal real counters (per-client atomic op/error counts incremented in `send_rr_data_item` — cheap and honest) or emit `null`/absent fields with a schema note; pick the counters option unless it measurably complicates CODEX-AL's touched code — coordinate if both tasks are in flight.
5. **`SubscriptionManager` + aliases** (`src/subscription.rs:169-227`, exported twice via `lib.rs:144-148` legacy aliases) and **`TagCache`** (`tag_manager.rs:73-114`, `#[allow(dead_code)]` *and* exported) and **`PlcManager`** (`src/plc_manager.rs` — health lifecycle unreachable, `&mut self` pooling that can't pool, superseded by `Fleet`) — deprecate all with notes pointing at the living replacements (`EipClient` subscriptions / `TagManager` / `Fleet`). `PlcManager` is still used by two test files; migrate those tests to `Fleet` or direct clients.
6. **`benches/udt_discovery_benchmark.rs`** — every body re-implements logic inline (hand-written `contains`, string formatting into HashMaps) and calls no crate code; regressions in the real functions are invisible. Delete it, or rewrite the two defensible cases to call the real `TagManager` functions.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §4 (each item's file:line + grep evidence), `docs/ROADMAP.md` items 5–9 + 2.0 section, `src/lib.rs` re-export block (the public-surface inventory this task edits), the CODEX-H history (board + log) — `TagCache` deferral rationale ("SemVer-major, belongs in a release-window bundle") which this brief honors via deprecation-not-deletion.

### Files to create or modify

`src/tag_manager.rs` (bulk), `src/client.rs` (`discover_udt_members` delegation), `src/monitoring.rs`, `src/config.rs`, `src/subscription.rs`, `src/plc_manager.rs`, `src/lib.rs` (deprecated re-exports), `src/client/diagnostics.rs` (+ `src/ffi.rs` if the JSON schema gains fields), `benches/udt_discovery_benchmark.rs`, affected tests (`integration_test.rs`, `comprehensive_test.rs` for PlcManager migration; `tag_manager` unit tests re-pinned), `docs/ROADMAP.md`, `CHANGELOG.md`.

### Behavior

- `discover_udt_members` returns template-derived truth or a typed error — never invented members.
- `discover_tags` stops silently dropping legal tag names; structure detection matches the type-word semantics.
- Deprecated modules compile with warnings for consumers but change no signatures; every deprecation note names the replacement.
- Diagnostics JSON either reports real counters or explicitly marks placeholders; nothing reads as "healthy zero" when unmeasured.

### Test requirements

- Re-pin the `tag_manager` request-byte tests to the *correct* path size (the current tests certify the bug — say so in the test comment).
- New: `validate_tag_name` accepts `_Tag`, `Program:Main.Tag`, `Arr[3]`; structure detection on a `0x02A0`-class type word; `discover_udt_members` against the sim's template path if modelled, else against `crates/udt` fixtures.
- Counters (if chosen): sim test asserting read/write/error counts increment.
- Deprecation hygiene: internal callers use `#[expect(deprecated)]` with reasons; `-D warnings` stays green.
- Full matrix incl. C# + Python (FFI JSON shape may gain fields — additive only; verify wrapper JSON parsing tolerates unknown fields before adding any).

### Acceptance criteria

- Explicit per-item disposition in the Codex log (deprecated / deleted / fixed / migrated), grep-clean for the deleted parser and its tests.
- No `RwLock` `.unwrap()` remains in `tag_manager.rs`; no `#[allow(dead_code)]` co-exists with a `pub` export anywhere touched.
- `cargo semver-checks` (baseline 1.1.0) green; ROADMAP 2.0 deletion list updated; CHANGELOG `### Deprecated`/`### Fixed` entries.

### Out of scope

- Subscription/fleet runtime lifecycle bugs — [[codex-ar-subscription-fleet-lifecycle]] (this brief only deprecates the dead `SubscriptionManager` type; the live path's fixes are AR's).
- The string/UDT strategy graveyard — [[codex-ap-string-udt-graveyard]].
- Building a real monitoring backend — deprecation is the decision *not* to.
- Actual 2.0 deletions.

### Risks and gotchas

- `discover_udt_members`'s return type may differ from what the template path produces — check the C#/Python wrappers and `eip_get_udt_definition` for shape coupling before changing what populates it.
- The tag-list resync heuristic (`:610-634`) may be masking a real off-by-one in entry parsing — before deleting it, run `discover_tags` against the sim's tag-list responses and confirm clean parses; if the heuristic ever fires there, find the real bug first.
- `config.rs` deprecation: `ProductionConfig` might appear in doc examples/README — sweep docs so deprecated types aren't taught to new users.
- Two tasks (this and CODEX-AL) may touch `send_rr_data_item` if counters land there — sequence with the maintainer (AL first is cleaner).

## Codex log

## Claude review

## Verdict
