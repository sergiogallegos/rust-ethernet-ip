---
id: CODEX-AZ
title: CIP fragmented read/write for structures larger than one packet (Str500+, large UDTs)
owner: codex
status: open
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

## Claude review

## Verdict
