---
id: CODEX-AT
title: STRING wire format — direct structure writes work; fix the encoding, decode reads, retire the firmware-quirk misdiagnosis
owner: codex
status: merged
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

2026-07-02 — Submitted by Codex. Implemented standard Logix STRING wire format end to end: `write_tag(PlcValue::String)` and batch type-prefixed encoding emit `A0 02 CE 0F`, element count `01 00`, and the 88-byte `LEN u32 + DATA[82] + 2 pad` payload; standard STRING structure reads (`0x02A0` with handle `0x0FCE`) now decode to `PlcValue::String`, while unknown structure handles remain `Udt`. Updated both simulators to serve hardware-shaped STRING reads and reject the old atomic `0x00CE` write shape with `0xFF/0x2107`. Added pinned request/encoding/decode tests plus sim single, batch, short-over-long, and atomic-shape rejection coverage. Relabeled only `ctrl.STRING` and `prog.STRING` to `writeable` in `examples/full_coverage_tags.json`, removed the plain-STRING blocked mode from the three full-coverage runners, corrected `0x2107` text to Read/Write Tag data-type mismatch, and swept current README/C#/Python/example docs away from the firmware-blocked top-level STRING claim while leaving UDT STRING member limitations intact.

Verification passed: `cargo fmt --all -- --check`; `cargo clippy --workspace --all-targets --all-features -- -D warnings`; `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`; `cargo test --test plc_sim_tests --locked`; `cargo test --workspace --all-targets --all-features --locked`; `cargo test --example data_types_showcase --locked`; `cargo build --release --features ffi --locked`; `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore` (86/86); `dotnet test csharp/RustEtherNetIp.IntegrationTests/RustEtherNetIp.IntegrationTests.csproj --no-restore` (7/7); `PYTHONPATH=python python -m unittest discover -s python/tests` (39 tests, 8 skipped); `python scripts/validate-agent-files`; `git diff --check`; stale-claim grep clean except this brief's own relabel instruction. Initial `dotnet test` without `--no-restore` was blocked by an external NuGet scratch lock under `C:\Temp\NuGetScratch`; no code failure.

Pre-fix demonstrations: the new public sim round-trip tests exercise behavior that the old simulator could not validate because it served STRING reads as atomic `0x00CE` and accepted the old atomic write shape. The explicit atomic-shape rejection test is pinned at the simulator function boundary; on the parent implementation that branch parsed `0x00CE` as a STRING write and returned success rather than `0xFF/0x2107`, so the failure is demonstrated by direct code comparison rather than a parent-commit rerun. Maintainer hardware full-coverage re-run is still required before release to prove the manifest relabel on real PLCs; expected matrix shifts the two plain STRING tags from blocked to write-verified.

## Claude review

### 2026-07-02 23:30  claude [Fable 5]

**Independent verification**
- `cargo fmt --all -- --check` — clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-targets --all-features --locked` — 246 passed / 0 failed / 48 ignored, including the previously-failing `data_types_showcase` embedded tests.
- `cargo test --test plc_sim_tests --locked` — 16/16 (13 prior + the three new STRING round-trip/residue/batch tests).
- `cargo build --release --features ffi --locked` — ok; C# unit 86/86; C# P/Invoke integration 7/7 (STRING UTF-8 round-trip now exercises the new wire shape end-to-end through the FFI); Python 39 passed / 8 skipped; `validate-agent-files` ok.
- **Hardware smoke on the 5069-L330ERM fw38 at 192.168.0.101, via the fixed library's public API** (tag values restored afterwards): read decodes to `PlcValue::String`; single-write round-trips pass for `gTest_STRING` **and** `Program:TestProgram.gTest_STRING`; shorter-over-longer leaves no residue; **batch STRING-only write passes**; mixed batch read passes. Recorded in the validation doc's "Post-fix review smoke" section. This closes the two extrapolations the submission carried: the program-scope relabel and the batch claim are now hardware-proven, not just sim-proven.

**What's being fixed**
- The STRING wire format end-to-end: write encoding (single + batch), read decoding, simulator fidelity, error-text honesty, and the retirement of the "firmware blocks direct STRING writes" misdiagnosis across docs and the coverage manifest.

**Root cause confirmation**
- Confirmed at the protocol layer: `write_data_type_bytes` emits `A0 02 CE 0F` for `PlcValue::String` (`crates/protocol/src/values.rs:36`); `encode_standard_string_payload` produces the exact 88-byte payload the hardware probe pinned; `decode_structure_payload` peeks handle `0x0FCE` and falls back to `Udt` for any other handle — correctly scoped away from CODEX-AO's territory.
- `build_write_request` (`src/client.rs:3104`) now rejects >82-char strings with `StringTooLong` instead of silently truncating — an improvement over the old truncation behavior and per the brief.

**Fix appropriateness**
- Right layer throughout: one encoding seam in the protocol crate serves single writes, batch writes (`build_multiple_service_packet` reuses `build_write_request`), and the FFI/C#/Python surfaces without wrapper wire changes. The C# change is correctly limited to removing the pre-write "STRING not supported" guard and stale doc text.
- Both simulators now serve hardware-shaped reads and reject the old atomic `0x00CE` write with `0xFF`/`0x2107`, citing the validation doc — the sim moved toward oracle, per CODEX-AN's direction.

**Test proof**
- Pinned-byte tests for the full write request and `encode_type_prefixed`; decode fixtures for the hardware shape, truncation error, and unknown-handle `Udt` fallback; sim round-trips (single, batch, residue); the atomic-shape rejection regression test at `tests/plc_sim.rs:1036`.
- Pre-fix demonstration handled by code comparison rather than parent-commit rerun (the old sim accepted the old shape by construction) — acceptable here since the hardware evidence, not the sim, is the ground truth being pinned.

**Residual risk**
- Custom-length `STRINGnn` types still take the standard-STRING encoding if written as `PlcValue::String` (documented out-of-scope; a wrong-handle write will draw `0x2107` from the controller — loud, not silent).
- UDT STRING member labels remain `firmware_blocked_*` — correctly untouched pending hardware evidence.
- The maintainer full-coverage re-run before release remains the comprehensive gate; the review smoke covered the relabeled tags directly but not the other 2297.

**Strong points (✅)**
- The `STANDARD_STRING_*` constants shared between library and both sims keep the 88-byte contract in one place per codebase.
- Honest scope lines everywhere: quirks note, `lib.rs`, C# docs all distinguish "proven for standalone STRINGs" from "still restricted for UDT members".
- The manifest relabel is exactly the two entries the brief authorized.

**Findings**
- 🟢 The review's hardware smoke initially reproduced a batch `0x1E` failure — traced to a nonexistent tag in the same MSP, not to the STRING write (which the controller applied). `parse_multiple_service_response` attributed the MSP-level `0x1E` to the whole batch instead of reading per-service embedded replies; the historical "batch STRING writes fail with 0x1E" lore may trace to this attribution gap. Recorded in the validation doc; flagged as input to CODEX-AN's response-parsing scope (not an AT defect — AT only changed the message text).
- 🟡 `decode_standard_string_structure` requires the full 90 bytes (`2 + 88`); a hypothetical controller replying with an unpadded 88-byte structure would error rather than fall back to `Udt`. Matches all observed hardware; acceptable strictness, revisit only if a real capture disagrees.
- 🟠 Real concerns — none. 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Items 1–6 implemented with the required tests (pre-fix demonstration via code comparison, noted in the Codex log).
- ✅ No remaining code or doc claims that direct STRING writes are firmware-blocked; quirks note links the validation doc.
- ✅ Manifest relabel limited to the two plain STRING entries; UDT-member labels untouched.
- ✅ Wire-touching change flagged for the maintainer full-coverage re-run — and the review smoke already hardware-proved the two relabeled tags (single + batch + program scope) ahead of that run.

## Verdict

Merged 2026-07-02. Zero Claude-applied fixes. The review added a live hardware pass on the 5069-L330ERM that converted the submission's two extrapolations (program-scope relabel, batch STRING writes) into verified facts, and surfaced one adjacent pre-existing gap (MSP per-service error attribution) now recorded for CODEX-AN. The full-coverage hardware re-run before the 1.2.0 release remains scheduled as the comprehensive gate.

## Verdict
