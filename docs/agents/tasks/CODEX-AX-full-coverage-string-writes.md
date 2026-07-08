---
id: CODEX-AX
title: Full-coverage harness never writes or blocked-probes STRINGs — release gate can't catch a STRING regression
owner: codex
status: open
created: 2026-07-08
last-update: 2026-07-08 claude [Opus 4.8]
---

> **2026-07-08 update:** CODEX-AY (handle-aware STRING writes) is **merged**, so `write_tag` /
> `write_string` / `eip_write_string` now write built-in *and* custom string members, and
> `read_string_tag` / `eip_read_string` read them back. This task is unblocked. Because the fix
> makes STRING members **writeable**, this task now *writes+verifies* the standard string entries
> (standalone + Member5) rather than blocked-probing them — most `encoding_blocked_udt_string_member`
> entries become `writeable` (blocked count drops toward 0; keep the blocked-probe only for any
> genuinely over-one-packet case). A committed `examples/test_plc_strings.rs` already covers the
> string surface as a standalone example and is the reference for the per-runner change. Adding
> custom `Member6`/`Member7` to the *shared* manifest is optional and cross-PLC-sensitive (they
> exist only on the extended test program) — prefer keeping them in the dedicated string example.

## Brief

### Goal

The full-coverage exercisers skip every STRING tag in the write and blocked-probe phases, so the
pre-release hardware gate does not exercise the CODEX-AT STRING-write fix at all. A STRING-write
regression would pass the gate as `RESULT=PASS`. Close the gap so the release gate proves the
STRING surface.

Root cause (`examples/test_plc_full_coverage.rs`): `rand_value()` (`:303`) and `nines()`
(`:313`) both `return None` for `Kind::String` (and `Kind::Udt`). Phase 2 (write, `:497`) and
Phase 4 (blocked-probe, `:531`) both `continue` on `None`. Consequence on 5069-L330ERM fw38
(2026-07-08): the manifest labels **2268 writeable / 17 expected-blocked / 19 read-only**, but
every runner reports `writes=2266` (the 2 standalone STRINGs skipped) and
`blocked_as_expected=0` (the 17 `encoding_blocked_udt_string_member` tags never probed). The
C#/Python runners mirror the same skip.

A correct reference implementation already exists and is hardware-validated: the new
`examples/cpp/full_coverage.cpp` (this session) writes+verifies the 2 standalone STRINGs and
blocked-probes the 17 UDT STRING members, reporting the full manifest intent
(`2304 read / 2268 write / 2268 verify / 17 blocked / 0 anomalies`). See
[`docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`](../../validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md)
finding 2, and [`docs/agents/notes/release-hardware-validation.md`](../notes/release-hardware-validation.md).

### Context to read first

- `examples/test_plc_full_coverage.rs` — `rand_value`/`nines` (`:303`/`:313`), Phase 2 (`:491`),
  Phase 3 verify (`:513`, `values_match`), Phase 4 (`:525`), Phase 5 settle (`:549`). The
  `Kind::String` short-circuits are the whole bug.
- `examples/cpp/full_coverage.cpp` — the corrected reference: how it generates a probe string,
  verifies read-back, blocked-probes members, and settles STRINGs to a terminal value.
- `examples/CSharpFullCoverage/Program.cs` and `python/examples/test_plc_full_coverage.py` — the
  mirror runners that need the same change (each has its own value generator with the same skip).
- [`docs/agents/notes/ab-firmware-quirks.md`](../notes/ab-firmware-quirks.md) — the contract:
  standalone STRING writes succeed; UDT-member STRINGs (`Member5_String`) must still reject
  `0x2107`. The blocked-probe asserts the rejection; a member that *accepts* a write is the
  anomaly to flag, not the reverse.

### Files to create or modify

`examples/test_plc_full_coverage.rs`, `examples/CSharpFullCoverage/Program.cs`,
`python/examples/test_plc_full_coverage.py`. No library `src/` changes — this is test-harness
correctness.

### Behavior

- Phase 2 writes standalone STRING tags (`Kind::String`, `writeable`) with a distinct probe
  value per tag; Phase 3 verifies the read-back equals it. Phase 5 settles them to a terminal
  string. STRING values compare by exact string equality.
- Phase 4 blocked-probe attempts a STRING write to each `encoding_blocked_udt_string_member` tag
  and counts a rejection as `blocked_as_expected`; a success is `blocked_unexpected_pass` (an
  anomaly). This requires distinguishing member-STRING (blocked) from standalone-STRING
  (writeable) — the manifest `writeability` already encodes it; drive off that, not off `Kind`.
- `Kind::Udt` stays skipped for direct writes (whole-UDT tags remain `read_only`; that is correct
  — do not start writing them).
- After the change, a clean run on validated hardware reports `writes=2268`,
  `blocked_as_expected=17`, `anomalies=0`, byte-identical across Rust/C#/Python and matching the
  C++ runner.

### Test requirements

- `--dry-run` on each runner still reports `writeable=2268 blocked=17 read-only=19` (unchanged —
  the manifest is already correct; only execution changes).
- No unit-test surface here (these are example binaries), but a dry-run assertion and a manual
  note suffice. Keep the untouched-code matrix green: `cargo fmt -- --check`,
  `cargo clippy -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`.
- Sanity-check the three runners still agree on counts under `--dry-run` and that the JSON result
  schema is unchanged (the `full_coverage_manifest_tests.sh` dry-run cross-binding check must
  still pass).
- Fix the incidental Windows-portability bug while here: the Python runner prints `✓`/`✗` and
  crashes under redirected stdout on cp1252 (`UnicodeEncodeError`); force UTF-8 or use ASCII
  markers (finding 3 in the validation record).

### Acceptance criteria

- All three runners write+verify the 2 standalone STRINGs and blocked-probe the 17 UDT STRING
  members; a clean hardware run reports `2304 / 2268 / 2268 / 17 / 0` on all four bindings.
- The Python runner no longer crashes on Windows under redirected stdout.
- `full_coverage_manifest_tests.sh` and the untouched-code matrix stay green.

### Out of scope

- Changing the manifest (it is already correct at 2268/17/19).
- Whole-UDT or UDT-member STRING *write* support (that is CODEX-AO wire-format territory; the
  blocked-probe here only asserts the current rejection).
- Restoring original STRING values instead of settling to terminal — match the existing
  settle-to-terminal convention the scalar phases already use.

### Risks / gotchas

- Drive writeable-vs-blocked off the manifest `writeability`, not `Kind::String` — both
  standalone and member STRINGs are `Kind::String`; only the label distinguishes them.
- STRING verify must be exact-equality; do not reuse a numeric tolerance path.
- Keep per-run STRING probe values ≤ the 82-char Logix STRING capacity.

## Codex log

## Claude review

## Verdict
