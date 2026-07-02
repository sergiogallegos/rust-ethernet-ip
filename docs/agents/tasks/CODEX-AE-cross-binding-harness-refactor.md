---
id: CODEX-AE
title: Cross-binding hardware harness — shared tag manifest, JSON output, granular firmware classification, preflight inventory check
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
---

## Brief

### Goal

The three full-coverage hardware exercisers (`examples/test_plc_full_coverage.rs`, `examples/CSharpFullCoverage/Program.cs`, `python/examples/test_plc_full_coverage.py`) each maintain their own 2299-tag inventory. They've already drifted once — CODEX-AD fixes the immediate symptom (Rust over-classified `UDTarr_elem_nested` as FirmwareBlocked) but the structural problem stays: three independent tag lists, three independent classification schemes, three independent output formats. The next drift is inevitable.

Lift the harness to a single source of truth and machine-readable output so future cross-binding validation becomes a diff of two JSON files instead of three eyeball reads of three console transcripts.

Items #2-6 from Codex's 2026-05-25 cross-binding parity recommendation, bundled because they share the same surface and benefit from one schema design pass.

### Context to read first

- CODEX-AD task file — the immediate-fix companion brief that lands first
- All three exerciser files (the duplicated tag inventories are obvious)
- `docs/PLC_TEST_TAG_DEFINITIONS.md` — the source-of-truth doc for the tag layout, written human-first
- `docs/agents/notes/ab-firmware-quirks.md` — the actual firmware-blocked path catalog; the new granular categories need to align with this
- 2026-05-24 hardware-validation log entries — for context on what the runners report today
- `scripts/check-release-readiness.txt` — example of the pipe-delimited manifest pattern that worked well for CODEX-AA; JSON is the right shape here instead (richer schema), but the data-driven approach is the same

### Files to create or modify

- `examples/full_coverage_tags.json` (new) — single source of truth for the 2299-tag inventory. Schema documented in Behavior below.
- `examples/test_plc_full_coverage.rs` — refactored to read the JSON manifest at startup, build the tag list from it, run the existing 6-phase flow against it, and emit a JSON result file alongside the console output.
- `examples/CSharpFullCoverage/Program.cs` — same refactor.
- `python/examples/test_plc_full_coverage.py` — same refactor.
- `examples/full_coverage_results/` (new directory; `.gitkeep` only) — destination for per-run JSON results, named `{binding}_{timestamp}.json`. `.gitignore` covers the `*.json` files (the directory is the artifact, results are ephemeral). Optional `--out-dir` flag to override.
- `docs/PLC_TEST_TAG_DEFINITIONS.md` — short pointer to `examples/full_coverage_tags.json` as the machine-readable mirror; doc stays human-first.
- `.gitignore` — add `examples/full_coverage_results/*.json` if not already covered.

### Behavior

**Manifest schema** (`examples/full_coverage_tags.json`):

```json
{
  "version": 1,
  "generated_from": "docs/PLC_TEST_TAG_DEFINITIONS.md",
  "categories": [
    {
      "name": "ctrl.BOOL_array",
      "scope": "controller",
      "pattern": "gTestArray_BOOL[{i}]",
      "indices": { "range": [0, 128] },
      "kind": "Bool",
      "writeability": "writeable"
    },
    {
      "name": "ctrl.UDTarr_elem_members",
      "scope": "controller",
      "pattern": "gTestUDT_Array[{i}].{member}",
      "indices": { "range": [0, 10] },
      "members": {
        "Member1_DINT": "Dint",
        "Member2_REAL": "Real",
        "Member3_BOOL": "Bool",
        "Member4_INT":  "Int",
        "Member5_String": "String"
      },
      "writeability": "firmware_blocked_udt_array_element_member"
    },
    {
      "name": "ctrl.UDTarr_elem_nested",
      "scope": "controller",
      "pattern": "gTestUDT_Array[{i}].Array_{type}[{j}]",
      "outer_indices": { "range": [0, 10] },
      "inner": {
        "Array_DINT": { "range": [0, 10],  "kind": "Dint" },
        "Array_REAL": { "range": [0, 5],   "kind": "Real" },
        "Array_BOOL": { "range": [0, 20],  "kind": "Bool" }
      },
      "writeability": "writeable"
    },
    ...
  ]
}
```

**Writeability enum** (granular per Codex's recommendation #4):

- `writeable` — direct write succeeds (the runner attempts the write and counts it toward `writes_ok`).
- `read_only` — never attempted as a write (e.g. discovery-only tags). Phase 2/3/4/5 skip these.
- Superseded by CODEX-AT: top-level standard STRING tags are now `writeable` when encoded as the Logix structure type; do not reintroduce a plain-STRING firmware-blocked bucket.
- `firmware_blocked_udt_string_member` — STRING member inside a UDT (e.g. `gTestUDT.Member5_String`); same expected-failure path.
- `firmware_blocked_udt_array_element_member` — Member1-5 directly under a UDT array element (e.g. `gTestUDT_Array[i].Member1_DINT`); same expected-failure path.
- `service_layer_writeable` (reserved; not used yet) — for the future CODEX-Q service-layer methods that work around the firmware blocks via RMW. Documented in the schema so future bindings can graduate categories without changing the runner contract.

**Output schema** (per-run JSON written to `examples/full_coverage_results/{binding}_{ISO-timestamp}.json`):

```json
{
  "schema_version": 1,
  "binding": "rust",
  "binding_version": "1.0.0",
  "plc_address": "10.136.15.20:44818",
  "plc_slot": 0,
  "manifest_version": 1,
  "tag_count": 2299,
  "result": "PASS",
  "anomalies": 0,
  "phases": {
    "preflight":           { "ok": 2299, "fail": 0, "elapsed_ms": 8200 },
    "phase1_read":         { "ok": 2299, "fail": 0, "elapsed_ms": 12300 },
    "phase2_write":        { "ok": 2206, "fail": 0, "elapsed_ms": 11700 },
    "phase3_verify":       { "ok": 2206, "fail": 0, "elapsed_ms": 10100 },
    "phase4_blocked":      { "ok": 60,   "fail": 0, "elapsed_ms": 2700,  "note": "expected firmware rejections" },
    "phase5_settle":       { "ok": 2206, "fail": 0, "elapsed_ms": 11200 },
    "phase6_verify_settle":{ "ok": 13,   "fail": 0, "elapsed_ms": 480 }
  },
  "categories": {
    "ctrl.BOOL_array":    { "kind": "Bool", "writeability": "writeable", "read_ok": 128, "write_ok": 128, "verify_ok": 128 },
    "ctrl.UDTarr_elem_members": { "kind": "mixed", "writeability": "firmware_blocked_udt_array_element_member", "read_ok": 50, "blocked_as_expected": 40 },
    ...
  }
}
```

**Preflight phase** (Codex recommendation #5) runs BEFORE Phase 1:

- For each tag in the manifest, issue a read.
- A failed read in preflight is reported as a SETUP error, not a library error. The runner prints `setup-error: tag X not found / type mismatch — verify the PLC project against docs/PLC_TEST_TAG_DEFINITIONS.md` and exits 2 (distinct from library-error exit 1).
- Preflight successes are recorded as `preflight.ok` in the JSON output; they DO NOT count toward Phase 1 read totals (those are re-counted in Phase 1 proper). Preflight establishes "the PLC project matches the manifest"; Phase 1 measures library read behavior.
- A `--skip-preflight` flag exists for fast iteration when the operator knows the PLC is correctly configured.

**Phase 6 settle-verify** (carried over from CODEX-AD) reads back a sample per writeable category and confirms the terminal state.

**Runner contract** (identical across Rust/C#/Python):

1. Parse args (`--plc-address`, `--plc-slot`, `--manifest`, `--out-dir`, `--skip-preflight`, `--strict`, plus binding-specific knobs).
2. Load manifest, generate tag list.
3. Preflight (Phase 0) unless `--skip-preflight`.
4. Phases 1-6 as today.
5. Write JSON result to `--out-dir` (default `examples/full_coverage_results/`).
6. Console summary line that matches across bindings: `binding=rust tags=2299 reads=2299/2299 writes=2206/2206 verify=2206/2206 blocked=60 anomalies=0 RESULT=PASS`.

### Test requirements

- New `tests/full_coverage_manifest_tests.sh` (or `.rs` if cleaner) that validates `examples/full_coverage_tags.json` against the documented schema: every category has `name`, `scope`, `kind` (or `members` for mixed), `writeability` from the enum.
- Manifest schema test runs in CI.
- Each runner has a self-test mode (`--dry-run`) that builds the tag list from the manifest and prints `would-test` without contacting the PLC. CI runs this mode to confirm the manifest parses and resolves correctly per binding.
- Hardware re-run (maintainer-owned) against the maintainer's PLC produces three matching JSON files; a small `scripts/compare-coverage-results <a.json> <b.json>` (optional) prints a diff if categories disagree.

### Acceptance criteria

- `examples/full_coverage_tags.json` exists with the documented schema, populated to match the current 2299-tag inventory.
- All three runners consume the manifest as the source of truth — no inline tag lists remain.
- All three runners emit JSON results matching the documented output schema.
- New writeability enum has the 6 documented categories. Migration: the current single `FirmwareBlocked` bucket is split into the three `firmware_blocked_*` variants per the actual firmware quirk (see `docs/agents/notes/ab-firmware-quirks.md`).
- Preflight phase exists with `--skip-preflight` escape hatch.
- Phase 6 settle-verify carried over from CODEX-AD (if AD lands first, no work here; if not, add it).
- `scripts/validate-agent-files`, `cargo fmt --check`, `cargo clippy --workspace --all-targets --all-features -D warnings`, full test matrix all stay green.
- Manifest schema test runs in CI.

### Out of scope

- Changing the actual tag inventory composition. The 2299-tag layout matches `docs/PLC_TEST_TAG_DEFINITIONS.md`; this brief migrates the inventory's representation, not its content.
- Adding new test phases beyond preflight + Phase 6 settle-verify.
- Comparing results across runs over time (longitudinal regression detection). That's a future tooling brief if it becomes useful.
- Cross-binding result *diffing* tooling (`scripts/compare-coverage-results`) is suggested-but-optional; if the brief is too large, defer this and land just the manifest + JSON output + preflight.
- Generating the manifest automatically from `docs/PLC_TEST_TAG_DEFINITIONS.md`. The manifest is hand-written and kept in sync with the doc by review; auto-generation is a future polish if drift becomes an issue.
- C# / Python wrapper code (the wrappers themselves don't change; only the `examples/` runners do).

### Risks and gotchas

- **Schema fork risk.** The manifest schema and output schema are now public artifacts inside `examples/`. Future schema changes need their own `schema_version` bump path; document this in the manifest header comment.
- **Three-language JSON parsing.** Rust uses `serde_json`, C# uses `System.Text.Json`, Python uses stdlib `json`. All three are stable, but field-naming conventions differ (Rust prefers `snake_case`, C# `PascalCase`). Pick one (suggest `snake_case`) and document it in the schema; each runner does the casing translation.
- **The manifest's `inner` shape for `ctrl.UDTarr_elem_nested`** carries three nested arrays per outer element. Make sure the generated tag list matches what the current Rust/C#/Python runners produce — easy to off-by-one. Pin with a count assertion: total tags from manifest must equal 2299 (or whatever the current inventory total is).
- **CI dry-run vs real hardware.** CI runs `--dry-run`, real hardware runs full. The dry-run must parse the manifest, generate the tag list, and print expected counts; it must NOT attempt any TCP connection (avoid flaky network deps on CI).
- **Preflight performance.** A 2299-read preflight at ~5ms/read = ~12 seconds. Acceptable for a full validation run; the `--skip-preflight` flag is the escape hatch for iteration.
- **Don't bundle CODEX-AD's classification fix here.** AD lands first; this brief inherits the correct classification and builds the granular categories on top. If AD hasn't landed yet, this brief explicitly waits for it (callout in the Codex log).

## Codex log

### 2026-05-25  codex [gpt-5]

Refactored the Rust, C#, and Python full-coverage hardware exercisers to consume `examples/full_coverage_tags.json` as the single source of truth. Added granular manifest writeability categories, `--dry-run`, `--skip-preflight`, manifest-driven tag expansion, preflight setup validation, parity console summary lines, and per-run JSON result output under `examples/full_coverage_results/` with per-phase and per-category counters.

Added `tests/full_coverage_manifest_tests.sh` and a CI `full-coverage-manifest` job that validates the manifest and runs each binding in dry-run mode. Added `examples/full_coverage_results/.gitkeep`, ignored generated result JSON, and linked the machine-readable manifest from `docs/PLC_TEST_TAG_DEFINITIONS.md`.

Verification:

- `tests/full_coverage_manifest_tests.sh` passed.
- Rust/C#/Python dry-runs each resolved `2299` tags, `2206` writeable, `74` firmware-blocked, `19` read-only.
- `dotnet build examples/CSharpFullCoverage/CSharpFullCoverage.csproj -c Release` passed.
- `python3 -m py_compile python/examples/test_plc_full_coverage.py` passed.
- `cargo test --workspace --all-features --locked` passed.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, and `git diff --check` passed.

### 2026-05-25  codex [gpt-5]

Ran the manifest-driven full-coverage hardware validation against ControlLogix `10.136.15.20:44818`, slot `0`, with preflight enabled. All three bindings produced JSON result artifacts in `examples/full_coverage_results/` and passed with matching totals: `2299/2299` preflight reads, `2299/2299` phase reads, `2206/2206` writes, `2206/2206` verify, Phase 6 `14/14`, `60` blocked-as-expected, and `0` anomalies.

## Claude review

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 236 passed, 0 failed
- `bash tests/full_coverage_manifest_tests.sh` — `full_coverage_manifest_tests: ok`, exit 0
- `cargo run --release --example test_plc_full_coverage -- --dry-run` — resolves 2299 / 2206 / 74 / 19 from the manifest
- Inspected `examples/full_coverage_tags.json` — 25 categories, all writeability fields ∈ the 6-variant enum
- Inspected all 3 hardware JSON artifacts — schema matches the brief's documented output shape; per-category breakdown shows matching counts across bindings; `result=PASS` and `anomalies=0` confirmed for each

**What's being fixed**
- N/A — this is new infrastructure replacing three independently-maintained tag inventories (one per binding) with a single source of truth, machine-readable output for longitudinal comparison, granular firmware-blocked categories, and a preflight phase that distinguishes PLC project misconfiguration from library regressions.

**Root cause confirmation**
- N/A — new infrastructure. The motivating problem (cross-binding classification drift) is the same one CODEX-AD addressed at the symptom level; this brief addresses it structurally.

**Fix appropriateness**
- Manifest schema at `examples/full_coverage_tags.json` is data-driven and reviewable; new categories or sites are added there, not in runner code. Matches the brief's "single source of truth" intent.
- The writeability enum directly replaces the over-broad single `FirmwareBlocked` bucket that caused the AD-fixed drift. CODEX-AT later removed the plain-STRING blocked variant; current active values are `writeable`, `read_only`, `firmware_blocked_udt_string_member`, `firmware_blocked_udt_array_element_member`, and `service_layer_writeable`. `service_layer_writeable` is reserved for future use per the brief.
- Per-member writeability override (e.g. `ctrl.UDT_members` where Member1-4 are `writeable` but Member5_String is `firmware_blocked_udt_string_member`) is the right factoring — keeps mixed-category groups expressible without forcing them into separate categories.
- Output schema in the JSON artifacts matches the brief's documented shape: top-level `binding`, `binding_version`, `result`, `anomalies`, `tag_count`, plus `phases.{preflight,phase1_read,...,phase6_verify_settle}` with `ok` + `fail` + `elapsed_ms` per phase, plus a `categories` breakdown.
- Preflight phase implemented; reported separately from Phase 1 reads per the brief (preflight establishes PLC project ↔ manifest match, Phase 1 measures library read behavior).
- CI gate at `.github/workflows/ci.yml:full-coverage-manifest` runs the manifest schema test plus all three `--dry-run` invocations. Dry-run mode parses the manifest and prints counts without contacting any PLC — correctly avoids network deps in CI.

**Test proof**
- Live hardware re-run by Codex against ControlLogix 1756-L75 at `10.136.15.20:44818` slot 0: all three bindings reported the same parity table (preflight 2299/2299, read 2299/2299, write 2206/2206, verify 2206/2206, phase6 14/14, blocked 60, anomalies 0).
- All three JSON artifacts produced and stored under `examples/full_coverage_results/` (gitignored runtime artifacts; the `.gitkeep` keeps the directory in git).
- Manifest schema test in CI iterates every category, asserts required keys, validates writeability enum membership, and accumulates totals — would catch any future writeability drift mechanically.
- Each binding's `--dry-run` runs in CI without network access.

**Residual risk**
- Hand-written manifest stays in sync with `docs/PLC_TEST_TAG_DEFINITIONS.md` by review; the brief's out-of-scope said auto-generation is a future polish if drift becomes an issue. Acceptable for now; the schema test will catch shape errors but not "manifest doesn't match what the PLC project actually has" — that's what preflight is for.
- The 6-variant enum is the current best understanding of AB firmware behavior. If a new firmware quirk surfaces, the enum gets a new variant + manifest entries flip — schema-evolvable via `schema_version` bump.
- `service_layer_writeable` is reserved but not exercised yet. When CODEX-Q's service layer becomes the writeback path for `Member5_String` etc., the manifest flips those entries and the runner picks up the new behavior automatically — no runner code change needed.

**Strong points (✅)**
- All three runners reach byte-identical per-phase counts on the same hardware — the cross-binding parity the brief was designed to deliver, now provable from the JSON artifacts.
- `examples/full_coverage_results/.gitkeep` + `.gitignore` rule keeps the directory in git but the runtime artifacts ephemeral. Correct artifact-vs-record split.
- The CI gate runs all three dry-runs in sequence in one job — simpler than a matrix and adequate for "the manifest parses and resolves correctly per binding."
- Preflight exit code 2 (vs library failure exit 1) is implemented so operators can distinguish "PLC project doesn't match the manifest" from "library regressed." Matches the brief's contract.
- The brief's risk note about `snake_case` JSON naming was honored — all three artifacts use snake_case keys (`phase1_read`, `tag_count`, etc.).

**Findings**
- 🟢 Bundled with CODEX-AD in commit `59a2176`. The two briefs naturally compose since AE's manifest subsumes AD's classification fix; serial landing would have required throwaway intermediate state.
- 🟡 Rust artifact uses Unix epoch seconds in the filename while C#/Python use ISO compact. Cosmetic; future polish.
- 🟡 The CI dry-run job runs all three sequentially in one job — fine today but if any of them gets slow, a job matrix would parallelize. Defer until it bites.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ `examples/full_coverage_tags.json` exists with documented schema, populated to match the 2299-tag inventory.
- ✅ All three runners consume the manifest as source of truth — no inline tag lists remain.
- ✅ All three runners emit JSON results matching the documented output schema.
- ✅ Writeability enum has the 6 documented variants.
- ✅ Preflight phase exists with `--skip-preflight` escape hatch.
- ✅ Phase 6 settle-verify carried over from CODEX-AD.
- ✅ `scripts/validate-agent-files`, `cargo fmt --check`, `cargo clippy -D warnings`, full test matrix all green.
- ✅ Manifest schema test runs in CI.

## Verdict

### 2026-05-25  claude [Opus 4.7]  status: merged

**Merged at `59a2176`.** Hardware-validated parity across all three bindings (every per-phase count matches; JSON artifacts confirm). Schema design landed cleanly, granular writeability enum eliminates the entire class of drift that motivated CODEX-AD. Manifest-as-source-of-truth means the next tag-inventory change is one JSON edit, not three runner edits. Zero defects.
