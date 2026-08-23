# Software Architecture

This document is the authoritative architecture map for `rust-ethernet-ip`.
It explains how the Rust core, protocol implementation, FFI layer, wrappers,
tests, and release compatibility rules fit together.

It is intended for:

- maintainers reviewing or refactoring the codebase
- contributors adding protocol, wrapper, or simulator-backed behavior
- release reviewers checking compatibility risk
- AI agents that need a stable map of module boundaries and design constraints

This document describes the current `v1.2.1` released line. The wiki may
contain investigation notes and synthesis, but this file is the user-facing
architecture reference.

## Project Scope

`rust-ethernet-ip` is a pure-Rust async EtherNet/IP client library focused on
Allen-Bradley CompactLogix and ControlLogix PLCs. The repository also includes
a C ABI boundary plus C# and Python wrappers, but the core product identity is
the Rust protocol library.

In scope:

- EtherNet/IP encapsulation and CIP explicit messaging
- Logix tag read/write support, including arrays, bits, strings, and UDT data
- ControlLogix route-path support for backplane and Ethernet hops
- Rust, C ABI, C#, and Python access paths over the same Rust implementation
- simulator-backed tests and real PLC validation records
- examples that demonstrate supported usage patterns

Out of scope:

- replacing SCADA, MES, historian, or plant-wide automation platforms
- implementing PLC firmware workarounds independently in each wrapper
- treating examples as independent products with separate protocol behavior
- promising real-time control semantics; this library is for communication and
  data access, not deterministic machine control

## Architectural Drivers

The important design goals are:

- protocol correctness over superficial API convenience
- stable public APIs for Rust users and wrapper consumers
- thin wrappers that do not reimplement PLC protocol logic
- high confidence through unit, simulator, wrapper, and hardware validation
- conservative refactors that preserve compatibility across the `v1.x` released line
- clear post-1.0 handling of any new public API and ABI debt

Key constraints:

- the Rust crate builds both an `rlib` and a `cdylib`
- the FFI layer is consumed by downstream C# and Python packages
- real hardware validation is required for PLC-specific behavior
- some Allen-Bradley firmware limitations are external constraints, not library
  bugs
- compatibility matters more than architectural purity on the released `v1.2.0`
  line

## Architecture Summary

The repository is organized around these layers:

1. **Rust public API:** `EipClient`, `PlcValue`, `RoutePath`, batch types,
   diagnostics, and error types exported from [src/lib.rs](../src/lib.rs).
2. **Rust implementation modules:** tag paths, routing, UDT handling, batching,
   monitoring, subscriptions, and configuration under `src/`.
3. **Protocol codec boundary:** encapsulation, CIP framing, and value encoding
   under [src/protocol/](../src/protocol/).
4. **FFI boundary:** C-compatible exported functions in [src/ffi.rs](../src/ffi.rs).
5. **Wrappers:** C# and Python convenience layers over the native ABI.
6. **Examples and validation:** Rust, C#, Python, simulator, and real PLC usage
   paths that exercise the same core implementation.

The intended ownership rule is simple:

- Rust owns protocol correctness, PLC behavior, and tag semantics.
- FFI owns ABI stability and marshaling.
- C# and Python own language-native ergonomics and resource lifetime.
- Tests and validation records prove parity between those surfaces.

## Module View

### Public API Surface

- [src/lib.rs](../src/lib.rs): crate root, public re-exports, tracing helpers,
  version string, and `EtherNetIpStream`.
- [src/client.rs](../src/client.rs): `EipClient` facade and current primary
  orchestration implementation.
- [src/types.rs](../src/types.rs): `PlcValue`, `UdtData`, and currently public
  session-related data types.
- [src/route.rs](../src/route.rs): `RoutePath` and `RouteHop`, including ordered
  route hops and compatibility fields for older grouped route construction.
- [src/error.rs](../src/error.rs): public error type and retry classification.
- [src/batch.rs](../src/batch.rs): batch read/write/execute data model.

### Internal Implementation Modules

- [src/protocol/](../src/protocol/): wire codec boundary for encapsulation, CIP,
  and value payloads.
- [src/tag_path.rs](../src/tag_path.rs): symbolic tag path parsing for arrays,
  bits, program scope, and nested UDT members.
- [src/tag_manager.rs](../src/tag_manager.rs): tag metadata and discovery support.
- [src/udt.rs](../src/udt.rs): UDT definitions, member layout, and serialization.
- [src/subscription.rs](../src/subscription.rs): subscription primitives.
- [src/tag_group.rs](../src/tag_group.rs): grouped polling and event aggregation.
- [src/plc_manager.rs](../src/plc_manager.rs): pooled PLC connection management.
- [src/monitoring.rs](../src/monitoring.rs): diagnostics, metrics, and health.
- [src/config.rs](../src/config.rs): production configuration data model.

### Workspace Crate Decomposition

The repository is a Cargo workspace. The main `rust-ethernet-ip` crate re-exports
four publishable sibling crates that hold the shared, wrapper-independent core:

- `rust-ethernet-ip-types` ([crates/types/](../crates/types/)): `PlcValue`,
  `UdtData`, `ConnectedSession`, and `ConnectionParameters`.
- `rust-ethernet-ip-tag-path` ([crates/tag-path/](../crates/tag-path/)): the
  `TagPath` parser for arrays, bits, program scope, and nested UDT members.
- `rust-ethernet-ip-protocol` ([crates/protocol/](../crates/protocol/)): the
  `Encode`/`Decode` wire codec boundary — encapsulation framing, CIP framing, and
  `PlcValue` codecs.
- `rust-ethernet-ip-udt` ([crates/udt/](../crates/udt/)): UDT discovery and
  serialization.

Because of this split, [src/protocol/mod.rs](../src/protocol/mod.rs),
[src/tag_path.rs](../src/tag_path.rs), and [src/udt.rs](../src/udt.rs) are thin
re-export shims over the corresponding sibling crates rather than standalone
implementations; the entries above describe the logical surface they expose.

## Runtime Flow

### Native Rust

1. A caller constructs an `EipClient` directly or through connection helpers.
2. The client registers or reuses the EtherNet/IP session.
3. Tag paths are parsed into the required symbolic, array, bit, or member form.
4. Requests are encoded through the protocol boundary and sent over Tokio I/O.
5. Responses are decoded into `PlcValue`, metadata, diagnostics, batch results,
   or subscription events.

### C# Wrapper

1. `EtherNetIpClient` loads the native library through P/Invoke.
2. The wrapper stores a native client ID returned by the FFI layer.
3. Wrapper methods call exported native functions for connect, read, write,
   batch, route, diagnostics, and lifecycle operations.
4. Native results are marshaled into .NET types.
5. The wrapper adds disposal, C# naming conventions, typed helpers, and tests.

### Python Wrapper

1. The Python package loads the native library through `ctypes`.
2. A Python client object stores the native client ID.
3. Python methods call the same FFI symbols used by other non-Rust consumers.
4. Returned values are converted into Python-native values and wrapper classes.
5. The wrapper remains a marshaling and ergonomics layer, not a protocol stack.

## Protocol Boundary

The protocol layer is responsible for bytes-on-the-wire behavior. New
encapsulation, CIP, route, or value encoding logic should be added under
[src/protocol/](../src/protocol/) unless there is a strong reason not to.

Rules:

- protocol encoders and decoders should be tested with pinned byte fixtures
- `client.rs` should orchestrate protocol calls, not inline new codecs
- wrapper tests should validate behavior, not duplicate protocol construction
- simulator-backed tests should cover realistic request/response flows
- real PLC validation should confirm behavior where firmware differences matter

## Client Facade and Internal Split

`EipClient` is the public facade and should remain the main Rust entry point.
The current implementation is still large, so the preferred refactor direction
is a facade-preserving internal split.

Likely future submodules:

- `client::session`: registration, session lifecycle, packet sizing, keepalive
- `client::tag_io`: scalar, array, bit, and generic tag read/write operations
- `client::udt`: UDT discovery, member access, and UDT write behavior
- `client::string`: Logix STRING-specific read/write behavior
- `client::batch_exec`: batch execution orchestration
- `client::diagnostics`: health and diagnostics snapshots
- `client::discovery`: tag and program discovery
- `client::subscriptions`: subscription and tag-group polling integration

The first split should be mechanical. It should move cohesive code without
changing public behavior. Deeper changes, especially concurrency or request
correlation changes, need separate design review and compatibility tests.

## Concurrency Model

The current client uses Tokio async I/O and shared ownership for long-lived
state. Some shared state is protected with mutexes because one PLC session and
one stream may serialize network operations.

Current rule:

- shared state should be scoped narrowly
- locks must not be held across network I/O unless serialization is intentional
- clone behavior must be treated as part of the API contract
- wrapper calls must not rely on accidental lock or clone behavior

The long-term direction may be an internal request worker with a cloneable
handle. That change should not be treated as a harmless implementation detail.
It can affect request ordering, cancellation, timeouts, and wrapper behavior.

## FFI ABI Boundary

The native ABI is a compatibility contract.

Rules:

- exported symbols in [src/ffi.rs](../src/ffi.rs) are part of the wrapper API
- FFI changes require Rust, C#, and Python compatibility review
- new exported functions should have explicit tests from at least one wrapper
- breaking payload or symbol changes require an ABI/versioning plan
- the FFI layer should remain thin and should not implement protocol semantics
- native client IDs, allocation ownership, and output buffers must be documented
  in the FFI function contracts

Before 1.0, the project should add an explicit ABI/version/capability query so
wrappers can fail clearly when loaded against an incompatible native library.

## Wrapper Architecture

Wrappers exist to make the Rust implementation usable from other ecosystems.
They should not become independent EtherNet/IP clients.

Wrapper responsibilities:

- locate and load the native library
- manage native client lifecycle and disposal
- marshal primitive values, strings, arrays, route paths, diagnostics, and batch
  results
- expose language-native naming and convenience helpers
- preserve parity with the Rust behavior through tests

Wrapper non-responsibilities:

- encoding CIP paths independently
- implementing firmware workarounds separately from Rust
- hiding native errors in a way that makes diagnosis impossible
- adding behavior that cannot be validated against the Rust core

## Firmware and Controller Behavior

Some behavior is constrained by Allen-Bradley firmware rather than this library.
Examples include certain UDT member write paths and the exact Logix STRING
structure encoding required on the wire. These rules should be represented as
documented Rust behavior and surfaced consistently through wrappers.

Preferred direction:

- keep firmware-specific workarounds in Rust
- document known limitations in user-facing docs
- synthesize controller-specific evidence in `wiki/controllers/` and
  `wiki/limitations/`
- validate route and firmware behavior against real hardware before release

## Testing Architecture

The test strategy has four layers:

1. **Rust unit and codec tests:** fast checks for value encoding, tag-path
   parsing, route encoding, errors, and pure logic.
2. **Simulator-backed integration tests:** no-PLC tests against an in-process or
   local simulated PLC protocol surface.
3. **Wrapper contract tests:** C# and Python tests that verify native loading,
   value conversion, client lifecycle, route paths, and simulator-backed flows.
4. **Real PLC validation:** maintainer-run validation against CompactLogix and
   ControlLogix hardware, recorded under `docs/validation/`.

Coverage expectations:

- Rust owns the deepest protocol and edge-case coverage.
- C# and Python tests must prove wrapper parity and marshaling correctness.
- Simulator tests should run in CI and cover realistic success/failure flows.
- Hardware validation is required for release claims involving firmware,
  routing, UDT, STRING, or controller-specific behavior.

## Release and Compatibility Model

`v1.2.1` is the current released line (tagged 2026-08-22). Future work
should preserve compatibility within the `v1.x` line per SemVer; new
SemVer-major work bundles into the next major release window.

Compatibility surfaces:

- Rust public API and semver-visible types
- FFI exported symbols and payload shapes
- C# NuGet API
- Python package API
- documented PLC behavior and validation claims
- simulator behavior used by CI and wrapper tests

Pre-1.0 breaking changes should be bundled deliberately instead of scattered.
Good candidates for the 1.0 cleanup window include:

- `#[non_exhaustive]` on public enums expected to grow
- structured FFI ABI versioning
- clearer error taxonomy for CIP and protocol failures
- private storage for route-path internals
- explicit clone/concurrency semantics for `EipClient`
- demoting internal wire/session state from the public API where possible

## Important Invariants

- `EipClient` is the semantic source of truth for PLC communication behavior.
- Protocol byte encoding belongs in `src/protocol/`.
- Wrappers must not silently drift from Rust behavior.
- FFI registry and lifetime changes are compatibility-sensitive.
- Real PLC validation records must back release claims about hardware behavior.
- Documentation for wrapper parity and release validation must match actual
  tests.

## Design Debt To Watch

- `client.rs` remains too large and should be split behind the facade.
- the public error model is still broad and string-heavy.
- the FFI layer needs explicit ABI/version/capability reporting.
- `EipClient` clone and concurrency semantics need stronger documentation or a
  cleaner handle model.
- route-path compatibility fields should be revisited for 1.0.
- internal session/wire-state types should not stay public unless users have a
  real construction use case.
- wrapper tests must remain strong enough that C# and Python changes cannot
  silently break native parity.

## Refactor Policy

When improving the architecture:

1. keep the public facade stable unless the change is intentionally versioned
2. move behavior toward narrower ownership boundaries
3. reduce duplicated implementations before adding new variants
4. keep protocol bytes in `src/protocol/`
5. keep the FFI ABI stable unless there is a deliberate versioned break
6. validate wrapper parity whenever exported native functions change
7. update this document, relevant user-facing docs, and wiki synthesis when the
   design changes materially

## Related Documents

- [README.md](../README.md)
- [docs/README.md](README.md)
- [programmer_manual.md](programmer_manual.md)
- [CONTROLLOGIX_ROUTING_IMPLEMENTATION.md](CONTROLLOGIX_ROUTING_IMPLEMENTATION.md)
- [OFFICIAL_SOURCES.md](OFFICIAL_SOURCES.md)
- [PYTHON_MVP_API_AND_FFI_MAPPING.md](PYTHON_MVP_API_AND_FFI_MAPPING.md)
- [PYTHON_WRAPPER_STRATEGY.md](PYTHON_WRAPPER_STRATEGY.md)
- [wiki/wrapper-parity/rust-vs-csharp.md](../wiki/wrapper-parity/rust-vs-csharp.md)
- [wiki/investigations/software-architecture-map.md](../wiki/investigations/software-architecture-map.md)
- [wiki/investigations/architecture-review-2026-05-18.md](../wiki/investigations/architecture-review-2026-05-18.md)
- [wiki/investigations/test-coverage-strength-2026-05-18.md](../wiki/investigations/test-coverage-strength-2026-05-18.md)
