# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust EtherNet/IP is a high-performance EtherNet/IP communication library for Allen-Bradley CompactLogix and ControlLogix PLCs. Written in pure Rust with a C FFI layer (`cdylib`) and a C# wrapper for .NET integration. The current development focus is on the .NET stack (C# wrappers and examples).

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (needed for C# FFI .dll/.so/.dylib)
cargo fmt -- --check                 # Check formatting (CI enforced)
cargo clippy -- -D warnings          # Lint with warnings as errors (CI enforced)
cargo test                           # Run all tests (integration tests need a PLC)
SKIP_PLC_TESTS=1 cargo test          # Run tests without a physical PLC
cargo test --test plc_sim_tests      # Run simulator-backed tests only (no PLC needed)
cargo test --lib                     # Run unit tests only
cargo test --test integration_test   # Run a specific test file
cargo test test_name                 # Run a single test by name
cargo bench                          # Run Criterion benchmarks
cargo run --bin plc_sim              # Start standalone PLC simulator
```

### Test Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `SKIP_PLC_TESTS` | unset | Set to any value to skip tests requiring a physical PLC |
| `TEST_PLC_ADDRESS` | `192.168.0.1:44818` | PLC IP address and port |
| `TEST_PLC_SLOT` | `0` | CPU slot (0 for CompactLogix) |

Most integration tests call `should_skip_plc_tests()` and return early when `SKIP_PLC_TESTS` is set. The `plc_sim_tests.rs` always run using an in-process `SimulatedPlc`.

### C# Wrapper

```bash
cd csharp/RustEtherNetIp && dotnet build
cd csharp/RustEtherNetIp.Tests && dotnet test
```

## Architecture

### Core Design

The library is built around `EipClient` (in `src/lib.rs`, ~7500 lines), which implements the EtherNet/IP encapsulation protocol and CIP (Common Industrial Protocol) over async TCP via Tokio. It is the single entry point for all PLC communication.

```
Rust/C# Application
        |
   EipClient (src/lib.rs) -- async TCP via Box<dyn EtherNetIpStream>
        |
   FFI layer (src/ffi.rs) -- #[no_mangle] extern "C", global Tokio runtime
        |
   C# P/Invoke wrapper (csharp/RustEtherNetIp/)
```

### Key Source Modules

| Module | Responsibility |
|---|---|
| `lib.rs` | `EipClient`, `PlcValue` enum (13 AB data types), `BatchOperation`, `RoutePath`, `UdtData`, protocol encoding/decoding, session management |
| `error.rs` | `EtherNetIpError` enum with `is_retriable()` for retry vs reconnect decisions |
| `tag_path.rs` | `TagPath` parser for complex addressing: arrays, bits, program-scoped, UDT members, nested paths |
| `udt.rs` | `UdtDefinition`, `UdtManager`, `UserDefinedType` for UDT discovery and serialization |
| `ffi.rs` | C FFI exports using `lazy_static` global `RUNTIME` and `FFI_CLIENTS: Mutex<HashMap<i32, EipClient>>` |
| `subscription.rs` | `TagSubscription`, `SubscriptionManager` with mpsc channels |
| `monitoring.rs` | `ProductionMonitor`, health checks, metrics collection |
| `config.rs` | `ProductionConfig` with connection/performance/monitoring sub-configs |

### Key Types

- **`EipClient`**: Primary client. Not thread-safe for concurrent use — wrap in `Arc<Mutex<>>` for shared access. Supports `connect()`, `with_route_path()`, and `connect_with_stream()` (stream injection for testing/metrics).
- **`PlcValue`**: Tagged enum covering all 13 AB types: `Bool`, `Sint`, `Int`, `Dint`, `Lint`, `Usint`, `Uint`, `Udint`, `Ulint`, `Real`, `Lreal`, `String`, `Udt(UdtData)`.
- **`UdtData`**: Opaque `{ symbol_id: i32, data: Vec<u8> }`. Must be parsed with a `UdtDefinition` obtained from the PLC. Always read before write to capture the `symbol_id`.
- **`EtherNetIpError`**: All operations return `Result<T, EtherNetIpError>`. Use `is_retriable()` to distinguish transient errors (timeout, connection lost) from permanent ones (protocol, CIP).
- **`RoutePath`**: Slot/port routing for ControlLogix backplane. When set, CIP messages are wrapped in Unconnected Send (service 0x52).

### Tag Path Addressing

The library handles full Allen-Bradley tag path syntax internally:
- Controller tags: `"MyTag"`
- Program-scoped: `"Program:MainProgram.MyTag"`
- Array elements: `"MyArray[5]"`, `"MyArray[1,2,3]"`
- BOOL arrays: `"gBoolArray[5]"` (automatic DWORD bit extraction)
- Bit access: `"StatusWord.15"` or `client.read_bit("StatusWord", 15)`
- UDT members: `"MotorData.Speed"`
- Nested: `"Cell_NestData[90].PartData.Member"`

### PLC Firmware Limitations

These are Allen-Bradley restrictions, not library bugs:
1. **Cannot write STRING tags directly** — CIP Error 0x2107. Workaround: write the entire containing UDT.
2. **Cannot write individual UDT array element members** — CIP Error 0x2107. Workaround: read entire UDT element, modify in memory, write back the whole element.

## Workspace Layout

The root `Cargo.toml` defines a workspace containing `.` (main crate) and `examples/desktop_app`. The crate produces both `rlib` and `cdylib` outputs. Rust MSRV is 1.95.

## CI

GitHub Actions runs on ubuntu/windows/macos with stable+beta Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --verbose`. C# tests and coverage (tarpaulin) run on ubuntu/stable only.
