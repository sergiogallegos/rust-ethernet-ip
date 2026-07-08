---
id: CODEX-AZ
title: CIP fragmented read/write for structures larger than one packet (Str500+, large UDTs)
owner: codex
status: merged
created: 2026-07-08
last-update: 2026-07-08 claude [Opus 4.8]
---

## Brief

### Goal

Support Logix strings/structures that don't fit in a single CIP packet. The handle-aware STRING
write (CODEX-AY) makes custom string types work, but only up to ~one packet: on 5069-L330ERM
fw38 the write request ceiling is ~494 bytes and the read reply ceiling ~500 bytes of value.
Above that:

- Reading a large structure (e.g. a `Str500`, 506-byte value) fails with CIP **`0x06` Partial
  Transfer** — the library's `read_tag` does not reassemble fragments.
- Writing a large structure fails with encapsulation **`0x03`** (bad length) — the request
  exceeds one packet. CODEX-AY returns a clear "over the single-packet limit" error here.

Implement the CIP **Read Tag Fragmented (`0x52`)** and **Write Tag Fragmented (`0x53`)** services
so reads reassemble multi-fragment replies and writes split into multiple requests, removing the
size ceiling for strings and structures.

### Context to read first

- `src/client.rs` — `read_tag_direct`/`parse_cip_response` (read reply handling; where `0x06`
  surfaces), `write_string_handle_aware` and the `SINGLE_PACKET_WRITE_LIMIT` guard (CODEX-AY),
  `build_read_request_with_count`, `build_tag_path`.
- [`docs/STRING_HANDLING.md`](../../STRING_HANDLING.md) — the measured ceilings and per-scope max
  DATA table; the size limits this task removes.
- [`docs/agents/notes/cip-framing.md`](../notes/cip-framing.md) and `unconnected-send.md` — MSP /
  Unconnected Send framing.
- Hardware evidence: the 2026-07-08 validation record — a raw fragmented read (`0x52`) at offset
  0 returned the full ~500-byte fragment with status `0x06` (more), confirming the services work
  on this controller; only the library support is missing.

### Behavior

- **Fragmented read:** when a Read Tag reply returns `0x06` Partial Transfer (or proactively for
  structures known to exceed the reply size), issue Read Tag Fragmented (`0x52`, request data =
  `count(2) + byte_offset(4)`) in a loop, advancing the offset by the received fragment size,
  until status is `0x00`. Reassemble the full structure and decode as today (`read_tag` /
  `read_string_tag`). The first fragment carries the type/handle prefix; subsequent fragments are
  raw data.
- **Fragmented write:** when a structure write would exceed the single-packet limit, split into
  Write Tag Fragmented (`0x53`, request data = `type(4) + count(2) + offset(4) + data_fragment`)
  requests of a safe size, advancing the offset. Replace the CODEX-AY hard error with the
  fragmented path.
- Small reads/writes keep the single-packet fast path (no fragmentation overhead).

### Test requirements

- Simulator support for a large structure/string type + tests that a value exceeding one packet
  round-trips via fragmentation (extend the sim, which currently models only the 88-byte STRING).
- Unit tests for the fragment loop (offset advance, reassembly, single vs multi fragment, the
  terminal `0x00` vs continuing `0x06`).
- Full matrix: `cargo fmt --check`, `cargo clippy --all-targets --features ffi -D warnings`,
  `SKIP_PLC_TESTS=1 cargo test --workspace --features ffi`, `plc_sim_tests`.
- Hardware re-validation (maintainer): `Str500`+ custom string members read+write in controller
  and program scope; confirm the CODEX-AY size-limit error no longer fires for supported sizes.

### Acceptance criteria

- `read_tag`/`read_string_tag` transparently read structures larger than one packet.
- `write_tag(String)` writes custom strings larger than one packet via fragmentation.
- No regression to the single-packet path; new sim + unit tests pass; hardware confirms `Str500`.

### Out of scope

- Connected (Class 3) messaging / Large Forward Open to raise the per-packet size — an alternative
  scaling axis, separate task.
- The batch-read byte-budget split (CODEX-AW) — related packet-size work but a different path.

### Risks / gotchas

- The response from `send_cip_request` includes the ~16-byte encapsulation/CPF prefix before the
  CIP reply (service at offset 16, status at offset 18) — parse the reply header at the right
  offset (an early hand-rolled probe misread `resp[2]`).
- Reassembly must strip the 2/4-byte type prefix only from the first fragment.
- Choose a conservative fragment size well under the measured ~494-byte request ceiling.

## Codex log

2026-07-08 codex [GPT-5] - Implementation submitted.

- Added client support for CIP Read Tag Fragmented (`0x52`) and Write Tag
  Fragmented (`0x53`) for large structure/string payloads.
- `read_tag` now detects CIP `0x06` Partial Transfer from a normal read and
  reassembles fragments starting at offset 0 before decoding the same
  type-prefixed payload shape as the single-packet path.
- Handle-aware string writes now fall through to fragmented writes when the
  built request exceeds the single-packet limit, splitting the structure
  payload into conservative chunks while preserving the target's real structure
  handle.
- Fixed the `0xD2` ambiguity for fragmented reads: `0xD2` can be an inner Read
  Tag Fragmented reply as well as an Unconnected Send reply, so direct-CIP
  fallback/extraction no longer unwraps fragmented-read partial replies.
- Extended the simulator with a 600-byte custom string tag and `0x52`/`0x53`
  handlers. Added a simulator integration test proving public
  `read_string_tag` / `write_tag(PlcValue::String)` round-trip a 520-byte value
  via fragmentation.
- Updated `docs/STRING_HANDLING.md` to describe fragmentation and the remaining
  hardware confirmation item for `Str500+`.
- Verification passed: `cargo test --test plc_sim_tests --locked`,
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`,
  `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`.
- Hardware re-validation remains pending for real `Str500+` controller and
  program-scope tags.

## Claude review

### 2026-07-08 claude [Opus 4.8]

Independent verification green: fmt, clippy `--all-targets --features ffi -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --features ffi` (0 failed), `plc_sim_tests` **25/25** (incl. the new `simulated_plc_large_custom_string_round_trips_with_fragmentation`, a 520-char round-trip exercising multi-fragment reassembly).

Implementation is sound: `READ_TAG_FRAGMENTED`/`WRITE_TAG_FRAGMENTED` (0x52/0x53); `read_tag_direct` routes `0x06` Partial Transfer to `read_tag_fragmented` (offset loop, empty-fragment + offset-overflow guards, reassemble → shared `decode_type_prefixed_value`); `write_string_fragmented` replaces the CODEX-AY size-limit error and sizes fragments from the real request overhead; the `0xD2`-status direct fallback is correctly excluded for fragmented reads.

**Claude fix-during-merge (mechanical, mirrors `read_tag_direct`):** the `0x06`→fragmented fallback was only in `read_tag_direct`, so `read_tag` on a whole large-UDT **array element** (`gTestUDT_Array[N]`, routed through `read_array_element_workaround`) still failed with `0x06`. Exposed by the maintainer's extended `gTestUDT` (now carrying Member6+Member7), whose whole-element size exceeds one packet. Two edits to `read_array_element_workaround`: (1) the BOOL-detection probe tolerates `0x06` (a partial-transfer element can't be a BOOL array), (2) the element read falls back to `read_tag_fragmented` on `0x06`. Hardware-validated: the full-coverage preflight went from 15 `gTestUDT_Array[N]` failures to **2304/2304 reads, PASS**.

Residual risk: single-tag `Str500+` write/read fragmentation is **simulator-validated only** — real-hardware confirmation is pending (the maintainer's Member7 is currently `Str400`, which fits one packet). `read_array_range` for huge scalar ranges is out of this task's string/UDT scope.

## Verdict

**Merged 2026-07-08** (with a Claude fix-during-merge for the array-element read path, hardware-validated on the extended UDT). CIP fragmentation removes the one-packet ceiling for large strings/UDTs; the string case is sim-covered and whole large-UDT array-element reads are hardware-proven. Real `Str500+` single-tag hardware confirmation remains a maintainer follow-up.
