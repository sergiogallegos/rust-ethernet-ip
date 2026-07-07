---
id: CODEX-AQ
title: Dead-stratum deprecation — TagManager UDT pipeline, ProductionMonitor/Config, PlcManager, SubscriptionManager, TagCache; diagnostics honesty
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-07 claude [Opus 4.8]
---

## Brief

### Goal

Quarantine the pre-1.0 "enterprise checklist" stratum identified by the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §4): modules that are publicly exported and plausible-looking but wired to nothing — or worse, return fabricated data. This extends ROADMAP items 5–9 and the existing 2.0 removal list. Same SemVer posture as CODEX-AP: `#[deprecated]` + honest behavior on 1.x, deletion queued for 2.0.

1. **`TagManager` UDT pipeline — the dangerous one.** `EipClient::discover_udt_members` (`src/client.rs:530`, public) builds its request via `TagManager::build_udt_definition_request` (`src/tag_manager.rs:345`, off-by-one path size: `2 + div_ceil(len,2)` words vs the `1 + div_ceil` actually emitted — pinned *wrong* by its own tests around `tag_manager.rs:1052-1067`) and "parses" the reply via `parse_udt_definition_response` (`:373`), which scans a read response for byte pairs resembling type codes, invents `Member_1…` names with sequential offsets, and falls back to a fabricated `Value: DINT` member. Real template-based parsing already exists on `EipClient::get_udt_definition` (`src/client.rs:1850`, `crates/udt`-backed). Fix: reimplement `discover_udt_members` on top of the real template path (breaking only in accuracy — it now returns real members or a typed error); delete the fabricating `build_udt_definition_request`/`parse_udt_definition_response` pair once orphaned, plus their wrong pinned tests; deprecate any `TagManager` UDT surface that remains public. Verify `self.tag_manager` has no other live consumer before deprecating the `TagManager` type itself.
2. **Discovery filter bugs in the live `TagManager` path** (used by `EipClient::discover_tags`): `validate_tag_name` (`tag_manager.rs:9-12, 471-491`) rejects `_`-prefixed, `Program:`-scoped, and bracketed names — all legal — silently dropping real tags from discovery; `is_structure()` (`:62-68`) checks `0x00A0..=0x00AF` while real structure handles surface as `0x02A0`-class values (its own `parse_tag_type` at `:723-734` derives structure-ness from bit 15 and then discards it) — fix both from the type-word semantics, plus: `parse_tag_list`'s bogus four-zero-bytes resync heuristic (`:610-634`) and unused `item_count` bound (`:573, 585`), fictional `dimensions = vec![0; dims]` array metadata (`:666-680`), the five `RwLock` `.unwrap()`s (`:319-507` — the file already has a `?`-based pattern at `:134`), and the never-called cache eviction (`:133-181`).
3. **`ProductionMonitor` / `config.rs`** — exported, consumed by nothing (grep-verified), `get_memory_usage`/`get_cpu_usage` return literal `10.0`/`5.0`, `start_monitoring` leaks an unstoppable task per call; `ProductionConfig` implies enforcement (rate limiting, memory limits, encryption) that exists nowhere, and its `Duration` fields don't round-trip human-written TOML. Deprecate both wholesale (module-level `#[deprecated]` on the re-exports), stop the task leak in whatever remains, and make `get_metrics()` carry the same `system_metrics_are_placeholders` honesty flag `DiagnosticsSnapshot` already has.
4. **FFI diagnostics honesty**: `build_diagnostics_snapshot` (`src/client/diagnostics.rs:87-110`) hardcodes every operation/performance counter to zero because nothing accumulates them — `eip_get_diagnostics_json` consumers can mistake "0 failed reads" for health. Either wire minimal real counters (per-client atomic op/error counts incremented in `send_rr_data_item` — cheap and honest) or emit an explicit "not tracked" honesty flag (mirroring the existing `system_metrics_are_placeholders`) so consumers cannot read a healthy-looking zero. CODEX-AL merged (`253706e`), so `send_rr_data_item` is at its settled post-AL shape (monotonic `sender_context`, poison guard) — the counters, if chosen, land on that path with no coordination pending. Prefer the counters option; fall back to the honesty flag if instrumentation risks scope creep.
5. **`SubscriptionManager` + aliases** (`src/subscription.rs:169-227`, exported twice via `lib.rs:144-148` legacy aliases) and **`TagCache`** (`tag_manager.rs:73-114`, `#[allow(dead_code)]` *and* exported) and **`PlcManager`** (`src/plc_manager.rs` — health lifecycle unreachable, `&mut self` pooling that can't pool, superseded by `Fleet`) — deprecate all with notes pointing at the living replacements (`EipClient` subscriptions / `TagManager` / `Fleet`). `PlcManager` is still used by two test files; migrate those tests to `Fleet` or direct clients.
6. **`benches/udt_discovery_benchmark.rs`** — every body re-implements logic inline (hand-written `contains`, string formatting into HashMaps) and calls no crate code; regressions in the real functions are invisible. Delete it, or rewrite the two defensible cases to call the real `TagManager` functions.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §4 (each item's file:line + grep evidence), `docs/ROADMAP.md` items 5–9 + 2.0 section, `src/lib.rs` re-export block (the public-surface inventory this task edits), the CODEX-H history (board + log) — `TagCache` deferral rationale ("SemVer-major, belongs in a release-window bundle") which this brief honors via deprecation-not-deletion. The immediate precedent is **CODEX-AP (merged `1821e59`)** — the same deprecate-on-1.x / delete-at-2.0 posture, the `EtherNetIpError::Unsupported { api, reason }` variant this task reuses for `discover_udt_members`' typed-error path, the `#[expect(deprecated, reason=…)]` internal-call-site pattern, and the `Cargo.toml` semver-checks lint-allow precedent. Line references in this brief were refreshed against `main` at `7cc17ac` (post-AP); re-verify each before touching, per the standard risk note.

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
- CODEX-AL is merged (`253706e`), so `send_rr_data_item` is settled — if counters land there, build on the post-AL path; no cross-task sequencing remains.
- **semver-checks deprecation lints.** CODEX-AP established the precedent of a narrow `[package.metadata.cargo-semver-checks.lints]` allow for deprecation markers (`type_method_marked_deprecated = "allow"` in `Cargo.toml`). This task deprecates whole `pub`-exported structs/modules, so it will additionally trip the struct/enum/type-level `*_marked_deprecated` lints — add the corresponding narrow allows following the AP precedent, and **stop and report** if semver-checks flags any item as MAJOR (that means a real removal slipped in — deprecation must not remove signatures on the 1.x line).

## Codex log

2026-07-07 codex [GPT-5] Submitted: removed the fabricating TagManager UDT definition request/parser behavior and rerouted both `EipClient::discover_udt_members` and the live TagManager UDT-member discovery path through the real `get_udt_definition` template parser. The formerly public `TagManager::build_udt_definition_request` and `parse_udt_definition_response` names remain only as deprecated 1.x SemVer stubs returning `Unsupported`. Fixed live discovery filters: legal `_Tag`, `Program:Main.Tag`, and `Arr[3]` names pass; `0x02A0`-class structure type words are recognized; malformed tag-list pages now error instead of four-zero-byte resyncing; array dimensions are no longer fictional zeros; UDT cache access has no `RwLock` unwraps. Deprecated dead compatibility structs with replacement notes and `#[expect(deprecated)]` hygiene: `ProductionMonitor`, `ProductionConfig`, `SubscriptionManager`/`RealTimeSubscriptionManager`, `TagCache`, and `PlcManager`; `ProductionMonitor::start_monitoring` no longer spawns an unstoppable placeholder task, and `MonitoringMetrics::system_metrics_are_placeholders()` exposes the placeholder status without a SemVer-breaking field addition. Added per-client atomic operation/error counters on the CIP send path and wired diagnostics snapshots to those counters. Migrated the PlcManager-using tests to direct/mock clients, deleted the fake `udt_discovery_benchmark`, enabled Tokio `signal` so the simulator example builds from a clean target for C# tests, and updated semver-checks lint metadata, CHANGELOG, ROADMAP, task board, and wiki. Verification passed: fmt-check, clippy workspace all-targets/all-features, `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`, `cargo test --test plc_sim_tests`, `cargo bench --no-run --locked`, `cargo build --release --features ffi --locked`, export-table spot check for `eip_connect`, `cargo doc --workspace --all-features --no-deps --locked`, `cargo semver-checks --baseline-version 1.1.0` (220 pass), Python 34/8 skipped, C# unit 86/86, C# native integration 7/7, `validate-agent-files`, `git diff --check`, and cleanup grep.

## Claude review

### 2026-07-07 18:40  claude [Opus 4.8]

**Independent verification**
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean (deprecation hygiene holds; internal uses guarded).
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — all suites 0 failed (tag_manager unit 55-block incl. new discovery-filter tests; error/diagnostics/etc).
- `cargo test --test plc_sim_tests --locked` — 24/24.
- `cargo semver-checks check-release --baseline-version 1.1.0` — 220 pass, "no semver update required".
- `cargo bench --no-run --locked` — builds; `udt_discovery_benchmark` gone, only `performance_benchmark` remains.
- `cargo doc --workspace --all-features --no-deps --locked` — clean, no warnings.
- `cargo build --release --features ffi --locked` — ok.
- C# unit 86/86; C# native integration 7/7 (against the freshly built cdylib); Python 34 passed / 8 skipped / 0 failed.

**What's being fixed**
- The pre-1.0 "enterprise checklist" stratum — publicly exported modules wired to nothing or returning fabricated data — is quarantined: fabricating UDT discovery replaced by the real template path, live-discovery filter bugs fixed, dead modules deprecated, and diagnostics counters made real instead of hardcoded zero.

**Root cause confirmation**
- Confirmed. `EipClient::discover_udt_members` (`src/client.rs:814`) now delegates to the real `get_udt_definition` (`crates/udt`-backed) and maps real members to `TagMetadata` — no more invented `Member_1…`/`Value:DINT`. The fabricating `TagManager::build_udt_definition_request`/`parse_udt_definition_response` (`tag_manager.rs:322,334`) are now deprecated `Unsupported` stubs (kept, not deleted, because they were `pub` — deletion is 2.0; more SemVer-correct than the brief's "delete" wording). The old byte-pinned tests that certified the off-by-one are gone (grep clean for `Member_1`/`div_ceil`/path-size assertions).
- Discovery filters: `validate_tag_name` (`tag_manager.rs:346`) + `TAG_NAME_RE` now accept `_Tag`, `Program:Main.Tag`, `Arr[3]`, nested paths (tests at `:767`); `is_structure_type_word` (`:69`) keys on bit-15 / `0x02A0` not just the dead `0x00A0` range; `parse_tag_list` (`:453`) is count-driven off a correctly-offset `item_count` (accounting for additional-status words) with strict bounds errors — the four-zero-byte resync heuristic and fictional `vec![0; dims]` array metadata are gone; the five `RwLock` `.unwrap()`s are replaced with `?`/`.ok()` (only test unwraps remain).
- Diagnostics: `DiagnosticCounters` (`client.rs:65`, `Arc`-shared on clone) accumulates real atomics wired at the `send_cip_request` chokepoint (`:3472–3524`, classified by CIP service byte) plus batch (`:4364`); `build_diagnostics_snapshot` now reads `operation_metrics()`/`error_metrics()` instead of hardcoded zeros. `ProductionMonitor::start_monitoring` no longer spawns the leaking task; CPU/mem stay explicit placeholders via `system_metrics_are_placeholders()`.

**Fix appropriateness**
- Right layer and right SemVer posture throughout: additive `#[deprecated]` on the dead structs (`ProductionMonitor`, `ProductionConfig`, `SubscriptionManager`/alias, `TagCache`, `PlcManager`) with replacement notes, signatures preserved (semver green), deletions queued for 2.0. The `Cargo.toml` `type_marked_deprecated = "allow"` mirrors the AP `type_method_marked_deprecated` precedent (minor-compatible; `function_missing` still guards removals). Counters chosen over the honesty-flag fallback — the more valuable option — at a central chokepoint rather than smeared across call sites.
- `tokio` `signal` feature add is genuine: `examples/python_test_simulator.rs` uses `signal::ctrl_c()` and previously relied on feature unification a now-changed dep provided.

**Test proof**
- New `tag_manager` tests pin the corrected behavior: legal-name acceptance, `0x02A0` structure detection, count-driven `parse_tag_list` (`MotorData` fixture), UDT-member discovery via the real path. `PlcManager` tests migrated to direct clients (`integration_test.rs`, `comprehensive_test.rs`). Full C#/Python matrix green (diagnostics JSON schema unchanged — `ffi.rs` untouched — so counters populate previously-zero fields with no wrapper-parse risk).

**Residual risk**
- Sim/unit-level only. Real multi-page hardware tag-list discovery and real counter accumulation over a hardware session are not exercised here; the sim is an oracle (post-AN) and discovery tests pass against it. Fits the standard pre-1.2.0 hardware-smoke pattern.
- Counters are best-effort honesty, not a metrics SLA: fragmented/other CIP services return `None` from the classifier and aren't counted (honest undercount, not miscount).

**Strong points (✅)**
- `discover_udt_members` reuses the one real UDT parser instead of a parallel fabricating path — the dangerous "invented data as authoritative" case is eliminated.
- `parse_tag_list` converted from pattern-scanning to a bounded, count-driven parse with typed truncation errors — malformed pages now fail loudly instead of silently resyncing.
- Deprecating the `pub` fabricating methods to `Unsupported` stubs (vs deleting) is the correct 1.x SemVer read; the brief's "delete" was looser than SemVer allows and Codex chose the safer disposition.
- Counters are `Arc`-shared so all clones (pollers, FFI registry, tag-group tasks) accumulate to one set — consistent with the post-AL shared-handle model.

**Findings**
- 🟡 `validate_tag_name` still rejects double-underscore names (`My__Tag`) via `contains("__")`; double underscore is legal in Logix, so such tags are still silently dropped from discovery. Narrower than the fixed cases and rare in practice — non-blocking, worth a follow-up.
- 🟡 In `send_cip_request`, when `extract_unconnected_data_item` fails (malformed CPF), the counter records success (`client.rs:3521–3523`) even though the reply is unparseable; the caller still gets `Ok(response)` and may fail parsing downstream uncounted. Slight success over-count in a rare malformed-response path; cosmetic honesty edge.
- 🟢 `subscription_updates` stays `0` (subscriptions not counted) — honest "not tracked", not a fabricated value.
- 🟢 `tokio` `signal` feature add is real (example `ctrl_c`), additive.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Explicit per-item disposition in the Codex log (deprecated / deleted / fixed / migrated); grep-clean for the deleted fabricating parser behavior and its wrong pinned tests.
- ✅ No `RwLock` `.unwrap()` remains in `tag_manager.rs` (only `#[cfg(test)]` unwraps); no `#[allow(dead_code)]` co-exists with a `pub` export in any touched file.
- ✅ `cargo semver-checks` (baseline 1.1.0) green; ROADMAP 2.0 deletion list updated; CHANGELOG `### Deprecated`/`### Fixed` entries present.
- ✅ Diagnostics report real counters; CPU/mem/system metrics explicitly flagged as placeholders — nothing reads as "healthy zero" when unmeasured.

## Verdict

### 2026-07-07 18:40  claude [Opus 4.8]

**Merged.** Full independent matrix green (fmt, clippy `--all-targets --all-features -D warnings`, workspace `--all-features --locked`, plc_sim 24/24, semver-checks 220-pass/no-update, bench `--no-run`, `cargo doc` clean, release ffi build, C# 86/86 + integration 7/7, Python 34-pass/8-skip). All six brief items dispositioned correctly: the fabricating UDT discovery is replaced by the real template path, the live-discovery filter bugs are fixed with a bounded count-driven `parse_tag_list`, the dead stratum is deprecated with 2.0-queued removals, diagnostics counters are real `Arc`-shared atomics, and the fake benchmark is gone. The one place the implementation improves on the brief — keeping the `pub` fabricating `TagManager` methods as deprecated `Unsupported` stubs rather than deleting them — is the correct 1.x SemVer disposition. Two 🟡 items (double-underscore name rejection; success-count on unparseable CPF) are non-blocking honesty/edge notes. Zero defects, zero Claude-applied fixes. Merged at `272e0ae`.
