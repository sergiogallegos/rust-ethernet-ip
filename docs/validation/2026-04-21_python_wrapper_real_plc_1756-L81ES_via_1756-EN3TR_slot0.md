# 2026-04-21 Python Wrapper Real PLC Validation - ControlLogix 1756-L81ES

Date: 2026-04-21
Tester: Codex + Sergio Gallegos
PLC model: 1756-L81ES
Network topology: Routed Ethernet connection to `192.168.0.101:44818` via `1756-EN3TR`, backplane slot `0`

## Scope

This record captures the Python wrapper validation pass run against the then-current `0.8.0` draft line on the routed ControlLogix target used for the Rust and C# real-PLC validations.

The focus of this pass was:

- routed Python wrapper connectivity
- live single-tag and batch reads
- live write behavior
- health and diagnostics access
- collector example behavior on a physical PLC

## Commands Executed

- `PYTHONPATH=python python3 -m unittest python.tests.test_integration`
- `PYTHONPATH=python RUST_ETHERNET_IP_PLC_ADDRESS=192.168.0.101:44818 RUST_ETHERNET_IP_PLC_SLOT=0 python3 python/examples/collector_service.py --config python/examples/collector_config.example.json --once`
- live routed Python smoke probe for:
  - `read_tag("gTestArray_DINT[0]")`
  - `read_tag("gTestUDT.Member1_DINT")`
  - `write_tag("gTestArray_DINT[5]", 424242)` with readback and restore
  - `read_tags(["gTestArray_DINT[0]", "gTestArray_REAL[0]", "gTestUDT.Member1_DINT"])`
  - `check_health()`
- live routed Python batch-write probe for:
  - `gTestArray_DINT[6]`
  - `gTestArray_REAL[0]`
  - `gTestArray_BOOL[0]`
  - readback and restore
- `PYTHONPATH=python python3 -m unittest python.tests.test_import python.tests.test_client_value_mapping python.tests.test_diagnostics_mapping`

## Result

- PASS: routed Python client connection succeeded through `1756-EN3TR` to CPU slot `0`.
- PASS: single-tag live reads succeeded for controller-scoped and UDT-member paths.
- PASS: routed batch reads succeeded for the exercised `gTest*` paths.
- PASS: `check_health()` returned `True` on the live routed target.
- PASS: `get_diagnostics_snapshot(detailed=True)` returned a valid mapped diagnostics object.
- PASS: collector example connected to the live PLC and wrote `4` rows to `python/examples/data/plc_samples.sqlite`.
- PASS AFTER FIX: single `DINT` writes now complete successfully in the Python wrapper with correct success reporting and readback verification.
- PASS AFTER FIX: `write_tags(...)` no longer misreports successful live `DINT` and `REAL` writes as failures on this routed ControlLogix target.
- PASS WITH OPEN FOLLOW-UP: `gTestArray_BOOL[0]` still returned `0x1E` on the exercised Python write path and did not change value during the final smoke probe.
- PASS: targeted pure-Python tests passed `7/7`.
- INFO: `python.tests.test_integration` is simulator-backed and skipped `3/3` tests because no simulator session was configured for this live-PLC pass.

## Issue Found and Fixed During Validation

The Python wrapper previously reported some successful live writes as failures on the routed ControlLogix target.

Observed behavior before the fix:

- `write_tag("gTestArray_DINT[5]", 424242)` raised a Python-side write error even though the PLC value changed and read back correctly
- `write_tags(...)` reported `0x1E` failures for live `DINT` / `REAL` writes that were actually applied on the PLC

Root cause:

- the Python wrapper relied on the native multi-write helper in a way that allowed a top-level embedded-service failure to collapse per-item status reporting on this live path

Fix applied:

- updated `python/rust_ethernet_ip/bindings.py`
- updated `python/rust_ethernet_ip/client.py`
- updated `python/README.md`

Behavior after the fix:

- `write_tag()` now uses the per-operation `eip_execute_batch` path
- `write_tags()` now executes sequentially per tag in the Python wrapper so per-tag success and error reporting stays truthful on the validated ControlLogix path
- live `DINT` and `REAL` writes now report success correctly and restore cleanly

## Known Limitations and Open Questions Observed

- CONFIRMED: this Python wrapper pass now reports live `DINT` and `REAL` write outcomes accurately on the exercised routed ControlLogix path
- OPEN QUESTION: `gTestArray_BOOL[0]` still returned `Multiple Service Response error: 0x1E` during the final Python write probe and the value remained unchanged
- UNCLEAR: whether that final BOOL outcome is a Python-wrapper path issue, a request-shape issue for this specific tag/path, or a controller-specific behavior difference that needs isolated follow-up

No evidence from this pass suggests a regression in routed reads, routed `DINT` writes, routed `REAL` writes, health checks, diagnostics mapping, or the collector example.

## Assessment

The Python wrapper was viable on the exercised routed ControlLogix feature set for the then-current `0.8.0` draft line, but this validation surfaced a real write-status handling bug that required a fix during the session. After the fix, the exercised live read, single-write, batch-read, health, diagnostics, and collector paths behaved correctly. One routed BOOL write path remained an explicit follow-up item rather than a closed limitation classification.
