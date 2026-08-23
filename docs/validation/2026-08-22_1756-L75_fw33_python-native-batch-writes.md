# 1756-L75 Firmware 33 Python Native Batch Writes

Date: 2026-08-22 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `786910d` plus the local
CODEX-BF implementation under review

## Purpose

This controlled rerun validates that Python `Client.write_tags()` sends
native-batch-safe writes through the Rust Multiple Service Packet engine while
retaining typed single-write fallbacks for operations whose semantics cannot
be safely combined.

The comparison baseline is the release-artifact Python rerun in
[the array-cache before/after record](2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md).
That baseline used the same target, route, DINT-array workload, logical sizes,
sampling floors, packet policy, and terminal value, but Python grouped writes
were sequential.

## Target and Method

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D` firmware `10.007` in chassis slot 1 to processor in
  backplane slot 0
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Build: optimized Rust/FFI `1.2.1` development artifact
- Tags: first 100 controller-scoped DINT array elements in
  [`full_coverage_tags.json`](../../examples/full_coverage_tags.json)
- Logical sizes: 1, 5, 10, 20, 50, and 100 tags
- Sampling floor: 30 seconds and 1,000 tag operations per size/direction,
  after ten read/write warm-up calls
- Packet policy: at most 20 operations and 504 bytes per CIP packet
- Correctness: 2,304/2,304 read-only preflight checks passed, every measured
  call and per-tag result succeeded, and terminal verification passed 100/100

The test address is intentionally omitted. The command was:

```bash
PYTHONPATH=python python3 python/examples/test_plc_full_coverage.py \
  --plc-address <PLC_ADDRESS> --plc-slot 0 \
  --batch-benchmark --batch-min-tag-operations 1000 \
  --batch-min-seconds 30 --allow-writes
```

Every write used the terminal value `999999`, which was already present before
the run, so the final controller state was unchanged. The ignored raw artifact
was `python_batch_benchmark_20260823T020359Z.json`; the complete summary needed
to reproduce comparisons is retained below.

## Dispatch Contract

- Native MSP: unique atomic scalar `BOOL`/numeric writes and numeric array
  elements, at controller or program scope.
- Typed sequential fallback: `STRING`/custom STRING, whole UDTs, member and bit
  paths, packed `BOOL` array elements, and duplicate tag names.
- Mixed input preserves execution order by batching only contiguous safe runs.
  Duplicate names execute sequentially and retain the existing mapping shape,
  where the final result for a duplicate key wins.

## Result

Latency columns are raw average / Tukey-filtered average in milliseconds.
Throughput uses the complete raw sample set.

| Size | Samples R/W | Read latency ms | Read tags/s | Write latency ms | Write tags/s | Write dispatch/call |
|---:|---:|---:|---:|---:|---:|---|
| 1 | 8,944 / 7,687 | 3.352 / 3.350 | 298.4 | 3.896 / 4.794 | 256.7 | 1 native / 0 fallback |
| 5 | 6,079 / 5,943 | 4.933 / 5.028 | 1,013.6 | 5.042 / 5.033 | 991.8 | 5 native / 0 fallback |
| 10 | 5,883 / 5,874 | 5.096 / 5.037 | 1,962.2 | 5.099 / 5.031 | 1,961.1 | 10 native / 0 fallback |
| 20 | 2,941 / 2,935 | 10.196 / 10.075 | 1,961.6 | 10.213 / 10.074 | 1,958.3 | 20 native / 0 fallback |
| 50 | 1,942 / 1,460 | 15.444 / 15.116 | 3,237.6 | 20.539 / 20.146 | 2,434.4 | 50 native / 0 fallback |
| 100 | 969 / 832 | 30.957 / 30.253 | 3,230.2 | 36.058 / 35.264 | 2,773.3 | 100 native / 0 fallback |

At size 100, write latency fell from `368.104 ms` to `36.058 ms` and
throughput rose from `271.7` to `2,773.3 tags/s`: a `10.21x` throughput gain
and `90.2%` latency reduction. The new Python result is within about 2% of the
previous native Rust/C#/C++ size-100 range (`2,829.5`–`2,832.2 tags/s`).

## Safety and Scope

The DINT-only hardware workload exercises the native path and its input/result
correlation. Special-write dispatch, mixed valid/invalid results, duplicate
names, packed BOOL indices above 32, STRING/UDT/member fallback, and terminal
read-back are covered by the Python contract and simulator integration suites.
C ABI batch behavior remains covered by the Rust FFI tests.

These numbers characterize repeated controller-scoped DINT array-element
traffic on this exact target and route. They are not throughput claims for
fallback writes, mixed types, long paths, STRINGs, UDTs, or other controllers.
