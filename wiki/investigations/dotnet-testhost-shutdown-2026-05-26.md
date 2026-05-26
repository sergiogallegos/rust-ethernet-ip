# .NET Testhost Shutdown - 2026-05-26

## Summary

`confirmed`: GitHub Actions run `26424069105` failed only in `Test (ubuntu-latest / stable)` after the C# suite reported `64/64` passing; VSTest then aborted because the testhost process crashed during shutdown.

`likely`: The immediate CI fix is to stop installing a .NET 10 preview SDK on top of the runner image and to use a .NET 10-aware test runner stack.

## Current Understanding

- The failing run used `actions/setup-dotnet@v4` with `dotnet-version: 10.0.x` and `dotnet-quality: preview`, which installed SDK `10.0.100-rc.2.25502.107` on Ubuntu.
- The C# test project was still pinned to `Microsoft.NET.Test.Sdk 17.9.0`, `xunit 2.6.6`, and `xunit.runner.visualstudio 2.5.6`.
- GitHub's Ubuntu stable job reached the C# test step, built the wrapper, ran the test assembly, reported all 64 tests passed, then returned exit code 1 with `Test host process crashed`.
- Local validation after updating the runner stack to `Microsoft.NET.Test.Sdk 18.5.1`, `xunit 2.9.3`, and `xunit.runner.visualstudio 3.1.5` passed `79/79` C# tests on .NET 10.

## Evidence

- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — .NET setup for CI test/package/build jobs.
- [`csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj`](../../csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj) — C# test runner package versions.
- [`csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs`](../../csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs) — simulator process and native-library staging path used by C# simulator tests.
- GitHub Actions run `26424069105`, job `Test (ubuntu-latest / stable)`, observed via `gh run view --log-failed`.

## Open Questions

- Whether a simulator-process teardown bug or native cdylib unload race still exists is `unclear`; the current evidence shows the runner/testhost stack was stale enough to be the first fix.
- If Ubuntu CI still crashes after the runner-stack update, the next investigation should add `--blame-crash`, inspect generated dumps, and isolate `SimulatorTestHarness.Dispose()` versus native library shutdown.

## Related Pages

- [test-coverage-strength-2026-05-18.md](test-coverage-strength-2026-05-18.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
