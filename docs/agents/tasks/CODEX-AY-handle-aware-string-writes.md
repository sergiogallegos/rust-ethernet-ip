---
id: CODEX-AY
title: STRING writes assume the built-in handle 0x0FCE — custom string types (Str82, etc.) reject with 0x2107
owner: codex
status: open
created: 2026-07-08
last-update: 2026-07-08 claude [Opus 4.8]
---

## Brief

### Goal

Make STRING writes work against **custom** Logix string types (user-defined name/length), not
only the built-in `STRING`. Studio 5000 lets a user define their own string type — same layout
as `STRING` (DINT `LEN` + SINT `DATA[n]`) but a different type name and its own **structure
handle**. This is common in real projects.

`crates/protocol/src/values.rs::write_data_type_bytes` emits `A0 02 CE 0F` (marker `0x02A0` +
`STANDARD_STRING_HANDLE 0x0FCE`) for **every** `PlcValue::String`, unconditionally. The
controller compares that handle against the target tag's real handle and returns `0xFF/0x2107`
("tag type used in request does not match the target's data type") on mismatch. So writes to a
built-in `STRING` succeed, and writes to any custom string type fail — which is exactly why the
`gTestUDT.Member5_String` (type `Str82`) writes were mislabelled "encoding-blocked".

Hardware proof, 2026-07-08, 5069-L330ERM fw38 (values restored):

- `gTestUDT.Member5_String` is type `Str82`, real structure handle **`0x9621`** (read reply
  prefix `21 96`); built-in `STRING` is `0x0FCE`.
- (A) `write_tag(String)` → `0x2107`, value unchanged. (B) raw write with `0x0FCE` → unchanged.
  (C) raw write with the correct handle `0x9621` → value changes, read-back confirms.

The member is fully writeable in a single direct request once the request carries the target's
real handle. See
[`docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`](../../validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md)
and [`docs/agents/notes/ab-firmware-quirks.md`](../notes/ab-firmware-quirks.md) (STRING Members).

### Goal of the fix

Discover the target's real structure handle and use it in the write instead of assuming
`0x0FCE`. This supersedes the "STRING member is blocked" understanding and the RMW-only member
path for the common cases.

### Context to read first

- `crates/protocol/src/values.rs` — `write_data_type_bytes` (`:36`), `STANDARD_STRING_HANDLE`
  (`:22`), `encode_standard_string_payload`; the read side that already recognizes `0x0FCE` and
  decodes to `PlcValue::String` while other handles fall through to `PlcValue::Udt` (that
  fall-through is where the real handle is currently visible — `read_tag` on a `Str82` member
  returns `Udt` whose `data[0..2]` is the handle).
- `src/client.rs` — `build_write_request` (`:3090`), the read path that surfaces the structure
  handle, and `get_tag_attributes`/template read (`:2210`, `read_udt_template`) which can also
  yield the template handle. Note `get_tag_attributes` currently fails with "Path segment error"
  on member paths (`gTestUDT.Member5_String`) — the read-reply handle is the reliable source.
- [`docs/agents/notes/ab-firmware-quirks.md`](../notes/ab-firmware-quirks.md) — STRING Members
  section (the mechanism + fix direction) and the `symbol_id` read-before-write precedent.
- CODEX-AO (UDT wire-format) — this is the STRING-member slice of that investigation; the other
  AO Phase-2 questions (whole-UDT write type, read handle-vs-offset) stay capture-gated.

### Files to create or modify

`crates/protocol/src/values.rs` (thread a handle into the STRING write encoding),
`src/client.rs` (capture/lookup the handle on the write path), and the STRING service-layer path
in `src/client/service_layer.rs` / `src/client/string.rs`. Tests in the protocol crate + a
simulator STRING-handle test. Manifest relabel of the 17 `encoding_blocked_udt_string_member`
entries once the write works (coordinate with CODEX-AX).

### Behavior

- A STRING write to a custom string type uses the target's real structure handle. Suggested
  mechanism: a read-before-write that captures the handle from the read reply (mirrors the
  `symbol_id` pattern), or a cached handle from a prior read of that tag; decide and document.
  A built-in `STRING` (`0x0FCE`) must keep working with no extra round-trip regression where the
  handle is already known.
- Works for standalone custom-string tags **and** STRING members inside UDTs / UDT array
  elements. `gTest_STRING` (built-in) unaffected.
- Payload length follows the target type's `DATA[n]` size (an `Str82` is 82 like STRING, but do
  not hardcode 82 for arbitrary custom lengths — derive from the type where a non-82 custom
  string is in play, or document the 82-only scope explicitly if that is all that is validated).

### Test requirements

- Protocol-crate unit test: STRING write with a supplied non-`0x0FCE` handle emits
  `A0 02 <handle LE>` + the correct payload.
- Simulator test proving a custom-handle STRING round-trips (extend the sim to model one custom
  string type/handle, mirroring how it already models `0x0FCE`).
- Full matrix: `cargo fmt -- --check`, `cargo clippy -- -D warnings`,
  `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`.
- Hardware re-validation (maintainer): `gTestUDT.Member5_String` (`Str82`, handle `0x9621`) and
  `gTestUDT_Array[i].Member5_String` write+read-back succeed via the public `write_tag`; record
  it. If the maintainer adds a built-in-`STRING` `Member6` to the UDT, confirm it writes through
  both the old and new paths (isolates handle vs member-path).

### Acceptance criteria

- `write_tag(PlcValue::String)` succeeds against a custom string type (real handle used), proven
  on the simulator and on hardware for `Str82` handle `0x9621`.
- Built-in STRING writes unchanged; no clippy/fmt/test regressions.
- The 17 `encoding_blocked_udt_string_member` manifest entries can be relabelled `writeable`
  (do it here or hand to CODEX-AX; note which).

### Out of scope

- Whole-UDT structure writes and the read handle-vs-offset question (CODEX-AO Phase 2, still
  capture-gated).
- Arbitrary custom-length strings beyond what the sim + hardware validate — scope explicitly if
  only 82-char custom types are proven.

### Risks / gotchas

- Do not drop the built-in-STRING fast path into a mandatory extra round-trip for every write if
  it can be avoided; measure the added latency.
- The read reply's structure prefix is `A0 02 <handle>` (4 bytes) for structures but a 2-byte
  atomic type for atomics — parse defensively.
- Keep the RMW service-layer fallback until the handle-aware path is hardware-proven.

## Codex log

## Claude review

## Verdict
