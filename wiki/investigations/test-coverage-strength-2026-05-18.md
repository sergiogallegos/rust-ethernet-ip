# Test Coverage Strength - 2026-05-18

## Summary

`confirmed`: Rust core coverage is strong for protocol encoding/decoding, route-path behavior, tag-path parsing, UDT parsing, diagnostics, simulator-backed read/write flows, and several failure paths.

`needs-review`: Wrapper coverage is uneven. C# has useful contract and smoke tests, but many tests are mocks or silently return when simulator/native prerequisites are missing. Python has good focused mapping tests but only a small wrapper surface and skipped simulator integration by default.

`confirmed`: Current local test commands on 2026-05-18 produced:

- `cargo test`: pass, including Rust unit/integration/doctests; many PLC-only tests remain ignored or gracefully skipped.
- Superseded during the 2026-05-18 follow-up: `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore` now passes `44/44` after C# simulator tests were changed to auto-start the deterministic simulator and stage an FFI-enabled native library.
- From `python/`: `python -m pytest tests`: pass with `7 passed, 3 skipped`.
- Superseded during the 2026-05-18 follow-up: From `python/`, `RUST_ETHERNET_IP_START_SIM=1 python -m pytest tests` now passes `12/12` after the Python harness was changed to prefer the prebuilt simulator binary and after the native library was built with `--features ffi`.
- Superseded again during the 2026-05-18 Python wrapper unit pass: From `python/`, `python -m pytest tests` now passes `21 passed, 5 skipped`; with `RUST_ETHERNET_IP_START_SIM=1`, the same suite passes `26/26`.

## Current Understanding

- `confirmed`: Rust has the strongest automated safety net. The suite includes unit tests inside core modules, simulator-backed tests, FFI boundary tests, route-path compatibility tests, and doctests.
- `confirmed`: The Rust simulator currently covers scalar BOOL/DINT/REAL/STRING, DINT and REAL arrays, batch partial failures, reconnect behavior, timeout behavior, and route-path compatibility.
- `confirmed`: 66 Rust tests are explicitly ignored or PLC-gated; this is appropriate for hardware-dependent scenarios, but normal `cargo test` does not prove the full real-PLC matrix.
- `confirmed`: C# tests exercise public wrapper contracts, DTO mapping, tag-group behavior, diagnostics shape, unsupported batch-config contract, and native simulator-backed paths by default.
- `needs-care`: C# files named `IntegrationTests` are still mostly Moq-based and should eventually be renamed or reframed so they do not imply real PLC/native integration.
- `confirmed`: C# wrapper-only contract coverage was expanded after the simulator work. The suite now covers `PlcValue` JSON parsing, legacy versus raw UDT value shapes, `RoutePath` grouped fields and FFI preparation, client not-connected/disposed behavior, batch input validation, and batch DTO defaults.
- `confirmed`: Python currently has 10 tests total: 7 pure mapping/import tests and 3 simulator integration tests that skip unless configured.
- `confirmed`: Python simulator integration now covers native connect/read/write/health, route-path connect, batch read/write, diagnostics retrieval, and partial batch read failure handling.
- `confirmed`: Python wrapper-only tests now cover fake-native lifecycle, route connect argument forwarding, read/write/batch behavior, batch partial errors, diagnostics retrieval, unconnected-operation errors, write-result parsing, write-request normalization, and native loader failure diagnostics.
- `needs-review`: Python does not yet test BOOL write follow-up behavior, UDT/tag metadata surfaces, or collector/MQTT examples in automated CI-style tests.
- `confirmed`: Existing coverage docs in `tests/TEST_COVERAGE_SUMMARY.md` and `tests/test_coverage_analysis.md` are stale in places; they list some areas as missing even though corresponding test files now exist.

## Strong Areas

- Rust protocol frame and value round-trips are well covered by [src/protocol/tests.rs](../../src/protocol/tests.rs).
- Rust route-path and tag-path behavior is covered by [src/route.rs](../../src/route.rs), [src/tag_path.rs](../../src/tag_path.rs), [tests/route_path_operations_tests.rs](../../tests/route_path_operations_tests.rs), and [tests/route_path_sim_compat_tests.rs](../../tests/route_path_sim_compat_tests.rs).
- Rust FFI input validation and JSON batch paths have targeted coverage in [tests/ffi_tests.rs](../../tests/ffi_tests.rs).
- Rust simulator coverage gives deterministic protocol-level checks without real hardware in [tests/plc_sim.rs](../../tests/plc_sim.rs) and [tests/plc_sim_tests.rs](../../tests/plc_sim_tests.rs).
- C# wrapper DTO and contract coverage exists for diagnostics, UDT data parsing, tag groups, write-batch routing, and unsupported batch config in [csharp/RustEtherNetIp.Tests](../../csharp/RustEtherNetIp.Tests).
- Python value decoding and diagnostics DTO mapping are covered by [python/tests/test_client_value_mapping.py](../../python/tests/test_client_value_mapping.py) and [python/tests/test_diagnostics_mapping.py](../../python/tests/test_diagnostics_mapping.py).

## Gaps To Prioritize

1. `done`: Make Python simulator integration reliable in local runs.
   - The harness now prefers `target/debug/examples/python_test_simulator(.exe)` and falls back to `cargo build --example python_test_simulator`.
   - The Python README now documents `cargo build --features ffi --example python_test_simulator` for wrapper integration testing.

2. `high`: Convert C# simulator tests from silent `return` skips to explicit skips or traits.
   - `done`: The C# simulator tests now use `SimulatorTestHarness`, auto-start the simulator when `SIM_PLC_ADDRESS` is absent, and stage/check a native library with required FFI exports.
   - Remaining cleanup: rename or reframe Moq-based `EtherNetIpClientIntegrationTests`.

3. `high`: Add wrapper parity tests for the current Python and C# surfaces against the same simulator fixtures.
   - Python now covers scalar read/write, batch read/write, route connect, partial batch failures, and diagnostics.
   - C# now covers scalar read/write, array ranges, batch read/write, mixed execute batch, route connect/diagnostics, and tag-group partial/read-failure style events against the same simulator.
   - Remaining parity work should add known unsupported/limitation behavior and deeper UDT/tag metadata surfaces.

4. `done`: Add first-pass C# wrapper unit coverage independent of simulator/native calls.
   - Added tests for `PlcValue`, `RoutePath`, `EtherNetIpClient` validation/lifecycle contracts, and batch operation/result DTOs.
   - Current C# suite result after this pass: `75/75` passing.

5. `done`: Add first-pass Python wrapper unit coverage independent of simulator/native calls.
   - Added fake-native tests for `Client` lifecycle, route argument forwarding, read/write/batch behavior, diagnostics retrieval, and unconnected errors.
   - Added binding-loader diagnostics tests and helper-contract tests for write normalization and result parsing.
   - Current Python suite result after this pass: `21 passed, 5 skipped` without simulator, `26/26` with simulator.

4. `medium`: Refresh stale Rust coverage docs.
   - [tests/test_coverage_analysis.md](../../tests/test_coverage_analysis.md) still describes batch, subscription, cache, route, and health tests as missing even though files now exist.

5. `medium`: Add native/FFI ABI contract tests for signatures and buffer ownership across wrappers.
   - C# and Python both depend on `eip_get_diagnostics_json` plus `eip_free_string`, route arrays, and JSON buffers.
   - Existing Rust FFI tests are useful, but wrapper-level misuse would not always be caught.

6. `medium`: Expand simulator behavior for UDT and tag-discovery paths.
   - Rust has unit tests for UDT parsing and real-PLC validation records, but deterministic simulator tests for UDT discovery, UDT member operations, and detailed tag attributes would reduce hardware dependence.

7. `medium`: Add regression coverage for the Python routed BOOL write follow-up.
   - The 2026-04-21 Python real-PLC record left `gTestArray_BOOL[0]` as an open follow-up after a `0x1E` result.

## Evidence

- [Cargo.toml](../../Cargo.toml)
- [tests/TEST_COVERAGE_SUMMARY.md](../../tests/TEST_COVERAGE_SUMMARY.md)
- [tests/test_coverage_analysis.md](../../tests/test_coverage_analysis.md)
- [tests/plc_sim.rs](../../tests/plc_sim.rs)
- [tests/plc_sim_tests.rs](../../tests/plc_sim_tests.rs)
- [tests/ffi_tests.rs](../../tests/ffi_tests.rs)
- [csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj](../../csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj)
- [csharp/RustEtherNetIp.Tests/SimulatorIntegrationTests.cs](../../csharp/RustEtherNetIp.Tests/SimulatorIntegrationTests.cs)
- [python/tests/test_integration.py](../../python/tests/test_integration.py)
- [python/tests/sim_harness.py](../../python/tests/sim_harness.py)
- [docs/validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Related Pages

- [../releases/0.8.0-validation-synthesis.md](../releases/0.8.0-validation-synthesis.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [../investigations/python-mvp-surface-2026-04-19.md](python-mvp-surface-2026-04-19.md)
