# Hardware Validation Program

## Summary

The user-facing matrix and test protocol live in
[`docs/HARDWARE_COMPATIBILITY.md`](../../docs/HARDWARE_COMPATIBILITY.md). This
page records the maintainer interpretation behind it.

## Current Understanding

- `confirmed`: Five exact processor/firmware combinations have physical
  evidence. The 5069-L330ERM fw38 is the `1.2.0` release gate, and the
  1756-L75 fw33 now has a four-binding functional and sequential-performance
  baseline on the `1.2.1` development line.
- A `Done` cell means an authoritative validation file exists for that exact
  binding and target; blank cells are invitations, not implied failures.
- Functional, endurance, and performance claims are separate. Passing one
  full-coverage run does not establish 24-hour stability or a universal
  throughput number.
- Performance records must include topology, sample count, latency
  distribution, errors, and resource/controller impact to support comparison.
- Any write-heavy contribution must identify a test-only program, starting
  values, and final restore/settle state.
- `confirmed`: The cross-binding companion gate now separates batch,
  whole-UDT, and discovery evidence from full-tag inventory coverage. Live
  mode requires explicit write opt-in and restores four dedicated DINT array
  elements after each binding.
- `confirmed`: On the 1756-L75/B fw33.011 through a 1756-EN2T/D fw10.007, three passes per
  direction completed 27,648 reads and 27,420 writes across the four bindings
  with zero failures. Median sequential latency was 5.070–5.579 ms; this is a
  heterogeneous manifest baseline, not a universal per-tag claim.
- `confirmed`: Repeating that sequential workload after the 1.2.1
  schema-cache sequence produced zero failures and observed average read
  latency 49–58% below the prior-day baseline across Rust, C#, Python, and
  C/C++. This is a cross-day observation, not isolated causal proof.
- `confirmed`: The same target has native batch distributions at logical sizes
  1, 5, 10, 20, 50, and 100. Native DINT writes reached about 2,830 tags/s at
  size 100 across Rust, C#, and C/C++. Python subsequently moved its safe
  atomic subset to the same native engine and reached 2,773 tags/s at size 100,
  up 10.21x from its controlled sequential baseline. STRING/UDT, member/bit,
  packed-BOOL-array, and duplicate-name writes remain typed fallbacks and are
  excluded from that throughput claim.
- `confirmed`: Positive/negative packed-BOOL array classification is now cached
  per connection, shared across FFI clones, and invalidated on route changes or
  explicit cache clearing.
  The controlled rerun reached approximately 3,305 size-100 DINT reads/s in all
  four bindings. Build-identical Rust/C#/C++ comparisons improved 10.9–11.6x
  while native write throughput remained stable.
- `confirmed`: the schema-change gate passes dynamic same-name mutation and
  explicit refresh across Rust, C ABI, C#, Python, and C++ using one release
  artifact, and is hardware-validated on a live 1756-L75 firmware-33: array
  schema-swap (both directions, both scopes, all four bindings), UDT
  layout-edit/download with session-survival confirmed, and the
  post-schema full-coverage/batch regression, all PASS with zero
  anomalies.

## Evidence

- [Hardware compatibility and test program](../../docs/HARDWARE_COMPATIBILITY.md)
- [Result template](../../docs/validation/REAL_HARDWARE_RESULT_TEMPLATE.md)
- [Cross-binding feature gate](../../docs/validation/CROSS_BINDING_FEATURE_GATE.md)
- [1756-L75 fw33 performance baseline](../../docs/validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md)
- [1756-L75 fw33 batch performance](../../docs/validation/2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md)
- [1756-L75 fw33 array-cache before/after](../../docs/validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md)
- [1756-L75 fw33 Python native batch writes](../../docs/validation/2026-08-22_1756-L75_fw33_python-native-batch-writes.md)
- [Schema-change gate procedure](../../docs/validation/SCHEMA_CHANGE_GATE.md)
- [1756-L75 fw33 schema-change record (live PASS)](../../docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md)
- [1.2.0 validation synthesis](../releases/1.2.0-validation-synthesis.md)

## Open Questions

- Select a second routinely available release-gate processor.
- Sweep batch packet policy on the 1756-L75 target before changing the
  conservative 20-operation/504-byte default. Compare packet sizes and
  operation limits while recording rejections, latency, and controller impact.
- Characterize cold-cache versus warm-cache reads, different tag shapes and
  scopes, and client/controller CPU and memory impact on the 1756-L75 target.
- Add the first 24-hour result with latency percentiles, data-gap detection,
  RSS/CPU trend, and reconnect accounting.

## Related Pages

- [firmware-behavior.md](firmware-behavior.md)
- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
