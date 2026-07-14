# Rust Toolchain Baseline 2026-04-19

## Summary

- `confirmed`: the workspace MSRV is Rust `1.88` and all six workspace
  packages inherit it from [Cargo.toml](../../Cargo.toml).
- `confirmed`: Rust `1.88.0` passes the locked, all-features workspace test
  suite; Rust `1.87.0` is rejected by the locked `time` dependencies.
- `confirmed`: the workspace remains on the Rust `2024` edition.
- `superseded`: the earlier policy of tracking the current stable compiler at
  Rust `1.95` and then `1.96` no longer describes the mainline MSRV.

## Current Understanding

- `[workspace.package].rust-version = "1.88"` is the single source of truth.
  The main library, four published sibling libraries, and desktop example
  inherit that value.
- The dedicated MSRV CI job reads the version from the root manifest and runs
  `cargo test --workspace --all-features --locked` with that toolchain.
- Rust `1.88` is the exact boundary for the repository's current locked graph:
  - Rust `1.87.0` fails dependency selection because `time 0.3.47` and
    `time-core 0.1.8` require Rust `1.88.0`.
  - Rust `1.88.0` passes the complete locked workspace test suite with all
    features.
- PR [#27](https://github.com/sergiogallegos/rust-ethernet-ip/pull/27)
  lowered the declared MSRV from Rust `1.96` without changing dependency
  versions, the Rust API, or the C ABI. Test-only `std::assert_matches!` uses
  were replaced with syntax available on Rust `1.88`.
- The repository's [API stability policy](../../docs/API_STABILITY.md) treats a
  future MSRV increase as a minor-version change; patch releases do not raise
  it.
- The `1.88` boundary is a verified repository baseline, not a promise that
  every future downstream dependency resolution will match this lockfile.
  Downstream consumers resolve and lock their own dependency graphs.

## Historical Context

- On `2026-04-19`, the workspace migrated to the Rust `2024` edition and set
  `rust-version = "1.95"`, matching the then-current stable compiler.
- Before that edition change, async temporary/drop-order sites were rewritten,
  the subscription stream moved from `async_stream::stream!` to
  `futures::stream::unfold`, and selected FFI unsafe operations were made
  explicit.
- Exported FFI entry points adopted Rust 2024 `#[unsafe(no_mangle)]`
  attributes. Those edition changes remain in the current code and do not
  require the former Rust `1.95` policy.
- On `2026-05-29`, the baseline moved from `1.95` to `1.96` to track stable and
  use `std::assert_matches!` in tests. PR #27 superseded that current-stable
  choice with the verified `1.88` workspace floor on `2026-07-14`.

## Evidence

- Current declarations and inheritance:
  - [Cargo.toml](../../Cargo.toml)
  - [protocol manifest](../../crates/protocol/Cargo.toml)
  - [tag-path manifest](../../crates/tag-path/Cargo.toml)
  - [types manifest](../../crates/types/Cargo.toml)
  - [UDT manifest](../../crates/udt/Cargo.toml)
  - [desktop example manifest](../../examples/desktop_app/Cargo.toml)
- Dependency boundary: [Cargo.lock](../../Cargo.lock)
- Automated MSRV gate: [.github/workflows/ci.yml](../../.github/workflows/ci.yml)
- Current user-facing policy and requirements:
  - [README.md](../../README.md)
  - [BUILD.md](../../BUILD.md)
  - [docs/API_STABILITY.md](../../docs/API_STABILITY.md)
  - [examples/desktop_app/README.md](../../examples/desktop_app/README.md)
- Historical Rust `1.96` change:
  [CODEX-AH task record](../../docs/agents/tasks/CODEX-AH-rust-1.96-msrv-and-assert-matches.md)
- Rust `1.88` boundary and validation: PR
  [#27](https://github.com/sergiogallegos/rust-ethernet-ip/pull/27)

## Open Questions

- `unclear`: whether future dependency updates should be held back when they
  would raise the locked MSRV without providing a material project benefit.

## Related Pages

- [wrapper-parity/rust-vs-csharp.md](../wrapper-parity/rust-vs-csharp.md)
- [releases/0.8.0-validation-synthesis.md](../releases/0.8.0-validation-synthesis.md)
