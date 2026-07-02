---
id: CODEX-AJ
title: C# wrapper critical fixes — WriteUdtMember deadlock, UTF-8 marshalling, keep-alive serialization + native P/Invoke integration tests
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
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

## Claude review

## Verdict
