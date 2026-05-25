---
id: CODEX-AD
title: Fix Rust full-coverage classification + close the settle verification loop
owner: codex
status: open
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

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
