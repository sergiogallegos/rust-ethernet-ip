---
id: CODEX-BF
title: Python native batch writes with safe typed fallbacks
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Post-1.2.1 performance follow-up; not a release blocker. Depends on
CODEX-BB/BC for the final schema and diagnostics contract.**

Python `write_tags()` currently issues singleton native batches. On the
1756-L75 firmware 33 size-100 workload this measured about 272 tags/s versus
about 2,830 tags/s for native Rust/C#/C++ batch writes. Add a genuine native
batch path where semantics are proven, retaining safe typed fallbacks for
operations that cannot share one Multiple Service Packet contract.

### Required work

1. Define which values and paths are native-batch-safe: atomic scalars, arrays,
   program scope, packed BOOL, STRING/custom STRING, and UDT/member cases must
   each have an explicit disposition.
2. Route safe items through one native batch call while preserving input/result
   correlation and per-item errors.
3. Keep handle-aware STRING, UDT read-modify-write, and packed-BOOL safety
   behavior correct; split/fallback rather than silently weakening semantics.
4. Preserve the public `write_tags()` result shape and document whether result
   execution order can differ from input order.
5. Add benchmark output that labels native batched versus sequential fallback
   operations separately.

### Test requirements

- Atomic DINT/REAL/BOOL and array batches.
- Controller/program scope and indices above 32 for packed BOOL.
- Built-in/custom STRING and UDT/member disposition tests.
- Mixed valid/invalid values, partial failure, duplicate tag names, result
  ordering/correlation, and terminal read-back.
- C ABI contract tests and Python unit/simulator integration tests.
- Controlled hardware comparison against the retained sequential baseline.

### Acceptance criteria

- Safe atomic Python size-100 writes use native MSP batching.
- Unsupported/special writes retain correct fallback behavior and labeling.
- Zero duplicate writes and zero false-success results.
- Documentation and benchmarks never describe fallback operations as native
  batch throughput.

### Out of scope

- Changing C# or C++ batch APIs.
- Automatically retrying ambiguous writes.
- Removing the sequential fallback.

## Codex log

## Claude review

## Verdict
