# 1756-L75 Firmware 33 Cross-Binding Performance Baseline

Date: 2026-08-21 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `2c87d11` plus the local
benchmark-runner changes described below

## Target and Host

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D` firmware `10.007` in chassis slot 1 to processor in backplane slot 0
- Test address: private lab address, intentionally omitted
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Build: optimized Rust/FFI release build; C#, Python, and C++ used that local
  native implementation
- Test program: isolated `TestProgram` and the shared
  [`full_coverage_tags.json`](../../examples/full_coverage_tags.json) manifest

## Method

Each binding opened a new routed connection and performed:

1. one untimed read-only warm-up/preflight over all 2,304 manifest paths;
2. three measured sequential-read passes: 6,912 total operations;
3. three measured sequential-write passes over all 2,285 writeable paths:
   6,855 total operations;
4. terminal-value writes only (`999999`, `9999`, `99.99`, `true`, and
   `SETTLED`) so no random values remained in the test tags.

Every individual call was measured with the language's monotonic
high-resolution clock. Percentiles use the nearest ranked sample. The workload
mix includes controller- and program-scoped atomic values, arrays, packed BOOL
elements, STRINGs, and UDT member paths. The 19 whole-UDT paths were read-only.

This is a sequential heterogeneous-manifest baseline, not a claim about the
latency of every individual data type or network. Each read and write direction
ran longer than 30 seconds and exceeded 1,000 measured operations. The four
bindings ran serially, so controller and network conditions were similar but
not simultaneous.

## Results

All 27,648 measured reads and 27,420 measured writes succeeded. There were no
operation failures in any binding.

### Sequential reads

| Binding | Samples | Avg ms | Min ms | p50 ms | p95 ms | p99 ms | Max ms | ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,912 | 5.602 | 0.629 | 5.070 | 9.691 | 10.149 | 237.169 | 178.5 | 0 |
| Python | 6,912 | 5.914 | 0.724 | 5.107 | 9.934 | 10.226 | 11.296 | 169.1 | 0 |
| C# | 6,912 | 6.065 | 0.615 | 5.141 | 10.014 | 10.225 | 13.613 | 164.9 | 0 |
| C/C++ | 6,912 | 6.645 | 0.639 | 5.246 | 10.163 | 10.443 | 15.436 | 150.5 | 0 |

### Sequential writes

| Binding | Samples | Avg ms | Min ms | p50 ms | p95 ms | p99 ms | Max ms | ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,855 | 6.178 | 0.634 | 5.124 | 10.176 | 15.008 | 236.642 | 161.9 | 0 |
| Python | 6,855 | 6.860 | 0.666 | 5.382 | 10.503 | 15.117 | 35.009 | 145.8 | 0 |
| C# | 6,855 | 6.572 | 0.645 | 5.208 | 10.435 | 15.092 | 17.238 | 152.2 | 0 |
| C/C++ | 6,855 | 7.362 | 0.642 | 5.579 | 14.741 | 15.289 | 236.045 | 135.8 | 0 |

The Rust read and write maxima and the C/C++ write maximum are isolated
roughly 236–237 ms tail events; their p99 values remained at or below 15.289
ms. This is why the distribution is more useful than either the minimum or
maximum alone. One run per binding is not enough to attribute small differences
solely to wrapper overhead.

## Functional Context

Immediately before this characterization, the same target passed the complete
four-binding functional run:

- 2,304/2,304 reads per binding;
- 2,285/2,285 writes per binding;
- 2,285/2,285 read-back verifications per binding;
- 18/18 settled-value samples per binding;
- zero anomalies.

The separate companion gate also passed native/grouped batch operations,
whole-UDT reads, and the discovery surfaces exposed by each binding. See the
[`CROSS_BINDING_FEATURE_GATE.md`](CROSS_BINDING_FEATURE_GATE.md) method for the
scope of that companion test.

## Reproduction

The four full-coverage runners accept the same benchmark options:

```text
--benchmark-passes 3 --allow-writes --out-dir <result-directory>
```

Benchmark mode requires the preflight warm-up and refuses to run writes unless
`--allow-writes` is supplied. JSON artifacts contain the exact sample counts,
failures, totals, distribution, throughput, binding version, route slot, and
workload name.

## Limitations and Next Measurements

- No controller task-load or EN2T utilization value was captured.
- No client CPU/RSS sampling or bytes-per-second measurement was captured.
- This run measures sequential single-tag calls. Batch sizes 1, 5, 10, 20, 50,
  and 100 are characterized separately in the
  [cross-binding batch performance record](2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md).
- A 24-hour soak and controlled reconnect test are still required before making
  endurance or recovery claims.
