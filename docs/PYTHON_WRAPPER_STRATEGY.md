# Python Wrapper Strategy

This document captures the current recommended strategy for adding Python support to `rust-ethernet-ip`.

Date: 2026-04-19

## Summary

Recommendation:

- build the Python wrapper on top of the existing external Rust FFI boundary
- keep Rust as the source of truth for protocol behavior
- keep Python thin and focused on usability
- treat service examples and data workflows as a second layer on top of the wrapper, not as part of the protocol core

This is the best fit for the repo's current shape and for the project's core vision:

- strong Rust EtherNet/IP core
- thin wrappers for user projects
- future support for more than one language without duplicating protocol logic

## Current Repo State

The repo already has the pieces needed to support this direction:

- Rust core crate builds both `rlib` and `cdylib` via [Cargo.toml](../Cargo.toml)
- the external native boundary is already explicit in [src/ffi.rs](../src/ffi.rs)
- the C# wrapper already consumes that boundary through `DllImport`
- release/build/test flows already validate the native + C# pairing

Important current architectural facts:

- [src/ffi.rs](../src/ffi.rs) exports a broad C ABI surface using `#[unsafe(no_mangle)]`
- [csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs](../csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs) demonstrates the current wrapper pattern cleanly
- [docs/SOFTWARE_ARCHITECTURE.md](SOFTWARE_ARCHITECTURE.md) now describes the intended ownership split

## Historical Note

The repo contains historical documentation that references prior wrapper work such as:

- `pywrapper/`
- `gowrapper/`
- earlier PyO3-based wrapper notes

Examples:

- [docs/ALL_WRAPPERS_UPDATE_COMPLETE.md](ALL_WRAPPERS_UPDATE_COMPLETE.md)
- [docs/WRAPPER_UPDATE_SUMMARY.md](WRAPPER_UPDATE_SUMMARY.md)

Those paths do not currently exist in the live repo tree. They should be treated as historical notes, not current maintained architecture.

## Strategy Options Considered

## Option A: Python on Top of the Existing C ABI

Shape:

- Rust core
- C ABI in `src/ffi.rs`
- Python binding layer using `ctypes`, `cffi`, or a small native-extension shim over the exported ABI

Pros:

- aligns with the existing wrapper strategy
- keeps one reusable external contract for multiple languages
- avoids splitting semantics between C# and Python
- strengthens long-term multi-language support
- keeps Rust as the only protocol implementation

Cons:

- Python API ergonomics must be built manually
- memory management and marshaling need careful design
- the current FFI surface may need some formalization for Python-friendliness

## Option B: Direct PyO3 / maturin Binding

Shape:

- Rust core
- Python extension module generated directly from Rust

Pros:

- can produce an ergonomic Python package quickly
- less manual marshaling for some types
- good Python packaging story in isolation

Cons:

- creates a second wrapper architecture separate from the current C# strategy
- weakens the value of the existing stable external boundary
- increases the risk of Python-only semantics drifting from C# and FFI consumers
- less aligned with long-term multi-language reuse

## Recommended Choice

Choose Option A.

Reasoning:

- the repo already has a meaningful external boundary
- the project vision is Rust-core-first, wrapper-second
- Python should be another adoption layer, not a parallel integration model
- a stable ABI layer is the most reusable foundation if more languages or service adapters are added later

## Recommended Python MVP Scope

The first Python release should stay narrow and practical.

Include:

- `Client(...)` construction / context-manager support
- connect / disconnect
- `read_tag(name)`
- `write_tag(name, value)`
- `read_tags([...])` batch read
- `check_health()`

Delay unless very clean:

- subscriptions
- tag groups
- UDT discovery convenience layers
- complex callback/event models

Those can follow after the core Python surface is validated.

## Proposed Package Shape

Suggested repo layout:

```text
python/
  rust_ethernet_ip/
    __init__.py
    client.py
    bindings.py
    exceptions.py
    types.py
  tests/
  examples/
```

Suggested implementation split:

- `bindings.py`: low-level FFI loading and function signatures
- `client.py`: public Python client API
- `types.py`: value wrappers or helper types only if needed
- `exceptions.py`: Python exception mapping from native failures

## FFI Fit Assessment

The current FFI surface is already strong enough for a first Python MVP because it includes:

- connection lifecycle
- typed reads/writes
- generic tag read
- batch reads/writes
- health checks
- metadata and UDT discovery hooks

Practical note:

- Python should prefer the generic/native-semantic paths where possible instead of re-probing types manually
- the C# wrapper already moved in that direction for subscriptions and generic reads

## Packaging Recommendation

For the MVP:

- do not start with a large PyO3 packaging path
- start with a thin Python package that loads the built native library
- document local build assumptions explicitly

Future packaging can decide whether to:

- bundle platform-native binaries
- publish per-platform wheels
- keep the package source-only with external native-library expectations

That decision should happen after the API shape is proven.

## Validation Strategy

Minimum validation for the Python path should include:

- import smoke test
- connect/disconnect test against simulator-backed flows if feasible
- one-tag read/write test
- batch-read test
- error-surface test for invalid client or tag failures

The existing Rust and C# validation should remain mandatory because Python must not destabilize the core or existing wrapper.

## Immediate To-Do

- [x] create a Python wrapper design skeleton without committing to implementation-heavy packaging choices
- [x] define the minimal FFI calls the Python package will consume
- [x] identify any FFI ergonomic gaps for Python consumers
- [x] create Python MVP examples focused on data collection and analytics workflows
- [ ] clean or reframe stale historical docs that still imply a maintained `pywrapper/` tree
- [x] add lightweight local development and test instructions for the Python package

Current implementation status:

- the `python/` package skeleton exists
- the package imports from the repo with `PYTHONPATH=python`
- the wrapper can load the current native library build outputs
- the current pure-Python tests run with `unittest` and do not require `pytest`
- optional simulator-backed integration tests now exist and use `SIM_PLC_ADDRESS` when configured
- the Python tests can now auto-launch the in-repo simulator with `RUST_ETHERNET_IP_START_SIM=1`
- simulator-backed validation now covers mixed `DINT` / `REAL` / `BOOL` / `STRING` batch flows
- Python `float` defaults to PLC `REAL` for practical wrapper ergonomics; `LREAL` must be explicit
- optional analytics and API examples now exist without adding mandatory package dependencies

Detailed MVP mapping:

- [PYTHON_MVP_API_AND_FFI_MAPPING.md](PYTHON_MVP_API_AND_FFI_MAPPING.md)

## Guardrail

The Python path should increase adoption of the Rust core, not shift the center of gravity of the repo away from it.

The repository remains:

- the core Rust EtherNet/IP library
- the protocol and semantics source of truth
- the place where correctness, performance, tests, and documentation are anchored
