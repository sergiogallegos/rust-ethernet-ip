# Python Wrapper Strategy

## Summary

The current recommendation is to build the future Python wrapper on top of the repo's existing Rust FFI boundary rather than introducing a separate primary wrapper architecture through PyO3.

## Current Understanding

- The crate already ships `cdylib` output and exports a broad C ABI in `src/ffi.rs`.
- The C# wrapper already demonstrates the intended thin-wrapper pattern over that ABI.
- This better supports the repo's core vision: strong Rust core first, wrappers second.
- The repo contains historical documents that reference `pywrapper/` and earlier wrapper experiments, but those are not current maintained paths in the live tree.
- A first Python package skeleton now exists under `python/` and follows the FFI-first approach.
- The package imports from the repo, loads the current native library build output, and currently validates with lightweight `unittest` checks.
- Optional simulator-backed integration tests now exist and are enabled through `SIM_PLC_ADDRESS`, matching the existing C# wrapper pattern.
- Optional analytics and API examples now exist as example-only paths, with dependencies isolated behind extras instead of the core package.

## Evidence

- [Cargo.toml](../../Cargo.toml)
- [src/ffi.rs](../../src/ffi.rs)
- [csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs](../../csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs)
- [docs/PYTHON_WRAPPER_STRATEGY.md](../../docs/PYTHON_WRAPPER_STRATEGY.md)
- [python/pyproject.toml](../../python/pyproject.toml)
- [python/rust_ethernet_ip/client.py](../../python/rust_ethernet_ip/client.py)
- [python/rust_ethernet_ip/bindings.py](../../python/rust_ethernet_ip/bindings.py)
- [docs/ALL_WRAPPERS_UPDATE_COMPLETE.md](../../docs/ALL_WRAPPERS_UPDATE_COMPLETE.md)
- [docs/WRAPPER_UPDATE_SUMMARY.md](../../docs/WRAPPER_UPDATE_SUMMARY.md)

## Open Questions

- Whether a generic `eip_write_tag` helper should be added to reduce Python reliance on one-item batch writes.
- Whether stale historical wrapper docs should be reframed now or after the Python path is more mature.
- Which integration tests should become the first simulator-backed or real-PLC checks for the Python path.

## Related Pages

- [ecosystem-platform-patterns-2026-04-19.md](ecosystem-platform-patterns-2026-04-19.md)
- [software-architecture-map.md](software-architecture-map.md)
- [../wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
