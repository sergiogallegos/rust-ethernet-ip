# 2026-04-16 Rust Real PLC Validation - ControlLogix 1756-L81ES

Date: 2026-04-16
Tester: Codex + Sergio Gallegos
PLC model: 1756-L81ES
Network topology: Routed Ethernet connection to `192.168.0.101:44818` via `1756-EN3TR`, backplane slot `0`

## Scope

This was a follow-up validation for the then-current `0.8.0` draft line against the same ControlLogix target and `gTest*` tag set used during the `0.7.0` release validation.

## Commands Executed

- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example readonly_plc_probe -- 192.168.0.101:44818`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test batch_operations_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test health_check_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test cache_management_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test route_path_operations_tests -- --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test subscription_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example test_comprehensive_arrays_udt`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example test_plc_test_tag_definitions`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --quiet --release --example perf_baseline_real_plc -- --iterations 100`
- `cargo test --workspace --all-targets`

## Result

- PASS: Routed session establishment succeeded through `1756-EN3TR` to CPU slot `0`.
- PASS: Read-only probe succeeded for 44/44 expected paths.
- PASS: Batch operation tests passed 5/5.
- PASS: Health-check tests passed 2/2.
- PASS: Cache-management tests passed 3/3.
- PASS: Route-path tests passed 5/5.
- PASS: Subscription tests passed 4/4. Expected invalid/nonexistent tag failures were logged and asserted correctly.
- PASS WITH DOCUMENTED LIMITATIONS: `test_comprehensive_arrays_udt` completed successfully and restored modified tags. The direct write to `gTestUDT_Array[3].Member1_DINT` failed with known extended status `0x2107`.
- PASS WITH DOCUMENTED LIMITATIONS: `test_plc_test_tag_definitions` produced 333 passed / 59 failed / 0 skipped, matching the previous ControlLogix limitation profile.
- PASS: Restore step reported 333 restored / 0 failed in the full tag matrix.
- PASS: `cargo test --workspace --all-targets` completed successfully.

## Known PLC Limitations Observed

The 59 expected write failures were all in documented firmware-limited categories:

- 2 direct standalone `STRING` writes
- 2 direct `STRING` member writes inside UDTs
- 55 direct writes to UDT array element members

No new Rust library regression was identified.

## Hardware Benchmark

Iterations per scenario: 100

| Metric | Total ms | Avg call ms | Logical ops/sec |
|---|---:|---:|---:|
| `single_read` | 151.624167 | 1.51624167 | 659.5254699733981 |
| `single_write` | 203.924834 | 2.03924834 | 490.3767630380908 |
| `batch_read` | 181.905125 | 1.81905125 | 5497.371225796965 |
| `batch_write` | 253.834416 | 2.53834416 | 1181.8728316179158 |
| `mixed_execute` | 395.1215 | 3.9512150000000004 | 1012.3468350874351 |

## Assessment

The Rust library remained stable on the exercised routed ControlLogix feature set for the then-current `0.8.0` draft line. The observed failures were unchanged controller firmware limitations, not new protocol regressions.

## 2026-04-21 Rerun

Date: 2026-04-21
Tester: Codex + Sergio Gallegos

### Commands Executed

- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test health_check_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test route_path_operations_tests -- --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example readonly_plc_probe -- 192.168.0.101:44818`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test batch_operations_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test cache_management_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo test --test subscription_tests -- --ignored --nocapture`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example test_comprehensive_arrays_udt`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --example test_plc_test_tag_definitions`

### Result

- PASS: Health-check tests passed `2/2`.
- PASS: Route-path tests passed `5/5`.
- PASS: Read-only probe again succeeded for `44/44` expected paths.
- PASS: Batch operation tests passed `5/5`.
- PASS: Cache-management tests passed `3/3`.
- PASS: Subscription tests passed `4/4`, with the expected invalid-tag assertions still behaving correctly.
- PASS WITH DOCUMENTED LIMITATIONS: `test_comprehensive_arrays_udt` again completed successfully and restored modified tags. The direct write to `gTestUDT_Array[3].Member1_DINT` again failed with the known extended status `0x2107`.
- PASS WITH DOCUMENTED LIMITATIONS: `test_plc_test_tag_definitions` again produced `333 passed / 59 failed / 0 skipped`, with `333 restored / 0 failed` in the restore step.

### Assessment

This rerun reproduced the same routed ControlLogix behavior and the same documented limitation profile as the earlier record. No new Rust regression was observed on the exercised live target.
