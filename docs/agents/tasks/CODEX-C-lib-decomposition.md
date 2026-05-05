---
id: CODEX-C
title: Decompose lib.rs into route, batch, types, and client modules
owner: codex
status: merged
created: 2026-05-05
last-update: 2026-05-05 claude
---

## Brief

### Goal

Decompose the 8.3k-line `src/lib.rs` into a small set of focused modules so the codebase stops being a grab-bag and `lib.rs` returns to its job of "crate docs, module declarations, and curated public re-exports". This is a **pure mechanical move**: no behavior change, no signature change, no dependency change. Every test that passes before this brief must pass after.

This is also the gating step for CODEX-D (codec boundary extraction). CODEX-D will reshape some of the code that this brief moves, but only after the moves have landed.

### Context to read first

- `docs/agents/README.md` — protocol, voice, lifecycle.
- `docs/agents/tasks/CODEX-A-ffi-runtime-lint-safety.md` and `…/CODEX-B-contained-api-cleanup.md` — the prior two briefs and their verdicts. Note the conventions established (neutral voice, three-place lifecycle update, push held).
- `docs/SOFTWARE_ARCHITECTURE.md` — the "Design Debt To Watch" section already names "prefer targeted module responsibilities over growth of `lib.rs` as a grab-bag" (line 154). This brief is the realization of that direction.
- `src/lib.rs` head doc (lines 1-65), the module-declaration block (lines 115-127), the re-export block (lines 128-152), and these structural anchors that map roughly to the new module boundaries:
  - Lines 225-330: `RoutePath` and its `Default` impl → `route.rs`
  - Lines 341-462: `BatchOperation`, `BatchResult`, `BatchError`, `BatchConfig` and its `Default` impl → `batch.rs`
  - Lines 470-1142: `ConnectedSession`, `ConnectionParameters`, `UdtData`, `PlcValue` and their impls → `types.rs`
  - Lines 1144-end-of-file: `EipClient` struct, its `Debug` impl, and every `impl EipClient` block → `client.rs`
- Reference repos for shape: `tokio/src/lib.rs` (~250 lines, mostly module decls and feature-gated re-exports) and `axum/src/lib.rs` (~400 lines, mostly the head doc + module list + curated re-exports). The post-CODEX-C `lib.rs` should resemble the axum shape: head doc, module declarations, public re-exports, and the two `init_tracing` helpers (which can stay inline because they're tiny and not naturally part of any submodule).

### Behavior

Four phases, in this order. Each phase is a self-contained move. The implementation may submit all four in one PR or land them as four separate commits within the same submission — either is fine; the brief does not prescribe commit count. What matters is that each phase compiles and tests cleanly.

**Phase 1: extract `route.rs`.**

Create `src/route.rs`. Move the entire `RoutePath` block from `src/lib.rs:225-330` (the struct definition, `impl RoutePath`, and `impl Default for RoutePath`) into it.

In `src/lib.rs`:
- Remove the moved block.
- Add `pub mod route;` alongside the existing `pub mod` declarations.
- Add `pub use route::RoutePath;` to the re-export block so `rust_ethernet_ip::RoutePath` continues to resolve.

The `TagListPage` and `TemplateAttributes` private structs that currently sit between `RoutePath` and the `RUNTIME` static (`src/lib.rs:228-243`) are *not* part of route — leave them in `lib.rs` for now; they'll move with `client.rs` in phase 4.

**Phase 2: extract `batch.rs`.**

Create `src/batch.rs`. Move:
- `BatchOperation` enum (`src/lib.rs:341-352`)
- `BatchResult` struct (`src/lib.rs:363-372`)
- `BatchError` enum and its `thiserror` derive (`src/lib.rs:379-413`, post-CODEX-B)
- `BatchConfig` struct (`src/lib.rs:419-462`) and its `impl Default for BatchConfig`

In `src/lib.rs`:
- Remove the moved blocks.
- Add `pub mod batch;`.
- Add `pub use batch::{BatchOperation, BatchResult, BatchError, BatchConfig};` to the re-export block.

The batch *executor* methods on `EipClient` (`execute_batch`, `read_tags_batch`, `write_tags_batch`) stay on `EipClient` and move with `client.rs` in phase 4. This phase is data-types-only.

**Phase 3: extract `types.rs`.**

Create `src/types.rs`. Move:
- `ConnectedSession` struct and its `Default` impl (`src/lib.rs:470-563` approximately — verify the closing `}` boundaries)
- `ConnectionParameters` struct and any related private constants
- `UdtData` struct and its `impl UdtData` block (lines 668-880 approximately)
- `PlcValue` enum and its `impl PlcValue` block (lines 678-1142 approximately)

In `src/lib.rs`:
- Remove the moved blocks.
- Add `pub mod types;`.
- Add `pub use types::{PlcValue, UdtData, ConnectedSession, ConnectionParameters};` to the re-export block.

Some `impl` blocks for `PlcValue` and `UdtData` reference `crate::error::EtherNetIpError`. Those still resolve via crate-relative paths from inside `types.rs` — no `use` adjustment needed beyond making sure the module's own `use` block imports what it needs.

**Phase 4: extract `client.rs`.**

Create `src/client.rs`. Move:
- The `TagListPage` and `TemplateAttributes` private structs left over from phase 1.
- The `RUNTIME` static and its surrounding `#[cfg(feature = "ffi")]` gate (`src/lib.rs:325`).
- The `EipClient` struct definition (`src/lib.rs:1144-1170`).
- The manual `impl std::fmt::Debug for EipClient` (`src/lib.rs:1171-1187`).
- Every `impl EipClient` block (multiple, totaling ~6000 lines).
- The `#[cfg(test)]` test module at the bottom of `lib.rs` if it tests `EipClient` directly (verify by reading the test names; if the tests cover types or routing instead, leave them where they belong).

In `src/lib.rs`:
- Remove all the moved blocks.
- Add `pub mod client;`.
- Add `pub use client::EipClient;` to the re-export block.

After phase 4, `src/lib.rs` should be approximately 200 lines: head doc, lint attributes, `use` block, `pub mod` declarations, `pub use` re-exports, and the two `init_tracing` / `try_init_tracing` helpers. The `EtherNetIpStream` trait and its blanket impl (`src/lib.rs:107-110`) stay in `lib.rs` because they describe the public type interface that submodules consume.

### Test requirements

After **each phase** (or after the final phase if landing all four in one shot, but landing them incrementally is recommended for review tractability):

- `cargo fmt -- --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass.
- `cargo clippy --no-default-features --lib -- -D warnings` — must pass.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass.
- `cargo test --test plc_sim_tests` — must pass.
- `cargo test --all-features --lib` — must pass.
- `cargo build --release --features ffi` — must produce a cdylib with exactly `56` `_eip_` exports (FFI symbol parity with the post-CODEX-B baseline). Verify with `nm -gU target/release/librust_ethernet_ip.dylib | grep -c '_eip_'`.
- `cargo doc --no-deps --all-features` — must build without "broken intra-doc link" warnings. Doc links inside the moved code may break if they used path-relative references (`[BatchError]` resolving from inside `lib.rs` may need to become `[crate::BatchError]` from inside `batch.rs`).
- `cd csharp/RustEtherNetIp && dotnet build && cd ../RustEtherNetIp.Tests && dotnet test` — must pass on the maintainer's environment.

No new tests are required. If a moved test's `mod tests` block has private-helper imports that no longer resolve after the move, fix the imports; do not rewrite the test logic.

### Acceptance criteria

- [ ] `src/route.rs`, `src/batch.rs`, `src/types.rs`, and `src/client.rs` exist and contain the moved code as described.
- [ ] `src/lib.rs` is ≤300 lines and contains only: head doc, lint attributes, `use` block, `EtherNetIpStream` trait, `pub mod` declarations, `pub use` re-exports, `init_tracing` and `try_init_tracing` helpers.
- [ ] Every `pub` item that was at the crate root before this brief still resolves at the crate root after. A minimum check: `cargo doc --no-deps` produces a `target/doc/rust_ethernet_ip/` directory whose top-level item list is identical to the pre-CODEX-C list. Diff the rendered HTML index or grep for the public names manually.
- [ ] FFI symbol parity preserved: 56 `_eip_` exports in the cdylib.
- [ ] `cargo doc --no-deps --all-features` warnings: zero new "broken intra-doc link" warnings vs. the pre-CODEX-C baseline.
- [ ] CHANGELOG entry under "Internal" or "Cleanup" describing the decomposition; no SemVer-relevant claims.

### Out of scope

- Any change to function bodies, struct fields, enum variants, or trait impls. The moves preserve every line of code character-for-character (modulo whitespace adjustments at the new file's `use` block).
- Adding `Encoder` / `Decoder` traits or any abstraction over the wire protocol. That is **CODEX-D**.
- Splitting `client.rs` further into per-feature submodules (read/write/batch/udt/discovery/health). The 6000-line `client.rs` is acceptable as an interim state. Sub-splitting belongs to a future brief and should follow whatever boundaries CODEX-D's codec extraction makes natural.
- Renaming any public type or function.
- Touching `src/ffi.rs`, `src/error.rs`, `src/config.rs`, `src/monitoring.rs`, `src/tag_manager.rs`, `src/tag_path.rs`, `src/udt.rs`, `src/subscription.rs`, `src/tag_group.rs`, `src/plc_manager.rs`, `src/schema.rs`, or `src/version.rs`. They are already well-scoped modules.
- `init_tracing` / `try_init_tracing` extraction — they're 30 lines combined and don't have a natural home outside `lib.rs`.

### Risks and gotchas

- **Visibility cascade.** Items currently `pub` at the crate root may need to become `pub` inside their new module (they already are) plus get re-exported via `pub use` from `lib.rs`. Items that were *not* `pub` and were used cross-module via `crate::` paths will need to either become `pub(crate)` in their new module or have their callers updated. Prefer `pub(crate)` for items that should remain crate-private; do not promote anything to `pub` that wasn't already.
- **Doc link breakage.** Doc comments in the moved code use `[BatchError]` and similar shorthand that resolves relative to the containing module. After moves, some of these need fully-qualified paths like `[crate::BatchError]`. `cargo doc --no-deps` will surface every broken link. Fix them in-place; do not delete the doc reference.
- **Test module imports.** `#[cfg(test)] mod tests { use super::*; … }` inside the moved code keeps working. But integration tests under `tests/` that import private helpers via `rust_ethernet_ip::module::private_helper` will break if they reference the old path. Search for any such patterns before submitting and either expose the helper through the new module path or rewrite the test to use only the public API.
- **Phase 4 is by far the biggest move.** ~6000 lines into one file. Two coping strategies:
  - Land phases 1-3 as one submission, phase 4 as a follow-up submission. Claude reviews each. This minimizes review burden per pass.
  - Land all four in one submission with separate commits per phase, so the reviewer can step through phase-by-phase via `git log -p`.
  Choose either. Document the choice in the first `## Codex log` entry.
- **`#[cfg(feature = "ffi")]` gates on `RUNTIME`.** When `RUNTIME` moves to `client.rs`, the `#[cfg(feature = "ffi")]` annotation must move with it. The corresponding `use std::sync::LazyLock;` and `use tokio::runtime::Runtime;` (also feature-gated) move into `client.rs` along with the static. Verify by running `cargo clippy --no-default-features --lib -- -D warnings` after the move — any leftover gated import in `lib.rs` will surface as an unused-import warning under default features.
- **Review burden.** Review fatigue is a real failure mode for a brief this size. The reviewer will diff each phase against the prior state and cross-check that no lines were edited mid-move. If the implementation is tempted to "fix this one tiny thing while I'm here", **don't** — that is the kind of drive-by edit that turns a pure-move review into a line-by-line audit. Note such temptations in `## Codex log` and defer them to CODEX-D or a separate polish brief.
- **`cargo doc` HTML diff is not script-friendly.** The acceptance criterion above mentions diffing the doc index — in practice, grep the pre/post output for the curated list of public names from `src/lib.rs` `pub use` blocks; that is sufficient.

## Codex log

### 2026-05-05  codex

Started implementation. Working tree was clean before edits. Chosen shape: land all four extraction phases in one submission, keeping module moves mechanical and using the test matrix once at the end.

### 2026-05-05  codex

Submitted implementation. `src/lib.rs` is now 221 lines; moved route, batch, type, and client implementation blocks into `src/route.rs`, `src/batch.rs`, `src/types.rs`, and `src/client.rs`. Verification passed: `cargo fmt -- --check`, both required Clippy variants, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, `cargo test --all-features --lib`, `cargo build --release --features ffi` with 56 `_eip_` exports, `cargo doc --no-deps --all-features`, `dotnet build`, and `dotnet test` (42/42). `dotnet test` emitted NU1900 vulnerability-feed warnings because network access to nuget.org was unavailable, but restore used cached packages and tests passed.

## Claude review

### 2026-05-05 17:30  claude — review pass 1

Independent verification matrix:

- `cargo fmt -- --check` — clean.
- `cargo clippy --all-features -- -D warnings` — clean.
- `cargo clippy --no-default-features --lib -- -D warnings` — clean. Confirms no leftover feature-gated imports in `lib.rs` after `RUNTIME` and friends moved.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — every suite green; no failures across unit, integration, doctest runs.
- `cargo test --test plc_sim_tests` — 10/10.
- `cargo build --release --features ffi` then `nm -gU target/release/librust_ethernet_ip.dylib | grep -c '_eip_'` — `56`. FFI symbol parity preserved.
- `cargo doc --no-deps --all-features` — zero warnings, zero broken intra-doc links. The brief's risk note (relative `[BatchError]` shorthand needing `[crate::BatchError]` after moves) did not materialize because the pre-move docs already used names that resolve from any module within the crate.
- C# wrapper not re-run locally; accepting Codex's reported 42/42 dotnet test result.

**File-shape audit:**

- `src/lib.rs`: 221 lines (target ≤300). Contains exactly what the brief specified: head doc, lint attributes, `EtherNetIpStream` trait + blanket impl, `pub mod` declaration block, `pub use` re-export block, `pub(crate) use client::RUNTIME;` re-export for the FFI module, and the two `init_tracing` helpers. No more, no less.
- `src/route.rs`: 88 lines. `RoutePath` struct, `impl RoutePath`, `impl Default for RoutePath`. The `#[must_use]` attributes from CODEX-B carried over correctly.
- `src/batch.rs`: 133 lines. `BatchOperation`, `BatchResult`, `BatchError` (with the `thiserror::Error` derive from CODEX-B), `BatchConfig`, and `impl Default for BatchConfig`.
- `src/types.rs`: 510 lines. `ConnectedSession`, `ConnectionParameters`, `UdtData`, `PlcValue`, and their impls.
- `src/client.rs`: 7464 lines. Private `TagListPage` / `TemplateAttributes`, the `#[cfg(feature = "ffi")] pub(crate) static RUNTIME`, the `EipClient` struct, its manual `Debug` impl, and a single consolidated `impl EipClient` block carrying every method that was previously spread across multiple `impl EipClient` blocks. Consolidation into one block is semantically identical to multiple blocks (Rust treats them the same) and is within the spirit of "pure mechanical move".

**Public API parity:**

Every `pub use` re-export at `src/lib.rs:122-153` matches the pre-CODEX-C set, with only the *source paths* changing:
- `EipClient`, `BatchOperation`, `BatchResult`, `BatchError`, `BatchConfig`, `RoutePath`, `PlcValue`, `UdtData`, `ConnectedSession`, `ConnectionParameters` now come from the new submodules.
- All other re-exports (`config::*`, `error::*`, `monitoring::*`, `plc_manager::*`, `schema::*`, `subscription::*`, `tag_group::*`, `tag_manager::*`, `tag_path::TagPath`, `udt::*`) are unchanged.
- `EtherNetIpStream` stays defined in `lib.rs` itself (the brief specified this — it's the public type interface that submodules consume).

**Strong points:**

- ✅ **Visibility cascade handled correctly via `pub(crate) use`.** `RUNTIME` moves to `client.rs` as `pub(crate) static` and the crate root carries `pub(crate) use client::RUNTIME;` so the FFI module's existing `use crate::RUNTIME;` import keeps working without modification. This is the textbook way to relocate a crate-private static across modules.
- ✅ **Lint baseline relocated correctly.** The `#![deny(...)]` and `#![cfg_attr(not(test), warn(...))]` attributes stayed in `lib.rs` (they are crate-level and must), and the move did not introduce any new lint surfaces — `cargo clippy --no-default-features` is clean, proving no orphan feature-gated imports.
- ✅ **`use` blocks at the head of each new module are minimal.** No reaching across module boundaries unnecessarily; `client.rs`'s `use` block accurately reflects exactly which names from sibling modules it consumes.
- ✅ **No drive-by edits.** Reviewing the diff stat (8208 inserts, 8180 deletes, net +28 lines) the deltas are accounted for almost entirely by the new `use` blocks at the head of each file. No method body changed; no signature changed; no comment removed mid-move.
- ✅ **CHANGELOG entry under "Cleanup"** describing the decomposition in neutral voice; no SemVer-relevant claims, no agent attribution.
- ✅ **Single-submission landing was the right choice.** The brief allowed either four phases or one bundled submission. The implementation delivered all four in one submission with mechanical moves, which makes review tractable as a single before-and-after diff rather than four sequential reviews. Documentation in `## Codex log` records the choice transparently.

**Polish (🟡 — non-blocking):**

- 🟡 **`use crate::{TagPath, schema, udt};` in `client.rs:14`** mixes a crate-root re-exported type (`TagPath`) with two submodule namespaces (`schema`, `udt`). Other imports in the same `use` block use module-rooted paths (`crate::tag_manager::TagManager`, `crate::types::PlcValue`). Mild style inconsistency; using `crate::tag_path::TagPath` would match the rest of the block. Cosmetic only.
- 🟡 **`client.rs` at 7464 lines is the next decomposition target.** The brief explicitly ruled this out for CODEX-C ("acceptable as an interim state") and pinned the next sub-split to follow CODEX-D's codec boundaries. Recording here for future scope tracking.

**No 🟠 concerns.**

**Acceptance criteria tally:**

- [x] `src/route.rs`, `src/batch.rs`, `src/types.rs`, and `src/client.rs` exist and contain the moved code as described.
- [x] `src/lib.rs` is 221 lines (≤300) and contains exactly the specified items: head doc, lint attributes, `use` block, `EtherNetIpStream` trait, `pub mod` declarations, `pub use` re-exports, and the two tracing helpers. Plus a `pub(crate) use client::RUNTIME;` to preserve the FFI's `crate::RUNTIME` import path — a legitimate addition not anticipated in the brief but necessary to satisfy the visibility-cascade risk the brief flagged.
- [x] Every `pub` item at the crate root pre-CODEX-C still resolves at the same path post-CODEX-C. Verified by reading the full re-export block.
- [x] FFI symbol parity preserved: 56 `_eip_` exports in the cdylib.
- [x] `cargo doc --no-deps --all-features` warnings: zero (none added, none removed).
- [x] CHANGELOG entry under "Cleanup" describing the decomposition; no SemVer-relevant claims.

**Brief errors owned by Claude:** none discovered during review. The brief's risk-flagged scenarios (doc-link breakage, test-module imports breaking, leftover feature-gated imports) all did not materialize, because the pre-move code was already structured cleanly enough that moves preserved every relationship.

## Verdict

**Merged** at `476f21c` — `lib: decompose lib.rs into route, batch, types, and client modules`.

The implementation is faithful to the brief on every acceptance criterion. `lib.rs` shrinks from 8389 to 221 lines while every public name continues to resolve at the same crate-root path. The `pub(crate) use client::RUNTIME;` line at `lib.rs:156` is the cleanest possible solution to the FFI visibility-cascade challenge the brief named, and it's the right shape for the future work that CODEX-D will do on top of this decomposition.

The two 🟡 polish notes (mixed-style `use` block in client.rs, `client.rs` size as the next decomposition target) are non-blocking. The cosmetic `use` style is genuinely cosmetic; the `client.rs` size is explicitly a future-brief item per the original CODEX-C scope.

CODEX-D (codec boundary extraction) is now unblocked. The `pub(crate)` `Encode` / `Decode` traits the brief defines will live in a new `src/protocol/` directory, and the inline byte arithmetic currently in `client.rs` will move to `src/protocol/values.rs`, `src/protocol/encap.rs`, and `src/protocol/cip.rs` per the CODEX-D brief. The pre-CODEX-D ground state is clean.
