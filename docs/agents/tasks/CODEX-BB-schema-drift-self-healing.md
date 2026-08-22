---
id: CODEX-BB
title: Schema-drift eviction and safe read self-healing
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
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

### 2026-08-22 10:40  codex [GPT-5]

Implementation started after CODEX-BA passed its required offline Rust gate.
Work is proceeding against the generation-scoped cache and must preserve batch
input/result correlation while bounding read recovery to one retry.

### 2026-08-22 12:25  codex [GPT-5]

Submitted response-driven schema recovery. Single and native-batch array reads
carry or consult generation-scoped classification, evict contradictory or
invalid paths, and retry only the affected logical read once. Packed-BOOL
writes can reclassify during their pre-write read, but no sent write is
replayed. The dynamic simulator now supports same-name array replacement,
temporary deletion, and request counts. Coverage includes DINT[] to/from
BOOL[], DINT[] to REAL[], controller/program paths, indices below/above 32,
batch correlation, bounded Symbol Not Found recovery, and fail-closed writes.
Focused simulator tests and formatting pass; the consolidated workspace gate
will run after CODEX-BC/BD integration.

## Claude review

### 2026-08-22 16:50  claude [Sonnet 5]

**Independent verification**
- Same full-matrix run as CODEX-BA (shared tree). Targeted reruns: `client::batch_exec::tests::generation_change_and_symbol_errors_are_recoverable_for_reads_only`, `::batch_reply_detects_both_packed_bool_transition_directions`, `::ordinary_array_type_change_dint_to_real_stays_addressing_compatible` — all pass individually.
- `scripts/schema-change-gate` (Rust leg: `cargo test --features ffi --test schema_drift_recovery_tests`) — 7/7, exercising this logic end-to-end through a real simulator TCP connection.

**What's being fixed**
- A cached packed-BOOL/ordinary-array classification could go stale under a live same-name schema change; this makes reads self-heal (bounded, generation-aware) and keeps writes fail-closed.

**Root cause confirmation**
- Confirmed: before this change, cached array classification had no path to detect that a returned response contradicted it — a stale `is_packed_bool=true` entry would keep selecting `index/32` DWORD addressing against what is now an ordinary array, or vice versa.

**Fix appropriateness**
- `read_tag` (`src/client.rs:1406`) wraps `read_tag_once` (`src/client.rs:1444`): on a drift-shaped error (`is_schema_drift_read_error`, `src/client.rs:1310`, matching `DataTypeMismatch` or CIP 0x04/0x05/0x16/0x2107/"expected bool array dword") it evicts only the affected array path (`evict_array_type_cache_entry`, `src/client.rs:1286`) and retries exactly once — no loop, no unbounded retry.
- Detection is response-driven, not speculative: the returned CIP datatype is checked against the cached classification and a contradiction returns `DataTypeMismatch` — recovery only fires on an actual observed mismatch or a symbolic-path failure, matching the brief's "contradicts the cached packed-BOOL classification" requirement precisely rather than reclassifying speculatively on every read.
- Batch preserves correlation: `execute_batch`'s result loop (`src/client/batch_exec.rs:490`) only reassigns `result` for the specific index whose `prepared_read_needs_schema_recovery` (`src/client/batch_exec.rs:970`) returns true; every other index's result is untouched. That function triggers on a generation mismatch or a `DataTypeMismatch`/`CipError{0x04|0x05|0x16|0xFF}` and explicitly only for `BatchOperation::Read`, never `Write` — the mechanism proving writes aren't silently retried through this same path.
- Packed-BOOL writes reclassify safely: in `build_batch_service_request`'s write branch (`src/client/batch_exec.rs:588` area) and the single-write equivalent (`write_bool_array_element_workaround`, `src/client.rs:2236`), a drift error on the *pre-write DWORD read* (not the write itself) triggers eviction + reclassification + a retried *read* before the write request is even constructed — the actual write is built and sent exactly once regardless. No code path re-sends a write after a timeout, connection-loss, or response-loss condition — `write_tag`/`write_tag_direct` have no retry wrapper at all, which is the correct fail-closed shape per the brief.

**Test proof**
- Below/above DWORD-boundary and both scope levels are exercised by `tests/schema_drift_recovery_tests.rs`: `controller_dint_array_to_bool_array_recovers_below_and_above_32` (indices 5 and 40, controller scope), `program_bool_array_to_dint_array_recovers_above_32` (program scope, index 40), `compatible_dint_array_to_real_array_needs_no_special_retry` (DINT→REAL, addressing-compatible, confirms exactly one extra read via `sim.read_count`), `deleted_tag_read_retries_once_then_recovers_after_recreation` (Symbol Not Found window, asserts exactly "initial read plus one retry" — i.e., bounded), `batch_recovers_only_changed_read_and_preserves_result_correlation` (proves the untouched-index claim above through a real batch call), `failed_write_is_never_replayed_after_send` (asserts `sim.write_count == 1` after a failed write). That file, and the `SimulatedPlc` mutation helpers it depends on in `tests/plc_sim.rs` (`replace_with_dint_array`/`replace_with_bool_array`/`replace_with_real_array`/`remove_tag`), are committed alongside CODEX-BD's offline deliverables rather than in this commit, since no other BA/BB test calls those helpers — grep confirms `schema_drift_recovery_tests.rs` is their only caller. The behavior is still fully proven on the merged tree; it's a commit-attribution note, not a gap in coverage.
- In-tree unit tests (`batch_reply_detects_both_packed_bool_transition_directions`, `ordinary_array_type_change_dint_to_real_stays_addressing_compatible`, `generation_change_and_symbol_errors_are_recoverable_for_reads_only`) prove the same contracts at the response-parsing layer without a live simulator, and do live in this commit.

**Residual risk**
- Same interleaving note as BA — `src/client.rs`/`batch_exec.rs` reviewed and merged as one change with BA.
- BB's own "same-name transitions without reconnect" / "simulator gate" acceptance criterion is proven by `tests/schema_drift_recovery_tests.rs`, which lands in the CODEX-BD commit rather than this one (see Test proof). Both commits are part of the same push, so the claim isn't left unproven in the tree as a whole.
- No live-PLC proof of the exact CIP status codes a real 1756-L75 returns for these transitions — deferred to CODEX-BD's live session by design.

**Strong points (✅)**
- The read/write asymmetry (bounded retry for reads, zero retry for writes, ever) is implemented as a structural fact (different code paths, not a flag), which is exactly what "fail-closed" should look like — it's not something a caller or config could accidentally disable.
- `prepared_read_needs_schema_recovery` explicitly checking that the operation is a read first is a clean, hard-to-regress guard against ever retrying a batch write.
- Diagnostics counters (`schema_type_contradictions`, `schema_read_recoveries_succeeded/failed`) are incremented at every eviction/retry site consistently, giving CODEX-BC's diagnostics real signal rather than best-effort counting.

**Findings**
- 🟢 `is_schema_drift_read_error`'s string-matching on `EtherNetIpError`'s `Display` output (`src/client.rs:1310`) is a little fragile stylistically (couples to error message text), but it's confirmed correct against `error.rs:76`'s `"CIP error 0x{code:02X}: {message}"` format and is the pragmatic choice given the CIP variant doesn't carry a raw status byte at this call site. Not blocking.
- 🟡 No real concerns beyond the cross-commit test-file attribution already called out under Residual risk.

**Acceptance criteria tally**
- ✅ No stale packed-BOOL classification can silently select the wrong element/DWORD — proven at both response-parsing and end-to-end levels.
- ✅ Read recovery bounded to one reclassification attempt — structural (no loop), proven by `deleted_tag_read_retries_once_then_recovers_after_recreation`'s exact read-count assertion.
- ✅ Ambiguous writes remain fail-closed — no retry-after-send path exists anywhere in the write call graph.
- ✅ Partial batch failure / correlation preserved — proven by `batch_recovers_only_changed_read_and_preserves_result_correlation`.
- ✅ Full offline Rust and simulator gates pass — true on the full pushed tree; the specific simulator-level proof lives in the CODEX-BD commit landing alongside this one.
- ✅ CHANGELOG and cache-lifecycle documentation describe the recovery contract.

## Verdict

Merged (`src/client.rs` + `src/client/batch_exec.rs`, combined with CODEX-BA in one commit — physically interleaved at the hunk level). Drift detection, bounded read recovery, and fail-closed writes are all real and independently traced through the code, not just claimed by the Codex log. Process note, not a defect: this task's own "simulator gate" acceptance criterion is proven by test files committed alongside CODEX-BD (`tests/plc_sim.rs` mutation helpers, `tests/schema_drift_recovery_tests.rs`) rather than in BB's own commit — both land in the same push.
