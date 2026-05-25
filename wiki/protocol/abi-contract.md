# FFI ABI Contract

## Summary

The native C FFI surface is pinned at ABI version `1` for the `1.0.0` release-candidate line. Wrappers should check `eip_abi_version()` at native-library load time and fail fast when the loaded `cdylib` does not match the wrapper's expected ABI.

## Current Understanding

- `confirmed`: `eip_abi_version()` returns `1`.
- `confirmed`: `eip_library_version()` returns a static null-terminated semver string sourced from `CARGO_PKG_VERSION`; callers must not free the pointer.
- `confirmed`: `eip_capabilities()` returns a `u64` bitmap for optional/stable FFI capabilities.
- `confirmed`: C# and Python wrappers are expected to reject native libraries whose ABI version differs from wrapper expectation.

## Capability Bits

| Bit | Name | Meaning |
|---|---|---|
| `0x0000_0000_0000_0001` | `ROUTE_PATH_ORDERED_HOPS` | Route-path FFI supports the post-CODEX-F ordered-hop behavior. |
| `0x0000_0000_0000_0002` | `BATCH_EXECUTE_V1` | `eip_execute_batch` is present and stable for ABI v1. |
| `0x0000_0000_0000_0004` | `DIAGNOSTICS_JSON` | `eip_get_diagnostics_json` is present and stable for ABI v1. |
| `0x0000_0000_0000_0008` | `TAG_GROUP_SUBSCRIPTIONS` | Tag-group subscription FFI exports are present for ABI v1. |

Reserved bits must remain `0` for ABI v1.

## Bump Policy

`ABI_VERSION` must be bumped for any change to:

- an exported `#[unsafe(no_mangle)] extern "C"` function signature
- ownership or lifetime rules for pointers crossing the C boundary
- struct layout passed through the C boundary
- call-convention or return-code meaning

Adding a new export can remain ABI-compatible when existing symbols and semantics are unchanged; expose discoverability through capability bits when wrappers may need feature detection.

## Evidence

- [src/version.rs](../../src/version.rs)
- [src/ffi.rs](../../src/ffi.rs)
- [docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md](../../docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md)

## Open Questions

- Future registry or client-handle refactors can remain on ABI version `1` only when exported signatures, pointer ownership rules, and return-code semantics stay unchanged.

## Related Pages

- [Route path behavior](route-path-behavior.md)
