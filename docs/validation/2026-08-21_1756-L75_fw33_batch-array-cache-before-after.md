# 1756-L75 Firmware 33 Batch Array-Type Cache Before/After

Date: 2026-08-21 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `2c87d11` plus the
benchmark and cache changes described here

## Purpose

The initial batch baseline showed that repeated DINT-array-element reads spent
most of their time probing each base array to distinguish ordinary arrays from
packed Logix `BOOL` arrays. This controlled rerun measures a per-connection
positive/negative array-type cache under the same hardware and workload.

The original record remains unchanged as the
[before baseline](2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md).

## Target and Method

- Controller: ControlLogix `1756-L75`, firmware major revision 33
- Route: `1756-EN2T` in chassis slot 1 to processor in backplane slot 0
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Tags: first 100 controller-scoped DINT array elements in
  [`full_coverage_tags.json`](../../examples/full_coverage_tags.json)
- Logical sizes: 1, 5, 10, 20, 50, and 100 tags
- Sampling floor: 30 seconds and 1,000 tag operations per size/direction,
  after ten read/write warm-up calls
- Packet policy: at most 20 operations and 504 bytes per CIP packet
- Correctness: zero call or per-tag failures; every binding verified all 100
  terminal values as `999999`

The implementation caches both packed-BOOL and non-BOOL classifications,
shares the cache across cloned clients used by the C ABI registry, and clears
it when the route changes or the caller invokes `clear_caches()`. Failed or
malformed probes are not cached.

## Build-Identical Before/After Result

Rust, C#, and C/C++ used release builds in both runs. These comparisons use raw
average read-call latency and derived tag throughput.

| Binding | Size | Before ms | After ms | Latency reduction | Before tags/s | After tags/s | Throughput |
|---|---:|---:|---:|---:|---:|---:|---:|
| Rust | 1 | 6.459 | 2.607 | 59.6% | 154.8 | 383.6 | 2.48x |
| Rust | 5 | 19.939 | 3.857 | 80.7% | 250.8 | 1,296.4 | 5.17x |
| Rust | 10 | 38.098 | 5.039 | 86.8% | 262.5 | 1,984.7 | 7.56x |
| Rust | 20 | 65.208 | 10.047 | 84.6% | 306.7 | 1,990.7 | 6.49x |
| Rust | 50 | 175.245 | 15.130 | 91.4% | 285.3 | 3,304.7 | 11.58x |
| Rust | 100 | 347.000 | 30.257 | 91.3% | 288.2 | 3,305.0 | 11.47x |
| C# | 1 | 5.884 | 2.820 | 52.1% | 169.9 | 354.6 | 2.09x |
| C# | 5 | 20.504 | 4.745 | 76.9% | 243.9 | 1,053.7 | 4.32x |
| C# | 10 | 37.641 | 5.041 | 86.6% | 265.7 | 1,983.9 | 7.47x |
| C# | 20 | 75.065 | 10.082 | 86.6% | 266.4 | 1,983.8 | 7.45x |
| C# | 50 | 174.177 | 15.121 | 91.3% | 287.1 | 3,306.6 | 11.52x |
| C# | 100 | 349.875 | 30.261 | 91.4% | 285.8 | 3,304.6 | 11.56x |
| C/C++ | 1 | 6.503 | 3.203 | 50.7% | 153.8 | 312.2 | 2.03x |
| C/C++ | 5 | 19.051 | 4.692 | 75.4% | 262.4 | 1,065.7 | 4.06x |
| C/C++ | 10 | 36.281 | 5.040 | 86.1% | 275.6 | 1,984.0 | 7.20x |
| C/C++ | 20 | 68.445 | 10.158 | 85.2% | 292.2 | 1,968.9 | 6.74x |
| C/C++ | 50 | 177.835 | 15.127 | 91.5% | 281.2 | 3,305.5 | 11.76x |
| C/C++ | 100 | 329.934 | 30.265 | 90.8% | 303.1 | 3,304.2 | 10.90x |

At size 100, the three build-identical native paths converged at approximately
3,304–3,305 tags/s, compared with 286–303 tags/s before the cache. Average
latency fell by 90.8–91.4%.

## Optimized Rerun Summary

The values below are raw average call latency and tags/s. Python reads use the
same native FFI batch path. Python writes remain grouped sequential operations,
not Multiple Service Packet writes.

| Binding | Size | Read ms | Read tags/s | Write ms | Write tags/s |
|---|---:|---:|---:|---:|---:|
| Rust | 1 | 2.607 | 383.6 | 2.681 | 373.0 |
| Rust | 5 | 3.857 | 1,296.4 | 4.297 | 1,163.5 |
| Rust | 10 | 5.039 | 1,984.7 | 5.041 | 1,983.9 |
| Rust | 20 | 10.047 | 1,990.7 | 10.082 | 1,983.7 |
| Rust | 50 | 15.130 | 3,304.7 | 20.170 | 2,478.9 |
| Rust | 100 | 30.257 | 3,305.0 | 35.330 | 2,830.5 |
| C# | 1 | 2.820 | 354.6 | 2.889 | 346.2 |
| C# | 5 | 4.745 | 1,053.7 | 4.925 | 1,015.2 |
| C# | 10 | 5.041 | 1,983.9 | 5.041 | 1,983.7 |
| C# | 20 | 10.082 | 1,983.8 | 10.082 | 1,983.8 |
| C# | 50 | 15.121 | 3,306.6 | 20.167 | 2,479.3 |
| C# | 100 | 30.261 | 3,304.6 | 35.308 | 2,832.2 |
| C/C++ | 1 | 3.203 | 312.2 | 3.296 | 303.4 |
| C/C++ | 5 | 4.692 | 1,065.7 | 4.934 | 1,013.4 |
| C/C++ | 10 | 5.040 | 1,984.0 | 5.042 | 1,983.2 |
| C/C++ | 20 | 10.158 | 1,968.9 | 10.092 | 1,981.7 |
| C/C++ | 50 | 15.127 | 3,305.5 | 20.171 | 2,478.8 |
| C/C++ | 100 | 30.265 | 3,304.2 | 35.342 | 2,829.5 |
| Python | 1 | 3.054 | 327.4 | 3.253 | 307.4 |
| Python | 5 | 4.896 | 1,021.3 | 16.251 | 307.7 |
| Python | 10 | 5.040 | 1,984.2 | 31.101 | 321.5 |
| Python | 20 | 10.077 | 1,984.7 | 68.347 | 292.6 |
| Python | 50 | 15.120 | 3,306.9 | 186.087 | 268.7 |
| Python | 100 | 30.258 | 3,304.9 | 368.104 | 271.7 |

## Python Artifact Audit

The original Python source-checkout run selected `target/debug` before
`target/release`. The loader now prefers the release artifact, and the rerun
was explicitly pinned to that release library. Python's optimized native-read
result agrees with the other bindings, but its exact before/after percentage is
not attributed solely to the cache because the builds were not identical.

## Conclusions

- Positive and negative array classification caching removes redundant
  network probes from repeated batch reads without changing public APIs.
- The four bindings converge on the native read path at larger sizes, showing
  negligible wrapper overhead for this workload.
- Native write throughput remains near the previous baseline, supporting that
  the change is specifically a read-preparation optimization and did not cause
  a write regression.
- These numbers apply to repeated DINT array elements on this exact target and
  route. Mixed types, long paths, STRINGs, UDTs, CPU/RSS, controller load,
  reconnect behavior, and endurance still require separate characterization.
