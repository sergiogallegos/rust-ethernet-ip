---
id: CODEX-AM
title: Tag addressing correctness — member-suffix drop, bit syntax, batch BOOL arrays, .DATA[i] segment, program discovery
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
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

## Claude review

## Verdict
