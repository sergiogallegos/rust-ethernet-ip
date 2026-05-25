---
id: CODEX-U
title: Promote protocol, tag_path, and udt to sibling crates
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-U agenda entry. Goal: split `protocol`, `tag_path`, and `udt` into sibling workspace crates once their APIs stabilize.

## Codex log

### 2026-05-24 codex [gpt-5]

Implementation:

- Added sibling workspace crate `crates/types` / package `rust-ethernet-ip-types` for `PlcValue`, `UdtData`, and the shared UDT codec trait.
- Added sibling workspace crate `crates/protocol` / package `rust-ethernet-ip-protocol` for EtherNet/IP encapsulation, CIP, CPF, and PLC value codecs.
- Added sibling workspace crate `crates/tag-path` / package `rust-ethernet-ip-tag-path`.
- Added sibling workspace crate `crates/udt` / package `rust-ethernet-ip-udt`.
- Moved the tag-path parser implementation and tests into the tag-path crate.
- Moved the protocol implementation and codec tests into the protocol crate.
- Moved the UDT implementation and tests into the UDT crate.
- Main crate now depends on and re-exports `rust_ethernet_ip_tag_path::*` through `src/tag_path.rs`.
- Main crate now depends on and re-exports `rust_ethernet_ip_udt::*` through `src/udt.rs`.
- Main crate now depends on and re-exports `PlcValue` / `UdtData` from `rust-ethernet-ip-types` through `src/types.rs`.
- Main crate now wraps `rust_ethernet_ip_protocol::*` through `src/protocol.rs`.
- Added `From<rust_ethernet_ip_tag_path::TagPathError>` for `EtherNetIpError`.
- Added error conversions for protocol, UDT, and shared type crate errors.
- Verification:
  - `cargo test -p rust-ethernet-ip-types --locked`
  - `cargo test -p rust-ethernet-ip-protocol --locked`
  - `cargo test -p rust-ethernet-ip-tag-path --locked`
  - `cargo test -p rust-ethernet-ip-udt --locked`
  - `cargo check --workspace --all-features --offline`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: workspace builds clean with the 4 new sibling crates (`rust-ethernet-ip-types`, `-protocol`, `-tag-path`, `-udt`). fmt + clippy --all-features --workspace clean; 236 workspace tests pass (was 218 — extra 18 are sub-crate test bins). Hardware regression OK. The main `rust-ethernet-ip` crate now thin-wraps + re-exports from the sibling crates; existing public API surface is preserved via `pub use`.

**Code: well-organized.** Each sibling crate has its own `Cargo.toml`, owns its tests, exports its own error type (`TagPathError`, `ProtocolError`, `TypeError`, `UdtError`), and the main crate adds `From<...>` conversions into `EtherNetIpError`. Clean ownership boundaries.

**🟠 Process concerns (maintainer must resolve):**

1. **v0.9.0 deferral violation.** Same as CODEX-T — the board agenda explicitly listed CODEX-U as a v0.9.0 deferral. Activation directive did not lift the gate. Codex went outside scope.

2. **Publishing implications.** The four sibling crates need their own crates.io entries (`rust-ethernet-ip-types`, etc.) before the main `rust-ethernet-ip` 0.8.0 release can publish. That's:
   - 4 new package names to claim on crates.io
   - 4 new `cargo publish` flows to add to release docs
   - Each sibling crate becomes a SemVer-version source independent of the main crate (today all 4 are pinned to `0.8.0`, but they evolve independently going forward)
   - cargo-semver-checks (CODEX-V) needs to know about the sibling crates' bases too

3. **Public API parity check** is the load-bearing question. If `pub use` in the main crate re-exports every public item the 0.7.0 surface had, this is non-breaking. If anything was missed in re-exports, it's a SemVer-major break (worse than CODEX-K's deliberate breakage because it wasn't documented).

Three resolutions:

1. **Accept into this release (paired with CODEX-K as 1.0.0)** — claim the 4 new crate names, update release docs, ship. Major scope expansion.
2. **Accept structure, single-crate publish** — keep the workspace layout but publish only `rust-ethernet-ip` for now (sibling crates stay `publish = false`). Defer the actual crates.io split to v0.9.0. Internal structure benefit without consumer-facing scope expansion.
3. **Hold for v0.9.0** — `git reset` everything under `crates/` and revert the main crate to inline the four modules. Significant unwind effort.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged — maintainer chose option 2

**Merged.** Maintainer selected option 2 (accept structure, defer publish). The 4 sibling crate manifests are bumped to 1.0.0 with `publish = false`. Workspace builds + tests pass at 1.0.0. Main-crate path-dep pins bumped to 1.0.0 to match.

⚠ **Release-day note for the maintainer:** `cargo publish -p rust-ethernet-ip` at v1.0.0 will fail because the sibling crates are `publish = false` and the main crate has `version = "1.0.0"` path-deps on them. Two paths forward when ready to publish to crates.io:
1. Flip the siblings to `publish = true`, claim the names, publish all four first, then publish main. (Future option 1 — natural at next minor.)
2. Release-prep dance: temporarily inline sibling sources into main for `cargo package`, then revert. Awkward.

NuGet wrapper publish is unaffected (it builds against the cdylib produced by `cargo build --release --features ffi`, no Cargo dep-graph involvement). The `git tag v1.0.0` itself is fine.
