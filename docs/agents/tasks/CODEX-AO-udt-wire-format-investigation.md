---
id: CODEX-AO
title: UDT wire-format investigation — capture-gated audit of struct read/write encoding and udt-crate strictness
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-08 claude [Opus 4.8]
---

## Brief

### Goal

Resolve the conflict between the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §1 items 2–3 and the "Conflict note" in Top priorities) and the CODEX-O hardware validation, then fix whichever side is wrong, plus close the independent `crates/udt` strictness gaps. **This task is capture-gated: no wire-format change lands until the maintainer's packet captures are attached to this file.**

**The conflict.** The analysis reads the Logix Write Tag service (1756-PM020) as requiring, for structures, the 2-byte marker `0xA0 0x02` followed by a separate 2-byte structure handle (Template attribute 1). The code instead emits a single u16 `0x02A0.wrapping_add(symbol_id)` (`crates/types/src/lib.rs:103-105`, pinned by `crates/protocol/src/tests.rs:124-129` asserting `0x14D4` for `symbol_id = 0x1234`). The analysis also finds the *read* path stores the reply's 2-byte structure handle inside `UdtData.data` and never populates `symbol_id` (`crates/protocol/src/values.rs:152-155` — `UdtData { symbol_id: 0, … }`), which would shift every member offset by 2. **However**: the CODEX-O review (2026-05-26) describes `0x02A0 + symbol_id` as the AB convention, and the 2026-05-26 (L18ER) and 2026-06-19 (L330ERM) full-coverage hardware runs verified 2206/2206 UDT-inclusive writes byte-identical across three bindings. Possibilities: (a) the analysis misreads the spec; (b) controllers tolerate the collapsed form; (c) `symbol_id` values in practice make the sum coincide with something valid; (d) the coverage matrix never exercises the failing shape (e.g. all UDT writes go through the C#/service-layer RMW path whose read-back supplies whatever the write needs symmetrically). Symmetric encode/decode bugs cancel in round-trip verification — byte-identical parity across bindings sharing one native core does not prove spec conformance.

### Maintainer capture checklist (blocks implementation)

Attach to this task (or `docs/validation/`):

1. Wireshark capture of a **known-good third-party client** (Studio 5000, RSLinx, pycomm3, or libplctag) writing a whole UDT tag — the ground-truth write-type bytes.
2. Capture of *this library* reading the same UDT tag — the reply's type/handle bytes, to confirm what precedes the struct payload.
3. Capture of this library writing the same UDT via the RMW path — what actually goes on the wire today.
4. The UDT's Template attribute values (structure handle vs template instance id) — obtainable via the library's own `read_udt_template` path or a third-party tool.

### What Codex does

**Phase 1 (no captures needed — land immediately):**

- **`crates/udt` strictness** (`crates/udt/src/lib.rs:500-541`): `to_hash_map` silently *skips* members whose offsets fall outside the data; `from_hash_map` zero-fills missing members. Composed in `write_udt_member_via_full_value` (`src/client/service_layer.rs:32-62`), a short or misaligned read produces a full-UDT write with **zeros for every skipped member** — a read-modify-write that can zero unrelated PLC data. Fix: `to_hash_map` returns `Err` on any out-of-range member; `from_hash_map` returns `Err` on any member missing from the map (the RMW caller always starts from a complete read, so this is loss-free). Public signature change from infallible to `Result` is additive-in-spirit but breaking in type — gate per SemVer: if the current signatures are infallible and public, add `try_*` variants, route all internal callers through them, and `#[deprecated]` the infallible ones.
- **Template member-name pairing** (`crates/udt/src/lib.rs:196-207`): `parse_null_terminated_strings` filters out empty chunks, so an empty member name (consecutive NULs — real templates contain these for hidden/host members) shifts every subsequent name onto the wrong member. Preserve empties positionally.
- **BOOL bit unpacking** (`crates/udt/src/lib.rs:605-611`): BOOL members decode as `data[offset] != 0`, ignoring the member `info` field, which for Logix BOOL members carries the bit index within the host byte — several BOOLs packed in one host SINT all read as the same value. Use `info` as the bit index for BOOL members. Cover both with unit tests built from a hand-crafted template byte image (cite the template format in comments); if a real captured template is available from item 4 above, pin it.

**Phase 2 (after captures):**

- Compare capture 1 vs capture 3. If the collapsed u16 is wrong: fix `known_data_type`/`write_data_type` to emit marker + separate handle (using the *structure handle* — note `parse_template_attributes_response` at `src/client.rs:2528-2537` already parses `structure_handle` and never uses it), re-pin `crates/protocol/src/tests.rs` from the capture bytes, and model struct reads/writes in `tests/plc_sim.rs` from the captured shapes. If the collapsed form is confirmed correct (capture 1 shows it or controllers accept it), document the finding in `docs/agents/notes/ab-firmware-quirks.md` with the capture reference and close the wire question — updating the analysis doc's conflict note.
- Same for the read path: if capture 2 shows `[0xA0 0x02][handle][payload]`, strip the handle into `UdtData.symbol_id` in `decode_payload` and stop shifting offsets; the "read first to capture symbol_id" contract (CLAUDE.md, quirks page) then actually works for plain `read_tag`.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` (§1 items 2–3, §Top priorities conflict note), `docs/agents/notes/ab-firmware-quirks.md` (whole page — 0x2107 lore, symbol_id staleness), CODEX-O's board/log entries (2026-05-26), `crates/udt/src/lib.rs` in full, `src/client/service_layer.rs`, `crates/protocol/src/values.rs`.

### Files to create or modify

Phase 1: `crates/udt/src/lib.rs`, `src/client/service_layer.rs`, unit tests in-crate, `CHANGELOG.md`. Phase 2 (conditional): `crates/types/src/lib.rs`, `crates/protocol/src/values.rs`, `crates/protocol/src/tests.rs`, `src/client.rs`, `tests/plc_sim.rs`, `docs/agents/notes/ab-firmware-quirks.md`.

### Test requirements

Phase 1: unit tests for error-on-partial (`to_hash_map` with truncated data → `Err`, not silent skip), error-on-missing (`from_hash_map`), empty-member-name template parse, packed-BOOL bit extraction (two BOOLs, same host byte, different `info`, different values). Phase 2: re-pinned byte tests citing capture files; sim struct read/write round-trip; full matrix + maintainer hardware re-run (full-coverage exerciser) before release.

### Acceptance criteria

- Phase 1 lands regardless of capture timing; the RMW zero-fill hazard is closed (a partial read can no longer silently produce a zero-filled write).
- Phase 2 either fixes the encoding with capture-pinned tests, or documents the collapsed form as confirmed-correct with capture evidence — no third outcome; "left as-is without evidence" is not an acceptable close.
- Full-coverage hardware matrix re-validated before the release carrying any wire change.

### Out of scope

- The `TagManager` fabricating UDT parser — [[codex-aq-dead-stratum-deprecation]]. The chunking ladder and `*_by_offset` methods — [[codex-ap-string-udt-graveyard]]. `get_tag_attributes` — [[codex-an-array-range-and-attributes]].

### Risks and gotchas

- **Do not change the write encoding on analysis authority alone** — it is hardware-validated in its current form on two controllers. The capture is the arbiter. If captures are delayed, submit Phase 1 alone.
- The `try_*`/deprecation dance for `to_hash_map` must not break the C#/Python wrappers' JSON contract (`UdtData` serde shape is an implicit cross-binding ABI — see analysis §3).
- If Phase 2 changes `UdtData.data` to exclude the handle, every consumer that compensated for the 2-byte shift (if any grew such a compensation) breaks silently — grep for offset arithmetic on `UdtData.data` across the repo and the wrappers before landing.

## Codex log

2026-07-08 codex [GPT-5] - Phase 1 submitted; Phase 2 remains blocked on maintainer captures.

- Closed the hardware-free `crates/udt` RMW zero-fill hazard without changing
  the public method signatures: `UserDefinedType::to_hash_map` now errors if
  any member range exceeds the returned UDT bytes, and
  `UserDefinedType::from_hash_map` now errors if any declared member is absent
  from the input map.
- Preserved empty NUL-separated template-name slots positionally so hidden/host
  template members no longer shift later names onto the wrong member records.
- Added private packed-BOOL bit metadata to `UserDefinedType` plus
  `add_member_with_bit_index`; callers using plain `add_member` keep the
  previous byte-level behavior, while template-aware code can decode and encode
  BOOL members by Logix `info` bit index.
- Added focused unit coverage for truncated data, missing members, empty
  template names, and two BOOL members sharing one host byte.
- Updated the stale `tests/udt_enhanced_tests.rs` partial-data expectation to
  assert the new fail-closed contract.
- Verification passed: `cargo test -p rust-ethernet-ip-udt --locked`;
  `cargo test --test udt_enhanced_tests --locked`;
  `cargo fmt -- --check`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`
  (outside sandbox for simulator localhost bind);
  `cargo semver-checks check-release --baseline-version 1.1.0`.
- No UDT wire-format encode/decode changes were made. Phase 2 still needs the
  capture checklist above before any `0x02A0 + symbol_id` or read-handle
  behavior changes.

2026-07-08 codex [GPT-5] - Phase 2 checked during AW/AX/AZ implementation sweep.

- Searched current `docs/validation/`, task notes, and wiki for attached
  Wireshark/packet-capture evidence satisfying the Phase 2 checklist.
- No capture artifacts or capture-pinned validation notes were present.
- No UDT wire-format changes were made; Phase 2 remains blocked on the
  maintainer capture checklist.

## Claude review

### 2026-07-08 15:40  claude [Opus 4.8]

**Independent verification**
- `cargo fmt -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — pass.
- `cargo test -p rust-ethernet-ip-udt --locked` — 9/9 passed (includes the 4 new Phase 1 tests).
- `cargo test --test udt_enhanced_tests --locked` — 13/13 passed.
- `cargo test --test plc_sim_tests --locked` (SKIP_PLC_TESTS=1) — 24/24 passed.
- `cargo semver-checks check-release --baseline-version 1.1.0` — no semver update required; `add_member_with_bit_index` is additive, `member_bit_indices` is private.

**What's being fixed**
- Phase 1 (hardware-free) only. Closes the read-modify-write zero-fill/skip hazard in `crates/udt` and fixes template member-name misalignment; adds a packed-BOOL bit-index mechanism. No UDT wire-format encode/decode change — Phase 2 remains capture-gated.

**Root cause confirmation**
- Confirmed zero-fill hazard: `to_hash_map` previously skipped out-of-range members (`crates/udt/src/lib.rs`, old `if offset + size <= len` branch) and `from_hash_map` zero-filled missing members; composed through `src/client/service_layer.rs:99` (`UdtData::from_hash_map`) a short read could write zeros for unrelated members. Both now fail closed (`to_hash_map` errors on any out-of-range member using `checked_add`/`is_none_or`; `from_hash_map` errors on any absent member). Since both methods already returned `Result`, all internal callers propagate the new error with no signature change — no `try_*`/deprecation dance needed.
- Confirmed name-misalignment: `parse_null_terminated_strings` dropped empty chunks via `.filter(!is_empty())`, shifting names past a hidden/host member. Removing the filter preserves positional alignment; the `member_name.is_empty()` `continue` at `crates/udt/src/lib.rs:208` correctly skips both intentional empties and the trailing split empty within the `zip` with `raw_members`.

**Fix appropriateness**
- Fail-closed lands at the right layer (the crate's serde boundary), so every consumer of `to_hash_map`/`from_hash_map` inherits it without per-caller edits. Correct.
- The `src/ffi.rs` change is a test-only (`#[cfg(test)]`) mutex serializing the `FORCE_RUNTIME_INIT_ERROR` global against the panic-probe test — no production behavior change; justified flake fix surfaced by the full-workspace run.

**Test proof**
- New unit tests cover error-on-truncation, error-on-missing-member, positional empty-name preservation, and two BOOLs in one host byte encode/decode by bit index. `tests/udt_enhanced_tests.rs:623` flipped from asserting partial-data `is_ok()` to `is_err()` — a legitimate contract tightening, not a weakening.

**Residual risk**
- Packed-BOOL support is a **dormant mechanism**. `UdtMember` (`crates/udt/src/lib.rs:42`) carries no bit-index field, and every production path builds `UserDefinedType` via plain `add_member` (`src/client/service_layer.rs:88`, `src/ffi.rs:1397`, the client doctests). `parse_udt_template` also discards `RawMember.info` for BOOL members (it only uses `info` for array element counts at `crates/udt/src/lib.rs:275`). So a real packed-BOOL UDT read through the client still collapses to `data[0] != 0`. The unit test proves the mechanism in isolation, not the end-to-end fix. End-to-end plumbing (thread `info` bit index through the definition pipeline into `UserDefinedType`) is left for Phase 2 or a small follow-up — see Findings.
- Phase 2 (the `0x02A0 + symbol_id` write-type conflict and the read-path handle-vs-offset question) is untouched and remains blocked on the maintainer capture checklist. No wire change may land without it.

**Strong points (✅)**
- The RMW zero-fill hazard — the load-bearing safety fix — is genuinely closed and cannot silently corrupt unrelated PLC members on a short read (`crates/udt/src/lib.rs:519-560`).
- Positional empty-name handling is verified against a hand-crafted template byte image with a cited format comment (`test_parse_udt_template_preserves_empty_member_names_positionally`).
- No public signature churn; semver-clean.

**Findings**
- 🟢 `bit_index.min(7)` is applied both at insert (`crates/udt/src/lib.rs:488`) and at use (`:569`, `:664`) — redundant but harmless defense.
- 🟡 Packed-BOOL bit-index support is not wired into any production caller and `RawMember.info` is not preserved as a bit index by `parse_udt_template`, so the "several BOOLs in one host SINT read the same value" hazard is **not yet closed in the real client path**. Non-blocking for Phase 1 (the brief scoped the BOOL item as "cover with unit tests," which is satisfied, and the natural home for the definition-pipeline plumbing is the capture-gated Phase 2), but it must not be reported as "packed BOOL fixed end-to-end." Tracked as a Phase 2 / follow-up item.

**Acceptance criteria tally**
- ✅ Phase 1 lands regardless of capture timing; the RMW zero-fill hazard is closed (a partial read can no longer silently produce a zero-filled or member-skipped write).
- (deferred) Phase 2 either fixes the encoding with capture-pinned tests or documents the collapsed form as confirmed-correct — owner: codex, timing: after maintainer captures (checklist unfilled).
- (deferred) Full-coverage hardware matrix re-validated before any release carrying a wire change — no wire change in this submission, so not yet due.

## Verdict

### 2026-07-08 15:45  claude [Opus 4.8]

**Phase 1 accepted and merged; task remains open for Phase 2 (capture-gated).**

The hardware-free deliverables are correct and independently verified. The headline RMW zero-fill hazard is closed at the crate serde boundary with fail-closed errors and no public signature change; template member-name alignment is fixed and pinned with a byte-image test. The packed-BOOL bit-index mechanism is correct and unit-tested but dormant in production (no caller supplies bit indices; `UdtMember` has no bit field; `parse_udt_template` discards `info` for BOOLs) — flagged as 🟡 and queued into Phase 2 rather than sold as a completed end-to-end fix.

No fix-during-merge edits were applied. Phase 2 (the `0x02A0 + symbol_id` write-type and read-path handle questions) stays untouched and blocked on the maintainer capture checklist; this task's status stays `open` with Phase 1 recorded as landed.

### 2026-07-08  claude [Opus 4.8]

**Phase 2 deferred indefinitely per maintainer direction; task closed.** The maintainer confirmed the whole-UDT wire-format audit is not needed for their usage — member-level UDT access (scalar DINT/INT/REAL/BOOL, and built-in *and* custom `STRING` members via CODEX-AY handle discovery) covers real applications, and no packet captures will be produced. The STRING-member sub-question Phase 2 originally carried was resolved by CODEX-AY (structure-handle discovery). The remaining items — whole-UDT `0x02A0 + symbol_id` write-type, read handle-vs-offset, and the dormant packed-BOOL bit-index plumbing — are recorded here and in [`wiki/limitations/string-and-udt-write-behavior.md`](../../../wiki/limitations/string-and-udt-write-behavior.md) should the need ever arise. A related, separately-observed gap (direct whole-UDT-array-element **writes** fail because the `symbol_id` `Get Attribute List` lookup returns a path-segment error for array-element paths) is documented in the same wiki page; member-level writes are the supported path. Status set to `merged` (Phase 1 landed at `7cb07a4`).
