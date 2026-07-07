---
id: CODEX-AP
title: Retire the string/UDT strategy graveyard — never-worked public paths return honest errors
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-07 claude [Opus 4.8]
---

## Brief

### Goal

The 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §1 "Public APIs that cannot ever have worked") identified a cluster of exploratory protocol code in `src/client.rs` / `src/client/string.rs` that is publicly exported but provably non-functional or hazardous. Removing public API is SemVer-major, so on the 1.x line this brief follows the established `eip_get_tag_metadata` precedent: each retired path returns an explicit `Unsupported`-style error (and gains `#[deprecated]`), with actual deletion queued for 2.0 (add to `docs/ROADMAP.md`'s 2.0 section). Internal dead code is deleted outright.

The retirement list (verification in the analysis doc; re-verify each before touching):

1. `write_string` (`src/client/string.rs:1003-1040`) — request missing the path-size byte; status read from the service-reply byte, so success reports as `WriteError { status: 0xCD }`. Two independent defects; never worked. The working path (`write_string_tag` → `write_tag(PlcValue::String)`) is untouched and becomes the documented redirect.
2. `write_ab_string_udt` (`string.rs:121-133`) — checks byte 2 of the raw CPF envelope (always 0), returns `Ok(())` unconditionally; payload is not a valid struct write. Silent false success — the most dangerous shape.
3. Connected messaging / Forward Open subsystem (`string.rs:190-452, 603-632` — `establish_connected_session`, `parse_forward_open_response`, `write_string_connected`, `send_connected_cip_request`) — response parsed at the wrong layer so every Forward Open fails after 6×100 ms; layered status checks read reserved bytes; connected reads have no timeout. Dead on arrival end-to-end.
4. `write_string_unconnected` (`string.rs:822-951`) — reverse-engineered payload ("Structure appears to be…", type `0x0FCE`, no element count) matching no documented Write Tag shape.
5. `write_ab_string_components` (`string.rs:13-68`) — one SINT per round trip (82 RTTs worst case), non-atomic `.LEN`-first sequence, and depends on the malformed `.DATA[i]` segment being fixed by CODEX-AM. **Decision point superseded by CODEX-AT:** top-level standard STRING writes work through the direct structure encoding, so this path is at most a non-atomic fallback for component experimentation, not the primary workaround. Record the decision + evidence in the log.
6. `read_udt_chunked` strategies B–D (`src/client.rs:1719-2021`) — `read_udt_chunk_advanced` emits protocol-invalid requests and returns error bytes as data; two loops don't terminate against a peer answering full-size chunks (unbounded growth); strategy D converts total failure into `Ok(Udt { data: vec![] })`. Keep strategy A only if it is the plain fragmented-read fallback that real code paths rely on — trace callers first; the dispatch trigger (`msg.contains("Partial transfer")`) must become a typed error match on whatever remains.
7. `read_udt_member_by_offset` / `write_udt_member_by_offset` (`src/client.rs:2045-2152`) — index into the CIP reply envelope, not the payload; the write path round-trips envelope bytes to the PLC as data.
8. Internal dead code, delete outright: `_build_ab_string_write_request`, `_get_connected_session` (`src/client.rs:3007-3084, 4084-4104`), the two `unreachable!` in `batch_exec.rs:186/263` (restructure to make the states unrepresentable or return a typed internal error), and whatever items 1–7 orphan.

Also correct `parse_extended_error` (`src/client.rs:3180-3319`) while in the area: extended status is signaled by the additional-status size field, not by general status `0xFF`; collapse the duplicated both-endianness match.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §1; `docs/agents/notes/ab-firmware-quirks.md` (which workarounds are load-bearing — the brief retires *strategies*, never the documented quirk handling); `docs/agents/notes/unconnected-send.md`; the FFI exports in `src/ffi.rs` that front any retired method (grep each name) — their behavior must change to the honest-error contract too, following the `eip_configure_batch_operations` "explicitly unsupported" pattern and its `ffi_batch_config_apis_are_explicitly_unsupported` test.

### Files to create or modify

`src/client/string.rs` (bulk), `src/client.rs`, `src/client/batch_exec.rs`, `src/ffi.rs` (matching honest-error stubs + capability note if applicable), `src/error.rs` (an `Unsupported { api, reason }`-style variant if none fits), C#/Python wrapper doc comments for any wrapped retired API (verify with grep; wrappers should surface the error, not pre-empt it), `docs/ROADMAP.md` (2.0 removal list), `CHANGELOG.md` (`### Deprecated` section), `tests/plc_sim_tests.rs` / `tests/ffi_tests.rs`.

### Behavior

- Every retired public API: `#[deprecated(note = "...never functioned; use X; removal in 2.0")]` + returns the typed unsupported error immediately (no network I/O). No silent false success remains anywhere in the string/UDT surface.
- Line count of `src/client/string.rs` drops substantially; every remaining line traces to a documented quirk or the working `write_tag` path.

### Test requirements

- Per retired API: a test asserting the immediate typed error (and, for the FFI stubs, the rc/last-error contract).
- Existing suites prove the *working* paths still work: STRING round-trip via `write_tag(PlcValue::String)` sim tests, UDT RMW via service layer, batch tests.
- `cargo doc` builds without broken intra-doc links to removed items; deprecation warnings do not trip `-D warnings` in-crate (use `#[expect(deprecated)]` at the few internal call sites that must remain until 2.0, with reasons).
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, C# + Python suites (FFI-touching).

### Acceptance criteria

- Items 1–8 each either retired-with-honest-error, deleted (internal), or kept-with-evidence (item 5's decision point, strategy A tracing) — an explicit disposition per item in the Codex log.
- No `msg.contains("Partial transfer")`-style stringly dispatch remains.
- `cargo semver-checks` (baseline 1.1.0, post-CODEX-AK) passes — deprecation + behavior change of never-working paths is not a signature break; if semver-checks disagrees on any item, stop and report.
- ROADMAP 2.0 section lists every deferred deletion. CHANGELOG updated.

### Out of scope

- Fixing any of these paths to actually work — that's what the retirement replaces; capture-gated UDT work is [[codex-ao-udt-wire-format-investigation]], attributes/array-range is [[codex-an-array-range-and-attributes]].
- The transport hardening in `send_rr_data_item` — [[codex-al-transport-session-hardening]] (note: retiring connected messaging removes `send_connected_cip_request`, which is why AL scoped it out).
- `TagManager`/monitoring dead strata — [[codex-aq-dead-stratum-deprecation]].

### Risks and gotchas

- **Verify "never worked" independently per item before retiring** — the analysis was verified by reading, not by hardware, and this repo has one precedent (CODEX-O / analysis conflict) of a read-based conclusion colliding with hardware evidence. For each item: reproduce the defect with a focused sim test or byte-level assertion first, record it, then retire. If any item works against the sim, stop and re-scope.
- The full-coverage manifest (2299 tags) may exercise some of these names — check `examples/full_coverage_tags.json` runners for calls into retired APIs; the exerciser must keep passing (it should — it uses the working paths — but verify).
- `read_udt_chunked` strategy A tracing: `read_tag` on large UDTs may genuinely depend on the fragmented-read fallback. If real fragmentation support is needed, that's the documented CIP service `0x52` Read Tag Fragmented — a *new* correctly-built path belongs to CODEX-AO's phase 2 or a follow-up, not a rescue of strategy B.
- Keep the diff reviewable: mechanical retirement first, `parse_extended_error` cleanup as a separate commit within the task.

## Codex log

2026-07-07 — Submitted by Codex.

Disposition by brief item:

1. `write_string`: deprecated and converted to an immediate `EtherNetIpError::Unsupported` compatibility stub. It no longer validates, builds, sends, or misparses the malformed request; replacement is `write_tag(..., PlcValue::String(...))` or `write_string_tag`.
2. `write_ab_string_udt`: deprecated and converted to an immediate unsupported stub because it parsed the raw CPF envelope as a CIP reply and could report false success.
3. Connected messaging / Forward Open subsystem: public `write_string_connected` is deprecated and returns unsupported. The private Forward Open builder/parser, connected send path, close path, connected-session cache, sequence counter, and connected-session wire-state structs were deleted as orphaned internal code.
4. `write_string_unconnected`: deprecated and converted to an immediate unsupported stub because the reverse-engineered payload does not match the maintained Logix STRING structure write.
5. `write_ab_string_components`: deprecated and converted to an immediate unsupported stub. CODEX-AT superseded the decision point by proving direct standalone STRING writes through the maintained structure encoding; component writes remain non-atomic experimentation, not a supported fallback.
6. `read_udt_chunked` strategies B-D: deleted. The compatibility method now delegates to the maintained `read_tag` UDT path and rejects non-UDT values; no `msg.contains("Partial transfer")` dispatch remains. Correct fragmented UDT reads are deferred to CODEX-AO.
7. `read_udt_member_by_offset` / `write_udt_member_by_offset`: deprecated and converted to immediate unsupported errors. Matching FFI exports return `-1` with last-error text; C# wrapper/interface methods are marked `[Obsolete]`.
8. Internal dead code: deleted `_build_ab_string_write_request`, `_get_connected_session`, connected-session types/fields, raw read/write helpers orphaned by offset API retirement, and replaced the two batch `unreachable!` calls with typed `BatchError::Other` results.

Also corrected `parse_extended_error`: additional status is decoded when the additional-status size field is nonzero, words are interpreted little-endian only, and `0x2107` now reports as Read/Write Tag data-type mismatch without the old BE shadow code.

Docs updated: `CHANGELOG.md`, `docs/ROADMAP.md`, `docs/programmer_manual.md`, C# XML docs/interface obsolete attributes, and `wiki/limitations/string-and-udt-write-behavior.md`.

Verification passed: `cargo fmt -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`; `cargo test --test plc_sim_tests --locked`; `cargo test --features ffi --test ffi_tests --locked`; `cargo doc --workspace --all-features --no-deps --locked`; `cargo semver-checks --baseline-version 1.1.0`; `cargo build --release --features ffi --locked`; `python -m unittest discover -s python\tests`; `dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --no-restore`; `dotnet test csharp\RustEtherNetIp.IntegrationTests\RustEtherNetIp.IntegrationTests.csproj --no-restore`; `git diff --check`.

## Claude review

### 2026-07-07 15:20  claude [Opus 4.8]

**Independent verification**
- `cargo fmt -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — ok across the workspace.
- `cargo test --test plc_sim_tests --locked` — 24/24 (two new AP tests: `retired_string_apis_return_unsupported_without_network_write`, `retired_udt_offset_apis_return_unsupported`).
- `cargo test --features ffi --test ffi_tests --locked` — 16/16.
- `cargo semver-checks check-release --baseline-version 1.1.0` — 221 checks pass, "no semver update required".
- `cargo doc --workspace --all-features --no-deps --locked` — clean, no broken intra-doc links to removed items.
- `cargo build --release --features ffi --locked` — ok.
- C# unit `RustEtherNetIp.Tests` — 86/86; C# native integration `RustEtherNetIp.IntegrationTests` — 7/7 (against the freshly built release cdylib); Python `python/tests` — 34 passed, 8 skipped, 0 failed.

**What's being fixed**
- A cluster of publicly exported STRING/UDT paths that were provably non-functional or hazardous (silent false success, fabricated empty payloads, malformed requests) are retired behind explicit `Unsupported` errors and `#[deprecated]`, with actual deletion queued for 2.0; orphaned internal code is deleted outright; `parse_extended_error` is corrected while in the area.

**Root cause confirmation**
- Confirmed per item. `src/client/string.rs` drops 1041→89 lines; the five retired writers (`write_string`, `write_ab_string_components`, `write_ab_string_udt`, `write_string_connected`, `write_string_unconnected`) return `EtherNetIpError::Unsupported` before any network I/O. The connected-messaging subsystem (`establish_connected_session`, `parse_forward_open_response`, `send_connected_cip_request`, session cache, sequence counter) and its wire-state structs `ConnectedSession`/`ConnectionParameters` (`src/types.rs`, `pub(crate)`) are deleted — grep confirms no residual references outside the retired `write_string_connected` stub itself.
- `read_udt_chunked` (`src/client.rs:1756`) now `validate_session` + delegate to `read_tag`, returning `PlcValue::Udt` or `DataTypeMismatch`; the `msg.contains("Partial transfer")` stringly dispatch is gone (grep clean).
- `parse_extended_error` (`src/client.rs:2900`) rekeyed on the additional-status size word (`cip_data[3]`), little-endian only, BE shadow removed, `0x2107` mapped to a data-type-mismatch message; `check_cip_error` gate corrected to trigger on `cip_data[3] > 0` regardless of general status.
- `batch_exec.rs` two `unreachable!` replaced with typed `BatchError::Other` preserving the tag name.

**Fix appropriateness**
- Lands at the right layer. Retirement follows the established `eip_get_tag_metadata` / `eip_configure_batch_operations` precedent (deprecate + honest error on 1.x, delete at 2.0). The `Unsupported { api, reason }` variant is added to the `#[non_exhaustive]` `EtherNetIpError` — additive, which is why semver-checks stays clean.
- Correct disposition on the FFI export boundary: `eip_write_string` is **repointed to the working `write_tag(PlcValue::String)` path**, not retired — the C ABI symbol is the stable contract the C#/Python `WriteString` wrappers call, so it must keep working; only the internal Rust `EipClient::write_string` method is retired. No consumer regression.
- C# offset-member wrappers carry `[Obsolete(..., false)]` (warn, not error) so downstream code still compiles while surfacing the migration path; the wrappers surface the native `-1`/last-error rather than pre-empting it.

**Test proof**
- New sim tests assert the immediate typed error for all five retired string APIs **and** that the maintained `write_tag(PlcValue::String)` path still round-trips; offset APIs assert `Unsupported { api, reason }` with api/reason content. `error.rs` gains a unit test for the new variant's Display. `ffi_tests` extended for the stub rc/last-error contract. Deprecated call sites use `#[expect(deprecated, reason = …)]`, so `-D warnings` holds.
- Working paths remain proven green: STRING round-trip, UDT RMW via the service layer, batch suites, C#/Python suites.

**Residual risk**
- `read_udt_chunked` on a genuinely oversized UDT (one that previously tripped "Partial transfer") now returns the propagated `read_tag` error instead of the old `Ok(Udt { data: vec![] })`. This is strictly an improvement (honest error vs fabricated empty payload) but is an observable behavior change; correct fragmented-read support is deferred to CODEX-AO phase 2 (documented).
- No hardware run in this review — all paths are sim/unit level. The retired paths never worked on hardware, and the maintained paths carry their own prior hardware evidence (CODEX-AT for STRING, CODEX-AV for UDT members), so no new hardware gate is introduced by this task.

**Strong points (✅)**
- FFI `eip_write_string` repoint preserves the C#/Python STRING contract while retiring the broken Rust method — the one place a naive retirement would have broken consumers.
- Every retired path returns before network I/O; no silent false success remains anywhere in the string/UDT surface (`write_ab_string_udt`'s unconditional `Ok(())` was the dangerous one — now gone).
- `parse_extended_error` correction is spec-accurate and collapses the duplicated both-endianness match the brief called out.
- Deprecation reasons name the concrete defect and the replacement API, and are consistent across Rust doc, FFI last-error text, C# `[Obsolete]`, CHANGELOG, and ROADMAP.

**Findings**
- 🟢 FFI `eip_write_string` correctly repointed to `write_tag`, not retired — no consumer regression (verified `src/ffi.rs:1037`).
- 🟡 `read_udt_chunked` oversized-UDT behavior change (fabricated empty payload → honest propagated error); improvement, deferred real fix owned by CODEX-AO. Non-blocking.
- 🟡 `check_cip_error` now routes to extended parsing whenever the additional-status size is non-zero, for any general status (previously effectively `0xFF`-gated). Spec-correct; affects only the human-readable error string, not error variant or control flow. Non-blocking.
- 🟢 `retired_string_apis_..._without_network_write` connects to the sim but asserts the typed error, not the absence of bytes on the wire; the stubs demonstrably return before I/O, so the name states intent rather than a wire assertion. Cosmetic.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Items 1–8 each retired-with-honest-error / deleted / delegated, with an explicit per-item disposition in the Codex log; item 5 decision recorded as superseded by CODEX-AT.
- ✅ No `msg.contains("Partial transfer")`-style stringly dispatch remains (grep clean).
- ✅ `cargo semver-checks` (baseline 1.1.0) passes — deprecation + behavior change of never-working paths is not a signature break.
- ✅ ROADMAP 2.0 section lists every deferred deletion (Rust + FFI + C# wrappers); CHANGELOG `### Deprecated` present and accurate.

## Verdict

### 2026-07-07 15:20  claude [Opus 4.8]

**Merged.** Full independent matrix green (fmt, clippy `--all-targets --all-features -D warnings`, workspace `--locked`, plc_sim 24/24, ffi_tests 16/16, semver-checks 221-pass/no-update, `cargo doc` clean, release ffi build, C# 86/86 + integration 7/7, Python 34-pass/8-skip). All eight brief items dispositioned correctly; the string/UDT surface no longer contains any silent-false-success or fabricated-payload path. `parse_extended_error` correction and the `unreachable!` removals are sound. The one judgment call — repointing the FFI `eip_write_string` export to the working `write_tag` path rather than retiring it — is the correct read of the C ABI contract boundary and prevents a consumer regression. Zero defects, zero Claude-applied fixes. Two 🟡 items (oversized-UDT honest-error behavior change → CODEX-AO; extended-status message gating) are non-blocking and documented. Merged at `1821e59`.
