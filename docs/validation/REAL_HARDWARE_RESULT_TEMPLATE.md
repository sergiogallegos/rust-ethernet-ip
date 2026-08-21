# YYYY-MM-DD Real-Hardware Validation — processor firmware

Library version:
Library commit:
Tester:

## Target

- Family:
- Processor catalog number:
- Processor firmware revision:
- Communication module and firmware:
- Topology and route (omit sensitive addresses):
- Host OS and architecture:
- Rust toolchain / .NET / Python / C++ compiler versions:

## Safety and Restore Plan

- Test-only controller/program:
- Write targets reviewed:
- Starting values captured:
- Restore or settle procedure:

## Bindings Tested

| Binding | Package/library version | Native ABI/library artifact | Result |
|---|---|---|---|
| Rust | | | Not run |
| C# | | | Not run |
| Python | | | Not run |
| C/C++ | | | Not run |

## Functional Results

| Binding | Reads | Writes | Verify | Expected rejections | Unexpected anomalies | Result |
|---|---:|---:|---:|---:|---:|---|
| Rust | | | | | | |
| C# | | | | | | |
| Python | | | | | | |
| C/C++ | | | | | | |

Record controller/program scope, arrays, packed BOOLs, strings, custom strings,
UDTs, fragmentation, batches, discovery, diagnostics, subscriptions, and route
behavior here.

## Endurance Results

- Profile and duration:
- Poll rates and tag counts:
- Total operations and errors:
- Reconnect count and recovery times:
- Latency p50/p95/p99/max:
- Initial/final/peak RSS and CPU:
- Controller communication/task-load observations:
- Data gaps, stale values, dropped events, or mismatches:

## Performance Results

Include warm-up, sample count, tag/payload shape, batch sizes, latency
distribution, throughput, error rate, and raw-result artifact paths.

## Anomalies and Interpretation

Distinguish library defects, harness defects, controller rejections, network
events, and unresolved observations. Include exact CIP general/extended status
when available.

## Restore Confirmation

- Final controller state:
- Values restored or intentionally settled:
- Temporary logic/tags removed or retained:

## Verdict

State exactly what this processor/firmware/topology/binding combination
confirmed. Do not generalize to untested firmware or models.
