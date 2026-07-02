---
id: CODEX-AU
title: C++ consumer support — C header with parity gate, RAII example, Qt integration guide
owner: codex
status: open
created: 2026-07-02
last-update: 2026-07-02 claude [Fable 5]
---

## Brief

### Goal

The maintainer has a C++/Qt vision-system PC application that needs this driver. The crate already ships everything a C++ consumer requires — the `cdylib` with a stable C ABI (`src/ffi.rs`, `--features ffi`), an ABI version + capability handshake (CODEX-L), and `eip_get_last_error` — but there is no C header (the C# wrapper declares P/Invoke imports in-code, so none was ever needed). This task adds first-class C/C++ consumption: a checked-in header whose accuracy is CI-enforced, a small C++ RAII example that smoke-tests against the in-process simulator, and an integration guide covering the threading model Qt consumers must respect.

Deliverables:

1. **`include/rust_ethernet_ip.h`** — C header (C99-compatible, `extern "C"` guarded) declaring the public FFI surface: connect/disconnect/route variants, the scalar read/write set, STRING and UDT operations, batch operations, discovery, subscription/polling exports if present, `eip_abi_version` + capability constants, and `eip_get_last_error`. **Exclude the three raw `*mut EipClient` exports** that CODEX-AS is privatizing — the header must never advertise them. Document each function group briefly (return-code convention, ownership of out-params, UTF-8 expectations).
2. **Header-accuracy gate in CI.** The 22 scalar wrappers are macro-generated, so cbindgen's syntactic pass cannot see them and `parse.expand` drags in a nightly-toolchain dependency — a hand-authored header with an enforced parity gate is the recommended shape (cbindgen is acceptable only if it works on stable and covers the macro-generated exports; record the decision either way). The gate must enforce both directions: (a) every header declaration links against the built cdylib — a C or C++ translation unit that takes the address of every declared function and links catches removals/renames at compile time; (b) every exported `eip_*` symbol in the cdylib appears in the header — a script comparing the dylib's export table (`nm`/`dumpbin`/`objdump`, platform-appropriate) against declarations parsed from the header catches additions. Wire both into CI on at least ubuntu + windows.
3. **`examples/cpp/`** — a minimal CMake project: a single-header RAII wrapper (`EipClient` class: factory/connect, destructor disconnects, methods returning a small result type that carries the `eip_get_last_error` detail on failure) plus a console demo that connects, round-trips a DINT/REAL/STRING, and runs a small batch. A ctest-driven smoke runs it against the standalone simulator (`cargo run --bin plc_sim` or the in-process sim's TCP listener — follow whatever `tests/ffi_tests.rs`/C# integration tests use to get a live endpoint) and asserts the round-trip values. Build + smoke wired into CI (ubuntu + windows; macOS optional).
4. **Qt integration guide** — `docs/CPP_INTEGRATION.md`: how to build (`cargo build --release --features ffi`), link, and deploy (ship the dylib next to the executable); the ABI handshake at startup; error handling; and the load-bearing section: **every FFI call blocks** (each call does a `block_on` against the library's global Tokio runtime), so Qt applications must keep the client off the GUI thread — one owning `QThread` worker per client handle, values published via signals/slots, writes delivered via queued invocations. Include a compact worker-class sketch (header-only listing in the doc, not a compiled example). State the concurrency contract explicitly from [`docs/agents/notes/ffi-safety.md`](../notes/ffi-safety.md): what the registry mutex serializes, and that a client handle should be treated as single-owner.
5. **README** — short "Using from C++" section pointing at the header, example, and guide. CHANGELOG `[Unreleased]` `### Added`.

### Context to read first

- `src/ffi.rs` end to end — the export inventory is the header's contract; note the macro that generates the scalar wrappers (the header must list each generated function by its concrete exported name).
- [`docs/agents/notes/ffi-safety.md`](../notes/ffi-safety.md) — the invariants the guide must restate for C++ consumers.
- `csharp/RustEtherNetIp/RustEtherNetIp.cs` — the existing consumer of record: P/Invoke signatures are the ground truth for parameter types/ownership the header must mirror; the Dispose/finalizer pattern informs the RAII wrapper.
- `docs/agents/tasks/CODEX-AS-*.md` — privatizes the raw-pointer exports and touches the last-error lifecycle. Sequence after AS if it is in flight; otherwise exclude the raw exports now and note the last-error semantics may tighten.
- `docs/agents/tasks/CODEX-AJ-*.md` — its simulator-backed P/Invoke integration-test rig is the pattern for the C++ smoke's live endpoint.
- CI workflow (`.github/workflows/ci.yml`) — where the new legs attach; keep the added wall-clock small (the C++ example is tiny; CMake + a system compiler are already present on the hosted runners).

### Files to create or modify

`include/rust_ethernet_ip.h`, `examples/cpp/` (CMakeLists.txt, wrapper header, demo, smoke), the parity-gate script under `scripts/`, `.github/workflows/ci.yml`, `docs/CPP_INTEGRATION.md`, `README.md`, `CHANGELOG.md`. No changes to `src/ffi.rs` behavior — this task is packaging; if the header work surfaces an FFI defect, report it to the board rather than fixing it here (likely CODEX-AS territory).

### Behavior

- A C++ consumer can `#include <rust_ethernet_ip.h>`, link the release cdylib, and round-trip tags against a PLC or the simulator with no other artifacts.
- Header drift fails CI in both directions (removed/renamed export → link error; new unlisted export → parity script failure).
- The example builds warning-clean at `/W4` (MSVC) and `-Wall -Wextra` (gcc/clang).

### Test requirements

- The ctest smoke: connect to the sim endpoint, write+read DINT/REAL/STRING, batch of ≥3 tags, assert values; failure surfaces `eip_get_last_error` text.
- The parity gate runs in CI and is demonstrated to catch a seeded drift once during development (note the demonstration in the Codex log — e.g. temporarily deleting a header line and observing the failure).
- Full matrix untouched-code sanity: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked` (no Rust changes expected, but the CI edit must not break existing legs).

### Acceptance criteria

- Header covers every public `eip_*` export except the CODEX-AS-excluded raw-pointer trio; parity gate green on ubuntu + windows.
- `examples/cpp` builds and its smoke passes in CI on ubuntu + windows.
- `docs/CPP_INTEGRATION.md` exists with the threading-model section and the Qt worker sketch; README section added.
- STRING round-trip in the smoke: if CODEX-AT has landed, assert the full write+read; if not, the smoke reads STRING and asserts the write returns the documented error — note which contract was tested.

### Out of scope

- A `cxx`-based Rust↔C++ bridge (adds a second FFI surface for no gain over the existing C ABI).
- Static-library distribution, vcpkg/conan packaging (ROADMAP candidates if C++ demand grows).
- A compiled Qt example in CI (Qt provisioning cost; the guide's sketch suffices — a maintainer-side Qt build is welcome validation but not a gate).
- Async/callback FFI (ROADMAP: optional C# true-async FFI is the related 2.0 item).

### Risks and gotchas

- Type mapping discipline: `c_int` vs `int32_t`, `usize` out-params, and bool conventions must match `ffi.rs` exactly — copy from the C# `DllImport` signatures, which are field-tested, not from memory.
- Windows link details: the smoke must link against the import lib or use delay-load/`LoadLibrary` consistently; document whichever the example uses. MSVC vs MinGW consumers both work against a C ABI cdylib — say so in the guide, but CI only proves MSVC.
- The simulator endpoint for the smoke must bind an ephemeral port or serialize with other sim-using CI legs to avoid flaky port collisions (see how existing sim tests handle it).
- Keep the RAII wrapper header-only and dependency-free — it will be vendored into consumer projects (the maintainer's vision app) and must not drag CMake targets with it.

## Codex log

## Claude review

## Verdict
