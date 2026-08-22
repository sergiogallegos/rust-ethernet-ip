---
id: CODEX-BC
title: Cross-binding schema refresh API and cache diagnostics
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1. Depends on CODEX-BA and CODEX-BB.**

Expose the comprehensive schema refresh and its diagnostics consistently
through the C ABI, C#, Python, and C/C++ surfaces.

### Required implementation

1. Add a handle-based C export such as `eip_refresh_schema(client_id)`.
2. Add thin wrapper methods:
   - C#: `RefreshSchema()`;
   - Python: `refresh_schema()`;
   - C++ convenience layer: `refreshSchema()`;
   - document the C function in the public header.
3. Update the FFI header parity gate and ABI capability bitmap. Determine and
   document whether the additive export requires an ABI version change; do not
   change the ABI number without coordinating every pin.
4. Extend diagnostics with:
   - schema generation;
   - refresh count;
   - array-classification hits, misses, and evictions;
   - datatype contradictions;
   - successful and failed read recoveries.
5. Ensure diagnostic JSON additions remain backward-compatible for existing
   wrapper parsers.
6. Document the maintenance workflow: pause writes, edit/download, refresh,
   optionally rediscover/verify, then resume writes.

### Test requirements

- FFI success, invalid-client, last-error, and clone-visibility tests.
- Header/export parity and ABI/capability tests.
- C#, Python, and C++ wrapper tests proving the same native generation changes.
- Diagnostics values increment for hits, misses, refresh, eviction,
  contradiction, and recovery without exposing proprietary tag values.
- Full Rust, C#, Python, and C++ offline gates pass against one release FFI
  artifact.

### Acceptance criteria

- Every supported language can explicitly refresh the same native schema
  state without reconnecting.
- Diagnostics make cache behavior measurable and are documented accurately.
- Existing consumers that ignore new diagnostics fields continue working.
- Public wrapper guides contain the schema-maintenance example.
- CHANGELOG and draft 1.2.1 release notes are updated.

### Out of scope

- UI or automatic Studio 5000 integration.
- Polling an undocumented controller project revision.
- Hardware editing and download execution (CODEX-BD).

## Codex log

### 2026-08-22 14:05  codex [GPT-5]

Submitted the cross-binding schema-maintenance surface. Added
`eip_refresh_schema`, ABI v3 and `CAP_SCHEMA_REFRESH`, header/export parity,
C# `RefreshSchema()`, Python `refresh_schema()`, and C++ `refreshSchema()`.
Diagnostics now include generation, explicit refreshes, classification
hits/misses/evictions, contradictions, and successful/failed read recoveries;
Python and C# tolerate missing additive JSON fields. Wrapper guides document
pause writes, edit/download, refresh, optionally rediscover/verify, then resume.
Verification passed against one release FFI artifact: Rust FFI ABI/state tests,
C# 89/89, Python contract plus 8/8 simulator integration, C++ build and 3/3
CTest, 60-symbol header parity, focused diagnostics tests, and clippy with
warnings denied.

## Claude review

### 2026-08-22 17:05  claude [Sonnet 5]

**Independent verification**
- `scripts/schema-change-gate` rerun clean this session (all 7 steps: Rust, C ABI, C#, Python, C header/export parity, C++ CMake+CTest).
- `cargo test --test ffi_abi` / `ffi_state_consistency` (schema-specific tests) confirmed passing as part of the full-suite and offline-gate runs.

**What's being fixed**
- Exposes CODEX-BA/BB's schema-refresh contract consistently across C ABI, C#, Python, and C++, plus the diagnostics needed to observe cache/recovery behavior from any binding.

**Root cause confirmation**
- N/A — this is new cross-binding surface, not a bug fix.

**Fix appropriateness**
- `eip_refresh_schema` (`src/ffi.rs:1606`) is a handle-based, safe (`extern "C"`, no `unsafe` block) export — correct given it only calls `client.refresh_schema()` through `ffi_block_on!` and returns an int code, no raw pointer traffic.
- ABI bump is coordinated, not silent: `src/version.rs` bumps `ABI_VERSION` 2→3 and adds `CAP_SCHEMA_REFRESH` into the `CAPABILITIES` bitmap; `include/rust_ethernet_ip.h` mirrors both the ABI version and the capability constant; `wiki/protocol/abi-contract.md` documents the policy change explicitly. All three wrapper ABI-version constants (C# `NativeRuntime.cs`, Python `bindings.py`) were bumped to 3 in the same diff — no drift between native and wrapper expectations.
- Wrapper method shapes match: C# `RefreshSchema()` (`EthernetNetIpClient.Diagnostics.cs`), Python `refresh_schema()` (`client.py`), C++ `refreshSchema()` (`eip_client.hpp`) — all thin wrappers over `eip_refresh_schema`, all documented in their respective READMEs with the same pause-writes/edit/refresh/rediscover/resume sequence, worded consistently across all three plus `docs/CPP_INTEGRATION.md`.
- Diagnostics additivity is proven, not assumed: Python's diagnostics parser uses `payload.get("schema_cache", {})` with a dataclass default (`types.py`), backed by a dedicated test, `test_missing_schema_cache_remains_backward_compatible`, that parses a full pre-BC-shaped payload with no `schema_cache` key and asserts it doesn't raise. C#'s `SchemaCache` property defaults to `new DiagnosticsSchemaCacheMetrics()` and `System.Text.Json` doesn't require the key present by default — same guarantee, though C# has no equivalent explicit missing-field test (see Findings).

**Test proof**
- `ffi_schema_refresh_is_clone_visible_and_invalid_handles_report_last_error` (`tests/ffi_state_consistency.rs`) proves clone-visibility through the FFI registry and the invalid-handle/last-error path in one test.
- `tests/ffi_abi.rs` asserts `CAP_SCHEMA_REFRESH` is set in the live capability bitmap, not just defined as a constant.
- C# `SimulatorIntegrationTests` asserts the schema generation and refresh count both advance by exactly one through the real native library.
- Python `test_integration.py` asserts the identical pair of deltas through the real native library.
- C++ `demo.cpp`'s new schema-generation helper parses the diagnostics JSON directly and asserts the generation advanced by one — exercised by `ctest -R cpp_smoke_demo`, confirmed passing in this session's gate rerun.

**Residual risk**
- No C# equivalent of Python's explicit "missing schema_cache key" backward-compatibility unit test — the backward-compat guarantee is real (verified by reading `System.Text.Json` default-deserialization semantics against the property declaration) but not proven by a dedicated C# test the way Python's is. Minor gap, not blocking.
- Live-hardware confirmation that `eip_refresh_schema` behaves identically against a real 1756-L75 across all four bindings is CODEX-BD's live session, still pending.

**Strong points (✅)**
- The ABI-bump-for-every-new-export policy (documented in `wiki/protocol/abi-contract.md`) is stricter than strictly required by SemVer/ABI-compat rules (an additive export doesn't normally require a version bump) but is a defensible, conservative choice that was applied consistently and documented, not just asserted.
- Wrapper doc wording (pause writes → edit/download → refresh → rediscover/verify → resume) is copy-consistent across C#, Python, and C++ integration docs — reduces the chance of one wrapper's guidance drifting from another's.
- The diagnostics JSON schema addition is genuinely additive at the wire level (new top-level `schema_cache` object, no existing fields touched), which is the correct shape for a non-breaking cross-language change.

**Findings**
- 🟢 C# lacks a dedicated "missing schema_cache field" unit test (Python has one). Low priority — the underlying `System.Text.Json` behavior is standard, but a test would make the guarantee explicit and regression-proof.
- 🟢 No other findings.

**Acceptance criteria tally**
- ✅ Every supported language can explicitly refresh the same native schema state without reconnecting — proven per-language against the real native library (C#, Python, C++) or the FFI layer directly (Rust, C ABI).
- ✅ Diagnostics make cache behavior measurable and are documented accurately.
- 🟡 partially "Existing consumers that ignore new diagnostics fields continue working" — proven explicitly for Python (dedicated test), proven by code inspection but not a dedicated test for C#.
- ✅ Public wrapper guides contain the schema-maintenance example (C#, Python, C++ all updated).
- ✅ CHANGELOG and draft 1.2.1 release notes updated.

## Verdict

Merged. Cross-binding surface is complete, consistently documented, and the additive-diagnostics claim is proven (not just asserted) for Rust/C/Python; C# has the same guarantee by inspection but lacks Python's dedicated regression test — worth a small follow-up, not a blocker.
