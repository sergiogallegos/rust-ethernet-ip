# .NET Testhost Shutdown - 2026-05-26

## Summary

`confirmed`: GitHub Actions run `26424069105` failed only in `Test (ubuntu-latest / stable)` after the C# suite reported `64/64` passing; VSTest then aborted because the testhost process crashed during shutdown.

`confirmed`: A 2026-08-21 crash-dump analysis superseded the earlier shutdown-race hypothesis. The faulting thread was inside `NativeLibrary.GetExport`, called by the test harness after another test had already loaded the same staged `.so` through P/Invoke.

## Current Understanding

- The failing run used `actions/setup-dotnet@v4` with `dotnet-version: 10.0.x` and `dotnet-quality: preview`, which installed SDK `10.0.100-rc.2.25502.107` on Ubuntu.
- The C# test project was still pinned to `Microsoft.NET.Test.Sdk 17.9.0`, `xunit 2.6.6`, and `xunit.runner.visualstudio 2.5.6`.
- GitHub's Ubuntu stable job reached the C# test step, built the wrapper, ran the test assembly, reported all 64 tests passed, then returned exit code 1 with `Test host process crashed`.
- Local validation after updating the runner stack to `Microsoft.NET.Test.Sdk 18.5.1`, `xunit 2.9.3`, and `xunit.runner.visualstudio 3.1.5` passed `79/79` C# tests on .NET 10.
- The durable 2026-08-21 fix makes MSBuild the sole owner of native-library staging. Running tests only check that the staged file exists and exercise it through CLR-managed P/Invoke; they never overwrite, manually reload, or unload it.

## Evidence

- [`.github/workflows/ci.yml`](../../.github/workflows/ci.yml) — .NET setup for CI test/package/build jobs.
- [`csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj`](../../csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj) — C# test runner package versions.
- [`csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs`](../../csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs) — simulator process and native-library staging path used by C# simulator tests.
- GitHub Actions run `26424069105`, job `Test (ubuntu-latest / stable)`, observed via `gh run view --log-failed`.

## Update 2026-05-29 — recurred after runner-stack bump, now scoped to beta

`confirmed`: The crash recurred after the runner-stack update, but only in `Test (ubuntu-latest / beta)`. Across recent runs the C# step is green on `ubuntu/stable`, `macos/{stable,beta}`, and `windows/{stable,beta}`, and crashes intermittently only on `ubuntu/beta` (e.g. runs `26662487474`, `26429129732`, `26428850556` crashed; `26430089760` passed). The beta job's Rust `cargo test` step passes — only the .NET testhost aborts.

`confirmed`: The reported pass-count before the abort varies between runs (31, 52, 64) with no managed stack trace or panic text, and the abort lands during testhost shutdown. The only variable that flips pass→crash is which Rust toolchain compiled the `cdylib` (stable vs beta), pointing at a native shutdown race: the leaked global Tokio runtime (`RUNTIME: LazyLock<Runtime>` in `src/client.rs`) keeps worker threads alive, and Linux testhost teardown races them. `unclear`: whether beta codegen exposes latent UB in the FFI surface or this is a transient beta-rustc/std regression. Not reproducible on macOS arm64 (full suite ran 13× clean).

`fix applied`: The C# test step (plus its cdylib build and .NET setup) is gated to `matrix.rust == 'stable'`. The C# wrapper P/Invokes a stable C ABI, so testing it against a stable-built cdylib is sufficient; the Rust code paths are still exercised by `cargo test` on beta, so no coverage is lost. `--blame-crash` was added to the surviving stable invocation so any future recurrence on stable produces a dump.

## Update 2026-07-02 — recurred on ubuntu/stable; retry + dump-upload applied

`confirmed`: The crash now reproduces on `Test (ubuntu-latest / stable)` — the leg the 2026-05-29 beta-scoping left as sole C# coverage. Runs `28567033593` (repo `b2fcf33`, docs-only push) and `28621972073` (repo `3caa686`) aborted with the same signature: all executed tests pass (78/78 in the latter), then "Test host process crashed" during teardown; run `28608241263` in between passed the same leg, confirming intermittency. The stable recurrence eliminates "beta-rustc regression" as the sole explanation — the native shutdown race (leaked global Tokio runtime threads vs. Linux testhost exit) stands as the primary hypothesis, pointing at the FFI teardown path (CODEX-AS territory).

`fix applied`: Both C# test steps retry once on failure (a genuine test failure still fails twice; only the post-pass teardown crash is absorbed), and a `--blame-crash` dump/sequence-file artifact upload (`if: always()`, 14-day retention) now preserves the evidence the 2026-05-29 open question asked for — including dumps from crashes the retry absorbed.

`next`: Pull `csharp-testhost-dumps-ubuntu-latest-*` from any run where the first attempt crashed and inspect whether the faulting thread is a Tokio worker inside the cdylib or the testhost's own teardown. A durable fix belongs to the FFI lifecycle work (CODEX-AS: unwind guard, teardown discipline), not to CI.

## Update 2026-08-21 — dump confirms unsafe shared-library restaging

`confirmed`: The crash dump from GitHub Actions run `32528795950` identifies the faulting managed stack as `System.Runtime.InteropServices.NativeLibrary.GetExport` → `SimulatorTestHarness.AssertNativeLibraryHasRequiredExports` → `SimulatorTestHarness.StageNativeLibrary` → the reported simulator test. No Tokio worker or batch-operation frame appears in the faulting path.

`confirmed`: ABI contract tests had already loaded the test-output `librust_ethernet_ip.so` through `DllImport`. The simulator harness then copied over that same output path and manually loaded, inspected, and freed it. Replacing and independently reloading an image already mapped by the CLR is unsafe on Linux. The batch test named by VSTest was only the next test constructing the harness, not evidence of a batch FFI defect.

`fix applied`: Both C# test projects now stage the release cdylib through MSBuild before testhost starts. The unit harness and native integration fixture only verify that the staged file exists. The manual `NativeLibrary.Load` / `GetExport` / `Free` test was replaced with a regression that loads through normal P/Invoke and proves repeated staging checks do not modify the file. CI retries were removed so every platform must pass once; crash-dump upload remains available for future failures.

`superseded`: The beta-codegen and leaked-Tokio-runtime shutdown theories were reasonable before a dump was available but do not match the captured faulting stack.

## Open Questions

- Whether unrelated FFI shutdown hazards exist remains a separate question; the captured Ubuntu testhost crash does not provide evidence for one.

## Related Pages

- [test-coverage-strength-2026-05-18.md](test-coverage-strength-2026-05-18.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
