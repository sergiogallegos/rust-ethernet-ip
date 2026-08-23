# 1756-L75 Firmware 33 Cross-Binding Batch Performance

Date: 2026-08-21 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `2c87d11` plus the local
benchmark-runner changes described below

> Historical before-optimization baseline. The controlled array-type-cache
> rerun is recorded in
> [the before/after validation](2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md).
> Keep this record for traceability; do not use it as the current expected
> batch-read performance.

## Target and Workload

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D` firmware `10.007` in chassis slot 1 to processor in backplane slot 0
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Build: optimized `1.2.1` development-line Rust/FFI build for Rust, C#, and
  C/C++. A post-run artifact audit found that Python's source-checkout loader
  selected `target/debug` ahead of `target/release`; the Python row remains
  valid historical evidence but is excluded from build-identical attribution.
- Tags: the first 100 dedicated controller-scoped DINT array elements from
  [`full_coverage_tags.json`](../../examples/full_coverage_tags.json)
- Logical batch sizes: 1, 5, 10, 20, 50, and 100 tags
- Default packet policy: at most 20 operations and 504 bytes per CIP packet;
  one logical size-50 or size-100 call may therefore use several packets

The test address is intentionally omitted. Every binding first completed the
2,304-tag read-only preflight. Each size and direction then ran for at least 30
seconds and at least 1,000 tag operations, after ten read/write warm-up calls.
Every write used the terminal value `999999`; no random values were introduced.

## API Equivalence

- Rust reads and writes use the native Rust batch engine.
- C# reads and writes use native `ExecuteBatch`.
- C/C++ reads use `eip_read_tags_batch`; writes use `eip_execute_batch`.
- Python reads use the native batch-read FFI. Python `write_tags()` in 1.2.x is
  intentionally a grouped public API implemented as sequential native writes,
  so its write results are reported separately and must not be described as
  Multiple Service Packet throughput.

All batch calls and per-tag results succeeded. A final 100-tag terminal-value
verification was added to every runner after this characterization; the same
100 elements were then verified on the controller as `999999`.

## Outlier Treatment

Raw measurements are retained. The tables include both the ordinary average
and a robust average after excluding values outside Tukey's two-sided `1.5 ×
IQR` fences. Min, p50, p95, p99, and max always use the complete sample set.
Because the filter is two-sided, it can remove unusually fast calls as well as
slow calls; consequently a filtered average can occasionally be higher than
the raw average. The median and percentiles should be preferred when comparing
the skewed or multi-modal latency distributions.

In each compact latency column below, values are ordered:
`average / filtered-average / minimum / p50 / p95 / p99 / maximum`, in ms.

## Rust

| Size | Samples R/W | Read latency ms | Read tags/s | Write latency ms | Write tags/s |
|---:|---:|---|---:|---|---:|
| 1 | 4,644 / 9,169 | 6.459 / 6.410 / 2.831 / 5.182 / 10.125 / 10.404 / 236.299 | 154.8 | 3.271 / 3.271 / 0.621 / 4.310 / 5.134 / 5.375 / 8.702 | 305.7 |
| 5 | 1,505 / 6,120 | 19.939 / 19.262 / 14.809 / 19.943 / 29.851 / 30.382 / 30.813 | 250.8 | 4.901 / 5.029 / 0.794 / 5.015 / 5.323 / 5.607 / 7.092 | 1,020.2 |
| 10 | 788 / 5,950 | 38.098 / 36.567 / 25.886 / 35.509 / 50.390 / 55.470 / 262.452 | 262.5 | 5.041 / 5.038 / 1.532 / 5.033 / 5.354 / 5.604 / 9.489 | 1,983.7 |
| 20 | 461 / 2,975 | 65.208 / 63.337 / 56.351 / 65.084 / 75.749 / 85.508 / 287.474 | 306.7 | 10.084 / 10.080 / 8.270 / 10.076 / 10.446 / 10.641 / 15.558 | 1,983.4 |
| 50 | 172 / 1,488 | 175.245 / 172.316 / 141.045 / 171.458 / 211.507 / 236.796 / 266.505 | 285.3 | 20.165 / 20.157 / 17.508 / 20.153 / 20.557 / 20.810 / 24.886 | 2,479.6 |
| 100 | 87 / 850 | 347.000 / 340.753 / 292.355 / 337.388 / 427.484 / 453.170 / 484.117 | 288.2 | 35.293 / 35.284 / 34.395 / 35.281 / 35.709 / 35.893 / 40.663 | 2,833.4 |

## C#

| Size | Samples R/W | Read latency ms | Read tags/s | Write latency ms | Write tags/s |
|---:|---:|---|---:|---|---:|
| 1 | 5,097 / 8,261 | 5.884 / 5.125 / 2.705 / 5.114 / 9.894 / 10.192 / 11.538 | 169.9 | 3.630 / 3.629 / 0.595 / 4.454 / 5.220 / 5.445 / 11.306 | 275.5 |
| 5 | 1,463 / 6,124 | 20.504 / 19.688 / 13.461 / 20.043 / 29.976 / 30.452 / 247.656 | 243.9 | 4.894 / 5.031 / 0.794 / 5.019 / 5.279 / 5.479 / 10.888 | 1,021.6 |
| 10 | 797 / 5,951 | 37.641 / 36.459 / 28.430 / 35.447 / 50.388 / 55.428 / 56.338 | 265.7 | 5.040 / 5.038 / 2.410 / 5.034 / 5.398 / 5.659 / 7.482 | 1,984.2 |
| 20 | 400 / 2,975 | 75.065 / 74.138 / 60.180 / 70.977 / 95.311 / 106.200 / 303.208 | 266.4 | 10.085 / 10.078 / 7.124 / 10.079 / 10.487 / 10.813 / 16.269 | 1,983.2 |
| 50 | 173 / 1,487 | 174.177 / 170.694 / 146.111 / 166.397 / 206.894 / 231.540 / 453.793 | 287.1 | 20.175 / 20.159 / 18.325 / 20.162 / 20.593 / 20.899 / 25.497 | 2,478.4 |
| 100 | 86 / 850 | 349.875 / 342.420 / 297.502 / 343.090 / 433.318 / 453.301 / 574.224 | 285.8 | 35.325 / 35.284 / 33.568 / 35.273 / 35.898 / 36.918 / 40.578 | 2,830.9 |

## C/C++

| Size | Samples R/W | Read latency ms | Read tags/s | Write latency ms | Write tags/s |
|---:|---:|---|---:|---|---:|
| 1 | 4,613 / 8,806 | 6.503 / 6.403 / 2.545 / 5.193 / 10.113 / 10.339 / 236.651 | 153.8 | 3.407 / 3.407 / 0.615 / 4.339 / 5.160 / 5.341 / 6.982 | 293.5 |
| 5 | 1,575 / 6,175 | 19.051 / 18.574 / 14.749 / 19.532 / 25.497 / 30.345 / 31.578 | 262.4 | 4.858 / 5.028 / 0.795 / 5.010 / 5.290 / 5.473 / 18.477 | 1,029.2 |
| 10 | 827 / 5,952 | 36.281 / 35.691 / 25.786 / 35.212 / 45.682 / 55.376 / 267.677 | 275.6 | 5.041 / 5.038 / 2.047 / 5.034 / 5.337 / 5.514 / 7.969 | 1,983.9 |
| 20 | 439 / 2,975 | 68.445 / 67.161 / 56.061 / 65.763 / 80.562 / 90.935 / 105.371 | 292.2 | 10.086 / 10.078 / 8.042 / 10.077 / 10.508 / 11.033 / 15.156 | 1,982.9 |
| 50 | 169 / 1,487 | 177.835 / 173.568 / 141.132 / 166.136 / 232.165 / 256.844 / 418.651 | 281.2 | 20.176 / 20.159 / 18.685 / 20.156 / 20.517 / 20.729 / 25.443 | 2,478.1 |
| 100 | 91 / 850 | 329.934 / 319.621 / 297.394 / 322.609 / 413.133 / 423.492 / 478.353 | 303.1 | 35.330 / 35.282 / 32.172 / 35.289 / 35.771 / 38.043 / 40.526 | 2,830.5 |

## Python

Python reads below are native batch reads. Writes are the public grouped-write
API's sequential native operations.

| Size | Samples R/W | Read latency ms | Read tags/s | Grouped-write latency ms | Write tags/s |
|---:|---:|---|---:|---|---:|
| 1 | 4,624 / 7,416 | 6.485 / 6.485 / 4.386 / 5.211 / 10.135 / 10.444 / 11.856 | 154.2 | 4.039 / 4.827 / 0.646 / 4.830 / 5.303 / 5.549 / 8.878 | 247.6 |
| 5 | 1,324 / 1,362 | 22.665 / 22.654 / 14.809 / 20.469 / 30.436 / 30.799 / 37.137 | 220.6 | 22.019 / 22.400 / 10.655 / 24.779 / 25.569 / 25.868 / 29.806 | 227.1 |
| 10 | 733 / 704 | 40.958 / 40.958 / 29.601 / 40.360 / 51.100 / 55.517 / 56.097 | 244.2 | 42.612 / 42.612 / 24.960 / 45.162 / 50.677 / 50.912 / 51.805 | 234.7 |
| 20 | 358 / 396 | 83.873 / 83.873 / 60.157 / 81.101 / 106.037 / 111.349 / 111.703 | 238.5 | 75.748 / 75.748 / 50.518 / 75.673 / 95.649 / 100.687 / 105.576 | 264.0 |
| 50 | 198 / 158 | 151.913 / 151.913 / 141.162 / 151.253 / 161.299 / 162.424 / 166.588 | 329.1 | 190.252 / 190.252 / 135.083 / 186.570 / 236.967 / 246.777 / 247.634 | 262.8 |
| 100 | 98 / 70 | 307.632 / 304.996 / 287.290 / 302.950 / 337.397 / 361.660 / 367.807 | 325.1 | 433.181 / 440.477 / 282.999 / 438.969 / 499.306 / 503.439 / 503.668 | 230.9 |

## Conclusions

- Native batch writes scaled from roughly 276–306 tags/s at size 1 to about
  2,830–2,833 tags/s at size 100: approximately a 9–10× throughput increase.
- Native batch-write results were effectively identical across Rust, C#, and
  C/C++, showing that wrapper overhead was small relative to network/controller
  time for this workload.
- Native batch reads rose from roughly 154–170 tags/s at size 1 to 286–325
  tags/s at size 100. The improvement is real but much smaller than writes.
- The current batch read preparation probes an array path to identify packed
  BOOL arrays before forming requests. Because this workload deliberately uses
  DINT array elements, the measured read result includes that current type-
  detection cost. Caching positive and negative array-type detection is a
  promising optimization to benchmark separately before changing published
  performance expectations.
- Python's public grouped writes stayed around 227–264 tags/s as size grew,
  which is expected for sequential native writes and confirms that this column
  is not a native batch-write result.

These results characterize repeated DINT array-element traffic only. Mixed
types, STRINGs, UDT members, long symbolic paths, bytes/second, client CPU/RSS,
controller task load, reconnect performance, and 24-hour behavior remain
separate measurements.
