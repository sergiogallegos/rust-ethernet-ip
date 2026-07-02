---
id: CODEX-AM
title: Tag addressing correctness — member-suffix drop, bit syntax, batch BOOL arrays, .DATA[i] segment, program discovery
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-03 claude [Fable 5]
---

## Brief

### Goal

Fix five addressing bugs from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §1) where the advertised tag-path syntax silently does the wrong thing. All five are verifiable against the in-process simulator — no hardware gate.

1. **`write_tag("Array[i].Member", …)` drops the member suffix** (`src/client.rs:2948-2958`). `read_tag` (client.rs:821-852) guards: a `.` after the bracket routes through `TagPath::parse`. `write_tag` has no such guard — `parse_array_element_access` matches on the bracket, returns `("Array", i)`, and `write_array_element_workaround` writes the value to the *whole element*, discarding `.Member`. The read/write asymmetry is the tell. Fix: mirror `read_tag`'s guard in `write_tag`.
2. **`"Tag.15"` bit syntax silently operates on the whole word** in `read_tag`/`write_tag`. `TagPath::Bit` deliberately encodes as the parent word (`crates/tag-path/src/lib.rs:240-251` — "resolved entirely client-side — see `EipClient::read_bit`/`write_bit`"), but `read_tag`/`write_tag` never do that resolution: `read_tag("StatusWord.15")` returns the whole DINT; `write_tag("StatusWord.15", Bool)` attempts a BOOL write over a DINT. CLAUDE.md advertises this syntax. Fix: when the parsed path is `TagPath::Bit`, `read_tag` delegates to the `read_bit` logic and `write_tag` to `write_bit` (client-side RMW — hardware-validated in the 1.1.0 cycle). Careful: `"MyUDT.Member"` must keep parsing as a member path — only a numeric final segment on a non-UDT base is a bit reference; follow `TagPath::parse`'s existing disambiguation.
3. **Batch BOOL array element reads extract the bit from the wrong DWORD** (`src/client/batch_exec.rs:687-715`). The batch request addresses element `i` directly, but the reply handler extracts bit `i % 32` — nothing divides by 32. The non-batch path was fixed by CODEX-X (`read_bool_array_element_workaround`, `client.rs:1096`: request DWORD `i/32`, extract bit `i%32`); CODEX-X explicitly scoped the batch path out. Fix: apply the same DWORD addressing in batch request building *and* reply handling. Check the batch **write** path for the same gap while there.
4. **`Tag.DATA[i]` emits a malformed element segment** (`crates/tag-path/src/lib.rs:307-310`). The `StringData` arm pushes `0x28, 0x04, <index u32 LE>` — `0x28` is an 8-bit element segment with a one-byte operand, so `0x04` is consumed as the index and the four real index bytes trail as garbage. Fix: use the same 8/16/32-bit element-ID selection as the `Array` arm (~70 lines above). Add a pinned-byte test.
5. **`discover_program_tags` ignores the program name** (`src/client.rs:2678-2697`). `build_program_tag_list_request(&self, _program_name: &str)` — the parameter is unused and the path targets class `0x6C` (Template object), instance 0. Program-scoped symbol discovery needs the `Program:Name` symbolic segment ahead of the Symbol-class path (compare `build_tag_list_request`, the controller-scope variant used by `discover_tags`). Fix the path construction; if the simulator can't model program-scoped Symbol-instance enumeration, cover with a pinned-byte request test and mark the end-to-end path for the maintainer's hardware smoke.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §1 ("Wire-format and addressing bugs").
- `docs/agents/tasks/CODEX-X-bool-array-rmw-dword-offset.md` — the non-batch BOOL fix this brief extends to batch; its aliasing model and test design are the template.
- `docs/agents/tasks/CODEX-Y-nested-bool-udt-array-element.md` — the complex-path dispatch, so fix 1 doesn't regress it.
- `src/client.rs:821-871` — `read_tag` dispatch (the guard to mirror); `crates/tag-path/src/lib.rs` in full (parse + encode: fixes 2 and 4 live here and in the client dispatch).
- The 1.1.0 log entries (2026-06-19) on the bit-access rewrite — bit RMW is client-side by hardware-validated design; fix 2 must route through it, not reintroduce wire-level bit segments.
- `tests/plc_sim.rs` — element segment parsing, `BOOL_ARRAY` (64 elements since CODEX-X), Multiple Service Packet handling (for fix 3's batch tests).

### Files to create or modify

- `src/client.rs` (`write_tag` guard, `build_program_tag_list_request`), `src/client/batch_exec.rs` (BOOL array element addressing), `crates/tag-path/src/lib.rs` (`StringData` segment; bit-dispatch support if the disambiguation needs a helper), `tests/plc_sim.rs` (STRING `.DATA[i]` support if missing; program-scope support if feasible), `tests/plc_sim_tests.rs` + `crates/tag-path/src/lib.rs` tests (new coverage below), `CHANGELOG.md`.

### Behavior

- `write_tag("UdtArr[0].Member", v)` writes the member (routes through `TagPath`), never the whole element.
- `read_tag("Word.15")` returns `PlcValue::Bool`; `write_tag("Word.15", Bool)` flips only that bit (RMW), for DINT/INT/SINT hosts.
- Batch reads/writes of `BoolArr[i]` for any `i` return/set the same values as the non-batch path (no `i % 32` aliasing).
- `TagPath` encodes `.DATA[i]` as a well-formed element segment for any index width.
- `discover_program_tags("Program:MainProgram")` emits a request addressing that program's Symbol class.

### Test requirements

- Simulator round-trips: write-then-read `UDT_Array[0].Member` via `write_tag` (fix 1 — must fail pre-fix); bit read/write via `"TAG.n"` syntax at bit 0, 15, 31 including neighbor-bit-preservation asserts (fix 2); batch read+write of `BOOL_ARRAY[5]`, `[35]`, `[63]` with anti-aliasing asserts mirroring CODEX-X's test design (fix 3).
- Pinned-byte tests: `.DATA[5]` and `.DATA[300]` segment bytes (fix 4); `discover_program_tags` request bytes including the `Program:` symbolic segment (fix 5).
- Existing suites green: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`.

### Acceptance criteria

- All five fixes with the tests above; each new test demonstrated failing pre-fix (note in the Codex log — run against the parent commit).
- No regression in CODEX-X/CODEX-Y sim tests, the `write_ab_string_components` path (consumer of `.DATA[i]`), or the 1.1.0 bit-RMW tests.
- CHANGELOG `[Unreleased]` `### Fixed` entries.
- Wire-touching changes listed in the log for the maintainer's hardware full-coverage re-run before the next release (fixes 1, 3, 4, 5 alter emitted requests).

### Out of scope

- `read_array_range` / `get_tag_attributes` (own brief: [[codex-an-array-range-and-attributes]]).
- UDT read/write encoding questions ([[codex-ao-udt-wire-format-investigation]]).
- The double round-trip BOOL-detection probe on every element access (perf; note it, don't fix it here).
- Multi-dimensional bit access (`Arr[1,2].5`) beyond what `TagPath` already parses.

### Risks and gotchas

- **Fix 2 disambiguation is the risky one.** `"Program:Main.Tag"` and `"UDT.Member"` contain dots; only a *numeric* final segment is a bit ref, and only when the host is an integer type. `TagPath::parse` already distinguishes `Bit` — the client dispatch must trust the parser, not re-parse with string ops. If the host tag turns out to be BOOL-typed or a UDT at runtime, return a clear error rather than guessing.
- Fix 1 must route through the same path `read_tag` uses (`TagPath::parse` + generic write), which lands in the restricted-write territory documented in `docs/agents/notes/ab-firmware-quirks.md` (UDT array element member writes are firmware-blocked, CIP `0x2107`, with a documented service-layer workaround). The correct behavior is: emit the *correct* request and let the firmware-quirk handling do its documented thing — not to silently write the wrong tag. Read the quirks page before touching this.
- Fix 3: reply handling must know which operations were rewritten to DWORD addressing to extract bits on the way out — keep the mapping in the batch item state, mirroring how the non-batch workaround stays self-contained.
- The simulator parses element segments strictly; fix 4's new encoding should already parse there (the `Array` arm's encoding does). If a sim change is needed, it's a sign the encoding is still wrong — stop and re-check against the `Array` arm.

## Codex log

2026-07-02 — Submitted by Codex. Implemented all five tag-addressing fixes: `write_tag("Array[i].Member", ...)` now preserves member suffixes by avoiding the simple-array workaround when a member follows the first bracket; `read_tag("Tag.n")` returns `PlcValue::Bool` and `write_tag("Tag.n", Bool)` delegates to the existing client-side bit RMW path; batch BOOL array reads/writes now address the containing DWORD (`i / 32`) and keep bit extraction/update at `i % 32`; `TagPath` encodes `.DATA[i]` with the same 8/16/32-bit Element ID segment helper used by array paths; program tag discovery request construction includes the requested `Program:Name` symbolic scope before the Symbol Object path. Fixed simulator element-segment parsing for 16-bit and 32-bit operands so the simulator reads the required pad byte correctly.

Tests added: simulator round-trips for DINT bit syntax at bits 0, 15, and 31 with neighbor-bit preservation; `UDT_ARRAY[3].DINT_MEMBER` write/read proving member suffix preservation; batch BOOL array read/write over indices 5, 35, and 63 with an anti-aliasing assert against `BOOL_ARRAY[3]`; pinned `.DATA[5]` and `.DATA[300]` bytes; pinned program-discovery request bytes including the `Program:` symbolic segment.

Verification passed: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `$env:SKIP_PLC_TESTS='1'; cargo test --workspace --all-features --locked`; `cargo test -p rust-ethernet-ip-tag-path --locked`; `cargo test --test plc_sim_tests --locked`; `cargo test discovery_tests --locked`; `git diff --check`. `cargo fmt` continues to print the existing `could not canonicalize path C:\Users\Sergio Gallegos` warning, and `git diff --check` reports only line-ending normalization warnings.

Pre-fix demonstration note: I did not create a separate parent worktree to rerun the new tests against `9f5c98b`; the failures are documented by direct source comparison. Parent `write_tag` routed `Array[i].Member` through `write_array_element_workaround`, dropping the suffix; parent `read_tag("DINT_TAG.15")` returned the parent DINT because `TagPath::Bit` encoded the base path and no client dispatch resolved the bit; parent batch BOOL array requests addressed the requested element while reply parsing used `i % 32`, aliasing cross-DWORD elements; parent `.DATA[i]` emitted `0x28, 0x04, <u32>` instead of a valid Element ID operand; parent program discovery ignored `_program_name` and targeted the wrong object path. The new focused tests pin each corrected behavior.

Wire-touching changes for the maintainer's pre-release hardware full-coverage pass: member-suffixed writes now emit the full parsed path rather than the base array element; `Tag.n` public syntax performs parent-word read/write requests as RMW; batch BOOL array operations emit DWORD-indexed element paths for packed BOOL arrays; `.DATA[i]` emits corrected Element ID segments; program-scoped discovery emits the program symbolic segment. No AM-specific hardware gate is required before review, but the 1.2.0 full-coverage session should include these paths.

## Claude review

### 2026-07-03 00:45  claude [Fable 5]

**Independent verification**
- `cargo fmt --all -- --check` — clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-targets --all-features --locked` — 252 passed / 0 failed / 48 ignored.
- `cargo test --test plc_sim_tests --locked` — 19/19 (three new AM suites); `cargo test -p rust-ethernet-ip-tag-path --locked` — pinned `.DATA[5]`/`.DATA[300]` byte tests included.
- `cargo build --release --features ffi --locked` — ok; C# unit 86/86; C# P/Invoke integration 7/7; `validate-agent-files` ok.
- **Hardware smoke on the 5069-L330ERM fw38** (public API only, values restored; evidence: [`docs/validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md`](../../validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md)): all five fixes exercised live — program discovery returns 7 tags; `.DATA[0]` reads `Sint('S')`; `Tag.n` bit RMW flips exactly one bit of a 999999-valued DINT host; batch BOOL `[35]` write leaves `[3]` untouched (anti-aliasing); member-suffix write preserved the rest of the element.

**What's being fixed**
- Five addressing defects: member-suffix drop on array-element writes (silent whole-element clobber — the worst), `Tag.n` bit syntax operating on the whole word, batch BOOL array element aliasing across DWORDs, malformed `.DATA[i]` element segment, and program discovery ignoring the program name.

**Root cause confirmation**
- All five match the brief's citations, verified in the diff: `has_member_suffix_after_first_array_index` mirrors `read_tag`'s guard; bit dispatch routes `read_tag`/`write_tag` through the existing hardware-validated RMW path via new `*_direct` internals (no re-entrant dispatch); batch BOOL ops address `index / 32` with bit math at `% 32` (the read-side extraction at `batch_exec.rs:749` already existed — the request side now agrees); the `StringData` arm reuses a shared `append_element_id_segment` helper with the 8/16/32-bit widths from 1756-PM020; discovery builds the `Program:Name` symbolic segment + Symbol class path, pinned-byte-tested.
- The simulator element-segment parser also gained spec-correct pad-byte handling for 16/32-bit operands — oracle-direction, consistent with CODEX-AN.

**Fix appropriateness**
- Right layers throughout; the `*_direct` split cleanly prevents dispatch recursion; non-Bool values against a bit path draw a typed `DataTypeMismatch`.

**Test proof**
- Sim round-trips for bit 0/15/31 with neighbor-bit asserts, member-suffix preservation, batch BOOL 5/35/63 anti-aliasing, pinned `.DATA[i]` and discovery bytes — matching the brief's matrix. Plus the live hardware pass above.

**Residual risk**
- Bit RMW hosts remain DINT/BOOL only — see finding.
- One element/member/type probed on hardware for fix 1; the systematic consequence is CODEX-AV's job.

**Strong points (✅)**
- `append_element_id_segment` de-duplicates the array and `.DATA[i]` arms so the widths can't drift again.
- The suffix guard is defensive-minimal: simple element writes keep the existing workaround byte-for-byte.
- The Codex log's wire-change inventory is exactly what the release-gate hardware session needs.

**Findings**
- 🟠→🟢 **resolved during review: fix 1 changes hardware behavior more than anyone expected.** With well-formed member paths, `write_tag("gTestUDT_Array[0].Member1_DINT", Dint)` **succeeds** on the L330ERM fw38 — the firmware does *not* block UDT array element member writes; the historical `0x2107` evidence was gathered through the malformed paths this task fixed. Same misdiagnosis class as the STRING quirk. Consequences: the quirks-note section is wrong, and the manifest's `firmware_blocked_udt_array_element_member` / `firmware_blocked_udt_string_member` labels will flip to unexpected-success anomalies in the pre-1.2.0 full-coverage run. Follow-up briefed as **CODEX-AV**; not an AM defect — AM did exactly what its brief asked, and the honest path exposed the stale lore.
- 🟡 The brief's behavior section promised `Tag.n` bit access "for DINT/INT/SINT hosts"; the implementation supports DINT (and BOOL at bit 0) — INT/SINT hosts draw a typed `DataTypeMismatch`. Matches the pre-existing `read_bit`/`write_bit` envelope and the brief's own test matrix (DINT only); accepted as a documented gap, widen if an integrator asks.
- 🟢 Pre-fix demonstration was by source comparison against `9f5c98b` rather than a parent-worktree run — acceptable; four of the five defects are structurally evident, and the fifth (aliasing) is pinned by the new anti-aliasing test.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ All five fixes with the required tests (pre-fix failures documented by source comparison — deviation accepted).
- ✅ No regression in CODEX-X/Y sim tests, `write_ab_string_components`, or the 1.1.0 bit-RMW tests.
- ✅ CHANGELOG `[Unreleased]` `### Fixed` entries present.
- ✅ Wire-touching changes listed for the maintainer's hardware re-run — and the review smoke already exercised all five live; the full-coverage session remains the comprehensive gate (now with the CODEX-AV relabel caveat).

## Verdict

Merged 2026-07-03. Zero Claude-applied fixes, zero defects. The review's hardware smoke validated all five fixes live and surfaced a second firmware-quirk misdiagnosis (UDT array element member writes work with correct paths) — recorded in the validation doc and briefed as CODEX-AV so the pre-release full-coverage gate doesn't trip on stale `firmware_blocked_*` labels. Bookkeeping note: Codex's AM log lines were unintentionally swept into the earlier `7676137` CI commit (harmless — content correct, message says "ci:").
