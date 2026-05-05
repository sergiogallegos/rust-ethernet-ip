---
id: CODEX-A
title: FFI safety, runtime hardening, and lint baseline
owner: codex
status: open
created: 2026-05-05
last-update: 2026-05-05 claude
---

## Brief

### Goal

Tighten the FFI boundary so failures inside the Rust core surface as actionable error codes to the C# host instead of process-wide panics, gate the `ffi` module behind its existing Cargo feature so pure-Rust consumers stop pulling the C ABI surface, and add a small, conservative crate-level lint baseline. This is the first contained-scope brief preparing the ground for later work (`lib.rs` decomposition, codec extraction). It must not change the public C ABI or the high-level Rust API surface.

### Context to read first

- `docs/agents/README.md` — protocol, voice, lifecycle.
- `CLAUDE.md` — project conventions; pay attention to "Workspace Layout" and "Architecture".
- `docs/SOFTWARE_ARCHITECTURE.md` — especially "Important Invariants" (FFI must not hold global registry locks across network operations) and "Design Debt To Watch" (avoid new global locks in the FFI boundary).
- `src/ffi.rs` — current FFI surface; note the existing `lock_clients()` / `lock_next_id()` helpers that already convert poison to `Err`. The pattern they establish should be the model for the std-Rust callsites that still `.unwrap()` on lock acquisition.
- `Cargo.toml` — note the existing but currently inert `ffi = []` feature.
- `src/lib.rs:1-122` — current crate-level docs and module declarations. The module `pub mod ffi;` line is what gets gated.

### Behavior

Five contained changes, in this order:

**1. Gate the FFI module on the `ffi` feature.**

In `src/lib.rs`, change `pub mod ffi;` to be feature-gated:

```rust
#[cfg(feature = "ffi")]
pub mod ffi;
```

The `cdylib` artifact must still build with the FFI surface intact. Update every cargo build invocation that produces the cdylib to pass `--features ffi`:

- `build.bat:7`
- `build-all.bat:15`
- `.github/workflows/ci.yml:90`
- `.github/workflows/release.yml:34`

The `rlib` build path (used by Rust consumers and `cargo test`) does **not** pass `--features ffi`. Verify by running `cargo build` (no flags) and confirming the `ffi` module is absent from the resulting `rlib`.

The `RUNTIME` static at `src/lib.rs:318` is currently used both by FFI callers and by Rust callers. Audit whether anything outside `src/ffi.rs` references `RUNTIME` — if not, move it under `#[cfg(feature = "ffi")]` along with the `ffi` module. If it has non-FFI references, leave it where it is and document that in a one-line comment.

**2. Replace `Runtime::new().unwrap()` with surfaced failure reporting.**

Current: `static RUNTIME: LazyLock<Runtime> = LazyLock::new(|| Runtime::new().unwrap());` (`src/lib.rs:318`).

New shape:

```rust
static RUNTIME: LazyLock<std::io::Result<Runtime>> =
    LazyLock::new(Runtime::new);
```

Every FFI entry point that currently does `RUNTIME.block_on(...)` needs to first match on `&*RUNTIME` and return a new error code (`EIP_ERROR_RUNTIME_INIT` or similar — pick the next free integer in the existing FFI return-code namespace and add it to the FFI header / C# constants). On the error branch, write a `tracing::error!` with the `io::Error` for diagnostics before returning.

This change is invisible to a healthy host (the `Runtime::new()` call almost always succeeds); it converts an undiagnosable process abort into a clean error code path.

**3. Convert `RwLock` poisoning unwraps at FFI-reachable sites.**

These are the `RwLock::read().unwrap()` / `RwLock::write().unwrap()` sites that sit on the path from an FFI call. A poisoned lock today panics across the C ABI:

- `src/tag_manager.rs:129` (`get_metadata`)
- `src/tag_manager.rs:140` (`update_metadata`)
- `src/tag_manager.rs:166` (`clear_cache`)
- `src/lib.rs:1396` (cache populate after tag discovery)
- `src/lib.rs:1427` (UDT definition cache write)
- `src/lib.rs:1649` (cache read on drill-down)
- `src/lib.rs:7225` — separate concern: this is a `HashMap::get(...).unwrap()`, not a lock unwrap. It assumes the prior `establish_connected_session` populated the entry. Replace with `.ok_or_else(|| EtherNetIpError::Connection(...))` for defense in depth; if the assumption is genuinely unviolable, justify with a comment instead.

For the lock unwraps, the project has a choice: convert them to `EtherNetIpError::Other("lock poisoned: …")`, or take a dependency on `parking_lot::RwLock` (which has no poisoning). Choose **the former** for this brief — fewer dependencies, and the caller still gets to react. A small helper in `src/error.rs` is fine:

```rust
impl<T> From<std::sync::PoisonError<T>> for EtherNetIpError {
    fn from(_: std::sync::PoisonError<T>) -> Self {
        EtherNetIpError::Other("lock poisoned".into())
    }
}
```

This lets the callsites use `?` instead of bespoke `.map_err`. Note: `tag_manager.rs:120-126`'s `new()` returning `Self` does not need to change. Only the `get_metadata` / `update_metadata` / `clear_cache` / `remove_stale_entries` (and similar) async methods need their signatures to return `Result<...>` if they don't already. `get_metadata` currently returns `Option<TagMetadata>` — it should now return `Result<Option<TagMetadata>>`.

Audit every caller of these methods after the signature change and propagate the `?` correctly.

**4. Add a conservative crate-level lint baseline at `src/lib.rs:1`.**

Insert *before* the existing `//! …` doc comment block (or directly above the first `use` after the docs — whichever the rest of the codebase prefers; tokio puts attributes after docs, axum puts them after docs too, so put them after the head doc but before `use crate::udt::UdtManager;`):

```rust
#![deny(unused_must_use, unsafe_op_in_unsafe_fn)]
#![cfg_attr(not(test), warn(clippy::print_stdout, clippy::dbg_macro))]
```

Specifically **do not** add `missing_docs`, `missing_debug_implementations`, or `rust_2024_idioms`:

- `missing_docs` and `missing_debug_implementations` produce a wall of warnings that should be cleared in a separate cleanup brief, not bundled here.
- `rust_2024_idioms` is not a real lint group. The rustc lint groups are `rust_2018_idioms` (still applicable on edition 2024) and `rust-2024-compatibility` (a migration group, not an ongoing style lint). Don't add either in this brief.

`unsafe_op_in_unsafe_fn` will likely surface a few diagnostics in `src/ffi.rs` where an unsafe operation sits inside an `unsafe extern "C" fn` without an inner `unsafe { ... }` block. Wrap each one in a minimal `unsafe { ... }` block; do not add `#[allow]`.

Run `cargo clippy --all-features -- -D warnings` and `cargo clippy --no-default-features -- -D warnings` and fix anything the new lints surface.

**5. Verify the C# native build still works end-to-end.**

After the feature-gating changes, run the full local C# wrapper test sequence:

```
cargo build --release --features ffi
cd csharp/RustEtherNetIp && dotnet build
cd ../RustEtherNetIp.Tests && dotnet test
```

If the native artifact location changed (it should not — `cdylib` output path is the same), update `pack-nuget.sh` / `pack-nuget.ps1` only as much as needed to match. Do not change unrelated logic.

### Test requirements

- `cargo fmt --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass with the new lints active.
- `cargo clippy --no-default-features -- -D warnings` — must pass; this is the path that proves the `ffi` module is absent from the default build.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass.
- `cargo test --test plc_sim_tests` — must pass.
- `cd csharp/RustEtherNetIp && dotnet build && cd ../RustEtherNetIp.Tests && dotnet test` — must pass.
- A new unit test under `src/error.rs` exercising the `From<PoisonError<_>>` impl: spawn a thread that panics while holding a `Mutex` guard, catch the poison on the main thread, convert via `?`, assert the variant.
- A new unit test confirming `RUNTIME` returns `Err` cannot be reproduced without injection — instead, add a small `#[cfg(test)]` shim that exercises the FFI error path by calling an FFI entry while a forced-error variant of the runtime accessor returns `Err`. If that shim is intrusive, document the gap in `## Codex log` and skip it.

### Acceptance criteria

- [ ] `cargo build` (no features) produces an `rlib` that does not contain symbols from `src/ffi.rs`. Verify with `cargo build && nm target/debug/librust_ethernet_ip.rlib | grep -ci eip_` returning `0`.
- [ ] `cargo build --release --features ffi` produces a `cdylib` whose exported symbol set is byte-for-byte identical to the prior release's, modulo build metadata. Diff symbol lists before/after to prove parity.
- [ ] All four cargo build invocations listed above (`build.bat`, `build-all.bat`, `ci.yml`, `release.yml`) pass `--features ffi`.
- [ ] `RUNTIME` initialization failure surfaces as a documented FFI return code (added to the public FFI header and to the C# constants), not a process abort.
- [ ] All seven `unwrap` sites listed in section 3 are converted; a `#[cfg(test)]` test demonstrates poison recovery via the `From<PoisonError<_>>` impl.
- [ ] The two new lint attributes are present at the top of `src/lib.rs`; clippy is clean under both feature configurations.
- [ ] C# wrapper build and tests pass on the maintainer's local environment (the maintainer will report the dotnet test run).
- [ ] CHANGELOG.md gets a new "Unreleased" section entry under "Fixed" / "Internal" describing the FFI hardening; no semver-relevant API changes are claimed (none should exist).

### Out of scope

- Decomposing `lib.rs` into submodules (CODEX-B candidate).
- Converting `BatchError` to `thiserror`, dropping the `async-trait` direct dep, removing `use serde_json;`, fixing the `0.7.0` docstring rot, adding `#[must_use]` to builder methods, the `tag_subscription.rs` shim, the underscore-prefixed `EipClient` fields. All belong to a separate "contained API cleanup" brief (CODEX-B candidate).
- Adding `#[non_exhaustive]` to public enums — that is a SemVer-breaking change and belongs to a release-window brief, not here.
- Switching `try_init_tracing` away from `Box<dyn Error>` — also a public signature change; defer to the cleanup brief with a typed parallel function.
- Adding broad lints (`missing_docs`, `missing_debug_implementations`).
- Reworking `Arc<Mutex<mpsc::Sender>>` patterns — those are perf hygiene, not safety, and belong with the broader subscription cleanup.
- Codec / encoder boundary extraction.

### Risks and gotchas

- **Symbol parity.** The C# wrapper depends on every exported symbol in the cdylib. Feature-gating is invisible to the cdylib build path *only if* `--features ffi` is added everywhere. Missing one workflow step leaves a broken release artifact. Verify with the symbol-list diff in the acceptance criteria.
- **`get_metadata` signature change ripples.** Changing `Option<TagMetadata>` → `Result<Option<TagMetadata>>` will touch every caller. Most should propagate via `?`; some may swallow via `.ok().flatten()` if the calling context cannot return a `Result`. Prefer propagation; if swallowing is necessary, comment why.
- **`PoisonError` conversion erases the inner data.** That is intentional here — the panicked guard's inner state is by definition not trustworthy. If a callsite needs to recover from a panic with state intact, it should opt out of the blanket `?` conversion and handle `PoisonError` explicitly. None of the listed callsites need this today.
- **`unsafe_op_in_unsafe_fn` will require touching `src/ffi.rs`.** Some `unsafe extern "C" fn` bodies use raw pointers without an inner `unsafe { ... }` block. Wrap each operation; resist the temptation to wrap the whole function body in one giant `unsafe { ... }` — that defeats the purpose of the lint.
- **Documentation rot.** `src/lib.rs:5` and `src/lib.rs:48` say `0.7.0` but the package is `0.8.0`. **Do not fix in this brief** — left for the cleanup brief. Mentioned only so it isn't accidentally edited here as a drive-by.

## Codex log

*(empty — codex appends entries on starting work)*

## Claude review

*(empty — claude appends after submission)*

## Verdict

*(empty — claude writes on merge or rejection)*
