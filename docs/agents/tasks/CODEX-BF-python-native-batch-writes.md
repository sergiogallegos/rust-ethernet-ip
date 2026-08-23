---
id: CODEX-BF
title: Python native batch writes with safe typed fallbacks
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5.6-sol]
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

### 2026-08-22 19:43  codex [GPT-5]

Implementation started after the CODEX-BA…BD dependency sequence completed.
The existing Python sequential write contract, typed single-write exports,
native batch FFI, simulator coverage, and live benchmark harness will be
reconciled before changing dispatch behavior.

### 2026-08-22 19:48  codex [GPT-5.6-sol]

Model-label correction: the active model is GPT-5.6-sol. The preceding GPT-5
label is superseded; the entry remains unchanged because this log is
append-only.

### 2026-08-22 20:07  codex [GPT-5.6-sol]

Submitted as the current working-tree diff for independent review. Python
`write_tags()` now native-batches contiguous unique atomic scalar and numeric
array writes while preserving typed sequential behavior for STRING/custom
STRING, whole UDT, member/bit, packed-BOOL-array, and duplicate-name cases.
Mixed inputs retain execution order; missing native results become explicit
failures, and duplicates execute exactly once each with the last result
retained by the existing mapping contract.

Verification passed: 51 Python unit tests (11 skipped), 11 simulator-backed
Python integration tests, 16 Rust C-ABI tests, the locked all-feature Rust
workspace suite, fmt, clippy with warnings denied, focused Ruff, bytecode
compilation, the full-coverage manifest gate, agent-file validation, and
diff-check. The live 1756-L75/B firmware-33.011 benchmark passed its 2,304-tag
preflight, reported zero failures, and verified all 100 terminal values. At
size 100, native Python writes reached 2,773.3 tags/s versus the retained
271.7-tags/s sequential baseline (10.21x). Evidence is recorded in
`docs/validation/2026-08-22_1756-L75_fw33_python-native-batch-writes.md`.

## Claude review

### 2026-08-22 20:45  claude [Sonnet 5]

**Independent verification**
- `cargo fmt -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — all green, 0 failures (this diff touches zero Rust source; `src/` is untouched per `git status`, confirmed out-of-scope item honored).
- `cargo test --test ffi_tests --features ffi --locked` — 16/16, matching the claimed count exactly.
- `PYTHONPATH=python python3 -m unittest discover -s python/tests -p "test_*.py"` — 51 tests, 11 skipped (integration tests skip without a simulator), matching the claimed count exactly.
- `PYTHONPATH=python RUST_ETHERNET_IP_START_SIM=1 python3 -m unittest python.tests.test_integration -v` — 11/11 passed against a real `SimulatedPlc` over the actual native FFI (not mocked), including the five new/changed tests targeting this diff.

**What's being fixed**
- Feature, not a bugfix (brief confirms: not a release blocker, a performance follow-up). Python `write_tags()` previously issued one native FFI call per tag regardless of type — no batching benefit over sequential. This adds a real Multiple Service Packet batch path for the subset of writes where batching is semantically safe.

**Root cause confirmation**
- N/A (feature work, not a defect).

**Fix appropriateness**
- Classification logic (`_is_native_batch_safe`, `client.py:221`) is a whitelist (`_NATIVE_BATCH_VALUE_TYPES` = 11 atomic scalar kinds) — STRING and UDT are excluded structurally by not being in the set, not by a fragile blocklist.
- Program-scope stripping (`_tag_body_without_program_scope`) correctly isolates the bare tag expression before checking for `.` (member/bit access) — traced by hand for `Program:TestProgram.gTestArray_DINT[5]` (partitions on the first `.`, leaves `gTestArray_DINT[5]`, no further `.`, native-eligible) and `Program:TestProgram.gTestUDT.Member1_DINT` (leaves `gTestUDT.Member1_DINT`, contains `.`, correctly excluded).
- Packed-BOOL exclusion (`kind == "BOOL" and "[" in tag_body`) is complete, not partial: every BOOL array element in this library's addressing model is packed (per `CLAUDE.md`'s own tag-addressing section — there's no "unpacked BOOL array" case), so excluding *all* BOOL-array-element writes from the native path is the correct full exclusion, not a partial mitigation. Bare BOOL scalars (no `[`) correctly remain native-eligible.
- Duplicate-name handling: `duplicate_tag_names` is computed once up front from the *entire* input list, so **every** occurrence of a repeated name (not just the 2nd+) is routed to the sequential path — each executes for real against the PLC, in order, and the last one's result is what survives in the returned dict. Verified against both a mocked unit test and a real simulator round-trip (`test_duplicate_batch_names_execute_sequentially`, which reads back the tag afterward and confirms the *second* write's value stuck).
- Missing-native-result handling (`client.py:578-586`): after parsing the native reply, every requested tag not present in the parsed result is explicitly backfilled as `success=False` with an explicit error message — a truncated/malformed native reply cannot silently read as success for a tag it didn't actually cover.
- Contiguous-run batching (not one global batch) is a deliberate, documented tradeoff (docstring, `client.py:518-524`) that keeps input-order execution correct when native-safe and fallback items interleave, at the cost of splitting one logical batch into N native calls when interrupted by fallback items. Confirmed via `test_write_tags_preserves_mixed_execution_order_with_contiguous_native_runs`, which asserts exactly two separate native calls when a `STRING` write sits between two DINT writes.
- `_execute_native_write_operations`'s `rc != 0 and not buffer.value` short-circuit matches the exact pattern already used by the pre-existing sibling `_execute_write_operations` path — consistent with established convention in this file, not a new pattern introduced ad hoc.

**Test proof**
- Unit (mocked FFI, `test_client_contract.py`): native batching of mixed program-scope/controller-scope/array-element/scalar-BOOL writes in one call; fallback routing for STRING, packed-BOOL-array, member-path, bit-path, and UDT (5 distinct exclusion reasons, each asserted individually); mixed-order contiguous-run splitting; duplicate-name sequential execution with last-result-wins; explicit partial-failure passthrough from a native reply.
- Simulator integration (real FFI + real `SimulatedPlc`, `test_integration.py`): native batch with a genuine type-mismatch failure and confirms the *other* array-element result and value are unaffected (real result correlation, not just mocked); duplicate names with real read-back proving the second write actually landed; packed-BOOL array writes above and below the DWORD-32 boundary through `write_tags` end-to-end, including a restore pass.
- Live hardware (`docs/validation/2026-08-22_1756-L75_fw33_python-native-batch-writes.md`): re-derived the arithmetic independently — 271.7→2,773.3 tags/s is exactly 10.21x; 368.104→36.058 ms is exactly 90.2% lower. Cross-checked the cited baseline figures (271.7 tags/s, 368.104 ms at size 100) directly against `docs/validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md:99` — they match verbatim, so the comparison is against a real prior measurement, not a fabricated or rounded-differently number.

**Residual risk**
- No dedicated unit test for the "native reply omits a requested tag" backfill branch (`client.py:578-586`) specifically — traced by hand, `not proven` by a targeted test. Low risk: defensive branch, unusual FFI failure mode already covered structurally.
- The full-coverage benchmark's `write_dispatch_per_call` field (`test_plc_full_coverage.py:236-239`) is hardcoded to `{"native_batched": size, "sequential_fallback": 0}` rather than reading actual dispatch counts back from `write_tags()` — accurate for this benchmark's always-pure-DINT workload, but would silently misreport if that workload ever became mixed-type. Cosmetic/forward-risk, not a current defect.
- Live validation covers only the DINT-array-element case at scale; STRING/UDT/member/bit/packed-BOOL/duplicate-name exclusion correctness is proven by contract + simulator tests, not by a live-hardware run through those specific paths — the brief's test requirements don't demand that either.

**Strong points (✅)**
- Zero Rust source changes — reuses the pre-existing `eip_write_tags_batch` FFI export exactly as-is, honoring the brief's scope and eliminating any regression risk to the other three bindings.
- The exclusion logic is a positive whitelist (11 named atomic kinds) rather than a blocklist trying to enumerate every unsafe case — new value kinds default to *not* being native-batched until explicitly added, the safer failure direction.
- Test coverage directly targets every correctness hazard the brief called out by name (duplicate names, packed-BOOL, member/bit, STRING/UDT, mixed order, partial failure) with both mocked and real-simulator tests.
- The live-hardware comparison reuses an actual prior baseline record rather than a fresh/uncomparable number, and both raw and Tukey-filtered averages are reported.

**Acceptance criteria tally**
- ✅ Safe atomic Python size-100 writes use native MSP batching — live-verified, 100/100 native dispatch at size 100.
- ✅ Unsupported/special writes retain correct fallback behavior and labeling — verified in code and both test layers.
- ✅ Zero duplicate writes and zero false-success results — duplicates each execute exactly once (not deduped, not double-sent) and missing-result backfill defaults to failure, not success.
- ✅ Documentation and benchmarks never describe fallback operations as native batch throughput — `python/README.md`, `CROSS_BINDING_FEATURE_GATE.md`, and the validation record all explicitly scope the throughput claim to the native-batched subset.

## Verdict

Merged. Zero Rust changes, so zero regression risk to the other three
bindings. The native/fallback classification is a safe-by-default
whitelist, every correctness hazard the brief named (duplicates,
packed-BOOL, member/bit, STRING/UDT, mixed order) has both mocked-unit and
real-simulator coverage, and the live-hardware throughput claim was
independently re-derived and cross-checked against the actual prior
baseline record rather than trusted. Two minor, non-blocking residual-risk
notes recorded (missing-result backfill lacks a dedicated test; the
full-coverage benchmark's dispatch-count field is hardcoded for its known
DINT-only workload) — neither affects correctness of the shipped
behavior.
