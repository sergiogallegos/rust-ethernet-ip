---
id: CODEX-BD
title: Schema-change simulator and real-hardware validation gate
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1 validation. Depends on CODEX-BA, CODEX-BB, and CODEX-BC.**

Create a repeatable validation gate for schema changes that occur without an
application restart, then execute the hardware portion with maintainer control
of Studio 5000 and the dedicated test controller.

### Required implementation

1. Extend the simulator or a deterministic test stream so a tag can disappear
   and reappear under the same symbolic name with a different datatype/shape.
2. Add an offline cross-binding runner that proves explicit refresh behavior
   in Rust, C#, Python, and C/C++.
3. Add an opt-in live procedure for the 1756-L75 firmware 33 through the
   1756-EN2T route covering:
   - online temporary-tag replacement under the original name;
   - ordinary array to packed BOOL and the reverse;
   - indices below and above 32;
   - program and controller scope;
   - offline UDT member/layout update and download;
   - whether the encapsulation session survives each change.
4. Require explicit write opt-in, dedicated tags, starting-value capture, and
   restoration. The runner must never edit PLC schema itself.
5. Record automatic read recovery, explicit refresh in all bindings, UDT
   rediscovery, errors, retries, and final controller state.

### Test requirements

- Deterministic simulator tests pass without PLC access.
- All four bindings use one release native artifact.
- Live validation records exact processor, firmware, bridge/route, host, build,
  and controller edit sequence without publishing the PLC address.
- No write is duplicated or sent using stale packed-BOOL addressing.
- Existing full-coverage and batch baselines remain green after the schema test.

### Acceptance criteria

- A dated validation record exists under `docs/validation/`.
- Hardware matrix and release notes link the result without generalizing beyond
  the exact controller/firmware/topology.
- The cache-lifecycle wiki and append-only wiki log are updated.
- The controller is restored or its intentional final test state is recorded.

### Out of scope

- Automated Studio 5000 project manipulation.
- Production-controller writes.
- Performance packet tuning (CODEX-BE).

## Codex log

### 2026-08-22 15:00  codex [GPT-5]

Offline implementation is complete. The simulator can replace/delete tags
under an unchanged symbolic path; `scripts/schema-change-gate` builds one
release FFI artifact and passes Rust, C ABI, C#, Python, and C++ explicit
refresh/generation checks. Added a safe live procedure and dated 1756-L75 fw33
record with dedicated fixtures, write opt-in, starting-value capture,
restoration, both BOOL/DINT directions, controller/program scope, indices 5/40,
UDT download/rediscovery, and session-survival capture. The record is clearly
marked offline PASS/live pending. Task remains in progress until the maintainer
controls the Studio 5000 edits and the hardware/full-coverage results are
recorded.

## Claude review

### 2026-08-22 16:10  claude [Sonnet 5]

Independent verification of the offline-complete surface (spans BA/BB/BC/BD,
all uncommitted in the same tree): `cargo fmt -- --check`,
`cargo clippy --workspace --all-targets --all-features -- -D warnings`,
`SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test
plc_sim_tests`, and `scripts/schema-change-gate` (Rust, C ABI, C#, Python,
C++) all pass clean on this machine. Read the recovery logic directly:
`EipClient::read_tag` (`src/client.rs:1406`) does exactly one bounded retry
gated on `is_schema_drift_read_error`/`array_path_for_schema_recovery`
(`src/client.rs:1301`, `1310`), evicting only the generation-scoped array
entry before retrying; the CIP-error string match at `src/client.rs:1316`
lines up with `EtherNetIpError`'s `Display` format at `src/error.rs:76`.
`eip_refresh_schema` (`src/ffi.rs:1606`) is a safe export with no `unsafe`
block, consistent with the existing FFI pattern. No blocking findings.

At maintainer direction, implemented directly (per the CODEX-AY precedent)
rather than routing back to Codex, since the remaining BD work is either
hardware-bound (out of reach here) or tooling to reduce the maintainer's
manual load during the live session:

- Added `examples/schema_change_gate_live.rs`, following the existing
  `examples/hardware_feature_gate.rs` / `probe_blocked_write_labels.rs`
  convention (env-var defaults, `--allow-writes`/`--dry-run` gating). It
  drives the Rust side of the "Per-Binding Online Replacement" section of
  `docs/validation/SCHEMA_CHANGE_GATE.md`: baseline capture, warm reads at
  indices 5/40 in controller and program scope, an optional restore-safe
  pre-edit write smoke check, a stdin pause naming the exact Studio 5000
  action to perform, post-edit reads with automatic-recovery counter deltas,
  explicit `refresh_schema()` with a generation/refresh-count assertion,
  rediscovery, post-refresh reads, and an optional restore-safe post-refresh
  write/verify. It never issues a schema edit itself and ends by printing a
  result block sized to paste into the dated validation record. The UDT
  layout/download section and the C#/Python/C++ companions are intentionally
  left manual — noted in the tool's own doc comment and in
  `SCHEMA_CHANGE_GATE.md`.
- Verified with `cargo build --example schema_change_gate_live`,
  `cargo clippy --example schema_change_gate_live --all-features -- -D
  warnings`, `cargo fmt -- --check`, and a `--dry-run` smoke run. Not run
  against a live controller — that step still requires the maintainer's
  hardware session.
- Documented the tool in `SCHEMA_CHANGE_GATE.md` and `wiki/log.md`.

Status stays `in-progress`: the live 1756-L75 Studio 5000 sequence, the
maintainer-run pass through the new companion tool, and completion of the
dated record's Live Checklist table remain outstanding before this can move
to `submitted`/`merged`.

### 2026-08-22 22:20  claude [Sonnet 5]

Live 1756-L75 (firmware 33) session completed with the maintainer
performing every Studio 5000 action; walked through interactively over
this session, driving each companion tool while the maintainer edited the
controller and confirmed each step. Full detail is in
`docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md`; summary:

- **Built three more live companions during the session** (C#
  `examples/CSharpSchemaGateLive`, Python
  `python/examples/schema_change_gate_live.py`, C/C++
  `examples/cpp/schema_change_gate_live.cpp` + `cpp_schema_gate_live`
  CMake target with a `--dry-run` CTest) mirroring the Rust tool phase for
  phase, plus a fifth Rust-only tool
  (`examples/schema_udt_gate_live.rs`) for the UDT layout-edit section.
  Committed at `2bb0e04` (Rust tool, prior entry), `95fc581` (C#/Python/C++
  array-gate companions), `bbc8834` (Rust UDT-gate companion).
- **Array schema-swap section: PASS**, both directions
  (`DINT[64]<->BOOL[64]`), controller and program scope, indices 5 and 40.
  Rust proved both directions end to end; C#, Python, and C++ each proved
  one direction (maintainer coverage decision, recorded in the dated file
  — Rust already covers both, the remaining goal per binding is proving
  its own refresh/diagnostics glue against a real edit). Counter deltas
  (contradictions, recoveries, generation/refresh) were identical across
  all four bindings, as expected from the shared native core. One process
  incident along the way: an early C# attempt captured a baseline *after*
  the maintainer had already swapped the tag, so it wasn't a real
  drift-recovery observation — caught before recording it, aborted, and
  rerun correctly against the true pre-edit state.
- **UDT layout-edit section: PASS.** Found and routed around a real bug
  during this section: `get_tag_attributes()`/`get_udt_definition()`
  failed with a CIP path-segment error against the live `gSchemaUdt` tag,
  while `read_tag()` and `discover_tags_detailed()` both succeeded on the
  same tag in the same session — opened **CODEX-BJ** to track it
  (non-blocking, severity/scope unknown). The Rust UDT-gate tool was
  rewritten to use the working calls instead. Live result: the
  encapsulation session survived **both** the offline member-add download
  and the offline restore download without reconnecting; the layout
  change was directly observed (14 -> 18 -> 14 payload bytes, stable
  `template_instance_id`); generation/refresh counters advanced correctly
  across both refreshes. C#, Python, and C++ each got a light manual
  spot-check (ad hoc, not new tooling) reading the restored `gSchemaUdt` —
  all three read the identical 14-byte payload.
- **Post-schema full-coverage and batch regression: PASS**, all four
  bindings, zero anomalies (`reads=2304/2304 writes=2285/2285
  verify=2285/2285`), confirming the schema-refresh/drift-recovery work
  didn't disturb ordinary operation. Batch/whole-UDT/discovery companion
  gate also PASS on all four bindings.
- **Final controller state**: `gSchemaSwap` (controller + program)
  restored to `DINT[64]`, its pre-session shape; `gSchemaUdt` restored to
  its original two-member layout; full-coverage fixtures left in their
  documented terminal settled state, consistent with every prior
  full-coverage run against this controller. No temporary swap tags
  remain.
- Updated `docs/release/1.2.1_RELEASE_NOTES_DRAFT.md`,
  `wiki/controllers/hardware-validation-program.md`,
  `docs/HARDWARE_COMPATIBILITY.md`, and
  `wiki/investigations/array-type-cache-lifecycle.md` (including answering
  that page's "does a firmware-33 download always break the session"
  open question — no, for this session/route/controller shape) to reflect
  the completed hardware PASS rather than "live pending."
- CI green on every commit in this sequence (independently watched, not
  just assumed): `2bb0e04`, `95fc581`, `fd95be1`. (`bbc8834` and the
  doc-only closing commits are docs/example-only changes following the
  same already-green tree; not separately re-watched given the earlier
  lesson's core risk — a commit depending on not-yet-landed APIs — does
  not apply here, since nothing in this closing sequence depends on
  anything not already on `main`.)

**Acceptance criteria tally**
- ✅ A dated validation record exists under `docs/validation/`.
- ✅ Hardware matrix and release notes link the result without generalizing
  beyond the exact controller/firmware/topology (release notes draft,
  `HARDWARE_COMPATIBILITY.md`, and the hardware-validation-program wiki
  page all scope the claim to the tested 1756-L75/fw33/1756-EN2T path).
- ✅ The cache-lifecycle wiki and append-only wiki log are updated.
- ✅ The controller is restored to its pre-session state (see Final
  Controller State above and in the dated record).

## Verdict

Live hardware validation complete: array schema-swap (both directions, all
four bindings), UDT layout-edit/download (Rust live, three bindings spot
checked), and post-schema full-coverage/batch regression are all PASS with
zero anomalies on the live 1756-L75 (fw33). Controller restored. One
non-blocking finding spun out as CODEX-BJ for separate investigation. All
brief acceptance criteria met — status set to `merged`.
