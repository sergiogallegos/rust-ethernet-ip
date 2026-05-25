---
id: CODEX-Y
title: BOOL workaround not applied to nested BOOL arrays inside UDT array elements
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

### Goal

`EipClient::read_tag` (`src/client.rs:854-902`) routes simple top-level array element accesses (`gTestArray_BOOL[5]`) through `read_array_element_workaround` (line 990), which detects the AB BOOL packing (data type `0x00D3` DWORD) and applies the bit-extraction workaround.

Complex paths — anything with member access after the first bracket, e.g. `gTestUDT_Array[3].Array_BOOL[5]` — skip that branch (line 894 onwards) and go directly to `build_read_request` (line 898). For nested BOOL arrays, that produces a CIP request that AB rejects.

Live evidence captured on the maintainer's ControlLogix 1756-L75 fw33 (2026-05-24):

```
ERR  gTestUDT_Array[3].Array_BOOL[5]  -> Protocol error: CIP Error 0x05: Path destination unknown
OK   gTestUDT_Array[3].Array_BOOL[0]  -> Udint(0)        # whole DWORD, not a Bool!
ERR  gTestUDT_Array[3].Array_BOOL[19] -> Protocol error: CIP Error 0x05: Path destination unknown
OK   gTestUDT_Array[3].Array_DINT[5]  -> Dint(999999)   # same depth, DINT works
OK   gTestUDT.Array_BOOL[5]           -> Bool(true)      # top-level UDT works
OK   gTestArray_BOOL[5]               -> Bool(true)      # plain array works
```

Two distinct symptoms inside the same broken dispatch:

- Index 0 succeeds at the wire level but returns the **whole DWORD** as `Udint` instead of extracting bit 0 as a `Bool`. The library never applied the workaround, so the response is parsed by `parse_cip_response` as a generic DWORD value.
- Indices != 0 fail with `CIP 0x05` because the CIP request uses element addressing into a path that AB does not allow element addressing on (nested BOOL arrays inside structures are addressed via the parent DWORD; the element segment confuses the firmware).

Fix scope: detect that a complex tag path resolves to a BOOL-array-element access (`...Array_BOOL[j]` where `Array_BOOL` is a `BOOL[]` member), and apply the same RMW-style workaround as `read_bool_array_element_workaround` but with the parent path being the UDT-nested base (`gTestUDT_Array[3].Array_BOOL` not just `Array_BOOL`).

Hardware impact: at the 2026-05-24 full-coverage run, 200 of 200 nested BOOL reads failed and 200 of 200 nested BOOL writes failed across all three bindings (controller scope; program scope dropped because the program UDT_Array doesn't include nested BOOLs in this test inventory). All other data types at the same path depth work — only BOOL is broken.

### Context to read first

- `src/client.rs:854-902` — `read_tag` dispatch. Lines 876-890 explicitly note that paths with member access after a bracket fall through to the complex-path branch. That's where the dispatch needs a new BOOL detection branch.
- `src/client.rs:990-1037` — `read_array_element_workaround`. Today's BOOL detection (line 1010-1018) lives here. The new complex-path detection should mirror this pattern but on the `gTestUDT_Array[3].Array_BOOL` parent path.
- `src/client.rs:1039-1118` — `read_bool_array_element_workaround`. The existing bit-extraction logic. The fix likely calls this (or a sibling) with the parent path as the `base_array_name`.
- `src/tag_path.rs` — `TagPath::parse`. The path `gTestUDT_Array[3].Array_BOOL[5]` parses to a nested `TagPath::Array { base_path: <gTestUDT_Array[3].Array_BOOL>, index: 5 }`. Use this to extract the parent path (everything before the final `[5]`).
- `src/client.rs:1480-1610` — write side. The write dispatch around `write_bool_array_element_workaround` has the same gap. Look at `client.rs:1481` to confirm where the equivalent dispatch happens and whether complex paths bypass the workaround on the write side too.
- [[codex-x-bool-array-rmw-dword-offset]] — companion brief fixing the related top-level RMW DWORD-offset bug. The DWORD-offset issue is **also present** in any new nested workaround (`Array_BOOL[5]` is bit 5 of DWORD 0, but `Array_BOOL[35]` is bit 3 of DWORD 1). Whoever lands second should rebase against the other's helper if [CODEX-X] introduces a shared `read_bool_array_element_at(base, index)` helper.
- 2026-05-24 log entries — context.

### Files to create or modify

- `src/client.rs` — add a complex-path BOOL detection step inside `read_tag` (probably after line 893, before the generic complex-path send). Pseudocode:
  ```
  if let Some((parent_path, index)) = parse_complex_array_element_access(tag_name) {
      // parent_path is e.g. "gTestUDT_Array[3].Array_BOOL"
      // index is e.g. 5
      if try_detect_bool_member(&parent_path).await? {
          return self.read_nested_bool_element_workaround(&parent_path, index).await;
      }
  }
  ```
  Detection can probe by issuing a count=1 read of the parent path (same trick `read_array_element_workaround` uses at line 1002) and checking for data type `0x00D3`. Cache the result on the per-tag attributes map if the cost matters.
- `src/client.rs` — analogous detection in `write_tag` for the write path (around the `write_bool_array_element_workaround` dispatch at line 1480).
- `src/client.rs` — new private helpers `read_nested_bool_element_workaround(parent_path, index)` and `write_nested_bool_element_workaround(parent_path, index, value)`. These can largely delegate to the top-level helpers if those are parameterized cleanly — e.g. if `read_bool_array_element_workaround` takes a fully-formed CIP path instead of just a base name, it becomes reusable. Otherwise duplicate-with-modification is acceptable for v0.8.0.
- `tests/plc_sim.rs` — add a UDT with a `BOOL[]` member (or extend an existing UDT). The simulator's existing UDT support is limited; this brief may need to expand `handle_read_cip_request` and `handle_write_cip_request` to model the nested-BOOL-DWORD shape. Keep simulator changes minimal and scoped to what the regression test needs.
- `tests/plc_sim_tests.rs` — add `simulated_plc_nested_bool_array_element_read_write` covering a `UDT_TAG.Array_BOOL[i]` round-trip.
- `CHANGELOG.md` — `### Fixed` line under `[Unreleased]`: "BOOL workaround now applied to nested BOOL array members inside UDT array elements (e.g. `gTestUDT_Array[3].Array_BOOL[5]`); fixes CIP 0x05 and DWORD-as-Udint return."

### Behavior

For `read_tag(tag_name)` where the parsed `TagPath` matches `Array { base_path: P, index: i }` and `P` resolves to a BOOL-typed member:

1. Build a parent path (everything before the final `[i]`).
2. Compute `dword_index = i / 32`.
3. Issue an element-addressed CIP read for `<parent_path>[dword_index]` (one DWORD).
4. Extract bit `(i % 32)` from the response.
5. Return `PlcValue::Bool(...)`.

For `write_tag(tag_name, PlcValue::Bool(value))` on the same path shape:

1. Same parent-path + DWORD-index resolution.
2. Read the DWORD (RMW phase 1).
3. Modify the bit at `(i % 32)`.
4. Write the modified DWORD back via element-addressed CIP write.

The detection step (BOOL vs DINT/REAL/etc.) needs to happen at runtime because the library doesn't currently maintain a typed UDT-member map. The cheapest approach is the test-read pattern already in `read_array_element_workaround` at line 1002 — issue a count=1 read, look at the response's data type, dispatch accordingly. Cache the result if profiling later shows it matters.

### Test requirements

**Simulator (no hardware, CI):**

- `simulated_plc_nested_bool_array_element_read` — extend the simulator UDT (or add a new one, e.g. `TEST_UDT_TAG` with member `BOOL_NESTED: BOOL[64]`). Pre-populate with a known pattern. Read `TEST_UDT_TAG.BOOL_NESTED[i]` for `i ∈ {0, 1, 31, 32, 33, 63}` and assert each returns the right bool.
- `simulated_plc_nested_bool_array_element_write` — write distinct values to two indices in the same DWORD and two in different DWORDs, read back, assert no aliasing.
- `simulated_plc_nested_bool_array_element_returns_bool_not_udint` — pin that the bit-0 case returns `PlcValue::Bool(...)`, not `PlcValue::Udint(...)`. This is the specific regression evidence from 2026-05-24.

**Hardware (operator runs):**

- After landing, re-run `examples/test_plc_full_coverage.rs`. Expect: `ctrl.UDTarr_elem_nested` reads `200/200` instead of the current `160/350` (the remaining gap is non-BOOL nested members like Array_DINT/REAL which already work, so the total category will hit `350/350`). Same category writes go from `150/350` (current) to `350/350`. Codex records the new line in `## Codex log`.

### Acceptance criteria

- `gTestUDT_Array[i].Array_BOOL[j]` reads return `PlcValue::Bool(...)` correctly for arbitrary `i` and `j`.
- `gTestUDT_Array[i].Array_BOOL[j]` writes set the right bit without aliasing (per [CODEX-X], DWORD-offset is computed from `j`).
- `gTestUDT.Array_BOOL[j]` (single-UDT, top-level — already works through `read_array_element_workaround`) continues to work — no regression.
- `gTestArray_BOOL[i]` (plain array — fixed by [CODEX-X]) continues to work — no regression.
- New simulator tests pass.
- `cargo fmt -- --check`, `cargo clippy -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests` all pass.
- `CHANGELOG.md` `[Unreleased]` section gains the `### Fixed` entry.

### Out of scope

- The top-level DWORD-offset bug — [[codex-x-bool-array-rmw-dword-offset]] owns that.
- BOOL member access on non-UDT-array bases (e.g. `gTestUDT.Member3_BOOL` — that's a simple BOOL member, not a BOOL array element; today's `parse_cip_response` already handles it correctly via the `0x00C1` data type path).
- Adding a generic typed-UDT-member cache so BOOL detection becomes free. Possible future polish; not gating.
- Multi-dim BOOL arrays. AB Logix doesn't support `BOOL[10,10]` natively (BOOL arrays are always 1-D DWORD-packed). Don't add complexity for a case the firmware doesn't have.

### Risks and gotchas

- **Path-rewriting fragility.** Extracting "everything before the final `[i]`" from a tag string is naive — `gTestUDT_Array[3].Array_BOOL[5]` has two bracket pairs. Use `TagPath::parse` to walk the structured representation, not string ops. The `TagPath::Array { base_path, index }` variant gives you the parent directly via `base_path.to_cip_path()`.
- **`base_path.to_cip_path()` for the parent.** `gTestUDT_Array[3].Array_BOOL` is itself a nested path — `Member(Array { base: gTestUDT_Array, index: 3 }, "Array_BOOL")`. Confirm `to_cip_path` handles this. If not, that's a `tag_path.rs` bug discoverable here — fix in the same brief.
- **Detection cost.** Each new BOOL access costs an extra round-trip to discover the data type. For hot paths this matters; for v0.8.0 correctness it doesn't. Don't pre-optimize.
- **Write detection symmetry.** The write dispatch doesn't have a `write_array_element_workaround` mirror — `write_tag` directly checks for the BOOL case at `client.rs:1480-ish`. Mirror the new detection on the write side without code-sharing if the read/write detection helpers don't compose cleanly.
- **CIP wire format for nested element-addressed BOOL.** The "right" CIP request for `gTestUDT_Array[3].Array_BOOL[dword_index]` is `symbolic(gTestUDT_Array) + element(3) + symbolic(Array_BOOL) + element(dword_index)`. AB *should* accept this for any 1-D array member of a UDT element. If it doesn't (worth confirming on hardware during implementation), the firmware-quirk page at `docs/agents/notes/ab-firmware-quirks.md` gets a new entry and the brief becomes "document, don't fix". The 2026-05-24 evidence suggests index=0 *did* succeed (returned the DWORD), which strongly implies the element segment is the problem, not the path shape — meaning the workaround should request just the parent (no element segment for the BOOL[] member) and slice the bit in software, exactly like the top-level workaround does.

## Codex log

- 2026-05-24 codex [gpt-5]: Submitted implementation. `read_tag` and `write_tag` now detect final array-element paths whose parent probes as packed BOOL-array DWORD data, including nested parents such as `UDT_ARRAY[3].BOOL_NESTED`, and route them through the shared BOOL DWORD RMW helper from CODEX-X. Simulator path parsing now preserves non-final array indices in canonical nested keys, and `UDT_ARRAY[3].BOOL_NESTED` regression coverage pins Bool-not-Udint reads plus same-DWORD and cross-DWORD writes. Verification so far: `cargo test --test plc_sim_tests`, `cargo clippy -- -D warnings`, `PYTHONPATH=python python3 -m unittest discover -s python/tests`, and `git diff --check` pass. Hardware full-coverage re-run not performed by Codex.

## Claude review

### 2026-05-24  claude  [Opus 4.7]

**Independent verification:**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 218/218
- `cargo test --test plc_sim_tests` — 13/13 (`simulated_plc_nested_bool_array_element_read_write` passes)
- **Hardware re-run against ControlLogix 1756-L75** via `examples/test_plc_full_coverage.rs` — see results below

**Hardware result:**

| Metric | Before fix | After fix |
|---|---|---|
| `ctrl.UDTarr_elem_nested` total reads | 160/350 (190 fails) | **350/350** ✅ |
| BOOL-specific subset (`Array_BOOL[j]` × 10 elements × 20 indices) | 0/200 | **200/200** ✅ |
| `gTestUDT_Array[i].Array_BOOL[0]` returns `Bool` not `Udint` | broken (`Udint(0)`) | **`Bool(...)`** ✅ |
| CIP 0x05 "Path destination unknown" on non-zero indices | 190 occurrences | **0 occurrences** ✅ |

Both symptoms from the 2026-05-24 evidence (`Udint(0)` regression at index 0, `CIP 0x05` at every other index) are gone.

**Strong points (✅):**
- Detection dispatch at `src/client.rs:894-902` (read) and `:3114-3121` (write) is symmetric. Both use the same `parse_final_array_element_access` + `detect_bool_array_path` pattern. The two-line guard is a clean diff against the existing dispatch flow.
- `parse_final_array_element_access` (`src/client.rs:986-993`) uses `TagPath::parse(...)` properly — pattern-matches on `TagPath::Array { base_path, indices } if indices.len() == 1`. Correctly bails on multi-dim arrays (`indices.len() > 1`) which is the right call: those have a separate CIP encoding path and aren't BOOL-array-shaped today.
- `detect_bool_array_path` (`src/client.rs:995-1004`) does a single test-read with `count=1`, checks the response's data type for `values::BOOL_ARRAY_DWORD` (0x00D3), and returns `false` on any CIP error or short response — defensive against false-positive routing into the workaround when the parent path isn't actually a BOOL array.
- `base_path.as_string()` is the right TagPath method to extract the parent — verified by reading `src/tag_path.rs:93`. Round-trips through CIP path encoding cleanly because the underlying `to_cip_path()` is unaffected.
- Same `read_bool_array_element_workaround` / `write_bool_array_element_workaround` is reused for both top-level (CODEX-X) and nested (CODEX-Y) paths. Single fix site, single test surface — DRY without becoming over-engineered.
- The new `simulated_plc_nested_bool_array_element_read_write` test (`tests/plc_sim_tests.rs:140`) covers (a) indices [0, 1, 31, 32, 33, 63] for cross-DWORD reads, (b) the specific Bool-not-Udint regression at index 0 (`matches!(..., PlcValue::Bool(_))`), and (c) same-DWORD + cross-DWORD distinct writes confirming no aliasing.
- Simulator was extended (per the brief's "may need to expand" warning) — `tests/plc_sim.rs` gained a `UDT_ARRAY[3].BOOL_NESTED[i]` tag with the right DWORD packing, and the request handler now preserves non-final array indices in canonical nested keys. Minimal, scoped change as the brief asked.

**Findings (🟡 polish, non-blocking):**
- 🟡 `detect_bool_array_path` silently returns `Ok(false)` on CIP errors (`src/client.rs:1000-1002`). That's correct for the false-positive-avoidance use case — if the parent path doesn't resolve, the complex-path branch will hit `build_read_request` which will produce its own diagnostic. But it does mean a transient connection issue during detection silently falls through to a less-precise error message at the next step. Minor.
- 🟡 The detection probe costs an extra round-trip per BOOL access (one test-read to discover data type, then the real read). Brief's "Out of scope: cost optimization" applies; future polish could cache per-`parent_path` resolution in a `HashMap<String, bool>` on the client.

**Findings (🟠 real concerns) — none.**

**Acceptance criteria tally:**
- ✅ `gTestUDT_Array[i].Array_BOOL[j]` reads return `PlcValue::Bool` correctly
- ✅ Writes set the right bit without aliasing
- ✅ `gTestUDT.Array_BOOL[j]` (single-UDT) still works — confirmed by hardware re-run's `ctrl.UDT_nested 35/35 read+`
- ✅ `gTestArray_BOOL[i]` (plain array, CODEX-X path) still works — `ctrl.BOOL_array 128/128`
- ✅ New simulator test passes
- ✅ `cargo fmt --check`, `cargo clippy -D warnings`, workspace tests, simulator tests all pass
- ✅ `CHANGELOG.md` `[Unreleased]` `### Fixed` entry present
- ✅ Hardware re-run captured (Claude ran it)

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Combined with CODEX-X, this clears every BOOL-array hardware finding from the 2026-05-24 full-coverage run. The Bool-not-Udint regression at index 0 was the most insidious symptom — calling code would see a `PlcValue::Udint(...)` and silently treat the whole DWORD as some unsigned 32-bit value with no indication anything was wrong. That class of silent type-mismatch is exactly what the new pinned `matches!(_, PlcValue::Bool(_))` assertion guards against forever.

Implementation cleanly factors the detect-and-route pattern via `parse_final_array_element_access` + `detect_bool_array_path`. No defects to fix during merge.

Combined hardware impact (CODEX-X + CODEX-Y):
- Reads: 2109/2299 → **2299/2299** (+190)
- Verify matches: 1728/1806 → **1806/1806** (+78)
- Real anomalies: 268 → **0**

(The 400 "blocked_unexpected_pass" count in the post-fix run is a test-side classification issue in `examples/test_plc_full_coverage.rs`, not a library bug — the exerciser labels `UDTarr_elem_nested` as FirmwareBlocked but those writes actually succeed; the test counter labels successful writes as unexpected. Worth a future test cleanup but doesn't affect the merge decision.)
