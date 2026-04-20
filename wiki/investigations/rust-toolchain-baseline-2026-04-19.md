# Rust Toolchain Baseline 2026-04-19

## Summary

- `confirmed`: local development is already on Rust `1.95.0`.
- `confirmed`: the crate now declares `rust-version = "1.95"` and `edition = "2024"` in [Cargo.toml](../../Cargo.toml).
- `likely`: no immediate Rust toolchain update is required for the repository.
- `confirmed`: the Rust 2024 migration is now applied for repo manifests.
- `confirmed`: the pre-pass removed the non-FFI Rust 2024 compatibility warnings before the edition bump.
- `confirmed`: the FFI export layer now uses Rust 2024 `#[unsafe(no_mangle)]` attributes.

## Current Understanding

- The current stable Rust release checked on `2026-04-19` is Rust `1.95.0`.
  Source: [Rust 1.95.0 release post](https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/)
- The repository now targets Rust `2024` edition and declares a current-stable baseline of Rust `1.95`.
  Source: [Cargo.toml](../../Cargo.toml)
- The repository builds successfully on Rust `1.95.0` with `cargo check --all-targets`.
- A Rust 2024 prep pass was completed on `2026-04-19` before the final manifest bump:
  - async tail-expression and lock/drop-order sites were rewritten to use explicit locals or explicit guard release.
  - the subscription stream path was rewritten away from `async_stream::stream!` to `futures::stream::unfold`.
  - selected `unsafe_op_in_unsafe_fn` sites in `src/ffi.rs` were wrapped explicitly.
- After that prep pass, the remaining repo-owned Rust 2024 warnings were limited to exported FFI entry points in `src/ffi.rs`.
- The final migration step raised the compiler baseline to Rust `1.95` and converted the exported FFI entry points to `#[unsafe(no_mangle)]`.
- The current state appears intentional:
  - `edition = "2024"` matches the compatibility work already completed in repo code.
  - `rust-version = "1.95"` matches the current stable toolchain baseline used for ongoing development.

## Evidence

- Manifest baseline:
  - [Cargo.toml](../../Cargo.toml)
  - [examples/desktop_app/Cargo.toml](../../examples/desktop_app/Cargo.toml)
  - [examples/web_app/backend/Cargo.toml](../../examples/web_app/backend/Cargo.toml)
- Current user-facing MSRV messaging:
  - [README.md](../../README.md)
  - [examples/desktop_app/README.md](../../examples/desktop_app/README.md)
  - [docs/VERSION_MANAGEMENT.md](../../docs/VERSION_MANAGEMENT.md)
- Migration probe outcome on `2026-04-19`:
  - `cargo check --all-targets` passed on local Rust `1.95.0`.
  - `cargo fix --edition --workspace --all-features --all-targets` reported Rust 2024 compatibility warnings instead of a clean mechanical migration.
- Prep-pass outcome on `2026-04-19`:
  - `cargo check --all-targets` still passes after the refactors.
  - `RUSTFLAGS='--force-warn rust-2024-compatibility' cargo check --all-targets` no longer reports repo-owned async/drop-order warnings outside `src/ffi.rs`.
- Final migration outcome on `2026-04-19`:
  - workspace manifests now target Rust `2024`.
  - the crate baseline is now Rust `1.95`.
  - `src/ffi.rs` now uses `#[unsafe(no_mangle)]` for exported FFI symbols.
- The pre-pass warnings were concentrated in async code where Rust 2024 changes temporary tail-expression drop order, including:
  - `src/plc_manager.rs`
  - `src/subscription.rs`
  - `src/main.rs`
  - `examples/desktop_app/src/main.rs`
  - `tests/subscription_tests.rs`

## Open Questions

- `unclear`: whether the project should keep the baseline at `1.95` for the next release line or continue advancing with each stable toolchain.
- `needs-review`: whether `docs/VERSION_MANAGEMENT.md` should distinguish historical release-line compatibility from the current mainline baseline more explicitly.

## Related Pages

- [wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [releases/0.7.1-validation-synthesis.md](../releases/0.7.1-validation-synthesis.md)
