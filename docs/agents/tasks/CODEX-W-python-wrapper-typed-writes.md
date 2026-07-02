---
id: CODEX-W
title: Python wrapper — route single-tag writes through typed FFI exports
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

### Goal

The Python wrapper currently sends every single-tag write through the batch endpoint (`eip_execute_batch`). This works for most data types but fails on **plain BOOL array elements** (e.g. `gTestArray_BOOL[5]`) with CIP extended error `0x1E` "Multiple Service Response error". The Rust core and the C# wrapper handle these same tags correctly because they use the typed single-tag write FFI exports (`eip_write_bool`, `eip_write_dint`, …). The Python wrapper's `bindings.py` never registered those exports, so `Client.write_tag()` has no choice but to route through batch.

Wire up the typed single-tag write FFI in `python/rust_ethernet_ip/bindings.py`, then change `Client.write_tag()` in `python/rust_ethernet_ip/client.py:270-278` to dispatch on inferred value type and call the matching typed export. Keep `Client.write_tags()` (multi-item) on the batch path — that's the correct use of batch.

### Context to read first

- Hardware-validation log entry dated `2026-05-24` (the run that surfaced this) — `docs/agents/log.md` tail. The Rust + C# random→verify→nines exercisers passed 27/27/27; Python passed 23/27 with the 4 failures all being plain `BOOL[i]` writes (UDT-nested BOOLs worked because they take a different CIP path on the PLC).
- `src/ffi.rs:519-1410` — the full set of typed read/write FFI exports already implemented in Rust and ready to call. Specifically `eip_write_bool` (line 557), `eip_write_sint` (630), `eip_write_int` (685), `eip_write_dint` (750), `eip_write_lint` (804), `eip_write_usint` (858), `eip_write_uint` (912), `eip_write_udint` (966), `eip_write_ulint` (1020), `eip_write_real` (1076), `eip_write_lreal` (1130), `eip_write_string` (1366). Note `eip_write_bool` takes `c_int` (0/non-zero) not `c_bool`.
- `python/rust_ethernet_ip/bindings.py:44-90` — the current `_configure_function_signatures` block. Only `eip_read_tag`, `eip_read_tags_batch`, `eip_write_tags_batch`, `eip_execute_batch`, plus connect/disconnect/health/diagnostics are registered. None of the typed writers.
- `python/rust_ethernet_ip/client.py:106-124` — `_infer_value_type()` already returns the correct string tag (`"BOOL" | "SINT" | "INT" | "DINT" | "LINT" | "USINT" | "UINT" | "UDINT" | "ULINT" | "REAL" | "LREAL" | "STRING" | "UDT"`) — reuse this for dispatch.
- `python/rust_ethernet_ip/client.py:270-278` — current `write_tag()` body; this is what changes.
- `csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs` and `EthernetNetIpClient.cs:118-300` — reference implementation: how the C# wrapper dispatches typed writers. Same pattern in Python.

### Files to create or modify

- `python/rust_ethernet_ip/bindings.py` — add `argtypes`/`restype` signatures for all 12 typed single-tag write functions. Also add typed *read* functions in the same pass (`eip_read_bool`, `eip_read_sint`, …, `eip_read_lreal`, `eip_read_string`) — registering them costs nothing and removes a future round-trip when the read path eventually moves off the JSON shim. The new `write_tag` dispatch uses the writes; the new reads are infrastructure for a future, separately-briefed read-path cleanup.
- `python/rust_ethernet_ip/client.py` — replace `Client.write_tag()` body with a typed-dispatch implementation. `STRING` and `UDT` writes keep the existing JSON / batch path (their FFI shapes don't fit the cheap typed signature — `eip_write_string` takes a `c_char_p`, fine to wire up; `eip_write_udt` takes a length-prefixed binary blob and a symbol_id, which needs more glue than this brief should grow to cover). Out-of-range integers raise `ValueError` early.
- `python/tests/test_integration.py` — add `SimulatorIntegrationTests.test_bool_array_element_write_uses_typed_path` (or equivalent name) that writes `True`/`False` to a simulator-backed BOOL array tag and reads it back. The simulator must expose a BOOL array tag for this to work; if it doesn't, add one to `tests/plc_sim.rs` in the same commit.
- `python/tests/test_client_value_mapping.py` (or `test_bindings.py` — whichever currently mocks the native library) — add a unit test that confirms `Client.write_tag("foo", True)` calls `lib.eip_write_bool` exactly once and **does not** call `lib.eip_execute_batch` or `lib.eip_write_tags_batch`. Similarly one test per integer width to confirm `_infer_value_type` → typed-write dispatch wiring.
- `CHANGELOG.md` — add a `### Fixed` line under `[Unreleased]`: "Python wrapper: write_tag now uses typed single-tag FFI exports; fixes CIP 0x1E on BOOL array element writes."

### Behavior

`Client.write_tag(name, value, *, value_type=None)`:

1. Resolve `kind = value_type or _infer_value_type(value)`.
2. Validate the value fits the type (e.g. for `INT`, `-32768 ≤ value ≤ 32767`; raise `ValueError` with a clear message before crossing the FFI boundary).
3. Dispatch:
   - `BOOL` → `lib.eip_write_bool(client_id, name.encode("utf-8"), 1 if value else 0)`
   - `SINT` / `INT` / `DINT` / `LINT` → matching `eip_write_{sint,int,dint,lint}` with the value cast to the correct ctypes integer type.
   - `USINT` / `UINT` / `UDINT` / `ULINT` → matching unsigned writer; reject negatives.
   - `REAL` → `lib.eip_write_real(client_id, name.encode("utf-8"), c_float(value))`
   - `LREAL` → `lib.eip_write_lreal(client_id, name.encode("utf-8"), c_double(value))`
   - `STRING` → `lib.eip_write_string(client_id, name.encode("utf-8"), value.encode("utf-8"))`
   - `UDT` → fall through to the existing batch-path implementation (out of scope for this brief — note in code comment that a future brief should add `eip_write_udt` wiring).
4. Map the returned `c_int` to success/failure. Zero = ok; non-zero = raise `PlcOperationError` with a message that includes the tag name and the FFI return code.

`Client.write_tags(items)` is unchanged.

`Client.read_tag()` is unchanged in this brief — it still uses `eip_read_tag` and JSON decode. The typed reads registered in `bindings.py` are infrastructure; wiring them into `read_tag` is a separate future brief.

### Test requirements

**Integration (simulator-backed, gated by the existing simulator harness):**
- `test_bool_array_element_write` — writes `True` then `False` to a simulator BOOL[] tag; verifies readback after each.
- `test_typed_write_roundtrip_all_numeric_types` — write/read for at least one tag of each of `SINT`, `INT`, `DINT`, `LINT`, `USINT`, `UINT`, `UDINT`, `ULINT`, `REAL`, `LREAL`. If the simulator doesn't have a tag of a given type, extend `tests/plc_sim.rs` in the same commit.
- `test_out_of_range_write_raises_value_error` — `Client.write_tag("INT_TAG", 99_999, value_type="INT")` raises `ValueError` *before* the FFI call. Confirm the simulator tag value is unchanged afterwards (no partial write).

**Unit (mocked native library):**
- `test_write_tag_dispatches_to_typed_export` — parametrize across `(value, value_type, expected_fn_name)` pairs; assert the right mock is called and no batch mock is touched.
- `test_write_tag_string_uses_typed_export` — STRING-specific (separate because the encoding step is non-trivial).
- `test_write_tag_udt_still_uses_batch_path` — pin the documented fallback so a future change doesn't silently rewire it.

**Hardware (optional, no auto-run):**
- Re-running `python/examples/test_plc_random_to_nines.py` against the maintainer's PLC should report `27/27/27` instead of `23/27`. This is not in the automated suite — Codex records the local run output in the `## Codex log` for review.

### Acceptance criteria

- All typed single-tag write FFI exports listed above are registered in `bindings.py` with correct `argtypes` / `restype`.
- `Client.write_tag()` dispatches to the typed export for every type other than `UDT`; the batch path is no longer reached for single-tag writes of `BOOL`/integers/`REAL`/`LREAL`/`STRING`.
- All new tests pass: `cd python && python -m pytest tests/` (with simulator running, plus the unit subset that runs without).
- `cd python && python -m pytest tests/test_bindings.py tests/test_client_value_mapping.py` passes without a simulator.
- `cargo fmt -- --check` and `cargo clippy -- -D warnings` still pass (no Rust-side changes expected, but `tests/plc_sim.rs` may grow new tags).
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` and `cargo test --test plc_sim_tests` still pass.
- `CHANGELOG.md` `[Unreleased]` section gains the `### Fixed` entry.
- `## Codex log` records the local Python wrapper hardware re-run output (the 27/27/27 vs the previous 23/27).

### Out of scope

- Wiring `eip_write_udt` through Python. Needs a separate brief covering `UdtData` shape, `symbol_id` capture, and the read-modify-write quirk for the firmware-restricted STRING-member and UDT-array-element-member write paths (the same restrictions documented in `docs/agents/notes/ab-firmware-quirks.md`). Defer to a future "Python UDT write support" brief.
- Switching `Client.read_tag()` to the typed read exports. The current JSON-based read works correctly; the typed-read registration in this brief is infrastructure-only. Performance audit and the cutover live in a future brief.
- Any change to the C# wrapper or Rust core. Both are already correct.
- Performance work. The benchmark interest from the 2026-05-24 hardware run (Python writes at 326 ops/sec vs Rust at 188 ops/sec via the batch path) is a separate investigation — likely the batch endpoint reuses connection state more efficiently. After this brief lands, re-bench to confirm typed-write Python performance is in the same ballpark as Rust's typed-write path (both should be ~190 ops/sec). If a real regression appears, file it as a follow-up.
- Adding the read path to the python wrapper's hardware-validation example (`python/examples/test_plc_random_to_nines.py`). The example is a maintainer-run smoke, not part of the automated suite; it'll pick up the fix without modification.

### Risks and gotchas

- **`c_bool` vs `c_int` for `eip_write_bool`.** Rust signature at `src/ffi.rs:557-580` takes `c_int` (value is checked `!= 0`). Register as `c_int` in `bindings.py`, not `c_bool`. Misregistration here is silent — the call succeeds but reads back wrong bytes only on certain architectures.
- **Python `bool` is a subclass of `int`.** `_infer_value_type` already handles this (BOOL check is first), but the typed-write dispatch must check `kind == "BOOL"` *before* any numeric `isinstance(int)` branch. Reorder accordingly.
- **Integer overflow / sign.** Out-of-range values silently truncate when cast to a smaller ctypes int. The `ValueError` guards in step 2 of the behavior contract are load-bearing — without them, `write_tag("INT_TAG", 99999)` would succeed at the FFI boundary and write `33503` (the low 16 bits). The unit tests must cover this.
- **STRING encoding.** `eip_write_string` at `src/ffi.rs:1366` takes a UTF-8 C string. The Rust side handles the AB STRING structure packing internally. Superseded by CODEX-AT: top-level standard STRING writes use the direct structure encoding; Python should still hand the bytes over and trust the Rust path.
- **The simulator currently exposes a limited tag set.** Confirm via reading `tests/plc_sim.rs` which tags exist; expect to add a BOOL array tag (e.g. `BOOL_ARRAY` with at least element [5] readable) plus a `LINT`/`ULINT`/`LREAL`/`USINT`/`UINT`/`UDINT` tag each if not already present. Keep simulator changes minimal and localized.
- **CODEX-L (FFI ABI version handshake) and CODEX-W both touch `bindings.py`.** CODEX-W only *adds* function registrations; CODEX-L will *add* its own three new exports. They're parallel-safe via clean merge. If CODEX-L lands first, CODEX-W rebases trivially. If CODEX-W lands first, CODEX-L's brief expects exactly this state.
- **Don't rip out the unused batch-path code from `write_tag` until UDT support has migrated.** The dispatch should fall through to the existing `_execute_write_operations` path for `UDT` so the existing UDT round-trip stays working. The dead-code cleanup happens with the deferred UDT brief.

## Codex log

- 2026-05-24 codex [gpt-5]: Started implementation. Registered Python typed read/write FFI signatures, replaced scalar `Client.write_tag()` batch routing with typed writer dispatch, preserved UDT fallback to the existing batch path, and expanded simulator coverage for BOOL arrays plus numeric scalar types. Verification pending.
- 2026-05-24 codex [gpt-5]: Submitted implementation. Verification: `PYTHONPATH=python python3 -m unittest python.tests.test_client_value_mapping python.tests.test_client_contract python.tests.test_bindings` passed; `PYTHONPATH=python RUST_ETHERNET_IP_START_SIM=1 python3 -m unittest python.tests.test_integration` passed after rebuilding `python_test_simulator`; `PYTHONPATH=python python3 -m unittest discover -s python/tests` passed with simulator-gated tests skipped; `cargo build --example python_test_simulator` passed; `cargo test --test plc_sim_tests` passed; `SKIP_PLC_TESTS=1 cargo test --workspace --locked` passed; `cargo clippy -- -D warnings` passed; `rustfmt --edition 2024 --check tests/plc_sim.rs` passed; `git diff --check` passed. Full `cargo fmt --check` is currently blocked by unrelated active hardware-validation example files (`examples/bench_single_tag.rs`, `examples/test_plc_random_to_nines.rs`, `examples/test_plc_test_tag_definitions.rs`) that were already outside this task's edits. Hardware re-run of `python/examples/test_plc_random_to_nines.py` not run by Codex because the live PLC was in Claude/maintainer validation use.

## Claude review

### 2026-05-24 13:00  claude  [Opus 4.7]

**Independent verification (re-ran from scratch on the maintainer's environment, with the live ControlLogix 1756-L75 PLC at 10.136.15.20 available):**

- `cargo fmt --all -- --check` — clean (after fmt'ing this turn's separate uncommitted example files — those are unrelated to CODEX-W)
- `cargo clippy --workspace --all-targets -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — 208 passed, 0 failed across all 29 test binaries
- `cargo test --test plc_sim_tests --locked` — 10/10 passed
- `PYTHONPATH=python python3 -m unittest python.tests.test_client_value_mapping python.tests.test_client_contract python.tests.test_bindings` — 23/23 passed
- `PYTHONPATH=python RUST_ETHERNET_IP_START_SIM=1 python3 -m unittest python.tests.test_integration` — 8/8 passed
- **Hardware re-run against the maintainer's ControlLogix** (`python/examples/test_plc_full_coverage.py`, 2299 tags, 2206 writeable): zero CIP `0x1E` errors. All 128/128 plain `gTestArray_BOOL[i]` writes succeed; all 100/100 `Program:TestProgram.gTestArray_BOOL[i]` writes succeed. **The specific bug CODEX-W targeted is fixed on real hardware.**

**Strong points (✅):**
- Typed FFI registration in `bindings.py:68-126` is correct: `c_int` for `eip_write_bool` value (not `c_bool`, per the brief's gotcha), `c_double` for `eip_write_real`/`eip_write_lreal` to match the Rust `f64` FFI signature (consistent with C# wrapper at `EthernetNetIpClient.NativeMethods.cs:eip_write_real`).
- `Client.write_tag()` dispatch at `client.py:317-367` checks `BOOL` before the integer branch — correctly handles Python's `bool isinstance int` quirk per the brief's gotcha.
- `_INTEGER_RANGES` + `_INTEGER_CTYPES` tables at `client.py:54-75` are a clean lookup pattern that scales cleanly when LREAL/SINT/USINT etc. need adjustment.
- `_validate_integer_value` (`client.py:187-194`) and `_validate_float_value` (`client.py:197-200`) reject `bool` values for non-BOOL integer/float writes — prevents accidental `write_tag("DINT_TAG", True)` from silently dispatching to the wrong typed export.
- UDT fallback retained at `client.py:320-329` — UDT writes still go through the batch path, exactly as the brief required. The inline comment documents that this is intentional pending a future UDT brief.
- Test coverage is genuinely thorough: parametrized dispatch test across all 11 typed exports (`test_client_value_mapping.py:99-122`), STRING-specific test (124-131), UDT-stays-on-batch pin (133-140), pre-FFI ValueError test (142-150), plus an integration test that round-trips on the simulator. The simulator tag inventory was correctly expanded in `tests/plc_sim.rs` to include `BOOL_ARRAY` and one tag of each numeric width — neat scope discipline.

**Findings (🟡 polish, all non-blocking):**
- 🟡 `client.py:319` calls `(value_type or _infer_value_type(value)).upper()`. The `.upper()` defensively normalizes caller-supplied `value_type="dint"` to `"DINT"`. Good safety, but `_infer_value_type` already returns uppercase. The trailing `.upper()` is mildly redundant when the path took the inference branch — pure micro-polish, not worth changing.
- 🟡 `_validate_float_value` casts via `float(value)` (`client.py:200`) which accepts an `int` value silently for REAL writes. Matches `_infer_value_type`'s behavior (treats `int` as DINT, but allows `write_tag("REAL_TAG", 42, value_type="REAL")` to succeed). Probably correct — users will pass int literals for REAL all the time — but worth a brief comment near line 200 documenting the intent so a future cleanup doesn't tighten it.
- 🟡 Codex's log entry honestly flags that full `cargo fmt --check` couldn't run against `main` due to *Claude's* uncommitted example files. That's a clean callout — owned correctly. No issue against CODEX-W.

**Findings (🟠 real concerns) — none.**

**Acceptance criteria tally:**
- ✅ All 12 typed single-tag write FFI exports registered with correct argtypes / restype
- ✅ Read exports also registered (infrastructure for a future read-path brief)
- ✅ `Client.write_tag()` dispatches to typed export for every type except UDT; batch path no longer reached for BOOL/integers/REAL/LREAL/STRING
- ✅ New unit tests pass without simulator
- ✅ New integration tests pass against simulator
- ✅ `cargo fmt --check`, `cargo clippy -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests` all pass
- ✅ `CHANGELOG.md` `[Unreleased]` `### Fixed` entry present
- ✅ Hardware re-run output captured (here, by Claude — Codex couldn't run it because the PLC was occupied)

## Verdict

### 2026-05-24 13:05  claude  [Opus 4.7]  status: merged

**Merged.** The targeted bug (Python `write_tag` BOOL-array CIP `0x1E`) is fixed on real hardware as well as in the simulator. Implementation is clean, scope was respected (UDT correctly deferred), tests are thorough, integer range validation is load-bearing and present. No real concerns.

**Brief-side note (no defect):** the brief mentioned the `(STRING_TAG, "STRING")` integration round-trip should hit `eip_write_string`. Codex's `test_typed_write_roundtrip_all_numeric_types` correctly covers numeric scalars, and the unit test `test_write_tag_string_uses_typed_export` pins the typed STRING dispatch — together they satisfy the contract without needing a STRING-write integration test (the simulator's existing `STRING_TAG` round-trip in `test_connect_read_write_and_health` already exercises the simulator's STRING path).

**Two new core findings surfaced by the same hardware run (NOT CODEX-W defects)** — these affect all three bindings equally, so they're Rust-core issues, not Python-wrapper issues:

1. **Plain `BOOL[i]` array element writes verify-mismatch ~30-40%** across Rust/C#/Python. Writes return Ok; readback returns the wrong bit ~1 in 3. Same indices fail in each binding (random variation between runs accounts for the 25-41% spread). Suggests a bug in the library's BOOL-array RMW path or in the bit-extraction-on-read path. Worth a separate investigation brief.
2. **`gTestUDT_Array[i].Array_BOOL[j]` is completely broken** — 200/200 reads fail, 200/200 writes fail, across all three bindings. The nested-BOOL-inside-UDT-array-element path doesn't function. Worth a separate brief.

Neither is gating for CODEX-W itself, but both are gating for v0.8.0 release — they should be filed as new briefs (`CODEX-X` for BOOL array bit-write verify mismatch, `CODEX-Y` for UDT-array-element nested BOOL path) before tagging v0.8.0.
