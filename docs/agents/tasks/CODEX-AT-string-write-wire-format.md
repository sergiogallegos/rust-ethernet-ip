---
id: CODEX-AT
title: STRING wire format — direct structure writes work; fix the encoding, decode reads, retire the firmware-quirk misdiagnosis
owner: codex
status: open
created: 2026-07-02
last-update: 2026-07-02 claude [Fable 5]
---

## Brief

### Goal

A 2026-07-02 maintainer-authorized hardware probe ([`docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md`](../../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md)) disproved the STRING section of [`docs/agents/notes/ab-firmware-quirks.md`](../notes/ab-firmware-quirks.md): a direct Write Tag to a controller-scoped STRING succeeds on a 5069-L330ERM fw38 when encoded per the Logix structure-write format. The library's wired path (`write_tag(PlcValue::String)`) fails with CIP extended `0x2107` because it emits atomic type `0x00CE` with an unpadded payload — and `0x2107` is the Logix Data Access "tag type mismatch" error, not a firmware prohibition. This is the second read-based conclusion overturned by hardware evidence (precedent: CODEX-O).

This task makes STRING round-trips honest end to end:

1. **Write encoding.** `write_tag(PlcValue::String)` (and every path that funnels into it: `write_string_tag`, the actor/service layers, `eip_write_string`, the C#/Python wrappers) emits the hardware-verified request: service `0x4D`, tag path, data type `A0 02 CE 0F` (structure marker `0x02A0` + standard-STRING template handle `0x0FCE`), element count `01 00`, then an **88-byte payload**: `LEN` u32 LE + `DATA[82]` zero-padded + 2 alignment pad bytes. The probe pinned both failure modes: 86-byte payload → `0x13` "not enough data"; atomic `0x00CE` type → `0xFF`/`0x2107`.
2. **Batch writes.** `encode_type_prefixed` (`crates/protocol/src/values.rs:58`) has the same defect (type `0x00CE`, pads `DATA` to 82 but omits the structure marker/handle and the 2 pad bytes). Fix identically so batch STRING writes stop dying with the batch-level `0x1E` embedded error documented in `src/client.rs:3407-3419` and `src/lib.rs:58`.
3. **Read decoding.** On real hardware a STRING read reply carries type `0x02A0` with the payload beginning `CE 0F` (handle) + `LEN u32` + `DATA[82]` + 2 pad — the current `AB_UDT | UDT` arm of `decode_payload` (`crates/protocol/src/values.rs:152`) returns that raw blob as `PlcValue::Udt { symbol_id: 0 }`. Add a peek: structure payloads whose handle is `0x0FCE` with a plausible length decode to `PlcValue::String`. Other handles keep the existing `Udt` behavior — broader structure-handle handling belongs to CODEX-AO, do not generalize here.
4. **Simulator alignment.** `tests/plc_sim.rs` currently *accepts* the library's broken write shape (`CIP_TYPE_STRING | CIP_TYPE_STRUCTURE` at line 531 — a client-derived mirror, the exact failure mode CODEX-AN targets) and *serves* reads as atomic `0x00CE` (line 842), which is why sim tests decode STRING reads to `PlcValue::String` while hardware returns `Udt`. Make the sim match the hardware evidence: STRING reads reply `A0 02 CE 0F` + 88-byte payload; STRING writes require the correct structure encoding and reject the old atomic shape with `0xFF`/`0x2107` (pinning the regression), citing the validation doc in comments.
5. **Error-text honesty.** The extended-error text for `0x2107` claims "vendor-specific or composite" (`src/monitoring.rs:407, 599`; `src/client.rs:3301-3302`) and the write-tag docs blame firmware. Correct to the documented meaning: Read/Write Tag Service data-type mismatch.
6. **Docs and manifest.** Rewrite the STRING section of `docs/agents/notes/ab-firmware-quirks.md` (state what was actually wrong, link the validation doc; leave the UDT-array-element-member and symbol_id sections untouched — they were not probed). Update the Known Limitations block in `src/client.rs:98-134` and the `src/lib.rs` mentions. In `examples/full_coverage_tags.json`, relabel the two plain STRING entries (`ctrl.STRING`, `prog.STRING`) from `firmware_blocked_string` to writable, and update the three runners' expectations accordingly; leave `firmware_blocked_udt_string_member` / `firmware_blocked_udt_array_element_member` labels as-is pending hardware evidence. Grep README/C#/Python docs for the "cannot write STRING" claim and sweep. CHANGELOG `[Unreleased]` `### Fixed`.

### Context to read first

- The validation doc above — it is the contract for the byte shapes. The probe source is reproducible from it (appendix of request layouts).
- `crates/protocol/src/values.rs` — `write_data_type` (23), `encode_payload` String arm (44), `encode_type_prefixed` String arm (58), `decode_payload` `AB_UDT | UDT` arm (152). Note `write_data_type` returns `u16`; the String structure type needs 4 bytes on the wire, so `build_write_request` (`src/client.rs:3095-3108`) needs a type-bytes seam rather than a wider return type bolted onto every caller.
- `src/client.rs:2969-2974` — the wired write path; `src/ffi.rs:1039` (`eip_write_string`) and `src/client/service_layer.rs:7` both funnel into `write_tag`, so no wrapper-side wire changes should be needed (verify with the C#/Python suites).
- `docs/agents/tasks/CODEX-AP-string-udt-graveyard.md` — item 5's decision point is now answered by hardware: `write_ab_string_components` *works* but is superseded by the fixed direct write; record that disposition in AP when it runs (retire it with a pointer to the working path, or keep as documented non-atomic fallback — AP's call, with this task's evidence).
- `docs/agents/tasks/CODEX-AN-*.md` — the sim-as-oracle rule this task's sim changes must follow; coordinate if concurrent.
- `docs/agents/tasks/CODEX-AM-tag-addressing-correctness.md` fix 4 — the probe's `.DATA[i]` success went through `write_tag`'s array workaround, *not* the malformed `TagPath` `StringData` segment; AM's fix stands, and acceptance there references `write_ab_string_components`.

### Files to create or modify

`crates/protocol/src/values.rs` (+ its pinned-byte tests in `crates/protocol/src/tests.rs`), `src/client.rs` (build_write_request seam, Known Limitations docs, 0x2107 text), `src/monitoring.rs` (0x2107 text), `tests/plc_sim.rs`, `tests/plc_sim_tests.rs`, `docs/agents/notes/ab-firmware-quirks.md`, `examples/full_coverage_tags.json` + the three full-coverage runners, `CHANGELOG.md`, plus any docs the limitation-claim grep surfaces.

Also `examples/data_types_showcase.rs`: its embedded `#[cfg(test)]` tests (run only under `--all-targets`, so invisible to the normal gate) pin stale STRING expectations — type code `0x00DA` and a u8-length short-string payload (`[2, 72, 105]`) — that contradict both the library's current `0x00CE`/u32-length behavior and this task's target encoding (found 2026-07-02 during the CODEX-AK review; pre-existing, two tests fail at lines ~395/~426). Update those expectations to the fixed wire format as part of this task.

### Behavior

- `write_tag("AnyStringTag", PlcValue::String(s))` succeeds against the sim (and hardware) in one request for `s.len() <= 82`; longer input keeps the existing `StringTooLong`-class rejection.
- Batch STRING writes succeed via the same encoding.
- `read_tag` of a standard STRING returns `PlcValue::String` from both sim and hardware byte shapes; non-`0x0FCE` structure handles still return `PlcValue::Udt`.
- Round-trip: write then read returns the written value; values shorter than the previous content read back clean (the 82-byte zero-padded `DATA` guarantees no residue).

### Test requirements

- Pinned-byte test for the full write request against the exact hardware-verified bytes (type `A0 02 CE 0F`, count `01 00`, 88-byte payload) — cite the validation doc.
- Pinned-byte test for `encode_type_prefixed` String output.
- Decode tests: hardware-shaped read payload (`CE 0F` + LEN + 82 + pad) → `PlcValue::String`; truncated payload → error, not panic; unknown handle → `Udt` unchanged.
- Sim round-trips: single write/read, batch write/read, write-shorter-over-longer residue check; regression test asserting the *old* atomic-`0x00CE` write shape is rejected by the sim with `0xFF`/`0x2107`.
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, C# + Python suites (FFI-consuming behavior changed even though signatures didn't).

### Acceptance criteria

- Items 1–6 implemented with the tests above; each new sim test demonstrated failing pre-fix (run against the parent commit, note in the Codex log).
- No remaining code or doc claims that direct STRING writes are firmware-blocked; the quirks note links the validation doc.
- Manifest relabel limited to the two plain STRING entries; UDT-member labels untouched.
- Wire-touching change flagged in the log for the maintainer's full-coverage hardware re-run before the next release — that run is also the gate for the manifest relabel (the runners must report the two STRING tags as write-verified, anomalies 0).

### Out of scope

- Custom-length `STRINGnn` types (template-specific handles and instance sizes — needs handle discovery; note as follow-up, return a typed error rather than emitting a guessed handle if the tag's type isn't standard STRING… in practice the write path cannot know the target type without a read, so the standard-STRING encoding is simply what `PlcValue::String` means; document that).
- STRING members inside UDTs / UDT array elements — labels stay `firmware_blocked_*` until hardware says otherwise (candidate extension of the maintainer's validation run; record results if attempted).
- Retiring `write_ab_string_components` and the rest of the graveyard — CODEX-AP.
- Structure-handle handling for general UDT reads (`symbol_id: 0` blob shape) — CODEX-AO.

### Risks and gotchas

- Single-controller evidence (5069-L330ERM fw38). The encoding matches the ecosystem-standard format (pycomm3, libplctag), so cross-firmware risk is low — but do not extrapolate to the UDT-member cases; those keep the RMW workaround.
- The read-decode change alters observable behavior: hardware consumers who learned to fish STRINGs out of `PlcValue::Udt` blobs will now get `PlcValue::String`. That's the documented contract (sim tests and docs always promised `String`), so it's a fix, but call it out in the CHANGELOG and verify the C# `ReadString`/Python `read_string` paths against the new shape.
- `ALT_STRING` (`0x00DA`) decode exists (`decode_short_string`) — leave it alone; it's a different reply shape, not part of this evidence.
- Batch-error text special-casing STRING writes (`src/client.rs:3407-3419`) becomes stale once batch STRING writes work — update or remove it with the fix, don't leave a message describing a failure that no longer happens.
- Sim changes and CODEX-AN both touch `tests/plc_sim.rs`; whichever lands second rebases. Same for CODEX-AP in `src/client/string.rs` and the 0x2107 text sites it also brushes (`parse_extended_error`).

## Codex log

## Claude review

## Verdict
