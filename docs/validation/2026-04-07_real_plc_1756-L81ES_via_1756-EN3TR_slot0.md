Date: 2026-04-07
Tester: Codex + Sergio Gallegos
PLC model: 1756-L81ES
Firmware revision: 37
Network topology: Routed Ethernet connection to `192.168.0.101:44818` via `1756-EN3TR`, backplane slot 0

Scenarios executed:
- Connect / disconnect / reconnect baseline through routed path
- Primitive read/write set on controller-scoped and program-scoped tags
- Batch read/write/mixed execute
- Subscription creation and polling behavior
- Health check APIs
- Tag discovery/cache clear/repopulation
- Route-path configuration and routed client creation
- UDT read paths used by the then-current 0.7.0 test tags
- Array element addressing including BOOL array elements
- Arrays of UDTs and nested member/array access

Commands executed:
- `cargo run --example readonly_plc_probe -- 192.168.0.101:44818`
- `cargo test --test batch_operations_tests -- --ignored --nocapture`
- `cargo test --test health_check_tests -- --ignored --nocapture`
- `cargo test --test cache_management_tests -- --ignored --nocapture`
- `cargo test --test route_path_operations_tests -- --nocapture`
- `cargo test --test subscription_tests -- --ignored --nocapture`
- `cargo run --example test_comprehensive_arrays_udt`
- `cargo run --example test_plc_test_tag_definitions`
- `cargo run --example perf_baseline_real_plc -- --iterations 100`

Environment used:
- `TEST_PLC_ADDRESS=192.168.0.101:44818`
- `TEST_PLC_SLOT=0`

Result:
- PASS: Routed session establishment succeeded through `1756-EN3TR` to the controller in slot 0.
- PASS: Read baseline probe succeeded for 44/44 expected test paths.
- PASS: Batch operations tests passed (5/5).
- PASS: Health check tests passed (2/2).
- PASS: Cache management tests passed (3/3).
- PASS: Route-path operation tests passed (5/5).
- PASS: Subscription tests passed (4/4), including fail-fast invalid-tag handling.
- PASS: Comprehensive arrays/UDT regression succeeded except for documented controller limitations on direct UDT array member writes.
- PASS WITH DOCUMENTED LIMITATIONS: `test_plc_test_tag_definitions` produced 333 passed / 59 failed. All 59 failures matched known controller write restrictions, not new library regressions.

Observed documented PLC limitations:
- Direct writes to UDT array element members return CIP extended error `0x2107`.
- Direct writes to standalone STRING tags fail on this controller path.
- Direct writes to STRING members inside UDTs fail on this controller path.

Observed deviations vs CompactLogix seed values:
- Controller-scoped test tags on this ControlLogix were mostly readable but seeded to zero/blank values.
- Program-scoped tags under `Program:TestProgram.*` retained the richer seeded dataset and validated normally.
- The limitation profile matched the CompactLogix validation despite the different initial values.

Hardware benchmark:
- Iterations per scenario: 100
- Tags used:
  - Single read: `gTestArray_DINT[0]`
  - Single write: `gTestArray_DINT[5]`
  - Batch read: `gTestArray_DINT[0-4]`, `gTestArray_REAL[0-1]`, `gTestArray_BOOL[0]`, `gTestArray_INT[0]`, `gTestUDT.Member1_DINT`
  - Batch write: `gTestArray_DINT[5-7]`
  - Mixed execute: read `gTestArray_DINT[0]`, write `gTestArray_DINT[5]`, read `gTestArray_REAL[0]`, read `gTestUDT.Member1_DINT`
- Results:
  - `single_read`: 238.6436 ms total, 2.3864 ms avg call, 419.03 ops/sec
  - `single_write`: 306.6707 ms total, 3.0667 ms avg call, 326.08 ops/sec
  - `batch_read`: 196.6781 ms total, 1.9668 ms avg call, 5084.45 logical ops/sec
  - `batch_write`: 306.5169 ms total, 3.0652 ms avg call, 978.74 logical ops/sec
  - `mixed_execute`: 335.0237 ms total, 3.3502 ms avg call, 1193.95 logical ops/sec

Benchmark interpretation:
- Batch reads on this ControlLogix target were substantially more efficient than repeated single reads for the same tag set.
- Batch writes also improved effective logical throughput versus single writes.
- The routed path through the `1756-EN3TR` did not show an obvious functional regression in the exercised scenarios.
- These numbers are hardware-specific and should be treated as a field baseline for this controller/network path, not a universal product claim.

Status assessment:
- The 0.7.0 hardening status on this ControlLogix target was acceptable for the exercised feature set.
- No unexpected regressions were found in routed connection, batch, cache, route-path, health-check, array addressing, program tag, or UDT read/nested access scenarios.
- Real-hardware evidence now covers both CompactLogix and ControlLogix families for the current release gate.

Follow-up issue links:
- None recorded from this validation run.
