# 1756-L75 Firmware 33 Post-BF/BI Cross-Binding Performance

Date: 2026-08-22 (America/Denver)  
Result: **PASS**  
Repository state: `1.2.1` development line, commit `9da389d`

## Target and Method

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D` firmware `10.007` in chassis slot 1 to processor in
  backplane slot 0
- Host: MacBook Pro (`Mac14,9`), Apple M2 Pro, 10 CPU cores, 16 GB RAM
- Host OS: macOS 26.5.2 (`25F84`), arm64
- Build: optimized `1.2.1` Rust/FFI artifact shared by all four bindings
- Test address: private lab address, intentionally omitted

Two workloads ran serially through Rust, C#, Python, and C/C++:

1. **Full-manifest sequential:** one 2,304-tag read-only preflight, then three
   measured read passes over all 2,304 paths and three write passes over all
   2,285 writeable paths.
2. **Native DINT batch:** logical sizes 1, 5, 10, 20, 50, and 100 over the
   first 100 dedicated controller DINT array elements. Each direction and size
   ran for at least 30 seconds and 1,000 logical tag operations after warm-up.

Every call used a monotonic high-resolution clock. `Filtered avg` applies
Tukey's two-sided `1.5 x IQR` rule; min and max retain every sample. Batch
throughput is logical tags per second, including packet splitting under the
default 20-operation/504-byte policy.

## Full 2,304-Tag Sequential Workload

All 27,648 reads and 27,420 writes succeeded. Each binding completed a
2,304/2,304 preflight.

### Reads

| Binding | Samples | Avg ms | Filtered avg ms | Min ms | Max ms | Ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,912 | 2.805 | 2.771 | 0.588 | 235.880 | 356.5 | 0 |
| C# | 6,912 | 2.857 | 2.857 | 0.600 | 9.474 | 350.0 | 0 |
| Python | 6,912 | 3.832 | 4.789 | 0.592 | 51.235 | 260.9 | 0 |
| C/C++ | 6,912 | 2.830 | 2.829 | 0.600 | 10.446 | 353.3 | 0 |

Python's two-sided filter removes unusually fast samples from its multi-modal
read distribution, so its filtered average is higher than the raw average.
Rust's 235.880 ms maximum is one isolated tail event; its filtered average is
2.771 ms.

### Writes

| Binding | Samples | Avg ms | Filtered avg ms | Min ms | Max ms | Ops/s | Failures |
|---|---:|---:|---:|---:|---:|---:|---:|
| Rust | 6,855 | 5.724 | 5.090 | 0.635 | 15.573 | 174.7 | 0 |
| C# | 6,855 | 5.648 | 5.091 | 0.652 | 236.605 | 177.1 | 0 |
| Python | 6,855 | 5.752 | 5.095 | 0.650 | 15.563 | 173.9 | 0 |
| C/C++ | 6,855 | 6.284 | 6.245 | 0.643 | 241.338 | 159.1 | 0 |

C# and C/C++ each observed one approximately 237–241 ms write tail. Rust,
C#, and Python otherwise converge at a filtered average near 5.09 ms. C/C++
was slower in this run at 6.245 ms filtered; the public behavior remained
correct with zero failures.

## Native DINT Batch Workload

Every size/direction completed with zero failures. Each binding passed its
2,304/2,304 preflight and final 100/100 terminal-value verification.

### Batch reads

| Binding | Size | Samples | Avg ms | Min ms | Max ms | Tags/s |
|---|---:|---:|---:|---:|---:|---:|
| Rust | 1 | 9,094 | 3.298 | 0.581 | 11.128 | 303.2 |
| Rust | 5 | 6,383 | 4.700 | 0.751 | 10.232 | 1,063.9 |
| Rust | 10 | 5,927 | 5.061 | 2.938 | 10.738 | 1,975.8 |
| Rust | 20 | 2,960 | 10.136 | 7.085 | 29.556 | 1,973.2 |
| Rust | 50 | 1,956 | 15.337 | 12.906 | 25.312 | 3,260.2 |
| Rust | 100 | 980 | 30.611 | 29.496 | 40.526 | 3,266.8 |
| C# | 1 | 10,489 | 2.858 | 0.602 | 10.011 | 349.9 |
| C# | 5 | 6,483 | 4.628 | 0.744 | 10.427 | 1,080.5 |
| C# | 10 | 5,931 | 5.058 | 2.999 | 10.674 | 1,977.2 |
| C# | 20 | 2,965 | 10.117 | 8.255 | 15.500 | 1,976.8 |
| C# | 50 | 1,958 | 15.328 | 11.807 | 25.221 | 3,262.1 |
| C# | 100 | 980 | 30.617 | 27.443 | 40.856 | 3,266.2 |
| Python | 1 | 8,444 | 3.550 | 0.619 | 10.923 | 281.7 |
| Python | 5 | 6,131 | 4.890 | 0.766 | 10.428 | 1,022.4 |
| Python | 10 | 5,934 | 5.052 | 2.721 | 10.609 | 1,979.3 |
| Python | 20 | 2,964 | 10.119 | 7.474 | 15.504 | 1,976.4 |
| Python | 50 | 1,958 | 15.320 | 14.233 | 25.620 | 3,263.6 |
| Python | 100 | 978 | 30.669 | 29.402 | 40.736 | 3,260.6 |
| C/C++ | 1 | 10,423 | 2.878 | 0.590 | 10.936 | 347.4 |
| C/C++ | 5 | 6,526 | 4.597 | 0.756 | 10.368 | 1,087.6 |
| C/C++ | 10 | 5,923 | 5.065 | 3.659 | 10.420 | 1,974.3 |
| C/C++ | 20 | 2,965 | 10.118 | 8.908 | 15.871 | 1,976.6 |
| C/C++ | 50 | 1,956 | 15.339 | 13.004 | 25.455 | 3,259.6 |
| C/C++ | 100 | 981 | 30.588 | 29.459 | 40.459 | 3,269.3 |

### Batch writes

| Binding | Size | Samples | Avg ms | Min ms | Max ms | Tags/s |
|---|---:|---:|---:|---:|---:|---:|
| Rust | 1 | 8,611 | 3.483 | 0.599 | 10.378 | 287.1 |
| Rust | 5 | 6,153 | 4.876 | 0.787 | 11.355 | 1,025.5 |
| Rust | 10 | 5,927 | 5.061 | 3.077 | 10.518 | 1,976.1 |
| Rust | 20 | 2,961 | 10.131 | 6.708 | 18.818 | 1,974.2 |
| Rust | 50 | 1,473 | 20.376 | 17.777 | 25.943 | 2,453.8 |
| Rust | 100 | 839 | 35.767 | 33.602 | 45.898 | 2,795.9 |
| C# | 1 | 10,165 | 2.951 | 0.620 | 10.456 | 338.9 |
| C# | 5 | 6,044 | 4.963 | 0.809 | 14.121 | 1,007.4 |
| C# | 10 | 5,912 | 5.074 | 1.556 | 10.706 | 1,970.8 |
| C# | 20 | 2,956 | 10.148 | 8.470 | 20.512 | 1,970.8 |
| C# | 50 | 1,472 | 20.391 | 18.178 | 30.477 | 2,452.1 |
| C# | 100 | 838 | 35.804 | 32.576 | 45.782 | 2,793.0 |
| Python | 1 | 9,339 | 3.223 | 0.634 | 232.284 | 310.2 |
| Python | 5 | 5,994 | 4.999 | 0.837 | 10.274 | 1,000.3 |
| Python | 10 | 5,923 | 5.057 | 2.148 | 10.657 | 1,977.3 |
| Python | 20 | 2,962 | 10.120 | 7.969 | 15.687 | 1,976.3 |
| Python | 50 | 1,473 | 20.358 | 18.791 | 30.543 | 2,456.1 |
| Python | 100 | 838 | 35.808 | 32.569 | 45.958 | 2,792.7 |
| C/C++ | 1 | 10,265 | 2.922 | 0.612 | 237.104 | 342.2 |
| C/C++ | 5 | 6,170 | 4.863 | 0.766 | 10.455 | 1,028.1 |
| C/C++ | 10 | 5,921 | 5.067 | 3.557 | 10.609 | 1,973.6 |
| C/C++ | 20 | 2,961 | 10.136 | 7.781 | 15.466 | 1,973.3 |
| C/C++ | 50 | 1,473 | 20.373 | 19.348 | 30.551 | 2,454.2 |
| C/C++ | 100 | 838 | 35.809 | 34.126 | 45.768 | 2,792.6 |

## Conclusions

- At size 100, all four bindings converge within `0.082 ms` on average read
  latency and within `0.042 ms` on average write latency.
- Size-100 throughput is 3,261–3,269 read tags/s and 2,793–2,796 write tags/s.
- Python's post-CODEX-BF native-safe DINT write throughput matches the native
  Rust, C#, and C/C++ paths at sizes 10 through 100. No fallback operation is
  included in these DINT-only batch numbers.
- Size 10 and 20 have similar tags/s because the default packet policy carries
  at most 20 operations per packet. Sizes 50 and 100 use multiple packets per
  logical call and amortize wrapper/call overhead further.
- Isolated maxima near 232–241 ms appeared in several size-1 or sequential
  windows. Raw averages and maxima are retained; filtered averages in the JSON
  artifacts prevent those tails from being mistaken for the steady state.

## Reproduction and Raw Evidence

Sequential runners used:

```text
--plc-slot 0 --benchmark-passes 3 --allow-writes \
  --out-dir examples/full_coverage_results/2026-08-22_post-bi-bf-performance
```

Batch runners used:

```text
--plc-slot 0 --batch-benchmark --batch-min-tag-operations 1000 \
  --batch-min-seconds 30 --allow-writes \
  --out-dir examples/full_coverage_results/2026-08-22_post-bi-bf-batch
```

The eight generated JSON artifacts are gitignored local evidence. All report
`PASS`; the four batch artifacts report terminal verification `100/100`. The
dedicated controller fixtures remain at terminal value `999999`.
