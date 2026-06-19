# API Stability Policy

This document describes the stability guarantees for the `rust-ethernet-ip`
1.0 line. It covers the public Rust API, the minimum supported Rust version
(MSRV), enum exhaustiveness, and the C ABI consumed by the wrappers.

## SemVer policy

The crate follows [Semantic Versioning](https://semver.org/). Within the
`1.x` line:

- The public Rust API is stable. Breaking changes to public items (removed or
  renamed types, functions, trait bounds, or changed signatures) require a new
  major version.
- Additive changes (new functions, new types, new enum variants on
  `#[non_exhaustive]` enums) ship in minor versions.
- Bug fixes that do not change the public API ship in patch versions.

## MSRV policy

- The current MSRV is **Rust 1.96**.
- An MSRV bump is treated as a **minor-version** change, not a patch. Patch
  releases never raise the required compiler.
- The MSRV is declared via `rust-version` in `Cargo.toml` and exercised by the
  dedicated CI MSRV job.

## Enum exhaustiveness

Public enums are marked `#[non_exhaustive]`. This lets new variants be added in
minor releases without breaking downstream code. Callers matching on these
enums **must** include a wildcard (`_ =>`) match arm; a match without one will
fail to compile against a future minor release.

## FFI ABI stability

The C ABI exported from `src/ffi.rs` (gated behind the `ffi` feature and
consumed by the C# and Python wrappers) is versioned independently of the crate
SemVer line:

- The current ABI version is **1** (`ABI_VERSION` in `src/version.rs`, exported
  as `eip_abi_version()`).
- Wrappers check `eip_abi_version()` at load time and refuse to run against an
  incompatible native library.
- Any ABI-breaking change (changed export signatures, struct layouts, or
  calling conventions) bumps the ABI version. Additive ABI changes that keep
  existing exports compatible do not.
