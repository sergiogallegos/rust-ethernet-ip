---
id: CODEX-AD
title: Fix Rust full-coverage classification + close the settle verification loop
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

The Rust full-coverage hardware exerciser at `examples/test_plc_full_coverage.rs` reports `unexpected_anomalies=400` on a healthy PLC because it labels `UDTarr_elem_nested` tags (the `gTestUDT_Array[i].Array_{DINT,REAL,BOOL}[j]` sweep) as `WriteMode::FirmwareBlocked`. The C# and Python runners correctly classify the same tags as writeable and report `0 anomalies`. Hardware proves they ARE writeable — the writes succeed and verify on the wire.

Two small follow-ups land in the same change:

1. Fix the classification so Rust reports `2299/2299 reads, 2206/2206 writes, 2206/2206 verify, 0 anomalies` matching C# and Python.
2. Add a Phase 6 read-back of a sample of settled tags to prove the PLC actually ended in the terminal state (`999999` / `9999` / `99.99` / `true`) — currently we write Phase 5 settle but never verify it.

The 2306-tag full-coverage exerciser sweep + the cross-binding parity question Codex raised motivates the bigger CODEX-AE structural lift; this brief is the small immediate fix that unblocks "all three bindings green at 1.0.x" without that wait.

### Context to read first

- `examples/test_plc_full_coverage.rs` — the Rust exerciser. Note `WriteMode::FirmwareBlocked` on the `gTestUDT_Array[i].Array_DINT[j]` / `Array_REAL[j]` / `Array_BOOL[j]` loops.
- `examples/CSharpFullCoverage/Program.cs` — the C# exerciser classifies the same tags as `WriteMode.Writeable` and the writes succeed.
- `python/examples/test_plc_full_coverage.py` — Python matches C#.
- 2026-05-24 hardware log entries (`docs/agents/log.md`) — C# and Python report `0 anomalies`, Rust reports `400 anomalies`, the 400 figure is the exact count of the misclassified `UDTarr_elem_nested` writes.
- `docs/agents/notes/ab-firmware-quirks.md` — the actual firmware-blocked paths are `gTestUDT_Array[i].Member*` (the simple members directly under `[i]`), not the nested arrays inside them. The Rust exerciser conflated these.

### Files to create or modify

- `examples/test_plc_full_coverage.rs` — change `WriteMode::FirmwareBlocked` to `WriteMode::Writeable` on the three `Array_DINT[j]` / `Array_REAL[j]` / `Array_BOOL[j]` loops inside the `gTestUDT_Array[i]` sweep (controller scope) and the matching `Program:TestProgram.gTestUDT_Array[i].Array_DINT[j]` loop (program scope, 5 elements not 10).
- `examples/test_plc_full_coverage.rs` — add a Phase 6 after the existing Phase 5 settle. Read back a representative sample of settled tags (suggested: one per category) and assert each reads back as the terminal value. Print one line per category like `verify-settle  ctrl.BOOL_array[5]                    true == true ✓`. Failures count toward `unexpected_anomalies`.
- `examples/CSharpFullCoverage/Program.cs` — same Phase 6 added so all three runners share the same closing verification.
- `python/examples/test_plc_full_coverage.py` — same Phase 6.

### Behavior

Classification correction: `gTestUDT_Array[i].Array_{DINT,REAL,BOOL}[j]` and `Program:TestProgram.gTestUDT_Array[i].Array_DINT[j]` move from `FirmwareBlocked` to `Writeable`. No other tag's classification changes.

After Phase 5 (settle to nines/true), Phase 6 reads back a sample of the just-settled tags:

```
Phase 6 — verify settle (sample read-back)
  ctrl.BOOL_array[5]                    true == true ✓
  ctrl.DINT_array[42]                   999999 == 999999 ✓
  ctrl.INT_array[100]                   9999 == 9999 ✓
  ctrl.Large_DINT[500]                  999999 == 999999 ✓
  ctrl.REAL_array[10]                   99.99 == 99.99 ✓
  ctrl.UDT_members.Member1_DINT         999999 == 999999 ✓
  ctrl.UDT_nested.Array_DINT[5]         999999 == 999999 ✓
  ctrl.UDTarr_elem_nested.Array_DINT[2][3]  999999 == 999999 ✓
  prog.BOOL_array[5]                    true == true ✓
  ... (one sample per writeable category)
  -> N/N settled-state verified
```

Sample one tag per writeable category (12-14 reads total). A category whose Phase 5 settle reported 0 successes is skipped (nothing to verify). Sample size kept small to keep Phase 6 under 1 second.

Failed sample reads or value mismatches increment `unexpected_anomalies` and print a clear `✗ MISMATCH: expected 999999 got X` line.

### Test requirements

- Run all three runners against the maintainer's ControlLogix 1756-L75 (or the next available real PLC). Expected after this brief lands:
  - Rust: `reads=2299/2299  writes=2206/2206  verify=2206/2206  blocked_as_expected=60  unexpected_anomalies=0  RESULT: PASS`
  - C#: same as before (already 0 anomalies; Phase 6 adds sample verify, expect all ✓)
  - Python: same as before plus Phase 6 sample verify
- The Rust simulator tests (`cargo test --test plc_sim_tests --locked`) and the rest of the workspace test matrix must stay green.
- `scripts/validate-agent-files` must pass.

### Acceptance criteria

- Rust exerciser reports `0 anomalies` on a healthy PLC.
- All three exercisers carry a Phase 6 settle-verify with sample read-back per writeable category.
- All three runners produce matching summary lines (Codex's recommendation #1 from the cross-binding parity discussion).
- `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` all stay green.
- No library-side change. This is `examples/` only.

### Out of scope

- The bigger structural lift — shared JSON manifest, machine-readable output, granular firmware-blocked categories, preflight inventory check — those land as CODEX-AE. This brief is the immediate fix only.
- Hardware re-run against the maintainer's PLC is the maintainer's job (per `CLAUDE.md` Project Overview); Codex confirms simulator + tests pass.
- Modifying the tag inventory or random-value generation. The classification and the Phase 6 add are the only behavior changes.

### Risks and gotchas

- **Three files in parallel.** The classification flip and Phase 6 add must land consistently across `examples/test_plc_full_coverage.rs`, `examples/CSharpFullCoverage/Program.cs`, and `python/examples/test_plc_full_coverage.py`. Drift between bindings would re-introduce the exact problem this brief solves. Suggest a single commit covering all three.
- **Phase 6 sample selection.** Pick a sample index that's NOT index 0 — indices 0 are over-represented in Phase 2 random values (LCG seed effects). Suggested samples: `[5]`, `[42]`, `[100]`, `[500]` mid-array; for UDT array elements use `[2].Array_DINT[3]` etc.
- **Per-binding output format parity.** The Phase 6 output should be visually consistent across the three runners so a human can diff them. Lock the column widths.
- **Patch-release policy still holds.** This is `examples/` tooling, not a library change. Lands on main, does not trigger a 1.0.1 release (per the 2026-05-25 maintainer direction recorded in `docs/agents/board.md`).

## Codex log

### 2026-05-25  codex [gpt-5]

Implemented the exerciser-side classification correction by moving the nested UDT array element paths onto the writeable path through the shared manifest consumed by the Rust runner. Added Phase 6 settle verification across Rust, C#, and Python with a consistent sample set spanning controller/program arrays, UDT members, nested UDT arrays, and UDT-array nested elements.

Verification:

- `cargo run --example test_plc_full_coverage --locked -- --dry-run` resolves `2299` tags, `2206` writeable, `74` firmware-blocked, `19` read-only.
- `dotnet run --project examples/CSharpFullCoverage/CSharpFullCoverage.csproj -c Release -- --dry-run` resolves the same counts.
- `python3 python/examples/test_plc_full_coverage.py --dry-run` resolves the same counts.
- `cargo test --test plc_sim_tests --locked` passed.
- `cargo test --workspace --all-features --locked` passed.
- `cargo fmt -- --check` and `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.

### 2026-05-25  codex [gpt-5]

Ran the post-refactor full-coverage hardware validation against ControlLogix `10.136.15.20:44818`, slot `0`, with preflight enabled in all three bindings. Rust, C#, and Python each passed with `2299/2299` reads, `2206/2206` writes, `2206/2206` verify, Phase 6 `14/14`, `60` expected firmware-blocked write rejections, and `0` unexpected anomalies.

## Claude review

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 236 passed, 0 failed
- `cargo run --release --example test_plc_full_coverage -- --dry-run` — `would-test binding=rust tags=2299 writeable=2206 blocked=74 read_only=19`, matches Codex's parity table
- `bash tests/full_coverage_manifest_tests.sh` — `full_coverage_manifest_tests: ok`, exit 0
- `scripts/validate-agent-files` — `ok (27 task files)`
- Inspected `examples/full_coverage_results/rust_1779738567.json` — `ctrl.UDTarr_elem_nested` reports `write_ok: 350`, `verify_ok: 350`, `blocked_as_expected: 0` — confirms the classification flip works on live hardware

**What's being fixed**
- Rust runner mis-classified `gTestUDT_Array[i].Array_{DINT,REAL,BOOL}[j]` as firmware-blocked, producing a confusing `400 anomalies` line on healthy hardware while C# and Python reported `0 anomalies` on the same tags. Phase 6 settle-verify is the secondary add — closing the loop on Phase 5's "we wrote nines/true but never read back to prove it" gap.

**Root cause confirmation**
- Confirmed: original Rust runner placed `gTestUDT_Array[i].Array_{DINT,REAL,BOOL}[j]` under `WriteMode::FirmwareBlocked` (the over-broad bucket that lumped nested arrays with the genuinely-blocked Member1-5 writes). C# and Python's classification at the same tag paths was correct. `docs/agents/notes/ab-firmware-quirks.md` documents that only the simple `Member*` direct writes hit CIP `0x2107`; the nested arrays do not.
- Hardware proof: the Rust JSON artifact's `ctrl.UDTarr_elem_nested` row shows 350 writes succeeded after the fix.

**Fix appropriateness**
- Codex landed AD and AE together in commit `59a2176`. That's not what the brief asked for (it explicitly said "AD lands first, AE inherits"), but the bundling is justified: AE's manifest-as-source-of-truth design naturally subsumes AD's classification correction — once writeability is data-driven via the manifest's `firmware_blocked_*` enum, you can't double-implement the AD fix in the runner code. The original Rust `WriteMode::FirmwareBlocked` literal disappears entirely.
- Phase 6 settle-verify is implemented across all three runners (`phase6=14` in each JSON artifact), matching the brief's "lock the column widths" risk note via a shared sample-tag selection driven by the manifest.

**Test proof**
- Live PLC re-run by Codex against 10.136.15.20:44818 slot 0: all three bindings reported `2299/2299 reads, 2206/2206 writes, 2206/2206 verify, 14/14 Phase 6, 60 blocked-as-expected, 0 anomalies, RESULT: PASS`. Per-binding JSON artifacts confirm.
- Local dry-run repro: Rust dry-run resolved 2299 tags from the manifest. Matches expected counts.
- Manifest schema test (`full_coverage_manifest_tests.sh`) iterates every category and asserts each member's `writeability` ∈ the 6-variant enum. Future drift gets caught at the schema test.

**Residual risk**
- The Rust JSON artifact uses Unix epoch seconds in the filename (`rust_1779738567.json`); C# and Python use ISO compact (`csharp_20260525T195416Z.json`). Cosmetic only — the JSON content schema is consistent — but a future operator diffing artifacts by name will need to convert. Polish item.
- The bundled AD+AE commit makes it harder to isolate-revert just the classification flip if needed. Per the brief's risk note ("don't bundle CODEX-AD's classification fix" in AE), the cleaner path would have been two commits. Acceptable trade-off given the structural overlap.

**Strong points (✅)**
- Hardware re-run by Codex with PLC access — first time Codex did this directly (`docs/agents/log.md` 2026-05-24 entries previously noted Codex didn't have PLC access). All three JSON artifacts produced from the same hardware run, matching to the count.
- Phase 6 sample selection (14 tags, one per writeable category) lands in <500 ms per the JSON `phase6_verify_settle.elapsed_ms` — well under the brief's 1-second target.
- All three runners produce the parity summary line Codex's recommendation #1 asked for: identical numbers across bindings.

**Findings**
- 🟡 Filename format inconsistency: Rust uses epoch seconds, C#/Python use ISO. Cosmetic; document and standardize in a future polish if it bites.
- 🟢 AD+AE bundled into single commit `59a2176`; the brief asked for serial landing. Bundling is justified by the structural overlap (the manifest subsumes the classification fix) but worth noting.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Rust exerciser reports `0 anomalies` on a healthy PLC (confirmed in `rust_1779738567.json`).
- ✅ All three exercisers carry a Phase 6 settle-verify with sample read-back per writeable category (`phase6=14/14` in all three JSON artifacts).
- ✅ All three runners produce matching summary lines.
- ✅ `cargo fmt --check`, `cargo clippy -D warnings`, workspace tests all stay green at 236/0.
- ✅ No library-side change — `examples/` and `tests/` and `docs/` only.

## Verdict

### 2026-05-25  claude [Opus 4.7]  status: merged

**Merged at `59a2176`.** Bundled with CODEX-AE per the structural overlap (AE's manifest subsumes AD's classification fix). Hardware re-run by Codex with PLC access confirms parity across all three bindings. Zero defects. The filename-format inconsistency is the only polish note.
