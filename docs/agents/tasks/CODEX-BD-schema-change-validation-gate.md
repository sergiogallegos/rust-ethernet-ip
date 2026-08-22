---
id: CODEX-BD
title: Schema-change simulator and real-hardware validation gate
owner: codex
status: in-progress
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

## Verdict
