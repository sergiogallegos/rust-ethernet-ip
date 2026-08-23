# Real-Hardware Compatibility and Test Program

Latest published library release: `1.2.0`; next patch in preparation: `1.2.1`

This page tracks tests run against physical Allen-Bradley controllers. A `Done`
cell means that binding was exercised on that exact processor and firmware; it
does not imply support for every processor or firmware in the product family.
Simulator-only results are intentionally excluded.

## Compatibility Matrix

| Family | Processor | Firmware | Topology | Rust | C# | Python | C/C++ | Latest library tested | Evidence |
|---|---|---:|---|---|---|---|---|---|---|
| CompactLogix 5380 | 5069-L330ERM | 38 | Direct EtherNet/IP | Done | Done | Done | Done | `1.2.0` | [1.2.0 release gate](validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md) |
| CompactLogix 5380 | 5069-L320ERMS3 | 35 | Direct EtherNet/IP | Done | Done | — | — | `0.7.0` development line | [Rust](validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md), [C#](validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md) |
| CompactLogix 5370 | 1769-L18ER-BB1B | 33 | Integrated Ethernet | Done | Done | Done | — | `1.0.0` | [cross-binding run](validation/2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md) |
| ControlLogix 5580 | 1756-L81ES | 37 | 1756-EN3TR to backplane slot 0 | Done | Done | Done | — | `0.8.0` development line | [Rust](validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md), [C#](validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md), [Python](validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md) |
| ControlLogix 5570 | 1756-L75/B | 33.011 | 1756-EN2T/D 10.007 to backplane slot 0 | Done | Done | Done | Done | `1.2.1` development line | [functional/sequential baseline](validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md), [post-schema sequential rerun](validation/2026-08-22_1756-L75_fw33_post-schema-cross-binding-performance.md), [batch baseline](validation/2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md), [array-cache rerun](validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md), [schema-change gate (live PASS)](validation/2026-08-22_1756-L75_fw33_schema-change-gate.md) |

Legend: `Done` = a traceable real-hardware pass exists; `—` = no submitted
evidence for that processor/firmware/binding combination.

## Help Expand the Matrix

The highest-value contributions are new processor/firmware combinations,
especially:

- CompactLogix 5380 processors across the 5069-L3xx family and multiple major
  firmware revisions.
- ControlLogix 5570 (`1756-L7x`) and 5580 (`1756-L8x`) processors, including
  routed chassis access through different EtherNet/IP bridge modules.
- Safety variants tested without changing or bypassing the safety task.
- C/C++ results on every target that currently has only Rust, C#, or Python
  evidence.
- Multi-hop routes that traverse more than one chassis or Ethernet segment.

Do not report an entire family as supported after testing one processor. Add
one row for each exact catalog number, firmware revision, topology, library
version, and binding.

## Safe Controller Setup

Use a development controller or a maintenance window. Never aim an automated
write test at production tags, safety logic, motion axes, outputs, recipes, or
setpoints that can affect equipment.

1. Create an isolated controller program named `TestProgram`.
2. Import or reproduce the tags in
   [PLC_TEST_TAG_DEFINITIONS.md](PLC_TEST_TAG_DEFINITIONS.md). The shared
   [full-coverage manifest](../examples/full_coverage_tags.json) defines what
   each binding will exercise.
3. Keep test tags controller-scoped and program-scoped, with atomic scalars,
   arrays, packed BOOL arrays, built-in `STRING`, at least one custom string
   type, UDTs, UDT arrays, and nested members.
4. Confirm from Studio 5000 that every write target is test-only and that the
   controller can be restored from a known project backup.
5. Record processor catalog number, complete firmware revision, communication
   module and firmware, chassis slot, network topology, host OS/architecture,
   and library commit/version before testing.
6. Read the starting values and retain them. Restore them after a smoke or
   performance run unless the standard full-coverage settle phase is intended.

## Required Functional Pass

Build one FFI library and use that same artifact for wrappers so parity results
are comparable:

```bash
cargo build --release --features ffi --locked

TEST_PLC_ADDRESS=192.168.0.10:44818 TEST_PLC_SLOT=0 \
  cargo run --release --example test_plc_full_coverage

TEST_PLC_ADDRESS=192.168.0.10:44818 TEST_PLC_SLOT=0 \
  dotnet run --project examples/CSharpFullCoverage -c Release

TEST_PLC_ADDRESS=192.168.0.10:44818 TEST_PLC_SLOT=0 PYTHONPATH=python PYTHONUTF8=1 \
  python3 python/examples/test_plc_full_coverage.py

cmake -S examples/cpp -B target/cpp-example
cmake --build target/cpp-example --config Release
# Run target/cpp-example/full_coverage with the address/route options shown by --help.
```

At minimum, report:

- connection, disconnect, and reconnect result;
- total reads, writes, read-back verifications, expected rejections, and
  unexpected anomalies for every binding;
- controller- and program-scoped behavior;
- scalar, array, packed BOOL, `STRING`, custom string, UDT member, UDT-array
  member, fragmentation, batch, discovery, diagnostics, and route behavior;
- whether all modified values were restored or intentionally settled.

## Endurance Tests

Start with a one-hour shakedown before a 24-hour run. Use a stable, wired test
network and collect host and controller diagnostics without increasing the PLC
task load beyond an agreed limit.

| Profile | Duration | Suggested workload | Pass criteria |
|---|---:|---|---|
| Read soak | 24 h | Repeated mixed batch reads at 100 ms, 500 ms, and 1 s groups | No unhandled error, leak, deadlock, or silent stale-data period; reconnects and all failed reads counted |
| Read/write soak | 8–24 h | Reads plus periodic writes/read-back on dedicated DINT, REAL, BOOL, STRING, and UDT-member tags | Zero unexplained mismatches; every write target restored; duplicate-write risk documented |
| Reconnect soak | 2 h | Controlled link interruption or controller program-mode transitions every 5–10 min | Recovery bounded and measured; no worker/task accumulation |
| Subscription soak | 24 h | Fast and slow consumers, including intentional backpressure | Polling remains live; dropped/partial/error events are observable and bounded |
| Multi-client soak | 4–24 h | Several clients and, when available, several PLCs | No cross-client state leakage; per-controller errors remain attributable |

Record operation counts, success/failure counts, reconnect count and duration,
maximum consecutive failures, latency percentiles, process RSS at intervals,
CPU use, and controller communication/task-load observations. A 24-hour test
that reports only “still running” is useful, but not sufficient for a `Done`
endurance claim.

## Performance Characterization

Performance numbers are field baselines, not universal claims. Hold constant
the processor, firmware, route, bridge module, network, tag paths, build mode,
host, and sampling method. Run a warm-up before timed samples and preserve raw
results.

Measure at least:

- connect and routed-connect latency;
- single read/write latency with p50, p95, p99, maximum, and error rate;
- batch throughput and latency at 1, 5, 10, 20, 50, and 100 tags;
- short and long symbolic paths;
- atomic, `STRING`, custom string, and fragmented structure payloads;
- sustained operations/second and bytes/second;
- reconnect time after a controlled interruption;
- CPU time and peak/resident memory on the client host;
- controller communication utilization or task impact when available;
- parity across Rust, C#, Python, and C/C++ using the same native build.

Use at least 30 seconds or 1,000 measured operations per scenario, whichever is
longer. Report sample count and distribution; do not publish only the fastest
observation. Existing hardware baselines are recorded in the linked validation
files and should not be compared across controllers without noting topology and
test-method differences.

The full-manifest runners provide a comparable sequential baseline across all
four bindings. After verifying the controller project and write targets, add
`--benchmark-passes 3 --allow-writes --out-dir <directory>` to each runner.
Benchmark mode performs an untimed 2,304-tag preflight, measures every
individual read and write, leaves the standard terminal values in writeable
test tags, and emits JSON with average, minimum, p50, p95, p99, maximum,
throughput, sample count, and failures. See the
[1756-L75 firmware 33 baseline](validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md)
for a complete example.

For logical batch-size characterization, add:

```text
--batch-benchmark --batch-min-tag-operations 1000 \
  --batch-min-seconds 30 --allow-writes
```

The first numeric minimum is a tag-operation floor; the runner derives the
necessary number of logical calls for each batch size. Results at sizes 50 and
100 may span multiple CIP packets under the conservative default packet policy.
See the historical [1756-L75 batch baseline](validation/2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md)
and its controlled [array-cache rerun](validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md).

## Submitting Results

1. Copy [REAL_HARDWARE_RESULT_TEMPLATE.md](validation/REAL_HARDWARE_RESULT_TEMPLATE.md)
   to `docs/validation/YYYY-MM-DD_<model>_fw<revision>.md`.
2. Attach or summarize raw JSON/CSV results without including plant IP
   addresses, controller project files, credentials, or proprietary tag names.
3. Add the exact row to this matrix. Use `Done` only for bindings actually run.
4. Update the maintainer wiki synthesis and `wiki/log.md` as described in
   [AGENTS.md](../AGENTS.md).
5. Open a pull request explaining anomalies, restore state, and any controller
   changes required for the test.
