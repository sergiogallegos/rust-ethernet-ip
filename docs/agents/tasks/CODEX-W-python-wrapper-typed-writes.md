---
id: CODEX-W
title: Python wrapper — route single-tag writes through typed FFI exports
owner: codex
status: open
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
- **STRING encoding.** `eip_write_string` at `src/ffi.rs:1366` takes a UTF-8 C string. The Rust side handles the AB STRING `LEN + DATA[82]` packing internally (incl. the documented STRING-tag write workaround for CIP `0x2107`). Don't reimplement that in Python; just hand the bytes over and trust the Rust path.
- **The simulator currently exposes a limited tag set.** Confirm via reading `tests/plc_sim.rs` which tags exist; expect to add a BOOL array tag (e.g. `BOOL_ARRAY` with at least element [5] readable) plus a `LINT`/`ULINT`/`LREAL`/`USINT`/`UINT`/`UDINT` tag each if not already present. Keep simulator changes minimal and localized.
- **CODEX-L (FFI ABI version handshake) and CODEX-W both touch `bindings.py`.** CODEX-W only *adds* function registrations; CODEX-L will *add* its own three new exports. They're parallel-safe via clean merge. If CODEX-L lands first, CODEX-W rebases trivially. If CODEX-W lands first, CODEX-L's brief expects exactly this state.
- **Don't rip out the unused batch-path code from `write_tag` until UDT support has migrated.** The dispatch should fall through to the existing `_execute_write_operations` path for `UDT` so the existing UDT round-trip stays working. The dead-code cleanup happens with the deferred UDT brief.

## Codex log

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
