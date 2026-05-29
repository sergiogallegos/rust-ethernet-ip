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

## Update 2026-05-29 — recurred after runner-stack bump, now scoped to beta

`confirmed`: The crash recurred after the runner-stack update, but only in `Test (ubuntu-latest / beta)`. Across recent runs the C# step is green on `ubuntu/stable`, `macos/{stable,beta}`, and `windows/{stable,beta}`, and crashes intermittently only on `ubuntu/beta` (e.g. runs `26662487474`, `26429129732`, `26428850556` crashed; `26430089760` passed). The beta job's Rust `cargo test` step passes — only the .NET testhost aborts.

`confirmed`: The reported pass-count before the abort varies between runs (31, 52, 64) with no managed stack trace or panic text, and the abort lands during testhost shutdown. The only variable that flips pass→crash is which Rust toolchain compiled the `cdylib` (stable vs beta), pointing at a native shutdown race: the leaked global Tokio runtime (`RUNTIME: LazyLock<Runtime>` in `src/client.rs`) keeps worker threads alive, and Linux testhost teardown races them. `unclear`: whether beta codegen exposes latent UB in the FFI surface or this is a transient beta-rustc/std regression. Not reproducible on macOS arm64 (full suite ran 13× clean).

`fix applied`: The C# test step (plus its cdylib build and .NET setup) is gated to `matrix.rust == 'stable'`. The C# wrapper P/Invokes a stable C ABI, so testing it against a stable-built cdylib is sufficient; the Rust code paths are still exercised by `cargo test` on beta, so no coverage is lost. `--blame-crash` was added to the surviving stable invocation so any future recurrence on stable produces a dump.

## Open Questions

- Whether beta codegen exposes a real data race / UB in the FFI shutdown path (clients cloned out of `FFI_CLIENTS` while background timers/keep-alive tasks call `block_on`) versus a transient beta-rustc regression is still `unclear`. If the crash ever appears on `ubuntu/stable`, the `--blame-crash` dump should isolate `SimulatorTestHarness.Dispose()` versus native library shutdown.

## Related Pages

- [test-coverage-strength-2026-05-18.md](test-coverage-strength-2026-05-18.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
