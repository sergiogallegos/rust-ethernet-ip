---
id: CODEX-K
title: Release-window SemVer bundle
owner: codex
status: merged
created: 2026-05-24
last-update: 2026-05-24 claude [Opus 4.7]
---

## Brief

Derived from the existing `docs/agents/board.md` CODEX-K release-window agenda. Goal: bundle deferred SemVer-significant cleanup into the v0.8.0 / 1.0.0-shape release window so breakage happens once.

## Codex log

### 2026-05-24 codex [gpt-5]

Completed CODEX-K items:

- Added `#[non_exhaustive]` to the public enums listed in the board brief:
  - `EtherNetIpError`
  - `BatchError`
  - `RouteHop`
  - `TagPath`
  - `HealthStatus`
  - `HealthCheckMode`
  - `ErrorCategory`
  - `TagGroupEventKind`
  - `TagGroupFailureCategory`
- Changed `try_init_tracing()` to return the crate `Result<()>` instead of `Box<dyn Error>`.
- Changed `ProductionConfig::from_file()` and `to_file()` to return the crate `Result`.
- Replaced stringly logging config fields with Serde-compatible enums:
  - `LogLevel`
  - `LogFormat`
  - `LogRotationSchedule`
- Demoted `ConnectedSession` and `ConnectionParameters` from public crate-root re-exports to `pub(crate)` internal wire state.
- Migrated Rust `RoutePath` to private ordered-hop storage; legacy public `slots` / `ports` / `addresses` fields are removed.
- Added ordered-hop FFI exports:
  - `eip_connect_with_route_hops`
  - `eip_set_route_path_hops`
- Kept the old grouped FFI route functions as compatibility shims.
- Updated C# `RoutePath` to store ordered `RouteHop` values and call the ordered-hop FFI.
- Updated Python `RoutePath` with ordered `RouteHop` support and routed connects through `eip_connect_with_route_hops`.
- Removed duplicate string-specific error variants in favor of existing read/write/invalid-response variants.

Verification so far:

- `cargo check --features ffi --locked`
- `cargo fmt -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo test --test udt_discovery_tests --locked`
- `PYTHONPATH=python python3 -m unittest python.tests.test_client_contract`
- `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj --no-restore -v minimal`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

Independent verification: fmt + clippy clean; workspace tests 236/0; `dotnet test` 79/79; Python 35/8 skipped; hardware regression OK.

**Code-side verdict: implementation is clean.** All items from the original board's CODEX-K agenda landed:
- ✅ `#[non_exhaustive]` on 9 enums (`EtherNetIpError`, `BatchError`, `RouteHop`, `TagPath`, `HealthStatus`, `HealthCheckMode`, `ErrorCategory`, `TagGroupEventKind`, `TagGroupFailureCategory`)
- ✅ `try_init_tracing` returns `crate::Result<()>` instead of `Box<dyn Error>`
- ✅ `ProductionConfig::from_file/to_file` typed
- ✅ Stringly-typed logging config → `LogLevel` / `LogFormat` / `LogRotationSchedule` enums with Serde compat
- ✅ `ConnectedSession`, `ConnectionParameters` demoted to `pub(crate)`
- ✅ `RoutePath` private storage; `slots/ports/addresses` public fields removed
- ✅ Ordered-hop FFI exports `eip_connect_with_route_hops`, `eip_set_route_path_hops` added; old grouped exports kept as compat shims
- ✅ C# and Python `RoutePath` updated to ordered-hop shape
- ✅ Duplicate STRING-specific error variants consolidated

**🟠 Process / release-strategy concern (maintainer must resolve before final merge):**

Cargo.toml is still pinned at `0.8.0`. CODEX-K's whole purpose was to bundle SemVer-major changes for the 1.0.0 cut. The current submission removes public fields from `RoutePath`, removes public re-exports from `lib.rs`, narrows return types (`Box<dyn Error>` → `Result`), and adds `#[non_exhaustive]` to 9 public enums. **All breaking vs the 0.7.0 crates.io baseline.**

Three possible resolutions, maintainer decision:

1. **Tag this as 1.0.0** — bump `Cargo.toml` to `1.0.0`, promote `CHANGELOG.md` `[Unreleased]` → `[1.0.0] - 2026-05-24`, tag `v1.0.0`. The release-window framing is honored; cargo-semver-checks (CODEX-V) will accept the major bump.
2. **Hold CODEX-K, ship 0.8.0 first** — `git reset` the K-specific changes (the enum non_exhaustive + RoutePath private storage + try_init_tracing return type), tag v0.8.0 with the rest (L/N/V/W/X/Y/M/J/P/Q/R/S/T/U), then re-land K targeting 1.0.0. Bigger churn, but honors the original "v0.8.0 minor, 1.0.0 release-window" sequencing.
3. **Tag as 0.8.0 anyway** — *not recommended*. cargo-semver-checks on the next push to main will block (CODEX-V job is required on main); downstream Cargo consumers picking up `^0.8.0` will silently break.

This brief stays `submitted` (not `merged`) until the maintainer picks 1, 2, or 3. **The code itself is correct and review-clean** — the question is purely about release-strategy timing.

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged — maintainer chose 1.0.0 cut

**Merged.** Maintainer selected option 1 (tag as 1.0.0). Cargo.toml bumped to 1.0.0, `src/lib.rs` head-doc release-line references bumped, CHANGELOG `[Unreleased]` promoted to `[1.0.0] - 2026-05-24` with a new empty `[Unreleased]` above. Sibling crate path-dep pins bumped to 1.0.0 to match. The actual `git tag v1.0.0` + crates.io / NuGet publish flow remains maintainer-owned (per the original gating items in `board.md`).
