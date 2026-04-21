# Real PLC Testing Guide

> Historical reference: this guide describes the pre-release `0.7.0` hardening workflow. Use the newer validation checklist and current release-validation records for the active process.

This guide defines the current real-hardware validation workflow for the unreleased `0.7.0` hardening line.

Use it when validating against CompactLogix or ControlLogix hardware with the dedicated `gTest*` tag set.

## Purpose

- Run a repeatable Rust + C# real-PLC validation pass.
- Capture evidence for gate `G9`.
- Leave the PLC test tags in a known state after the run.
- Separate real firmware limitations from library regressions.

## Preconditions

- Use dedicated test tags only.
- Confirm the PLC is reachable and not being used for production control.
- Confirm route-path/slot settings for the target family:
  - CompactLogix `5069-L320ERMS3` validated here with slot `0`
  - ControlLogix should use the actual CPU slot
- Set `TEST_PLC_ADDRESS` when needed. Default in current examples is `192.168.0.1:44818`.

## Recommended Run Order

### Rust

```powershell
cargo run --example readonly_plc_probe -- 192.168.0.1:44818
cargo test --test batch_operations_tests -- --ignored --nocapture
cargo test --test health_check_tests -- --ignored --nocapture
cargo test --test cache_management_tests -- --ignored --nocapture
cargo test --test route_path_operations_tests -- --nocapture
cargo test --test subscription_tests -- --ignored --nocapture
cargo run --example test_comprehensive_arrays_udt
cargo run --example test_plc_test_tag_definitions
cargo run --quiet --release --example perf_baseline_real_plc -- --iterations 100
```

### C#

```powershell
$env:DOTNET_CLI_HOME=(Join-Path $PWD '.dotnet')
$env:DOTNET_SKIP_FIRST_TIME_EXPERIENCE='1'
$env:DOTNET_NOLOGO='1'
$env:TEST_PLC_ADDRESS='192.168.0.1:44818'
New-Item -ItemType Directory -Force -Path $env:DOTNET_CLI_HOME | Out-Null

dotnet run --project examples\CSharpWrapperSmoke\CSharpWrapperSmoke.csproj
dotnet run --project examples\CSharpWrapperTest\CSharpWrapperTest.csproj
dotnet run --project examples\CSharpWrapperBenchmark\CSharpWrapperBenchmark.csproj -- --iterations 100
```

## Restore Policy

Current expectation:

- `examples/perf_baseline_real_plc.rs`
  - restores benchmark write targets at the end
- `examples/CSharpWrapperBenchmark/Program.cs`
  - restores benchmark write targets in a `finally` path
- `examples/CSharpWrapperSmoke/Program.cs`
  - restores the tags it modifies
- `examples/test_comprehensive_arrays_udt.rs`
  - restores the tags it modifies at the end of the run
- `examples/test_plc_test_tag_definitions.rs`
  - restores all successfully written tags in its restore step

Operational rule:
- If a PLC-backed runner writes test tags, it should either restore them before exit or clearly print that it is intentionally stateful.

## How To Classify Failures

Treat these as likely firmware limitations when they match the validated CompactLogix pattern:

- direct standalone `STRING` writes
- direct `STRING` member writes inside UDTs
- direct writes to UDT array element members

Observed CompactLogix `5069-L320ERMS3` / firmware `35` error shapes:

- direct `STRING` writes:
  - batch-level `0x1E` (`Embedded service error`) or extended `0x2107`
- direct UDT array element member writes:
  - extended `0x2107`

Treat these as regressions unless proven otherwise:

- failed reads for primitive tags that previously passed
- route-path/session failures on the same hardware/network setup
- subscription setup succeeding for invalid tags
- batch read/write regressions on the validated `gTest*` scenarios
- restore-step failures for tags that were previously restorable

## Evidence Naming

Store one Markdown record per hardware target under `docs/validation/`:

```text
YYYY-MM-DD_real_plc_<model>_fw<firmware>.md
YYYY-MM-DD_csharp_wrapper_real_plc_<model>_fw<firmware>.md
```

Each record should include:

- date
- tester
- PLC model
- firmware revision
- network topology / route-path
- commands executed
- pass/fail summary
- documented firmware limitations observed
- benchmark results
- remaining gaps

## Current Reference Records

- `2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md`

## Current CompactLogix Baseline

Validated today on:

- PLC: `5069-L320ERMS3`
- Firmware: `35`
- Address: `192.168.0.1:44818`
- Slot: `0`

Validated working areas:

- primitive reads/writes
- program-scoped tags
- route-path connection
- subscriptions
- UDT reads and nested array/member access
- Rust/C# mixed batch operations
- native BOOL batch reads on CompactLogix packed `0x00D3` responses

Remaining known hardware limits on that target:

- direct standalone/UDT `STRING` writes
- direct writes to UDT array element members
