---
id: CODEX-AO
title: UDT wire-format investigation — capture-gated audit of struct read/write encoding and udt-crate strictness
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
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

## Claude review

## Verdict
