---
id: CODEX-AV
title: Re-validate the firmware_blocked_* write labels — the 0x2107 lore is falling to correct paths
owner: codex
status: in-progress
created: 2026-07-03
last-update: 2026-07-03 claude [Fable 5]
---

## Brief

### Goal

Two hardware sessions on 2026-07-02 (both on the 5069-L330ERM fw38) overturned two "firmware blocks this write" beliefs in one day, and both had the same root cause — the historical CIP `0x2107` evidence was collected through requests the library built incorrectly:

- Standalone STRING writes work with the correct structure encoding ([`docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md`](../../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md), fixed by CODEX-AT).
- **UDT array element member writes work with correct member paths** — `write_tag("gTestUDT_Array[0].Member1_DINT", Dint(777))` succeeded outright after CODEX-AM fixed the member-suffix drop ([`docs/validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md`](../../validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md)).

This leaves `examples/full_coverage_tags.json` carrying stale `firmware_blocked_udt_array_element_member`, `firmware_blocked_udt_string_member`, and related labels. After AM, those writes may now **succeed**, so the pre-1.2.0 full-coverage hardware run — whose gate is zero unexpected anomalies — will trip on its own stale expectations. This task makes the labels evidence-based again before that run.

Work items:

1. **Systematic hardware matrix (maintainer-executed, Codex-prepared).** Extend or script a probe that attempts, on real hardware, one representative write per currently-blocked class: UDT member (each scalar type present in `gTestUDT`), UDT STRING member, UDT array element member (scalar + STRING), program-scoped variants of each. Record per-class: success / CIP error code / read-back verification / element-integrity check (sibling member untouched). The probe must restore every value it changes and must be runnable by the maintainer with one command.
2. **Relabel from evidence.** Update `examples/full_coverage_tags.json` writeability labels to match the matrix results — nothing extrapolated: a class flips to writeable only on a green probe result; anything unprobed or failing keeps a blocked label with the observed error code noted in the manifest comment/docs. Update the three full-coverage runners' expectations and the pinned counts in `tests/full_coverage_manifest_tests.sh` in the same change.
3. **Rewrite the quirks note honestly.** `docs/agents/notes/ab-firmware-quirks.md`'s "UDT array element member writes" section (and the `symbol_id` section if the matrix touches it) gets the same treatment the STRING section got in CODEX-AT: state what was actually wrong, what is now known, on which hardware, with links. Update the Known Limitations block in `src/client.rs` and `src/lib.rs`, and sweep C#/Python doc claims (grep for `0x2107`, "firmware limitation", "cannot write").
4. **Service-layer consequences.** `write_udt_member` / `write_udt_array_member` (`src/client/service_layer.rs`) currently do read-modify-write of the whole UDT because direct member writes were "impossible". If the matrix proves direct member writes reliable, prefer the direct write with RMW as fallback on `0x2107` — but only flip the default with hardware evidence for every scalar type the service layer routes, and keep the RMW path for STRING members unless proven. Document the decision either way.

### Context to read first

- Both validation docs above — they are the evidence contract and the misdiagnosis pattern (paths were malformed; `0x2107` = data-type mismatch, not prohibition).
- `docs/agents/notes/ab-firmware-quirks.md` — the current claims under review.
- `examples/full_coverage_tags.json` + the three runners + `tests/full_coverage_manifest_tests.sh` (pinned counts: currently 2299/2208/72/19 after the CODEX-AT relabel — this task moves those numbers again).
- `docs/agents/tasks/CODEX-AO-udt-wire-format-investigation.md` — AO phase 2's capture checklist overlaps; if the maintainer captures packets during this task's matrix run, both tasks are served by one bench session.
- CODEX-AT's review (manifest-count CI fallout at `7676137`) — update the pinned counts in the same commit as the manifest this time.

### Files to create or modify

A probe example or extension of the full-coverage runner (Codex's choice — prefer reusing the manifest/runner machinery over a bespoke example), `examples/full_coverage_tags.json`, the three runners, `tests/full_coverage_manifest_tests.sh`, `docs/agents/notes/ab-firmware-quirks.md`, `src/client.rs` / `src/lib.rs` Known Limitations, `src/client/service_layer.rs` (item 4, evidence-gated), C#/Python doc sweeps, `CHANGELOG.md`, a new `docs/validation/` evidence doc for the matrix run.

### Behavior

- Every writeability label in the manifest traces to a recorded hardware result (or an explicit "unprobed, kept blocked" note).
- The pre-1.2.0 full-coverage run passes with zero unexpected anomalies because expectations match reality.
- No silent behavior flips: service-layer routing changes only where the matrix proves the direct path.

### Test requirements

- Sim coverage for any newly-writeable path the sim can express (the sim's UDT support is thin — extend minimally or note the gap per CODEX-AN's oracle rule; do not fake success shapes the hardware hasn't shown).
- `tests/full_coverage_manifest_tests.sh` green with the new counts.
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `plc_sim_tests`, C# + Python suites.

### Acceptance criteria

- Hardware matrix executed and recorded in `docs/validation/` (maintainer runs the bench; Codex ships the probe + interprets results).
- Manifest, runners, pinned counts, quirks note, and library docs all agree with the recorded evidence; grep finds no unqualified "firmware blocks X" claim that the matrix disproved.
- Item 4 decision documented with evidence either way.
- Wire-affecting expectation changes flagged for the release-gate full-coverage run (which this task exists to un-break).

### Out of scope

- General UDT wire-format work and struct-handle decoding — CODEX-AO.
- Custom STRINGnn handles — noted follow-up from CODEX-AT.
- Retiring graveyard APIs — CODEX-AP (though its item-5 evidence now includes both validation docs).

### Risks and gotchas

- **Single-controller evidence so far.** The L330ERM fw38 results may not hold on older firmware (the full-coverage history includes an L18ER fw33 and a 1756-L81ES fw37). If the maintainer can bench a second controller, per-controller labels beat global ones; otherwise record the firmware scope explicitly in the manifest docs.
- The matrix mutates UDT members — restore discipline is non-negotiable, and the probe must read-verify sibling members to catch partial-element corruption (the failure mode AM just eliminated; prove it stays eliminated).
- Batch variants of newly-writeable classes may behave differently from single writes inside Multiple Service Packets — the STRING evidence says batch works, but verify per class before relabeling any batch-specific expectation.
- MSP per-service error attribution is weak (`parse_multiple_service_response` — see the STRING validation doc's false-alarm note); don't let a batch-level `0x1E` from one bad service masquerade as a class-level "blocked" result. Single-service probes are the arbiter.

## Codex log

2026-07-03 — Submitted by Codex, probe stage. Added `examples/probe_blocked_write_labels.rs`, a maintainer-run Rust hardware probe that expands `examples/full_coverage_tags.json`, selects one representative per current `firmware_blocked_*` class by default (11 targets), and can sweep every blocked manifest tag with `--all-blocked` (72 targets). Each probe target reads the original value, selects a kind-specific test value, writes through the public `write_tag` path, verifies read-back on success, checks a sibling member where the path has one, restores the original value, verifies restore, and writes JSON evidence under `examples/full_coverage_results/`. The probe exits non-zero on setup, verify, sibling-integrity, or restore failures; ordinary write failures are recorded as class evidence rather than treated as tool failures.

Maintainer commands: `cargo run --example probe_blocked_write_labels --locked` for the representative matrix, or `cargo run --example probe_blocked_write_labels --locked -- --all-blocked` for the full 72-label sweep. Set `TEST_PLC_ADDRESS` and `TEST_PLC_SLOT` or pass `--plc-address` / `--plc-slot`. Added `docs/validation/2026-07-03_blocked_write_label_probe_plan.md` as the evidence template and runbook.

Documentation changed without relabeling: `docs/agents/notes/ab-firmware-quirks.md`, `src/lib.rs`, and `src/client.rs` no longer state that all UDT array element member writes are inherently firmware-blocked. They now record the CODEX-AM DINT-member success, keep the manifest labels under CODEX-AV revalidation, and keep read-modify-write as the conservative fallback until the matrix proves which direct writes are reliable. `CHANGELOG.md` records the probe and documentation correction.

Verification passed: `cargo fmt --all`; `cargo run --example probe_blocked_write_labels --locked -- --dry-run` (11 representative targets); `cargo run --example probe_blocked_write_labels --locked -- --dry-run --all-blocked` (72 targets); `cargo clippy --example probe_blocked_write_labels --locked -- -D warnings`; `cargo test --example probe_blocked_write_labels --locked`. `tests/full_coverage_manifest_tests.sh` could not run locally because this Windows environment has no WSL/bash distribution installed; its new assertion is covered by the direct probe dry-run command above and should run in CI's ubuntu shell.

Pending hardware/evidence work: no manifest labels, full-coverage runner counts, or service-layer routing defaults were changed yet. Those are intentionally gated on the maintainer-run probe JSON and should land in the evidence/relabel follow-up commit.

## Claude review

### 2026-07-03 02:00  claude [Fable 5] — probe stage review + hardware matrix executed

**Independent verification**
- `cargo fmt --all -- --check` — clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `bash tests/full_coverage_manifest_tests.sh` — **ok on this host** (Git Bash; Codex's "no bash" note was over-cautious), including the new `blocked_targets=11` dry-run assertion.
- Dry-runs: 11 representative / 72 all-blocked targets enumerate correctly from the manifest; no `Udt`-kind targets exist in the blocked set (the `candidate_value` `None` path is dead for current data — fine).
- Probe source audited before any hardware use: per-target read-original → write-candidate → verify → sibling check → restore → verify-restore, non-zero exit on any discipline failure. Sound.
- **Hardware matrix executed on the 5069-L330ERM fw38** (maintainer-authorized bench, both commands): representative 11 and full 72-target sweep, both `RESULT=PASS` with `unexpected=0` — every mutated value restored and verified, every sibling untouched. Evidence recorded in the plan doc (now the evidence doc) with JSON artifact names.

**The matrix answer**
- **All 60 scalar targets (DINT/REAL/BOOL/INT × controller/program × UDT-array elements) write successfully.**
- **All 12 STRING-member targets fail with CIP `0xFF`/`0x2107`** — consistent, both scopes, standalone-UDT and array-element members alike. With the current encoding (`PlcValue::String` → standalone STRING handle `0x0FCE`), STRING members are genuinely refused; whether a member-tailored encoding exists is CODEX-AO wire-format territory, not a labeling question.
- Manifest asymmetry surfaced: `prog.UDTarr_elem_members` lacks a blocked `Member5_String` entry that its controller-scope twin has — reconcile in the relabel.

**Findings**
- 🟢 Probe-stage scope discipline was exactly right: docs corrected without premature relabels, everything evidence-gated.
- 🟡 The class-representative selector keys on `(scope, mode, category, member, kind)`, which made the default matrix 11 rather than a minimal 6 — harmless (more evidence), noted only so nobody "fixes" it into less coverage.
- 🟠/🔴 none.

**Directive for the relabel stage (now unblocked)**
1. Flip the 60 scalar `firmware_blocked_udt_array_element_member` entries to writeable; update the three runners and the pinned counts (2299 / 2268 / 12 / 19 — recompute and verify via the shell suite) in the same commit.
2. STRING members stay blocked; make the label/docs say what the evidence says (rejected `0x2107` under the current encoding, L330ERM fw38) and fix the prog-array `Member5_String` asymmetry.
3. Quirks note: UDT-array-element section becomes scalar-vs-STRING-member split with links to the evidence doc.
4. Item 4 (service layer): direct writes are proven for scalars — route scalar member writes direct with RMW fallback on `0x2107`; STRING members keep RMW unconditionally. Document.

Probe stage approved and merged to `main`; task returns to `in-progress` for the relabel stage.

## Verdict
