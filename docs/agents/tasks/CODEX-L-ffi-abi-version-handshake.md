---
id: CODEX-L
title: FFI ABI version + capability handshake
owner: codex
status: merged
created: 2026-05-18
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

### Goal

Export a small fixed set of versioning and capability symbols from the C FFI surface so that downstream wrappers (`csharp/RustEtherNetIp`, `python/rust_ethernet_ip`, and any future binding) can verify at load time that they were built against a compatible `cdylib`. Pin the current FFI shape as `eip_abi_version() == 1`. This establishes the contract that protects every subsequent FFI-touching refactor (CODEX-M registry audit, eventual CODEX-P actor wiring, the CODEX-K release-window FFI ordered-hop change).

Driven by the architecture review at [`wiki/investigations/architecture-review-2026-05-18.md`](../../../wiki/investigations/architecture-review-2026-05-18.md). Phase 0, item 1 of the post-books roadmap. **This brief runs before CODEX-M and CODEX-N**; the ABI baseline must be in place before any FFI internals move.

### Context to read first

- `src/ffi.rs:1-100` — current FFI runtime singleton, client registry layout, existing `#[no_mangle] extern "C"` exports.
- `src/version.rs` (46 lines) — any existing version helpers; reuse if present.
- `Cargo.toml:7-20` — `version`, `rust-version`, the publish `include = [...]` list, the `ffi` feature gate.
- `csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs` — how the C# wrapper declares `[DllImport]` against `rust_ethernet_ip`.
- `python/rust_ethernet_ip/bindings.py` — how the Python wrapper resolves symbols via `ctypes.CDLL` and `_configure_function_signatures`.
- `wiki/investigations/architecture-review-2026-05-18.md` — the parent synthesis document.

### Files to create or modify

- `src/ffi.rs` — add three new `#[no_mangle] extern "C"` functions (signatures below).
- `src/version.rs` — add a `pub const ABI_VERSION: u32 = 1;` constant and a `pub const CAPABILITIES: u64 = ...;` bitmap. Keep them in this small file so they're easy to bump.
- `tests/ffi_abi.rs` — new Rust integration test (gated on `cfg(feature = "ffi")`) calling each new export.
- `csharp/RustEtherNetIp/NativeRuntime.cs` (new file) — static class exposing `AbiVersion`, `LibraryVersion`, `Capabilities`. Add static-constructor mismatch check that throws `BadImageFormatException` when `AbiVersion != 1`.
- `csharp/RustEtherNetIp.Tests/AbiContractTests.cs` (new file) — verifies the three values via the new static class.
- `python/rust_ethernet_ip/bindings.py` — add `load_native_library()` ABI verification: read `eip_abi_version()` after `CDLL`, raise `NativeLibraryLoadError` on mismatch.
- `python/tests/test_abi_contract.py` (new file) — verifies module-level globals.
- `wiki/protocol/abi-contract.md` (new file) — single-page reference: what ABI versions mean, what capability bits exist, what triggers a bump.

### Behavior

Three new exported symbols, signatures fixed by this brief:

```rust
#[unsafe(no_mangle)]
pub extern "C" fn eip_abi_version() -> u32 { crate::version::ABI_VERSION }

/// Returns a static, null-terminated semver string sourced from CARGO_PKG_VERSION.
/// Pointer is valid for the process lifetime; the caller must NOT free.
#[unsafe(no_mangle)]
pub extern "C" fn eip_library_version() -> *const std::os::raw::c_char {
    static VERSION_C: std::sync::OnceLock<std::ffi::CString> = std::sync::OnceLock::new();
    VERSION_C
        .get_or_init(|| std::ffi::CString::new(env!("CARGO_PKG_VERSION")).expect("static"))
        .as_ptr()
}

#[unsafe(no_mangle)]
pub extern "C" fn eip_capabilities() -> u64 { crate::version::CAPABILITIES }
```

Capability bitmap — fixed table for ABI v1 (do not change values without bumping ABI):

| Bit | Name | Meaning |
|---|---|---|
| `0x0000_0000_0000_0001` | `ROUTE_PATH_ORDERED_HOPS` | `eip_connect_with_route` understands ordered hops (post-CODEX-F). |
| `0x0000_0000_0000_0002` | `BATCH_EXECUTE_V1` | `eip_execute_batch` present and stable. |
| `0x0000_0000_0000_0004` | `DIAGNOSTICS_JSON` | `eip_get_diagnostics_json` present and stable. |
| `0x0000_0000_0000_0008` | `TAG_GROUP_SUBSCRIPTIONS` | `eip_subscribe_to_tag_group` and friends. |
| Other bits | reserved | Must be 0 in ABI v1. |

`src/version.rs` exposes the bitmap and the constant. Future tasks set the appropriate bits when they wire new capabilities.

Wrapper-side behavior:

- **C#**: `NativeRuntime` static class with `AbiVersion`, `LibraryVersion`, `Capabilities` properties. Static constructor reads the values once and throws `BadImageFormatException` if `AbiVersion != NativeRuntime.ExpectedAbiVersion` (where `ExpectedAbiVersion` is a `const int = 1` in the wrapper).
- **Python**: `bindings.load_native_library()` reads `eip_abi_version()` after `CDLL` and before `_configure_function_signatures`; raises `NativeLibraryLoadError` with a clear message ("native library ABI version X, wrapper expects Y") on mismatch. Expose `rust_ethernet_ip.ABI_VERSION`, `LIBRARY_VERSION`, `CAPABILITIES` as module-level constants populated at first load.

### Test requirements

- `tests/ffi_abi.rs` calls all three new symbols, asserts `eip_abi_version() == 1`, `eip_library_version()` returns a non-null pointer with content `env!("CARGO_PKG_VERSION")`, `eip_capabilities() & ROUTE_PATH_ORDERED_HOPS != 0`.
- `csharp/RustEtherNetIp.Tests/AbiContractTests.cs` asserts the three properties match the same values via `[Fact]` xUnit tests. Add an explicit test that confirms the static constructor mismatch path by mocking — or, if mocking is awkward across native boundary, document why the live happy-path test is sufficient.
- `python/tests/test_abi_contract.py` asserts the three module-level constants. Use the harness already established in `python/tests/test_import.py` (no PLC required).
- Run all three wrapper test suites in CI; the new `package` CI job will catch any wheel/NuGet packaging miss.

### Acceptance criteria

- `nm target/release/librust_ethernet_ip.so` on Linux shows `eip_abi_version`, `eip_library_version`, `eip_capabilities` as `T` (text/exported).
- `dumpbin /exports target/release/rust_ethernet_ip.dll` on Windows shows the same three names.
- C# `dotnet test` includes the new `AbiContractTests` cell and it passes on ubuntu / windows / macos.
- Python `python -m unittest discover -v tests` includes `test_abi_contract` and passes on all three OSes × py 3.10 / 3.11 / 3.12.
- New `wiki/protocol/abi-contract.md` documents the bitmap, the bump policy, and the load-time handshake behavior.
- No existing FFI symbol changes signature.
- CI: every job stays green (including the new package + version-check jobs).

### Out of scope

- Defining the exact criteria for "ABI break" beyond the seed rule "any change to a `#[no_mangle] extern "C"` signature, struct layout passed through the boundary, or call-conv contract." That goes in `wiki/protocol/abi-contract.md` as opening prose; refining it is a future doc task.
- Actually bumping `ABI_VERSION` to 2. This brief only establishes the mechanism. The next FFI-breaking brief is responsible for the bump.
- Auditing the *contents* of every existing FFI export for correctness. That is CODEX-M's job.
- Cross-language enum sync (the codex `BadImageFormatException` is a hard fail, not a semver-aware downgrade). Soft compatibility checks live in a future brief if ever needed.

### Risks and gotchas

- Pinning `ABI_VERSION = 1` *right now* declares the current FFI surface as the v1 baseline. Any pre-existing FFI design mistake (CODEX-M may find some) ships under v1. Acceptable — better to lock the baseline than to keep the door open indefinitely.
- The `eip_library_version()` pointer lifetime: callers (C#/Python) must not free the buffer. Document this loudly in the brief output (`wiki/protocol/abi-contract.md` should call it out) and in the function's Rust doc comment.
- `OnceLock` requires Rust ≥ 1.70 (already satisfied; MSRV is 1.95).
- The C# `BadImageFormatException` thrown from a static constructor surfaces in user code as `TypeInitializationException`. Document the unwrap pattern (`ex.InnerException is BadImageFormatException`) in the wrapper README so consumers can write reasonable error handling.
- The Python `ABI_VERSION` constant evaluated at import time means the *first* `import rust_ethernet_ip` triggers native load. If a consumer's test setup imports without the native lib present and previously got away with it (because loading was lazy via `Client.__init__`), this brief makes the load eager. Verify against existing wrapper tests; if regression, gate the ABI check behind a deferred `_ensure_compatible()` called from `Client.__init__` instead.

## Codex log

### 2026-05-24 codex [gpt-5]

- Added ABI v1 constants and capability bits in `src/version.rs`.
- Exported `eip_abi_version()`, `eip_library_version()`, and `eip_capabilities()` from `src/ffi.rs` without changing existing FFI symbol signatures.
- Added Rust FFI ABI coverage in `tests/ffi_abi.rs`.
- Added Python load-time ABI verification, module-level ABI metadata accessors, and ABI mismatch tests in `python/tests/test_abi_contract.py`.
- Added C# `NativeRuntime` ABI metadata and live contract tests in `csharp/RustEtherNetIp.Tests/AbiContractTests.cs`.
- Added `wiki/protocol/abi-contract.md` and indexed/logged the new maintainer synthesis page.
- Verification:
  - `PYTHONPATH=python python3 -m unittest python.tests.test_abi_contract python.tests.test_bindings python.tests.test_import`
  - `cargo test --features ffi --test ffi_abi`
  - `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore -v minimal`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

**Independent verification reproduced everything Codex claimed plus a full workspace re-run:**

- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — **218 passed, 0 failed** across all test binaries (was 208 before this bundle; the +10 reflects the new ABI, BOOL, and CIP-validation tests landing together)
- `cargo test --test plc_sim_tests` — 13/13 (was 10/10)
- `PYTHONPATH=python python3 -m unittest discover -s python/tests` — 35 passed, 8 skipped (simulator-gated, expected)
- `dotnet test` after rebuilding `--release --features ffi` — 79/79 (3 ABI contract tests included)
- `nm target/release/librust_ethernet_ip.dylib | grep eip_` confirms the three new exports (`eip_abi_version`, `eip_library_version`, `eip_capabilities`) are present

**Strong points (✅):**
- `src/version.rs:6-25` is the cleanest possible factoring: `ABI_VERSION: u32 = 1`, four named `CAP_*` constants, and a single `CAPABILITIES: u64` that ORs them. The capability bit naming is self-documenting (`CAP_ROUTE_PATH_ORDERED_HOPS` etc.) and matches the wiki entry exactly — no drift risk.
- `eip_library_version()` (`src/ffi.rs:35-40`) uses `OnceLock<CString>` correctly — the returned pointer is valid for process lifetime, no leak, no alloc per call. The `expect("static version")` is sound because `CARGO_PKG_VERSION` cannot contain interior NULs.
- C# `NativeRuntime` (`csharp/RustEtherNetIp/NativeRuntime.cs`) uses a static constructor + `BadImageFormatException` on mismatch — exactly the right exception class for "the native code I was compiled against doesn't match what loaded".
- Python `_verify_abi` (`bindings.py:173-188`) gates the typed-signature configuration on a successful ABI check; if the ABI check fails, the typed sigs aren't configured and the wrong-symbol failure mode can't happen at a later call site. Defensive ordering.
- The `__getattr__` lazy export in `__init__.py:29-32` lets `rust_ethernet_ip.ABI_VERSION` work without forcing native-lib load at import time (the test that mocks `CDLL` validates this).
- Pinned tests on both wrappers cover the mismatch path (`AbiMismatchPathIsDocumentedByExpectedVersionConstant`, `test_load_native_library_rejects_abi_mismatch`) — future ABI changes will trip these immediately if the wrapper's expected version isn't bumped.
- `wiki/protocol/abi-contract.md:1-48` is short and load-bearing: capability bits, bump policy, mismatch semantics. Exactly what the bump rules should look like for v0.8.0 onwards.

**Findings (🟡 polish, non-blocking):**
- 🟡 `src/version.rs:27-34` carries pre-existing dead `MAJOR_VERSION = 0; MINOR_VERSION = 5; PATCH_VERSION = 5` constants that are not updated by anything and don't match `CARGO_PKG_VERSION`. **Not Codex's introduction** — these predate the brief. Worth a follow-up cleanup line in CODEX-H. Don't touch in this brief.
- 🟡 `python/tests/test_abi_contract.py:30-33` — `FakeNativeLibrary.__getattr__` returns a fresh `FakeFunction(0)` for any unknown attribute, then stores it. Test convenience helper, fine, but flag for future readers: any new ABI export will silently default to 0 in the mocked path until tests pin the expected return. Not a bug, just a footgun for future Codex.

**Findings (🟠 real concerns) — none.**

**Acceptance criteria tally** (against the brief's "Acceptance criteria" section):
- ✅ `ABI_VERSION = 1` exported
- ✅ Capability bitmap exported with four named bits, doc'd in wiki
- ✅ C# wrapper rejects mismatch at NativeRuntime static init
- ✅ Python wrapper rejects mismatch at `load_native_library()`
- ✅ Rust `ffi_abi.rs` test exercises all three exports
- ✅ C# + Python contract tests cover happy path and mismatch path
- ✅ Wiki page lands at `wiki/protocol/abi-contract.md` with bump policy

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** Foundational Phase 0 work that pins the FFI contract before CODEX-M (registry semantics) and CODEX-N (CIP validation) touch it. The capability bitmap leaves a clean expansion path for future ABI v1 additions (set higher bits) and makes ABI v2 cutover explicit. Both wrapper handshakes throw the semantically-correct exception class on mismatch. Pinned tests guarantee no silent ABI drift slips through.

No defects to fix during merge. Brief authored cleanly, implementation matched the brief without scope creep.
