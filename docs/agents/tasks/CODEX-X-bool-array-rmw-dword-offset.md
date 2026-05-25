---
id: CODEX-X
title: BOOL array element RMW addresses the wrong DWORD for indices ≥ 32
owner: codex
status: open
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

### Goal

`EipClient::read_bool_array_element_workaround` (`src/client.rs:1039-1118`) and `EipClient::write_bool_array_element_workaround` (`src/client.rs:1516-1609`) implement the AB BOOL-array RMW dance — read the parent DWORD, extract or modify bit `(index % 32)`, write the DWORD back. Both functions correctly compute `bit_index = (index % 32) as u8` (lines 1114, 1578), but the CIP request for the parent DWORD is hardcoded to read DWORD[0]:

```rust
// client.rs:1052 (read path) and :1529 (write path)
self.send_cip_request(&self.build_read_request_with_count(base_array_name, 1))
```

`build_read_request_with_count(base_array_name, 1)` returns the **base** array without an element-index segment, which AB resolves to "element 0 of the array" — i.e. **DWORD[0] only**. The DWORD that actually holds bit `(index / 32)` is never requested, so every BOOL access where `index >= 32` reads/writes bit `(index % 32)` of DWORD[0] instead of the correct DWORD.

Bug surfaced by the 2026-05-24 full-coverage hardware run (`docs/agents/log.md`). Verify-rate evidence on `gTestArray_BOOL[0..127]` (4 DWORDs):

- Rust binding: 79/128 verified (61.7%)
- C# binding: 75/128 verified (58.6%)
- Python binding (post-CODEX-W): 90/128 verified (70.3%)

The 62.5% expected verify rate exactly matches the aliasing model: 128 bool writes mapped onto 32 physical bits, each bit retains the last of its 4 aliased writes; reading each index gets the last-written value for its `(i mod 32)` group; one match per group is guaranteed + the others match with random 50%. `(1 + 3·0.5) / 4 = 0.625`. Verifies the diagnosis to a fraction of a percent.

Fix scope: pass `index / 32` as the DWORD index to the CIP request, using `EipClient::build_read_array_request` (`src/client.rs:4340`) which already supports element-addressed reads via 0x28/0x29/0x2A segments. Same change in the write path (use `build_write_request_with_index` or equivalent — `src/client.rs:1662`).

### Context to read first

- `src/client.rs:854-902` — `read_tag` dispatch. Simple-array-element paths (`gTestArray_BOOL[5]` style) take the BOOL workaround branch; complex paths (`gTestUDT_Array[3].Array_BOOL[5]` style) don't and fail differently — that's [[codex-y-nested-bool-udt-array-element]].
- `src/client.rs:990-1037` — `read_array_element_workaround`. Test-reads the array with count=1 to detect BOOL (data type `0x00D3` DWORD), then dispatches to `read_bool_array_element_workaround`. Detection logic is fine; only the workaround's DWORD-index is wrong.
- `src/client.rs:1039-1118` — `read_bool_array_element_workaround`. Bug site #1 at line 1052.
- `src/client.rs:1516-1609` — `write_bool_array_element_workaround`. Bug site #2 at line 1529.
- `src/client.rs:4340-4400` — `build_read_array_request`. Reference implementation of element-addressed reads. Use this for DWORD-indexed reads.
- `src/client.rs:1662-1730` — `build_write_array_request_with_index`. Reference for element-addressed writes.
- `tests/plc_sim.rs` — simulator's `BOOL_ARRAY` currently has 6 elements (added by CODEX-W). Needs at least 64 elements to exercise the cross-DWORD bug in a regression test.
- 2026-05-24 log entries — context for what surfaced this.

### Files to create or modify

- `src/client.rs` — modify `read_bool_array_element_workaround` (line 1039) to request DWORD `(index / 32)` instead of DWORD 0; same for `write_bool_array_element_workaround` (line 1516). Either inline the element-addressed request build or call `build_read_array_request(base_array_name, index / 32, 1)` and the matching write helper. Keep the bit-extraction at line 1114 / bit-modify at lines 1578-1583 unchanged (the `% 32` arithmetic is already correct).
- `tests/plc_sim.rs` — expand `BOOL_ARRAY` to at least 64 elements (two DWORDs) so the simulator can exercise cross-DWORD bit access. Suggest pattern: `BOOL_ARRAY[i] = (i % 3 == 0)` — gives unambiguous true/false expectations for every index.
- `tests/plc_sim_tests.rs` — add `simulated_plc_bool_array_cross_dword_read_write` that writes BOOL_ARRAY[5], BOOL_ARRAY[35], BOOL_ARRAY[63] (one per DWORD) with non-aliased values, reads each back, and verifies the writes don't alias.
- `CHANGELOG.md` — `### Fixed` line under `[Unreleased]`: "BOOL array element read/write no longer aliases all indices to DWORD[0]; index ≥ 32 now addresses the correct DWORD."

### Behavior

For `read_bool_array_element_workaround(base, index)`:

1. Compute `dword_index = index / 32`.
2. Build a read request for `base[dword_index]` (one DWORD) using element-addressed CIP — i.e. call `build_read_array_request(base, dword_index, 1)`.
3. Send, extract CIP response, check error.
4. Verify the response data type is `0x00D3` (DWORD) as before.
5. Extract the DWORD bytes from the response; compute `bit_index = (index % 32) as u8`.
6. Return `PlcValue::Bool((dword >> bit_index) & 1 != 0)`.

For `write_bool_array_element_workaround(base, index, value)`:

1. Compute `dword_index = index / 32`.
2. Read DWORD `dword_index` via the same element-addressed request as above.
3. Modify the bit at `(index % 32)` per the existing code at `client.rs:1578-1583`.
4. Build an element-addressed write request for `base[dword_index]` with the modified DWORD bytes, data type `0x00D3`, element count 1.
5. Send, check write response for errors.

The semantics of the public API don't change — callers see `read_tag("gTestArray_BOOL[50]")` return the right bit, and `write_tag("gTestArray_BOOL[50]", PlcValue::Bool(true))` set the right bit. Only the wire-level CIP changes (an element-addressing segment is added to the path).

### Test requirements

**Simulator (no hardware needed, runs in CI):**

- `simulated_plc_bool_array_cross_dword_read` — read BOOL_ARRAY at indices [0, 31, 32, 33, 63] (boundary cases on both sides of every DWORD edge). Verify the returned bool matches the simulator's stored value, independently for each index.
- `simulated_plc_bool_array_cross_dword_write` — write distinct values to BOOL_ARRAY[5], BOOL_ARRAY[35], BOOL_ARRAY[63]. After all three writes, read each back and confirm no aliasing — each index returns the value last written to it specifically, not whatever was last written to a `(i % 32)` collision.

**Pinned-bytes (CIP wire format, in `tests/plc_sim_tests.rs` or a new `tests/bool_array_wire_tests.rs`):**

- One test that captures the bytes built by the new request for BOOL_ARRAY[50] and asserts they include element segment `0x28 0x01` (8-bit index = 1, i.e. DWORD[1]) — guards against a future refactor silently reverting to DWORD[0].

**Hardware (operator runs, not automated):**

- After landing, re-run `examples/test_plc_full_coverage.rs` against the maintainer's PLC. Expect: `ctrl.BOOL_array` `verify+` column rises from ~79/128 to **128/128**; `prog.BOOL_array` rises from ~71/100 to **100/100**. Codex records the new exerciser output line in `## Codex log`.

### Acceptance criteria

- BOOL array writes at any valid index do not corrupt bits in adjacent DWORDs (simulator regression test passes).
- BOOL array reads at any valid index return the value that was last written to *that specific index*, not to its `(i % 32)` alias.
- Existing simulator BOOL-array tests in `tests/plc_sim_tests.rs` (the 6-element coverage from CODEX-W) still pass — backwards-compatible on the index 0..5 path.
- `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests` all pass.
- `CHANGELOG.md` `[Unreleased]` section gains the `### Fixed` entry.
- Hardware re-run output captured in `## Codex log` if Codex has hardware access; otherwise maintainer captures during merge.

### Out of scope

- The matching bug for **complex paths** like `gTestUDT_Array[i].Array_BOOL[j]` — that's [[codex-y-nested-bool-udt-array-element]]. CODEX-X only fixes the simple top-level path (`gTestArray_BOOL[i]`). Both bugs are real; both should land before tagging v0.8.0; but they touch different dispatch branches and warrant separate review.
- Batch BOOL writes via `write_tags`. The batch path serializes through `_execute_write_operations` which has its own code path; out of scope here.
- Performance work. The element-addressed read is one extra path-segment byte on the wire — no measurable cost.
- Removing the `dword_value = u32::from_le_bytes([value_data[0], value_data[1], value_data[2], value_data[3]])` boilerplate or factoring out a shared `read_dword_at_index` helper. Yes, the read and write functions duplicate the parse logic — leave it for a follow-up tidy. Don't bundle structural cleanup into the bug fix.

### Risks and gotchas

- **Element segment encoding choice (8 / 16 / 32-bit index).** `build_element_id_segment` already picks the smallest encoding (`0x28` for 0..255, `0x29` for 256..65535, `0x2A` for 65536+). For BOOL arrays the maximum reasonable DWORD index is `BOOL_ARRAY.length / 32` — for a 1024-bool array that's DWORD 31, still 8-bit. Don't bypass the existing helper.
- **Write response status check.** The existing write path checks `check_cip_error(&write_cip_data)` at line 1605 — keep that. AB returns specific errors when an element-addressed DWORD write is rejected (e.g. tag isn't really an array, wrong type). The errors should bubble up as `EtherNetIpError::Protocol`.
- **Backwards compat on the simulator path.** The simulator (`tests/plc_sim.rs`) currently honours `BOOL_ARRAY` reads with count=1. If the new CIP request adds an element segment, the simulator's request parser needs to also accept the segment — verify by reading any failing test trace. Likely the simulator already handles element segments because `gTestArray_DINT[5]` (DINT array element) works there; if not, extend `handle_read_cip_request` to accept the element segment.
- **Don't change the dispatch in `read_array_element_workaround` (`client.rs:990`).** That function's job is detection + routing; it correctly routes to `read_bool_array_element_workaround` after seeing the DWORD data type. The bug is *inside* the workaround, not in the routing.
- **The user-facing API contract is silent on whether the BOOL array fits in one DWORD.** Today's behavior would be correct only for arrays of ≤ 32 BOOLs. Many real PLCs have BOOL[128] / BOOL[1024]; the spec says nothing limiting array size. Fix without adding any size-cap guard.

## Codex log

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
