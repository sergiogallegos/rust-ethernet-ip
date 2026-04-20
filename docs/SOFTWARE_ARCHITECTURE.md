# Software Architecture

This document describes the current software architecture for `rust-ethernet-ip`.

It is intended for:

- maintainers reviewing or refactoring the codebase
- contributors adding protocol or wrapper features
- AI agents that need a stable map of module boundaries and design constraints

## Core Objective

The repository's primary objective is to remain a strong Rust EtherNet/IP core library:

- protocol-correct
- safe
- fast
- tested
- documented
- current

Wrappers and example systems are important enablement layers for users, but they are not meant to replace the core-library identity of the project.

## Summary

The repository is organized around a layered design:

1. `EipClient` is the primary application-facing API and orchestration boundary.
2. Protocol and tag-path logic live in Rust library modules under `src/`.
3. The FFI layer in [src/ffi.rs](../src/ffi.rs) exposes a stable C ABI for native consumers.
4. The C# wrapper in `csharp/` turns that ABI into a .NET-friendly object model.
5. Examples exercise the same core library through native Rust, FFI, desktop, and web entry points.

The intended direction is:

- keep protocol and PLC semantics in Rust
- keep FFI thin and data-marshaling-focused
- keep the C# wrapper ergonomic, but not a second protocol implementation

## Layer Map

## Application Surface

- [src/lib.rs](../src/lib.rs): exports the main Rust API surface, the `EipClient` type, shared value types, and top-level modules
- `csharp/RustEtherNetIp/`: .NET wrapper and higher-level usability layer over the native library
- `examples/`: integration examples for desktop, web, and direct Rust usage

## Core Rust Modules

- [src/lib.rs](../src/lib.rs): protocol orchestration, client API, batch operations, tag reads/writes, and shared data model types
- [src/tag_path.rs](../src/tag_path.rs): parses symbolic paths, array access, bit access, and nested UDT addressing
- [src/tag_manager.rs](../src/tag_manager.rs): tag metadata, discovery, cache-oriented tag support
- [src/udt.rs](../src/udt.rs): UDT metadata and payload representation
- [src/subscription.rs](../src/subscription.rs): subscription primitives and manager used by the current exported API
- [src/tag_group.rs](../src/tag_group.rs): grouped polling/event aggregation behavior
- [src/plc_manager.rs](../src/plc_manager.rs): pooled PLC connection management and reuse policy
- [src/monitoring.rs](../src/monitoring.rs): health checks, metrics, and diagnostics-oriented state
- [src/config.rs](../src/config.rs): runtime configuration types
- [src/error.rs](../src/error.rs): project error taxonomy and conversion boundary
- [src/ffi.rs](../src/ffi.rs): C ABI surface for non-Rust consumers

## Wrapper Layer

- [csharp/RustEtherNetIp/EthernetNetIpClient.cs](../csharp/RustEtherNetIp/EthernetNetIpClient.cs): main .NET client facade
- `csharp/RustEtherNetIp/*.cs`: value types, route-path helpers, subscriptions, groups, diagnostics, and contracts
- `csharp/RustEtherNetIp.Tests/`: parity and contract tests against the native library

## Primary Design Patterns

## Thin Wrapper Pattern

The intended ownership split is:

- Rust owns protocol correctness, tag semantics, batching, routing, and PLC behavior
- FFI owns ABI stability and marshaling
- C# owns .NET ergonomics, resource lifetime, and API usability

The wrapper should not reimplement protocol logic that already exists in Rust.

## Facade + Module Decomposition

`EipClient` is the facade for the library. It exposes high-level operations while delegating to specialized modules for:

- path parsing
- metadata discovery
- monitoring
- subscriptions
- pooled client management

This keeps callers on a stable API while allowing internal refactors behind the facade.

## Shared-Ownership Async State

The library uses Tokio-based async I/O and shared ownership through interior mutability where long-lived state must be shared across async tasks.

Key rule:

- shared state should be scoped narrowly and guards should not be held across network I/O unless the serialization is intentional

Recent refactor work explicitly moved subscription and FFI call paths away from broad lock-across-await patterns.

## Stable ABI Boundary

The native boundary is intentionally explicit:

- Rust FFI exports are declared in [src/ffi.rs](../src/ffi.rs)
- symbol names are part of the C# contract
- Rust 2024 requires `#[unsafe(no_mangle)]` on exported functions

Any FFI change should be treated as an API compatibility review, not only a code change.

## Current Runtime Flow

## Native Rust

1. A caller constructs or obtains an `EipClient`.
2. The client creates or reuses the underlying PLC session state.
3. Tag paths are parsed into the required symbolic/array/member addressing form.
4. The client issues EtherNet/IP and CIP requests over Tokio-managed I/O.
5. Results are mapped into `PlcValue`, typed helpers, metadata objects, or subscription events.

## C# Wrapper

1. `EtherNetIpClient` connects through `DllImport`-backed FFI functions.
2. The wrapper stores a native client ID.
3. Each read/write/subscription/group operation calls into exported Rust functions.
4. Returned buffers and result structs are marshaled into .NET types.
5. The wrapper adds disposal, convenience APIs, and integration-friendly surface area.

## Important Invariants

- `EipClient` is the semantic source of truth for PLC communication behavior.
- FFI should not hold global registry locks across network operations.
- Subscription/update code should avoid lock-across-await patterns.
- Wrapper code should preserve native symbol parity and not silently drift from exported behavior.
- Documentation for release validation and wrapper parity should be kept in sync with actual tests.

## Current Refactor Seams

These are the main places to scrutinize before adding new features:

- FFI registry and lifetime design in [src/ffi.rs](../src/ffi.rs)
- subscription/event model consolidation around [src/subscription.rs](../src/subscription.rs)
- pooled-connection policy in [src/plc_manager.rs](../src/plc_manager.rs)
- C# wrapper lifecycle and concurrency behavior in `EthernetNetIpClient`

If new functionality crosses more than one of these seams, document the ownership boundary before implementing it.

## Design Debt To Watch

- keep duplicate protocol logic out of the C# wrapper
- avoid creating parallel subscription implementations
- avoid adding new global locks in the FFI boundary
- prefer targeted module responsibilities over growth of `lib.rs` as a grab-bag
- keep examples aligned with the real supported architecture, not as one-off prototypes

## Recommended Refactor Posture

When improving the architecture:

1. move behavior toward narrower ownership boundaries
2. reduce duplicated implementations before adding new variants
3. keep the C ABI stable unless there is a deliberate versioned break
4. validate parity with the C# wrapper whenever exported native functions change
5. update this document, the relevant user-facing docs, and the wiki when the design changes materially

## Related Documents

- [README.md](../README.md)
- [docs/README.md](README.md)
- [programmer_manual.md](programmer_manual.md)
- [CONTROLLOGIX_ROUTING_IMPLEMENTATION.md](CONTROLLOGIX_ROUTING_IMPLEMENTATION.md)
- [OFFICIAL_SOURCES.md](OFFICIAL_SOURCES.md)
- [wiki/wrapper-parity/rust-vs-csharp.md](../wiki/wrapper-parity/rust-vs-csharp.md)
- [wiki/investigations/rust-toolchain-baseline-2026-04-19.md](../wiki/investigations/rust-toolchain-baseline-2026-04-19.md)
