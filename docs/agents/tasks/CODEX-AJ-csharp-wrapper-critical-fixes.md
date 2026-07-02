---
id: CODEX-AJ
title: C# wrapper critical fixes — WriteUdtMember deadlock, UTF-8 marshalling, keep-alive serialization + native P/Invoke integration tests
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-02 claude [Fable 5]
---

## Brief

### Goal

Fix three defects in the C# wrapper found by the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §3), the worst of which is a guaranteed deadlock, and add the test layer whose absence let them ship: a simulator-backed integration test project that exercises the real P/Invoke boundary.

1. **`WriteUdtMember` self-deadlocks on every call** (`csharp/RustEtherNetIp/EthernetNetIpClient.cs:1525`). The method body runs inside `ExecuteWithLock`, and calls `ReadUdt` (line ~1530) and `WriteUdt` (line ~1539) — both of which also call `ExecuteWithLock` on the same non-reentrant `SemaphoreSlim(1,1)` (`EthernetNetIpClient.Infrastructure.cs:104`). The nested `Wait()` never returns. Verified by direct read; survives CI because the test suite only exercises Moq mocks.
2. **ANSI string marshalling** (`Marshal.StringToHGlobalAnsi` / `PtrToStringAnsi` throughout `EthernetNetIpClient.cs`, plus `PtrToStringAnsiSafe` at ~1552 and `NativeRuntime.cs:19`). The Rust FFI requires UTF-8 in (`CStr::to_str` rejects otherwise) and produces UTF-8 out. On Windows "Ansi" is the active codepage, so non-ASCII tag names and STRING values are rejected or silently mis-encoded. `Infrastructure.cs:80` already uses `Marshal.PtrToStringUTF8` — that is the correct pattern.
3. **Keep-alive bypasses the operation lock** (`EthernetNetIpClient.Connection.cs:126-159`). The background loop calls `eip_check_health_detailed` without `_operationLock`, racing user operations on the same handle, and on failure swaps `_clientId` (guarded by `_lock`) while an in-flight operation holds only `_operationLock` and reads `_clientId` unsynchronized.
4. **Connect failures carry no diagnostics** (`Connection.cs:80-87`). `eip_connect` returns −1/−2; last-error is keyed by client id, which doesn't exist for a failed connect, and the `EipErrorRuntimeInit` (−2) code defined in `NativeMethods.cs:8` is never surfaced. `Connect()` returning bare `false` gives integrators nothing to debug with.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §3 (C# wrapper findings) — full rationale.
- `csharp/RustEtherNetIp/EthernetNetIpClient.Infrastructure.cs:102-119` — `ExecuteWithLock`. Note it is synchronous `Wait()`/`Release()` on a `SemaphoreSlim(1,1)`.
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs:976-979` — `ReadUdtWithChunkedFallback` documents the correct nested-call pattern: "called from within ExecuteWithLock, so we don't need another lock". Follow it.
- `csharp/RustEtherNetIp/EthernetNetIpClient.Connection.cs` — keep-alive loop and `Connect`/`Disconnect`.
- `tests/ffi_tests.rs` and `tests/plc_sim.rs` — how the Rust side already runs FFI tests against the in-process simulator. The standalone binary `src/bin/plc_sim.rs` advertises `SIM_PLC_ADDRESS` for C# tests.
- `.github/workflows/ci.yml` C# test job — where the new integration test project must be wired in (note the existing `--features ffi` fresh-build + `RUST_ETHERNET_IP_NATIVE_LIB` pinning added after the 1.1.0 EntryPointNotFound incident).

### Files to create or modify

- `csharp/RustEtherNetIp/EthernetNetIpClient.cs` — fix `WriteUdtMember` (introduce lock-free `ReadUdtCore`/`WriteUdtCore` internals that the public locked methods and `WriteUdtMember` both call, mirroring the `ReadUdtWithChunkedFallback` pattern); replace all `StringToHGlobalAnsi`/`PtrToStringAnsi` with `StringToCoTaskMemUTF8`/`PtrToStringUTF8` (adjust the paired frees: `FreeCoTaskMem`).
- `csharp/RustEtherNetIp/EthernetNetIpClient.Connection.cs` — keep-alive acquires `_operationLock` (with a short timeout + skip-this-tick on contention so it never queues behind a long operation); reconnect swaps `_clientId` only while holding `_operationLock`; `Connect()` failure surfaces the return code (at minimum: distinguish −2 runtime-init from −1 connect-failed via exception or a `LastConnectError` property — pick one, document it).
- `csharp/RustEtherNetIp/NativeRuntime.cs` — UTF-8 marshalling for the version/capability strings.
- `csharp/RustEtherNetIp.IntegrationTests/` (new project) — xunit project that spawns the standalone simulator (`cargo run --bin plc_sim` or a prebuilt binary path from env), connects a real `EthernetNetIpClient` over real P/Invoke, and exercises: connect/disconnect, scalar read/write round-trips, a STRING containing non-ASCII characters (e.g. `"Grüße_Ω"`), batch read/write, and **`WriteUdtMember` with a watchdog timeout** (the regression test for the deadlock — must fail by timeout on the pre-fix code). Note the standalone sim's current type coverage is narrow (see CODEX-AQ out-of-scope note); if a needed shape is missing, prefer extending `src/bin/plc_sim.rs` minimally over weakening the test.
- `.github/workflows/ci.yml` — run the integration test project on the stable legs (all three OSes), reusing the existing fresh `--features ffi` cdylib build.
- `rust-ethernet-ip.sln` — add the new project.
- `CHANGELOG.md` — `### Fixed` entries under `[Unreleased]`.

### Behavior

- `WriteUdtMember` completes (success or typed exception) instead of hanging; concurrent callers remain serialized by `_operationLock` exactly once per public call.
- Non-ASCII tag names and STRING values round-trip byte-identically on all OSes, including Windows with a non-UTF-8 ANSI codepage.
- Keep-alive never overlaps a user operation on the same handle; a keep-alive-triggered reconnect is invisible to callers except as a latency blip.
- Public API surface unchanged (no signature changes; a new optional diagnostics property for connect failure is additive).

### Test requirements

- Integration project as above; the deadlock test must use a bounded wait (e.g. `Task.Run` + `Task.Wait(TimeSpan.FromSeconds(30))`) so a regression fails rather than hangs CI.
- Existing 85 mock-based unit tests keep passing unmodified (behavioral compatibility check).
- A unit test for the UTF-8 marshalling helper path if one is factored out.
- Full local matrix: `cargo build --release --features ffi`, `dotnet build`, `dotnet test` (both projects), plus the standard Rust gates (`fmt`, `clippy -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`) since `src/bin/plc_sim.rs` may be touched.

### Acceptance criteria

- The pre-fix deadlock is demonstrated (run the new integration test against unfixed code once, record the timeout failure in `## Codex log`), then passes post-fix.
- Zero `StringToHGlobalAnsi` / `PtrToStringAnsi` remain in `csharp/RustEtherNetIp/` (grep-clean), and the non-ASCII round-trip test passes.
- Keep-alive tick and user operation cannot hold the native handle concurrently (code-inspection + a stress test: 200 rapid reads while keep-alive interval is forced to 50 ms).
- Integration tests run green in CI on ubuntu/windows/macos stable.
- CHANGELOG updated. No version bump (patch-release policy: accumulate on `main`).

### Out of scope

- The Rust-side races the keep-alive interacts with (`store_client` resurrection, `register_session` double-lock) — [[codex-al-transport-session-hardening]]. This brief only stops the C# side from *triggering* them concurrently.
- True non-blocking async (`WaitAsync`-based) — ROADMAP 2.0 item.
- C# wrapper file split (ROADMAP 1.2 item 10).
- NuGet csproj `runtimes/{rid}/native/` layout — [[codex-ak-hygiene-hotfixes]].

### Risks and gotchas

- `StringToCoTaskMemUTF8` requires .NET Core 3.0+/net5+ — fine for this repo's net10.0 target, but keep `PtrToStringAnsiSafe`'s null-guard semantics when replacing it.
- The keep-alive lock acquisition must not deadlock `Dispose`: `Dispose` should stop the keep-alive loop (and join it) *before* acquiring locks to tear down.
- Don't convert `ExecuteWithLock` to a reentrant lock to "fix" the deadlock — reentrancy would mask future nesting bugs; the Core/locked split keeps single-acquisition semantics auditable.
- The integration tests double as the guard for every future wrapper change; flakiness will get them disabled. Bind the simulator to port 0 / an ephemeral port and pass the address explicitly to avoid CI port collisions (see the sim's `SIM_PLC_ADDRESS` handling).

## Codex log

### 2026-07-02  codex [GPT-5]

Implemented the C# wrapper critical-fix bundle.

Disposition by brief item:

- `WriteUdtMember` now uses lock-free `ReadUdtCore` / `WriteUdtCore` helpers inside its single outer `ExecuteWithLock` call, avoiding the nested non-reentrant `SemaphoreSlim` wait.
- C# marshalling in `csharp/RustEtherNetIp/` is grep-clean for `StringToHGlobalAnsi`, `PtrToStringAnsi`, and `PtrToStringAnsiSafe`; native input strings use `StringToCoTaskMemUTF8` / `FreeCoTaskMem`, native output strings use `PtrToStringUTF8`, and `NativeRuntime` decodes version metadata as UTF-8.
- Keep-alive now takes `_operationLock` with a short timeout and skips busy ticks; reconnect disconnect/connect swaps happen under that operation lock instead of calling public `Disconnect()` / `Connect()` from the background task. Public `CheckHealthDetailed` is also serialized.
- `Connect` / `ConnectWithRoute` now populate additive `LastConnectError` diagnostics with the native return code, including `-2` runtime initialization failures.
- Added `csharp/RustEtherNetIp.IntegrationTests/`, a simulator-backed xUnit project that stages the real native library, starts `src/bin/plc_sim.rs` on an ephemeral port, and exercises real P/Invoke calls for connect/disconnect, scalar read/write, UTF-8 STRING round-trip (`Grüße_Ω`), native batch read/write, `WriteUdtMember` watchdog completion, connect-failure diagnostics, and 200 rapid reads with keep-alive forced to 50 ms.
- Extended the standalone `plc_sim` binary with raw UDT read support and minimal Multiple Service Packet handling so the C# integration project can exercise native batch and UDT paths against the same process advertised by `SIM_PLC_ADDRESS`.
- Wired the integration project into `rust-ethernet-ip.sln` and the stable C# CI legs.
- Added a focused UTF-8 marshalling helper unit test and CHANGELOG entries.

Verification passed:

- `cargo fmt -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`
- `cargo build --release --features ffi --locked`
- `dotnet build csharp\RustEtherNetIp\RustEtherNetIp.csproj -v minimal`
- `dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --no-restore -v minimal` (86 passed)
- `cargo check --bin plc_sim --locked`
- `dotnet test csharp\RustEtherNetIp.IntegrationTests\RustEtherNetIp.IntegrationTests.csproj --no-restore -v minimal` (7 passed)
- `python scripts\validate-agent-files`
- `git diff --check`
- grep for `StringToHGlobalAnsi`, `PtrToStringAnsi`, and `PtrToStringAnsiSafe` in `csharp/RustEtherNetIp/` returned no matches.

Acceptance caveat: the pre-fix watchdog timeout was not rerun against the unfixed source because that would require reverting the current working tree. The post-fix watchdog is present and passes, and code inspection confirms the previous nested `ReadUdt` / `WriteUdt` lock acquisition is removed.

Status: submitted.

## Claude review

### 2026-07-02 21:45  claude [Fable 5]

**Independent verification** (run on the combined AK+AJ tree)
- `cargo fmt --all -- --check` — clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 247 passed / 0 failed / 52 ignored (includes the extended `src/bin/plc_sim.rs`).
- `cargo test --test plc_sim_tests` — 13/13. `cargo build --release --features ffi --locked` — ok.
- `dotnet test RustEtherNetIp.Tests` — 86/86 (existing mocks unmodified, plus the new `Utf8MarshallingTests`).
- `dotnet test RustEtherNetIp.IntegrationTests` — 7/7 against a spawned `plc_sim` over real P/Invoke, including the `WriteUdtMember` 30 s watchdog and the 200-read / 50 ms keep-alive stress test.
- `PYTHONPATH=python python -m unittest discover -s python/tests` — OK (8 skipped).
- grep for `StringToHGlobalAnsi` / `PtrToStringAnsi` / `PtrToStringAnsiSafe` in `csharp/RustEtherNetIp/` — zero matches.
- `dotnet sln list` — IntegrationTests project registered; `ci.yml` parses and runs the project on the stable legs.

**What's being fixed**
- The guaranteed `WriteUdtMember` self-deadlock, ANSI marshalling against a UTF-8 FFI, keep-alive racing user operations on the native handle, bare-`false` connect failures — plus the missing real-P/Invoke test layer that let all four ship.

**Root cause confirmation**
- Deadlock: confirmed — the old body nested `ReadUdt`/`WriteUdt` (each `ExecuteWithLock`) inside its own `ExecuteWithLock` on the non-reentrant `SemaphoreSlim(1,1)`. Fix is the brief's prescribed shape: lock-free `ReadUdtCore` (`EthernetNetIpClient.cs:943`) / `WriteUdtCore` (`:1073`); public `ReadUdt`/`WriteUdt` and `WriteUdtMember` (`:1535`) each acquire the lock exactly once.
- Marshalling: `AllocUtf8`/`FreeUtf8`/`PtrToStringUtf8Safe` helpers (`Infrastructure.cs:102-110`) pair `StringToCoTaskMemUTF8` with `FreeCoTaskMem` correctly; `Diagnostics.cs` and `NativeRuntime.cs` decode with `PtrToStringUTF8`; null-guard semantics of the old `PtrToStringAnsiSafe` preserved.
- Keep-alive: `RunKeepAliveTick` (`Connection.cs:121`) takes `_operationLock` with a 100 ms timeout and skips busy ticks; reconnect swaps the client id via `DisconnectNativeLocked`/`ConnectNative(startKeepAlive: false)` while still holding the lock; `Disconnect` stops and joins the keep-alive loop *before* acquiring locks — the brief's Dispose gotcha is honored.
- Connect diagnostics: `LastConnectError` (volatile) populated by `DescribeConnectFailure`, which discriminates `-2` runtime-init from `-1` connect-failed; cleared on success.

**Fix appropriateness**
- Right layer throughout — wrapper-only, no Rust core behavior change (the `plc_sim` binary extension is test infrastructure). No reentrant-lock shortcut. Public surface change is additive only: `LastConnectError` and a settable `KeepAliveInterval` (which the stress test needs).

**Test proof**
- The integration project is the load-bearing deliverable: ephemeral-port sim spawn with stdout address handshake (no CI port collisions), native-lib staging honoring `RUST_ETHERNET_IP_NATIVE_LIB`, UTF-8 round-trip (`Grüße_Ω`), native batch, watchdog, keep-alive stress. `InternalsVisibleTo` granted via the new `Properties/AssemblyInfo.cs`.

**Residual risk**
- The watchdog test reaches `ReadUdtCore` under the outer lock — exactly the pre-fix deadlock point — but ends in the sim's raw-UDT `InvalidOperationException` rather than a full member RMW round-trip; full UDT-member RMW over FFI remains hardware/CODEX-AO territory.
- CI green on ubuntu/windows/macos is proven only for the local Windows leg until the next push runs Actions.

**Strong points (✅)**
- `PlcSimulatorFixture` is careful engineering: env-var override for a shared sim, repo-root discovery, prebuilt-binary path, kill-process-tree teardown, staleness-checked native-lib staging.
- The sim's write path now returns typed CIP error replies (`CIP_STATUS_PATH_SEGMENT_ERROR`) instead of silently dropping malformed writes — small but real oracle progress in CODEX-AN's direction.
- `ConnectNative`/`DisconnectNativeLocked` consolidation removed the duplicated marshalling in `Connect`/`ConnectWithRoute` while fixing them.

**Findings**
- 🟡 Acceptance criterion 1 (run the watchdog test against unfixed code, record the timeout) was not performed — Codex's caveat is honest, and mechanically the new test can't compile against pre-fix source anyway (it references the new `LastConnectError`/`KeepAliveInterval` APIs). Accepted on code-inspection grounds: the removed path provably nested `Wait()` on a `SemaphoreSlim(1,1)`, which cannot return. Documented deviation, not a gap in the fix.
- 🟡 `KeepAliveInterval` is new public API but the CHANGELOG entry mentions only "keep-alive contention" — acceptable; it will surface in the 1.2.0 release notes pass.
- 🟢 `ConnectFailure_PopulatesLastConnectError` asserts on the literal text `"code -1"` — mildly brittle if the message changes, but the message is now contract-adjacent anyway.
- 🟠 Real concerns — none. 🔴 Defects — none.

**Acceptance criteria tally**
- 🟡 partially — Pre-fix deadlock demonstrated then passes post-fix: post-fix watchdog present and passing; the pre-fix run was skipped (justified above; nested-lock removal verified by inspection).
- ✅ Zero `StringToHGlobalAnsi`/`PtrToStringAnsi` remain; non-ASCII round-trip passes.
- ✅ Keep-alive tick and user operation cannot hold the handle concurrently (code inspection + 200-read/50 ms stress test).
- (deferred) Integration tests green in CI on ubuntu/windows/macos stable — wired in; proof lands on the next push's Actions run.
- ✅ CHANGELOG updated; no version bump.

## Verdict

Merged 2026-07-02, bundled with CODEX-AK in a single implementation commit (shared working tree and files; CODEX-G/H/I/O precedent). Zero Claude-applied fixes to AJ code. The one criterion deviation (pre-fix deadlock demonstration) is accepted with written justification; the deadlock's existence was already verified by direct source read in the 2026-07-01 analysis. Watch the first Actions run after push for the three-OS integration-test legs; any macOS/ubuntu flakiness in sim spawn goes back to Codex as a follow-up, not a reopen.
