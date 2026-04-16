# 2026-04-16 C# Wrapper Real PLC Validation - ControlLogix 1756-L81ES

Date: 2026-04-16
Tester: Codex + Sergio Gallegos
PLC model: 1756-L81ES
Network topology: Routed Ethernet connection to `192.168.0.101:44818` via `1756-EN3TR`, backplane slot `0`

## Scope

This is a follow-up C# wrapper validation for the `0.7.1` draft line against the same ControlLogix target and `gTest*` tag set used during the `0.7.0` release validation.

## Commands Executed

- `cargo build --release`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperSmoke/CSharpWrapperSmoke.csproj`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperBenchmark/CSharpWrapperBenchmark.csproj -- --iterations 100`
- `TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpWrapperTest/CSharpWrapperTest.csproj`
- `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -v minimal`

## Result

- PASS: C# wrapper smoke validation passed.
- PASS: C# wrapper benchmark completed successfully.
- PASS WITH DOCUMENTED LIMITATIONS: Comprehensive wrapper matrix produced 333 passed / 59 failed / 0 skipped, matching the previous ControlLogix limitation profile.
- PASS: C# wrapper tests passed 41/41.

## Issue Found and Fixed During Validation

The C# validation example projects were hardcoded to copy `target/release/rust_ethernet_ip.dll`. On macOS this prevented the examples from running because the native library is `target/release/librust_ethernet_ip.dylib`.

Fix applied:

- Updated `examples/CSharpWrapperSmoke/CSharpWrapperSmoke.csproj`
- Updated `examples/CSharpWrapperBenchmark/CSharpWrapperBenchmark.csproj`
- Updated `examples/CSharpWrapperTest/CSharpWrapperTest.csproj`
- Updated `csharp/RustEtherNetIp/RustEtherNetIp.csproj`

The projects now use `MSBuild::IsOSPlatform(...)` to copy the correct native library for macOS and Windows.

## Known PLC Limitations Observed

The 59 expected write failures were all in documented firmware-limited categories:

- 4 direct `STRING` write paths surfaced as batch-level `0x1E` embedded service errors
- 55 direct writes to UDT array element members surfaced as the known `0x2107` limitation

No new C# wrapper behavior regression was identified.

## Hardware Benchmark

Iterations per scenario: 100

| Metric | Total ms | Avg call ms | Logical ops/sec |
|---|---:|---:|---:|
| `single_read` | 146.7322 | 1.467322 | 681.5136691196615 |
| `single_write` | 303.8717 | 3.0387169999999997 | 329.0862558112519 |
| `batch_read` | 173.2961 | 1.732961 | 5770.470310641728 |
| `batch_write` | 295.1683 | 2.951683 | 1016.3693052404341 |
| `mixed_execute` | 177.3201 | 1.773201 | 2255.807435254097 |

## Assessment

The C# wrapper remains stable on the exercised routed ControlLogix feature set for the `0.7.1` draft line. The validation example project portability issue was fixed, and the remaining matrix failures are unchanged controller firmware limitations.
