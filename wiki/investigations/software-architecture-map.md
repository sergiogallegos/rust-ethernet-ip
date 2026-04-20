# Software Architecture Map

## Summary

The repository currently follows a layered architecture centered on `EipClient` as the Rust facade, a thin C ABI in `src/ffi.rs`, and a .NET usability layer in `csharp/`.

The main design rule is:

- Rust owns protocol behavior and PLC semantics
- FFI owns stable ABI and marshaling
- C# owns ergonomics and managed-resource integration

The primary human-facing architecture document is [docs/SOFTWARE_ARCHITECTURE.md](../../docs/SOFTWARE_ARCHITECTURE.md).

## Current Understanding

- `src/lib.rs` is still the dominant orchestration surface and exports the main public API.
- `src/ffi.rs` is a compatibility-critical boundary because the C# wrapper binds directly to exported symbols.
- `src/subscription.rs` is now the active subscription implementation; `src/tag_subscription.rs` is currently a compatibility re-export layer rather than a second implementation.
- `src/plc_manager.rs` owns connection-pool policy and should not drift into duplicate acquisition strategies.
- The C# wrapper should remain thin relative to Rust and avoid reimplementing PLC behavior locally.

## Evidence

- [src/lib.rs](../../src/lib.rs)
- [src/ffi.rs](../../src/ffi.rs)
- [src/subscription.rs](../../src/subscription.rs)
- [src/tag_subscription.rs](../../src/tag_subscription.rs)
- [src/plc_manager.rs](../../src/plc_manager.rs)
- [csharp/RustEtherNetIp/EthernetNetIpClient.cs](../../csharp/RustEtherNetIp/EthernetNetIpClient.cs)
- [docs/SOFTWARE_ARCHITECTURE.md](../../docs/SOFTWARE_ARCHITECTURE.md)

## Open Questions

- Whether `lib.rs` should be reduced further by moving more orchestration into smaller modules.
- Whether the C# wrapper should be split into smaller partials or service-oriented classes as the managed feature surface grows.

## Related Pages

- [rust-toolchain-baseline-2026-04-19.md](rust-toolchain-baseline-2026-04-19.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
