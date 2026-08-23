# 1756-L75 Firmware 33 Schema-Change Gate

Status: **live session complete — hardware PASS**

Every row of the live checklist is PASS: array schema-swap (both
directions, both scopes, all four bindings), UDT layout-edit/download
(Rust live, C#/Python/C++ spot check), and the post-schema
full-coverage/batch regression (all four bindings, zero anomalies). This
record is now citable as hardware compatibility evidence for the CODEX-BA
through CODEX-BD schema-safety work, scoped exactly to the controller,
firmware, and topology below — see the coverage-decision footnotes for
what was and wasn't exercised per binding/direction.

## Target

- Processor: ControlLogix `1756-L75`, backplane slot 0, chassis slot 0
- Processor firmware: major revision 33 (full minor revision not read from
  Studio 5000/module properties during this session)
- Chassis: `1756-EN2T` bridge in slot 1
- Route: bridge TCP endpoint to backplane slot 0
- Bridge firmware: not read from module properties during this session
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 16 GB RAM, macOS 26.5.2 (`25F84`), arm64
- Library: `1.2.1` development line, C ABI v3, commit `958e9f3`
- PLC address: intentionally omitted

## Offline Result — 2026-08-22

Command:

```bash
scripts/schema-change-gate
```

One `cargo build --release --features ffi --locked` artifact backed every
wrapper. Results:

| Surface | Result | Evidence exercised |
|---|---|---|
| Rust | PASS | 7 dynamic simulator tests: DINT/BOOL/REAL transitions, delete/recreate, controller/program scope, indices 5/40, batch correlation, no write replay |
| C ABI | PASS | refresh success, clone-visible generation, invalid handle and last-error |
| C# | PASS | refresh advances diagnostics generation and refresh count through the simulator |
| Python | PASS | refresh advances diagnostics generation and refresh count through the simulator |
| C/C++ | PASS | header/export parity (60 symbols) and C++ refresh-generation smoke |

No proprietary tag names or values are emitted by schema-cache diagnostics.

## Live Checklist

Follow [SCHEMA_CHANGE_GATE.md](SCHEMA_CHANGE_GATE.md). Complete one row only
after capturing the starting values, session behavior, counters, retry count,
write count, and restoration result.

| Scenario | Rust | C# | Python | C/C++ | Session survived? | Restored? |
|---|---|---|---|---|---|---|
| Controller DINT[64] -> BOOL[64], indices 5/40 | **PASS** | n/a¹ | **PASS** | n/a¹ | yes | yes |
| Controller BOOL[64] -> DINT[64], indices 5/40 | **PASS** | **PASS** | n/a¹ | **PASS** | yes | yes |
| Program DINT[64] -> BOOL[64], indices 5/40 | **PASS** | n/a¹ | **PASS** | n/a¹ | yes | yes |
| Program BOOL[64] -> DINT[64], indices 5/40 | **PASS** | **PASS** | n/a¹ | **PASS** | yes | yes |
| UDT layout edit + download + rediscovery | **PASS** | **PASS**² | **PASS**² | **PASS**² | yes | yes |
| Post-schema full coverage and batch baseline | **PASS** | **PASS** | **PASS** | **PASS** | n/a | yes |

¹ Coverage decision (2026-08-22, maintainer direction): Rust already proved
both directions end to end, so C#, Python, and C++ each exercise **one**
direction only (whichever the live tag state made available at the time) to
confirm each wrapper's refresh/diagnostics glue against a real controller
edit, rather than repeating the full direction matrix per binding. `n/a`
marks the direction a given binding did not exercise, not a failure.

² Coverage decision (2026-08-22, maintainer direction): the UDT section
proves a different mechanism than the array section (offline
download/session-survival, not online rename), so it's exercised on Rust
only via a dedicated tool; C#, Python, and C++ get a lighter manual
whole-UDT-read spot check instead of three more dedicated companions.

## Rust — Controller/Program DINT[64] -> BOOL[64] detail (2026-08-22)

Run via `examples/schema_change_gate_live.rs --allow-writes` against
`TestProgram`, tag `gSchemaSwap`, indices 5 and 40. The maintainer performed
the online replacement (delete original `DINT[64]`, rename the
`BOOL[64]` `gSchemaSwapReplacement` onto `gSchemaSwap`) without an offline
program download, in both controller and program scope.

- Baseline reads (pre-edit): `Dint(0)` at all four points (controller/program
  x index 5/40).
- Restore-safe pre-edit write smoke check: exercised and restored to
  `Dint(0)` at all four points.
- Post-edit reads, **before** calling `refresh_schema()`: all four points
  returned the correct `Bool(false)` automatically. 2 datatype contradictions
  detected (one per array path, not per index), 2/2 read recoveries
  succeeded, 0 failed, 0 automatic generation change (as designed — automatic
  recovery evicts/retries without bumping the shared generation).
- `refresh_schema()`: generation 2 -> 3, refresh count 0 -> 1 (exactly one).
- Rediscovery: controller discovery returned 282 tags including `gSchemaSwap`;
  program discovery returned 6 tags including `gSchemaSwap`.
- Post-refresh reads: `Bool(false)` at all four points, consistent with the
  post-edit reads.
- Restore-safe post-refresh write/verify: exercised the new packed-BOOL
  addressing shape and restored to `Bool(false)` at all four points
  (indices 5 and 40, i.e. both below and above the packed-BOOL DWORD
  boundary).
- Session: a single connection was held for the entire run (never
  reconnected) and reported healthy throughout — this is the evidence for
  "session survived" on this binding, since the raw encapsulation handle is
  not part of the public API.
- Final cumulative counters (baseline -> end of run): generation 2 -> 3,
  refreshes 0 -> 1, array classification hits/misses/evictions 0/0/0 ->
  28/6/4, datatype contradictions 0 -> 2, read recoveries succeeded/failed
  0/0 -> 2/0.

## Rust — Controller/Program BOOL[64] -> DINT[64] detail (2026-08-22)

Immediate reverse-direction run, same session/process pattern. The
maintainer performed the online replacement (delete the `BOOL[64]`, rename
the pre-staged `gSchemaSwapReplacementDint`/`Program:TestProgram.gSchemaSwapReplacementDint`
`DINT[64]` tags onto `gSchemaSwap`) online, no offline download, in both
controller and program scope.

- Baseline reads (pre-edit, i.e. the `BOOL[64]` state from the prior
  direction): `Bool(false)` at all four points; restore-safe write smoke
  check exercised and restored to `Bool(false)` cleanly at all four points.
- Post-edit reads, before `refresh_schema()`: all four points returned the
  correct `Dint(0)` automatically. 2 datatype contradictions (one per array
  path), 2/2 read recoveries succeeded, 0 failed.
- `refresh_schema()`: generation 2 -> 3, refresh count 0 -> 1.
- Rediscovery: controller 282 tags / program 6 tags, `gSchemaSwap` found in
  both.
- Post-refresh reads: `Dint(0)` at all four points.
- Restore-safe post-refresh write/verify: exercised the new ordinary-array
  (non-packed) addressing shape and restored to `Dint(0)` at all four points.
- Session: single connection held for the entire run, healthy throughout.
- Final cumulative counters: generation 2 -> 3, refreshes 0 -> 1, array
  classification hits/misses/evictions 0/0/0 -> 28/6/4, datatype
  contradictions 0 -> 2, read recoveries succeeded/failed 0/0 -> 2/0.

Both directions are now proven end to end for the Rust binding at both
scopes and both DWORD-boundary indices. `gSchemaSwap` is currently
`DINT[64]` (controller and program) — the same shape it started this
session in.

## C# — Controller/Program BOOL[64] -> DINT[64] detail (2026-08-22)

Run via `examples/CSharpSchemaGateLive/Program.cs --allow-writes`, a new C#
companion mirroring the Rust tool's phases (built following this session's
one-direction-per-binding coverage decision). First attempt captured a
`BOOL[64]` baseline where the maintainer had already performed the swap
before the process's pre-edit reads ran, so that run was aborted without
being recorded — it would not have exercised a real drift/recovery event.
Rerun below is the recorded result.

- Baseline reads (pre-edit, true `BOOL[64]` state): `Bool(False)` at all four
  points (controller/program x index 5/40). Restore-safe pre-edit write
  smoke check exercised and restored to `Bool(False)` cleanly.
- Maintainer performed the online replacement (delete `BOOL[64]`, rename the
  pre-staged `gSchemaSwapReplacementDint` / `Program:TestProgram.gSchemaSwapReplacementDint`
  `DINT[64]` tags onto `gSchemaSwap`) online, no offline download, both
  scopes.
- Post-edit reads, before `RefreshSchema()`: all four points returned the
  correct `Dint(0)` automatically. 2 datatype contradictions (one per array
  path), 2/2 read recoveries succeeded, 0 failed — same shape as the Rust
  result.
- `RefreshSchema()`: generation 2 -> 3, refresh count 0 -> 1 (exactly one).
- Rediscovery: controller discovery returned 282 tags including
  `gSchemaSwap`. Program-scoped discovery is not exposed by the C# 1.2.x
  wrapper (documented gap, not a defect of this gate).
- Post-refresh reads: `Dint(0)` at all four points.
- Restore-safe post-refresh write/verify: exercised the new ordinary-array
  addressing shape and restored to `Dint(0)` at all four points.
- Session: single connection (`EtherNetIpClient`, disposed at process exit)
  held for the entire run, `CheckHealth()` true throughout.
- Final cumulative counters: generation 2 -> 3, refreshes 0 -> 1, array
  classification hits/misses/evictions 0/0/0 -> 28/6/4, datatype
  contradictions 0 -> 2, read recoveries succeeded/failed 0/0 -> 2/0 —
  byte-for-byte identical counter deltas to the Rust run, as expected since
  both share the same native core.

`gSchemaSwap` is currently `DINT[64]` (controller and program) going into
the next binding's pass.

## Python — Controller/Program DINT[64] -> BOOL[64] detail (2026-08-22)

Run via `python/examples/schema_change_gate_live.py --allow-writes`, a new
Python companion mirroring the Rust/C# tools' phases; program-tag/attribute
discovery is not exposed by the Python 1.2.x wrapper (documented gap,
matching `hardware_feature_gate.py`'s existing N/A convention), so Phase 6
is reported N/A rather than executed.

- Baseline reads (pre-edit, true `DINT[64]` state): `Dint(0)` at all four
  points. Restore-safe pre-edit write smoke check exercised and restored to
  `Dint(0)` cleanly.
- Maintainer performed the online replacement (delete `DINT[64]`, rename the
  pre-staged `gSchemaSwapReplacement` / `Program:TestProgram.gSchemaSwapReplacement`
  `BOOL[64]` tags onto `gSchemaSwap`) online, no offline download, both
  scopes.
- Post-edit reads, before `refresh_schema()`: all four points returned the
  correct `Bool(False)` automatically. 2 datatype contradictions (one per
  array path), 2/2 read recoveries succeeded, 0 failed.
- `refresh_schema()`: generation 2 -> 3, refresh count 0 -> 1.
- Rediscovery: N/A (Python wrapper does not expose tag/attribute discovery
  in 1.2.x).
- Post-refresh reads: `Bool(False)` at all four points.
- Restore-safe post-refresh write/verify: exercised the new packed-BOOL
  addressing shape and restored to `Bool(False)` at all four points.
- Session: single `Client` context manager held for the entire run,
  `check_health()` true throughout.
- Final cumulative counters: generation 2 -> 3, refreshes 0 -> 1, array
  classification hits/misses/evictions 0/0/0 -> 28/6/4, datatype
  contradictions 0 -> 2, read recoveries succeeded/failed 0/0 -> 2/0 —
  identical counter deltas to the Rust and C# runs, as expected (shared
  native core).

`gSchemaSwap` is currently `BOOL[64]` (controller and program) going into
the C++ pass.

## C/C++ — Controller/Program BOOL[64] -> DINT[64] detail (2026-08-22)

Run via `examples/cpp/schema_change_gate_live.cpp` (`cpp_schema_gate_live`,
new CMake target), built and CTest-registered (`cpp_schema_gate_live_dry_run`)
alongside the existing C/C++ example suite; built against the raw C ABI
(like `hardware_feature_gate.cpp`) rather than the `eip_client.hpp` RAII
wrapper, since that wrapper does not expose bool reads/writes or routed
connect. Program-tag/attribute discovery is not exposed by the C ABI in
1.2.x (documented gap, matching the existing companion gate's convention),
so Phase 6 program discovery is reported N/A.

- Baseline reads (pre-edit, true `BOOL[64]` state): `Bool(false)` at all
  four points. Restore-safe pre-edit write smoke check exercised and
  restored to `Bool(false)` cleanly.
- Maintainer performed the online replacement (delete `BOOL[64]`, rename the
  pre-staged `gSchemaSwapReplacementDint` / `Program:TestProgram.gSchemaSwapReplacementDint`
  `DINT[64]` tags onto `gSchemaSwap`) online, no offline download, both
  scopes.
- Post-edit reads, before `eip_refresh_schema()`: all four points returned
  the correct `Dint(0)` automatically. 2 datatype contradictions (one per
  array path), 2/2 read recoveries succeeded, 0 failed.
- `eip_refresh_schema()`: generation 2 -> 3, refresh count 0 -> 1.
- Rediscovery: controller discovery returned 282 tags including
  `gSchemaSwap`; program discovery N/A (C ABI gap).
- Post-refresh reads: `Dint(0)` at all four points.
- Restore-safe post-refresh write/verify: exercised the new ordinary-array
  addressing shape and restored to `Dint(0)` at all four points.
- Session: single client handle held for the entire run (`eip_check_health`
  true throughout), disconnected cleanly at exit.
- Final cumulative counters: generation 2 -> 3, refreshes 0 -> 1, array
  classification hits/misses/evictions 0/0/0 -> 28/6/4, datatype
  contradictions 0 -> 2, read recoveries succeeded/failed 0/0 -> 2/0 —
  identical counter deltas to the Rust, C#, and Python runs (shared native
  core).

All four bindings have now exercised this gate against the live 1756-L75:
Rust proved both directions; C#, Python, and C++ each proved one direction
per the maintainer's coverage decision above. `gSchemaSwap` is currently
`DINT[64]` (controller and program).

## Rust — UDT layout edit + download + rediscovery detail (2026-08-22)

Run via `examples/schema_udt_gate_live.rs --allow-writes` against the
controller-scope `gSchemaUdt` (instance of `SchemaGateUdt`: `Marker` DINT +
`Flags` BOOL[64]).

**Finding, recorded before this run started:** `get_udt_definition()` /
`get_tag_attributes()` failed against `gSchemaUdt` with `Protocol error: Get
Attribute List for 'gSchemaUdt' failed: Path segment error`, even though
plain `read_tag("gSchemaUdt")` and `discover_tags_detailed()` both succeeded
immediately (the latter already reporting `template_instance_id: Some(2970)`
for the same tag). Isolated with a throwaway probe before building the live
tool; not investigated further here — see CODEX-BJ. The live tool below was
rewritten to use `read_tag()` (whole-UDT payload length) and
`discover_tags_detailed()` (`template_instance_id`) as its layout-change
signal instead of the broken path, so this finding did not block validating
the actual schema-recovery mechanism this section exists to test.

- Baseline (before any edit): `payload_bytes=14`,
  `template_instance_id=Some(2970)` (out of 282 discovered controller tags).
- Maintainer went offline in Studio 5000, added a dedicated non-I/O `DINT`
  member (`Marker2`) to `SchemaGateUdt`, and downloaded.
- Post-edit, before `refresh_schema()`: read succeeded on the **first**
  attempt — **the encapsulation session survived the offline download
  without a reconnect** (`session survived without reconnect: true`).
  `payload_bytes=18` (already reflecting the new layout at this point).
- `refresh_schema()`: generation 2 -> 3. Post-refresh snapshot unchanged
  from the post-edit read (`payload_bytes=18`,
  `template_instance_id=Some(2970)` — the controller reused the same
  template instance id rather than issuing a new one for the edited
  layout).
- Maintainer went offline again, removed `Marker2`, and downloaded to
  restore the original layout.
- Post-restore, before the second `refresh_schema()`: again succeeded on
  the first read — **session survived this download too**
  (`session survived without reconnect: true`).
- Second `refresh_schema()`: generation 3 -> 4. Post-restore-refresh
  snapshot: `payload_bytes=14`, `template_instance_id=Some(2970)` —
  matches the original baseline exactly (`matches original baseline:
  true`).
- Cumulative: generation 2 -> 4, refreshes 0 -> 2 (one per edit event, as
  expected).
- Final controller state: `gSchemaUdt` restored to its original
  `Marker`/`Flags` layout.

C#, Python, and C++ manual spot check (per the coverage decision above):
all three read `gSchemaUdt` and its members against the restored (original)
layout, via throwaway ad hoc probes (not committed — each confirmed working
API usage, not new reusable tooling):

| Binding | `gSchemaUdt` payload | `Marker` | `Flags[0]` | Result |
|---|---|---|---|---|
| Python | `{'symbol_id': 0, 'data': [80, 8, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]}` (14 bytes) | `0` | `False` | PASS |
| C# | `ReadTagWithDetails` success, `UdtData.Data.Length == 14` | `0` | `false` | PASS |
| C/C++ | `eip_read_tag` rc=0, `{"Udt":{"symbol_id":0,"data":[80,8,0,0,0,0,0,0,0,0,0,0,0,0]}}` (14 bytes) | `0` | `0` (false) | PASS |

All three bindings observed the identical 14-byte payload the Rust tool's
post-restore baseline reported — good cross-binding consistency evidence,
independent of the `get_tag_attributes` finding above (none of these three
spot checks touch that code path; they use each binding's plain tag-read
surface).

## Post-schema full coverage and batch regression detail (2026-08-22)

Re-ran the existing (pre-1.2.1) full-coverage and batch/whole-UDT/discovery
companion gates against this same controller, on the current tree/build
(commit `2028748`, C ABI v3), after all of the schema-generation refresh
activity above, to confirm ordinary operation is unaffected.

**Full-coverage** (`test_plc_full_coverage` / `CSharpFullCoverage` /
`python/examples/test_plc_full_coverage.py` / `cpp_full_coverage`), all
four bindings against the shared 2304-tag manifest:

| Binding | Reads | Writes | Verify | Blocked | Anomalies | Result |
|---|---|---|---|---|---|---|
| Rust | 2304/2304 | 2285/2285 | 2285/2285 | 0 | 0 | PASS |
| C# | 2304/2304 | 2285/2285 | 2285/2285 | 0 | 0 | PASS |
| Python | 2304/2304 | 2285/2285 | 2285/2285 | 0 | 0 | PASS |
| C/C++ | 2304/2304 | 2285/2285 | 2285/2285 | 0 | 0 | PASS |

All four settled every writeable tag to its terminal value
(`999999`/`9999`/`99.99`/`true`/`SETTLED`-family) with `settle_ok=2285`,
`settle_fail=0`, and 18/18 sample settle-verify reads, byte-identical
across bindings.

**Batch/whole-UDT/discovery companion gate** (`hardware_feature_gate`, all
four bindings), restore-safe against the separate `gTestArray_*`/`gTestUDT`
fixtures:

| Binding | Controller discovery | Program discovery | Whole UDT reads | Batch read | Batch write (restore-safe) | Result |
|---|---|---|---|---|---|---|
| Rust | PASS (282 tags) | PASS (6 tags) | 4/4 | 10/10 | 4/4 restored | PASS |
| C# | PASS (282 tags) | N/A (not exposed) | 4/4 | 10/10 (batch + native) | 4/4 restored | PASS |
| Python | N/A (not exposed) | N/A (not exposed) | 4/4 | 10/10 | 4/4 restored | PASS |
| C/C++ | PASS (282 tags) | N/A (not exposed) | 4/4 | 10/10 | 4/4 restored | PASS |

Rust's program-scope discovery output listed `gSchemaSwap (DINT)` among the
6 program tags — an independent confirmation that the array-swap section's
final state (`DINT[64]`, restored) is exactly what discovery reports too.

Both gates are unchanged from their pre-1.2.1 shape (same manifests, same
fixtures); nothing about the schema-refresh/drift-recovery work changed
their pass criteria. Zero anomalies across all 8 runs.

## Final Controller State

All test fixtures were restored to their pre-session state or an explicit
terminal value:

- `gSchemaSwap` (controller and program): `DINT[64]`, matching the state
  before this session started. Values at indices 5/40 restored to their
  original pre-edit values after every write-verification pass.
- `gSchemaUdt` (controller): `SchemaGateUdt` restored to its original
  two-member layout (`Marker` DINT + `Flags` BOOL[64]); values unchanged
  (`Marker=0`, `Flags` all false).
- `gSchemaSwapReplacement`/`gSchemaSwapReplacementDint` (and their
  program-scoped equivalents): consumed by the online renames during the
  array-swap section; none remain as separate tags.
- The pre-existing `gTestArray_*`/`gTestUDT`/`gTest_STRING` full-coverage
  fixtures were left in their documented terminal settled state
  (`999999`/`9999`/`99.99`/`true`/`SETTLED`) by the post-schema
  full-coverage re-run above, consistent with every prior full-coverage
  gate run against this controller.

This file is now a complete hardware-validation record for the CODEX-BA
through CODEX-BD schema-safety work on the live 1756-L75 (fw33): array
schema-swap (both directions, both scopes, all four bindings), UDT
layout-edit/download/session-survival (Rust live, C#/Python/C++ spot
check), and the post-schema full-coverage/batch regression, all PASS with
zero anomalies.

