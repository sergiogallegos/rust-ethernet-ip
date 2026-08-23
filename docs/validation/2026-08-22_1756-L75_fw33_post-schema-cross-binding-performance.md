# 1756-L75/B 33.011 Post-Schema Cross-Binding Performance

Date: 2026-08-22 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `67c093a`

## Target and Host

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D`, firmware `10.007`, chassis slot 1 to processor in backplane slot 0
- Test address: private lab address, intentionally omitted
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Build: optimized Rust/FFI release build; all wrappers used the same local native implementation
- Manifest: [`full_coverage_tags.json`](../../examples/full_coverage_tags.json), 2,304 paths

## Method

This repeats the methodology from the
[2026-08-21 baseline](2026-08-21_1756-L75_fw33_cross-binding-performance.md)
after the CODEX-BA through CODEX-BD schema-cache safety sequence:

1. one untimed read-only warm-up/preflight over all 2,304 paths;
2. three measured sequential-read passes, 6,912 operations per binding;
3. three measured sequential-write passes over all 2,285 writeable paths,
   6,855 operations per binding;
4. established terminal test values only, leaving the dedicated fixtures in
   their documented settled state.

Every call used a monotonic high-resolution clock. Percentiles use nearest
rank. The bindings ran serially. Tukey `1.5 x IQR` filtering is reported only
for the filtered average; min, percentiles, and max retain every sample.

## Results

All 27,648 reads and 27,420 writes succeeded. Every preflight was 2,304/2,304.

### Sequential reads

| Binding | Samples | Avg ms | Filtered avg ms | Min ms | p50 ms | p95 ms | p99 ms | Max ms | ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,912 | 2.844 | 2.835 | 0.597 | 4.196 | 5.014 | 5.320 | 11.086 | 351.6 | 0 |
| Python | 6,912 | 2.972 | 2.966 | 0.621 | 4.214 | 5.062 | 5.407 | 10.707 | 336.4 | 0 |
| C# | 6,912 | 2.989 | 2.949 | 0.582 | 4.225 | 5.034 | 5.358 | 232.352 | 334.5 | 0 |
| C/C++ | 6,912 | 2.810 | 2.806 | 0.600 | 4.181 | 4.979 | 5.258 | 10.907 | 355.9 | 0 |

### Sequential writes

| Binding | Samples | Avg ms | Filtered avg ms | Min ms | p50 ms | p95 ms | p99 ms | Max ms | ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,855 | 5.773 | 5.084 | 0.650 | 5.077 | 10.026 | 10.809 | 237.243 | 173.2 | 0 |
| Python | 6,855 | 5.697 | 5.086 | 0.643 | 5.077 | 9.996 | 10.751 | 19.723 | 175.5 | 0 |
| C# | 6,855 | 5.723 | 5.099 | 0.623 | 5.078 | 10.000 | 14.469 | 232.254 | 174.7 | 0 |
| C/C++ | 6,855 | 5.993 | 5.082 | 0.653 | 5.088 | 10.161 | 15.055 | 232.665 | 166.9 | 0 |

The roughly 232–237 ms C#/Rust/C++ maxima are isolated tail events. The
filtered write averages cluster tightly at 5.082–5.099 ms, and write p99 stays
at or below 15.055 ms.

## Comparison with 2026-08-21

| Binding | Read avg before -> after | Observed read change | Write avg before -> after | Observed write change |
|---|---:|---:|---:|---:|
| Rust | 5.602 -> 2.844 ms | 49.2% lower | 6.178 -> 5.773 ms | 6.6% lower |
| Python | 5.914 -> 2.972 ms | 49.7% lower | 6.860 -> 5.697 ms | 16.9% lower |
| C# | 6.065 -> 2.989 ms | 50.7% lower | 6.572 -> 5.723 ms | 12.9% lower |
| C/C++ | 6.645 -> 2.810 ms | 57.7% lower | 7.362 -> 5.993 ms | 18.6% lower |

The read improvement is consistent across all four bindings and is compatible
with the shared Rust-core array-shape cache improvements. It is still an
observed before/after result, not a controlled causal attribution: controller
task load, bridge utilization, and network utilization were not captured in
either run, and the runs occurred on different days.

## Reproduction

Each full-coverage runner used:

```text
--plc-slot 0 --benchmark-passes 3 --allow-writes \
  --out-dir examples/full_coverage_results/2026-08-22_post-schema-performance
```

The generated JSON artifacts are gitignored local evidence. They retain exact
sample totals, full distributions, failure counts, versions, and workload
identity.

## Evidence Boundary

- This is heterogeneous sequential single-tag traffic, not batch throughput.
- No controller task-load, EN2T utilization, client CPU/RSS, or bytes/second
  telemetry was captured.
- It characterizes this exact host/controller/firmware/route and should not be
  generalized to every Logix controller or network.
- The dedicated full-coverage fixtures remain in their established terminal
  settled state; no temporary schema tags were created.
