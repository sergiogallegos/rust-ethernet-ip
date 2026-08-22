---
id: CODEX-BB
title: Schema-drift eviction and safe read self-healing
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1. Depends on CODEX-BA.**

Make cached array classification self-correct when a stable symbolic name is
deleted/recreated or changes between packed BOOL and an ordinary array while
the connection remains alive.

### Context to read first

- `docs/agents/tasks/CODEX-BA-schema-cache-generation.md`
- `wiki/investigations/array-type-cache-lifecycle.md`
- `src/client/batch_exec.rs`
- `src/client.rs`
- `docs/agents/notes/ab-firmware-quirks.md`

### Required implementation

1. Carry the prepared array path, classification, and schema generation far
   enough into response handling to validate the returned datatype.
2. Evict the affected schema entry on Symbol Not Found, invalid symbolic path,
   structure/type mismatch, or a returned datatype that contradicts the cached
   packed-BOOL classification.
3. For read-only operations, rebuild and retry once after eviction and fresh
   classification. The retry must be bounded and observable.
4. Preserve per-tag batch results. One stale tag must not silently reorder or
   corrupt unrelated results.
5. For writes, reclassify before sending a packed-BOOL read-modify-write when
   stale state is detected. Never replay a write after an ambiguous timeout,
   connection loss, or response-loss condition.
6. Add counters for type contradictions and successful/failed self-healing
   reads. CODEX-BC exposes them across bindings.

### Test requirements

- Same-name transitions without reconnect: `DINT[] -> BOOL[]`, `BOOL[] ->
  DINT[]`, and `DINT[] -> REAL[]`.
- Delete/temporary Symbol Not Found/recreate under the same name.
- Indices below and above 32 to prove packed-BOOL DWORD addressing is rebuilt.
- Controller- and program-scoped paths.
- Single reads and native batch reads.
- A read retries at most once; a write is not replayed after an ambiguous
  transport outcome.
- Partial batch failure and input/result correlation remain correct.
- Full offline Rust and simulator gates pass.

### Acceptance criteria

- No stale packed-BOOL classification can silently select an ordinary-array
  element at `index / 32`, or vice versa.
- Read recovery is bounded to one reclassification attempt.
- Ambiguous writes remain fail-closed.
- Existing warm-cache throughput behavior is preserved within normal benchmark
  noise; no per-call type probe is reintroduced.
- CHANGELOG and cache-lifecycle documentation describe the recovery contract.

### Out of scope

- General automatic PLC project-version discovery.
- Automatic write retry after transport ambiguity.
- Wrapper APIs and diagnostics presentation (CODEX-BC).
- Real-controller editing (CODEX-BD).

## Codex log

## Claude review

## Verdict
