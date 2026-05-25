# Sibling Crate Boundaries

## Summary

CODEX-U splits previously main-crate-owned internals into sibling workspace crates while preserving main-crate re-exports or wrappers for existing callers.

## Current Understanding

- `confirmed`: `rust-ethernet-ip-types` owns `PlcValue`, `UdtData`, and the `UdtCodec` trait.
- `confirmed`: `rust-ethernet-ip-protocol` owns encapsulation, CIP/CPF, and PLC value codec logic.
- `confirmed`: `rust-ethernet-ip-tag-path` owns Logix tag-path parsing and CIP path generation.
- `confirmed`: `rust-ethernet-ip-udt` owns UDT definitions, tag attributes, UDT parsing, and `UserDefinedType` helpers.
- `confirmed`: the main crate keeps compatibility by re-exporting/wrapping these crates through `src/types.rs`, `src/protocol.rs`, `src/tag_path.rs`, and `src/udt.rs`.

## Evidence

- Workspace manifest: [`../../Cargo.toml`](../../Cargo.toml)
- Shared types crate: [`../../crates/types/src/lib.rs`](../../crates/types/src/lib.rs)
- Protocol crate: [`../../crates/protocol/src/lib.rs`](../../crates/protocol/src/lib.rs)
- Tag-path crate: [`../../crates/tag-path/src/lib.rs`](../../crates/tag-path/src/lib.rs)
- UDT crate: [`../../crates/udt/src/lib.rs`](../../crates/udt/src/lib.rs)
- Main-crate error conversions: [`../../src/error.rs`](../../src/error.rs)
- Task record: [`../../docs/agents/tasks/CODEX-U-sibling-crates.md`](../../docs/agents/tasks/CODEX-U-sibling-crates.md)

## Open Questions

- Whether these sibling crates should be published independently or kept as workspace-internal packages for the v0.8.0 release.
- Whether wrapper package artifacts should include any explicit version/capability signal for the internal crate split.

## Related Pages

- [`architecture-review-2026-05-18.md`](architecture-review-2026-05-18.md)
- [`client-actor-service-retry-2026-05-24.md`](client-actor-service-retry-2026-05-24.md)
