# FFI ABI Contract

## Summary

The native C FFI surface is pinned at ABI version `2` on the unreleased `1.2.0` line after CODEX-AS removed three raw `*mut EipClient` exports from the public symbol table. Wrappers should check `eip_abi_version()` at native-library load time and fail fast when the loaded `cdylib` does not match the wrapper's expected ABI.

## Current Understanding

- `confirmed`: `eip_abi_version()` returns `2` on current mainline.
- `confirmed`: ABI v2 removes the unusable raw-pointer exports `eip_get_udt_definition`, `eip_get_tag_attributes`, and `eip_discover_tags_detailed` from the C symbol table; handle-based `_by_id` exports remain public. The three functions are retained in the crate's Rust API as non-exported (`#[no_mangle]`-free) `pub unsafe extern "C" fn`s so the crate stays SemVer-compatible with 1.1.0 on the 1.2.0 minor line — the C ABI symbol table is versioned here (by `ABI_VERSION`), not by crate SemVer. `cargo-semver-checks` is told to skip the `function_export_name_changed` lint (see `[package.metadata.cargo-semver-checks.lints]` in the root `Cargo.toml`) because that lint fires only on `#[no_mangle]` functions and would otherwise force a crate-major bump for what is an ABI-version event; ordinary Rust API removals stay gated by `function_missing`.
- `confirmed`: `eip_library_version()` returns a static null-terminated semver string sourced from `CARGO_PKG_VERSION`; callers must not free the pointer.
- `confirmed`: `eip_capabilities()` returns a `u64` bitmap for optional/stable FFI capabilities.
- `confirmed`: C# and Python wrappers are expected to reject native libraries whose ABI version differs from wrapper expectation.

## Capability Bits

| Bit | Name | Meaning |
|---|---|---|
| `0x0000_0000_0000_0001` | `ROUTE_PATH_ORDERED_HOPS` | Route-path FFI supports the post-CODEX-F ordered-hop behavior. |
| `0x0000_0000_0000_0002` | `BATCH_EXECUTE_V1` | `eip_execute_batch` is present and stable. |
| `0x0000_0000_0000_0004` | `DIAGNOSTICS_JSON` | `eip_get_diagnostics_json` is present and stable. |
| `0x0000_0000_0000_0008` | `TAG_GROUP_SUBSCRIPTIONS` | Tag-group subscription FFI exports are present. |
| `0x0000_0000_0000_0010` | `LAST_ERROR` | `eip_get_last_error` is present and stable. |

Reserved bits must remain `0` unless a future release documents a new capability.

## Bump Policy

`ABI_VERSION` must be bumped for any change to:

- an exported `#[unsafe(no_mangle)] extern "C"` function signature
- removal of an exported `#[unsafe(no_mangle)] extern "C"` symbol
- ownership or lifetime rules for pointers crossing the C boundary
- struct layout passed through the C boundary
- call-convention or return-code meaning

Adding a new export can remain ABI-compatible when existing symbols and semantics are unchanged; expose discoverability through capability bits when wrappers may need feature detection.

## Evidence

- [src/version.rs](../../src/version.rs)
- [src/ffi.rs](../../src/ffi.rs)
- [docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md](../../docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md)

## Open Questions

- Future registry or client-handle refactors can remain on ABI version `2` only when exported signatures, pointer ownership rules, and return-code semantics stay unchanged.

## Related Pages

- [Route path behavior](route-path-behavior.md)
