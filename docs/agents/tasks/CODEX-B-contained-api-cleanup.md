---
id: CODEX-B
title: Contained API cleanup — thiserror, dead deps, dead state, must_use
owner: codex
status: merged
created: 2026-05-05
last-update: 2026-05-05 claude
---

## Brief

### Goal

A contained sweep of mechanical cleanup items that make the public surface tidier without changing the semver-major API contract: convert the last hand-rolled `Display`/`Error` impl to `thiserror`, remove a direct dependency that no source file uses, fix two docstring references to a stale version number, kill a one-line module shim, delete three unused `EipClient` fields, and add `#[must_use]` to a handful of builder/getter methods. Pair with two small polish items carried over from CODEX-A's review.

This brief deliberately excludes anything that changes the public API in a SemVer-major way (no `#[non_exhaustive]`, no enum-ifying stringly-typed config fields, no `try_init_tracing` signature change). Those belong in a release-window brief.

### Context to read first

- `docs/agents/README.md` — protocol, voice, lifecycle.
- `docs/agents/tasks/CODEX-A-ffi-runtime-lint-safety.md` — the prior brief's Verdict and the four 🟡 polish notes; two of them are folded into this brief and the other two are explicitly deferred.
- `CLAUDE.md` and `docs/SOFTWARE_ARCHITECTURE.md` — design debt list.
- `src/lib.rs` head doc (lines 1-65), `BatchError` (lines 368-413), `EipClient` struct (lines 1153-1177), the two `EipClient` constructors (`from_stream` ~1206 and `new_unconnected_for_testing` ~1245).
- `src/subscription.rs` — the real implementation.
- `src/tag_subscription.rs` — the one-line re-export shim (delete target).
- `src/version.rs` — pure getters that should be `#[must_use]`.
- `Cargo.toml` — `async-trait = "0.1"` is still listed; no source file uses it after the AFIT migration.

### Behavior

Eight contained changes. They are independent and can land in any order, but the suggested order minimizes diff conflicts.

**1. Convert `BatchError` to `thiserror`.**

`src/lib.rs:368-413` currently hand-rolls `impl std::fmt::Display for BatchError` and `impl std::error::Error for BatchError`. Replace with a `#[derive(Debug, Clone, thiserror::Error)]` annotation matching the existing `EtherNetIpError` style in `src/error.rs`. Each variant gets an `#[error("…")]` attribute carrying the same format string the manual `match` arm currently produces. Verify the `Display` output is byte-for-byte identical for every variant — the existing tests in `tests/concurrency_tests.rs` and elsewhere that match on the formatted string must continue to pass.

Keep `BatchError` enum variants exactly as-is. Do **not** add `#[non_exhaustive]` (that is a SemVer-major change, deferred).

**2. Drop the `async-trait` direct dependency.**

`Cargo.toml:51` lists `async-trait = "0.1"`. No source file under `src/` uses `#[async_trait]` after the AFIT migration. Remove the line. Run `cargo update -p async-trait --precise 0.0.0` is not the right approach — just delete the dep line and let `cargo build` regenerate `Cargo.lock` with the indirect dependencies still present (other crates pull `async-trait` transitively; that is fine and unrelated).

After the change: `cargo tree -i async-trait` should show only transitive uses (e.g. through `tracing-subscriber` or test deps), never the root crate.

**3. Fix the `0.7.0` docstring rot.**

`src/lib.rs:5` reads `//! The current released crate line is `0.7.0`.` and `src/lib.rs:48` reads `//! Real-hardware validation for the `0.7.0` release line confirmed that some`. The package version is `0.8.0` (`Cargo.toml:5`). Update both to `0.8.0`. While in this docstring, audit nothing else — drive-by edits are out of scope. Just the version strings.

For long-term durability: do **not** introduce `env!("CARGO_PKG_VERSION")` in the doc comment — doc comments don't expand macros. Use a literal string and accept that this needs periodic updating with each release. Add a one-line note to `CHANGELOG.md` reminding future release-prep that the lib.rs head doc carries a literal version reference.

**4. Remove the `src/tag_subscription.rs` shim.**

The file is a single line: `pub use crate::subscription::{SubscriptionManager, SubscriptionOptions, TagSubscription};`. The crate-root re-exports at `src/lib.rs:154-157` go through the shim:

```rust
pub use tag_subscription::{
    SubscriptionManager as RealTimeSubscriptionManager,
    SubscriptionOptions as RealTimeSubscriptionOptions,
    TagSubscription as RealTimeSubscription,
};
```

Replace with a direct re-export from `subscription`:

```rust
pub use subscription::{
    SubscriptionManager as RealTimeSubscriptionManager,
    SubscriptionOptions as RealTimeSubscriptionOptions,
    TagSubscription as RealTimeSubscription,
};
```

Then delete `src/tag_subscription.rs` and remove `pub mod tag_subscription;` from `src/lib.rs:127`.

**Surface impact:** The crate-root names `rust_ethernet_ip::RealTimeSubscriptionManager` etc. are unchanged. The path `rust_ethernet_ip::tag_subscription::*` *is* removed. A grep across `src/`, `tests/`, `csharp/`, `python/`, and `examples/` finds zero external uses of that path; the only references are the re-exports themselves at `src/lib.rs:154-157`. Treat the path removal as practically internal but document it in the CHANGELOG under "Removed" (not "Breaking"); call it "consolidated subscription re-exports" so users searching for `tag_subscription` find a hint.

**5. Delete three unused `EipClient` fields.**

`src/lib.rs:1160-1175` declares `_connection_id`, `_connected`, and `_session_timeout`. Each is initialized in `from_stream` (~`src/lib.rs:1213`) and `new_unconnected_for_testing` (~`src/lib.rs:1251`). None of the three is read anywhere except the manual `Debug` impl at `src/lib.rs:1184-1207`, which prints `_session_timeout` only.

Audit:

```
grep -n "_connection_id\b\|_connected\b\|_session_timeout\b" src/
```

Should show only the field declaration sites, the two constructors, and the `Debug` impl. If grep finds any other use, **stop and ask** in `## Codex log` — the original brief's audit may be wrong.

If grep matches the audit, delete:

- the three field declarations on the `EipClient` struct.
- the three field initializations in both constructors.
- the `_session_timeout` field on the `Debug` impl.

This is a `#[derive(Clone)]`-compatible change (the struct stays `Clone`; the layout change is internal).

Do **not** rename or repurpose any of the three. If a future feature needs a "session timeout" or "connected status" field, it should be added with a real name and real usage in that feature's PR, not resurrected here.

**6. Add `#[must_use]` to selected builder and getter methods.**

The minimum target list:

- `src/lib.rs:243` — `pub fn new() -> Self` on `RoutePath`. Annotate as `#[must_use]`.
- `src/lib.rs:252, 258, 264` — `add_slot`, `add_port`, `add_address` on `RoutePath`. All return `Self`; all should carry `#[must_use]`.
- `src/lib.rs:278` — `to_cip_bytes(&self) -> Vec<u8>` on `RoutePath`. `#[must_use]`.
- `src/error.rs:107` — `is_retriable(&self) -> bool` on `EtherNetIpError`. `#[must_use]`.
- `src/version.rs:31, 36, 41` — `get_version`, `get_name`, `get_description`. `#[must_use]`.

Do **not** add `#[must_use]` to `Result`-returning or `Option`-returning functions; `Result` and `Option` already carry the warning by virtue of their own `#[must_use]` declarations.

Do **not** add `#[must_use]` to `Default::default` impls — the trait method does not propagate the attribute, and the standard pattern is `#[must_use] pub fn new() -> Self` already covered above.

**7. Polish carry-over from CODEX-A: RAII guard for `FORCE_RUNTIME_INIT_ERROR`.**

The test in `src/ffi.rs::tests::forced_runtime_init_error_returns_documented_code` does `store(true) → call → store(false)`. If the test panics between the two stores, the flag stays on and pollutes any subsequent test in the same process. Wrap the flag in an RAII guard:

```rust
struct ForceRuntimeInitErrorGuard;
impl ForceRuntimeInitErrorGuard {
    fn enable() -> Self {
        FORCE_RUNTIME_INIT_ERROR.store(true, std::sync::atomic::Ordering::SeqCst);
        Self
    }
}
impl Drop for ForceRuntimeInitErrorGuard {
    fn drop(&mut self) {
        FORCE_RUNTIME_INIT_ERROR.store(false, std::sync::atomic::Ordering::SeqCst);
    }
}
```

Update the test to bind `let _guard = ForceRuntimeInitErrorGuard::enable();`. The Drop runs even if the test panics.

**8. Polish carry-over from CODEX-A: doc comment on `ffi_block_on!`.**

The macro at `src/ffi.rs:42-50` early-returns the runtime-init error code if the global runtime is unavailable. That early return only makes sense in a function returning `c_int`. Add a one-line doc comment above the macro definition:

```rust
/// Awaits a future on the FFI Tokio runtime, early-returning
/// `EIP_ERROR_RUNTIME_INIT` if the runtime is unavailable. Only call
/// from inside an `unsafe extern "C" fn ... -> c_int` body.
```

Nothing more — no full rustdoc with examples; the macro is private to `ffi.rs`.

### Test requirements

- `cargo fmt --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass (the new `#[must_use]` annotations may surface call sites that discard the return; if so, fix the call sites to use the value or add an explicit `let _ = …` with a comment about why).
- `cargo clippy --no-default-features -- -D warnings` — must pass.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass.
- `cargo test --test plc_sim_tests` — must pass.
- `cd csharp/RustEtherNetIp && dotnet build && cd ../RustEtherNetIp.Tests && dotnet test` — must pass.
- `cargo tree -i async-trait` should not list the root crate as a direct user.
- `grep -rn "RealTimeSubscription" src/ tests/ csharp/ python/ examples/` should show only the re-export site at `src/lib.rs` (no orphan references after the shim removal).
- `grep -rn "_connection_id\b\|_connected\b\|_session_timeout\b" src/` should return zero matches after the deletions.

No new tests are required for the cleanup itself — every change is mechanical and existing tests cover the surface.

### Acceptance criteria

- [ ] `BatchError` is a `thiserror::Error`-derived enum; no manual `Display` / `Error` impls remain in `src/lib.rs`. `Display` output is identical to the prior implementation, verified by manual inspection of the test assertions that touch batch errors.
- [ ] `Cargo.toml` no longer lists `async-trait` as a direct dependency. `cargo tree -i async-trait` confirms no root-crate use.
- [ ] `src/lib.rs:5` and `src/lib.rs:48` both read `0.8.0`. CHANGELOG carries a one-line release-prep reminder.
- [ ] `src/tag_subscription.rs` is deleted; `pub mod tag_subscription;` is removed from `src/lib.rs`; the crate-root `RealTimeSubscription*` aliases are unchanged in name and re-export the same types.
- [ ] `_connection_id`, `_connected`, and `_session_timeout` are removed from `EipClient`, both constructors, and the `Debug` impl. No grep hits in `src/`.
- [ ] All seven `#[must_use]` annotations are present at the listed sites; clippy is clean under both feature configurations.
- [ ] The `ForceRuntimeInitErrorGuard` RAII type exists in `src/ffi.rs::tests` and the existing forced-runtime-init test uses it. The test still passes; if intentionally panicking inside the test would prove the guard works, that is fine but not required.
- [ ] The `ffi_block_on!` macro carries a doc comment pinning its calling-context contract.
- [ ] CHANGELOG entry under "Internal" or "Cleanup" describes the contained changes; no semver-relevant claims.

### Out of scope

- `#[non_exhaustive]` on any enum (SemVer-major; deferred to a release-window brief).
- Converting `try_init_tracing` away from `Box<dyn Error>` (public signature change; deferred).
- Stringly-typed config fields (`level`, `format`, `schedule`) → enums (Serde representation change; deferred).
- `lib.rs` decomposition (CODEX-C candidate).
- Codec / encoder boundary extraction (CODEX-D candidate).
- `Arc<Mutex<mpsc::Sender>>` over-locking cleanup (separate brief; perf hygiene).
- The two larger polish items from CODEX-A: dedupe runtime-init logging via `Once`, and considering a `Result<Option<_>>` form for `get_tag_metadata` in a future release. Both are documented in CODEX-A's review for future briefs.
- Any drive-by edit to the `lib.rs` head doc beyond the two version-string fixes. If something else looks wrong, write a `## Codex log` note and stop.

### Risks and gotchas

- **`#[must_use]` may surface unused-result warnings at existing call sites.** This is the point — if a caller was discarding the value, that is a latent bug. Fix the caller; do not suppress the warning. If the caller genuinely needs to discard (rare), use `let _ = …;` with a one-line comment.
- **`thiserror`'s `#[from]` is not the right tool here.** `BatchError` variants do not carry source errors from foreign types; they are constructed directly. Just use `#[error("…")]` per variant.
- **Shim removal touches one public path.** `rust_ethernet_ip::tag_subscription::*` no longer exists after this brief. The grep audit should confirm no in-tree user. If a downstream user (outside this repo) imports that path, they need to switch to the crate-root alias. The CHANGELOG entry under "Removed" is the heads-up.
- **`async-trait` removal must not break `Cargo.lock`.** Other crates pull `async-trait` transitively; deleting it from `[dependencies]` only removes the root-crate edge. `cargo build` should regenerate `Cargo.lock` cleanly; if it produces a churn-heavy lock diff, commit it as part of this brief without alarm.
- **The `Debug` impl on `EipClient` currently uses dummy strings (`"<stream>"`, `"<tag_manager>"`, …) for the `Arc`-wrapped fields.** Removing the `_session_timeout` field from the `Debug` impl is the only Debug change. Do not touch the dummy-string pattern; that is a deliberate Clone-handle ergonomics choice and belongs in a separate review if it ever changes.
- **Don't bundle CODEX-A polish items beyond the two listed.** The other two (`Once`-based log dedupe, `get_tag_metadata` Result return) are explicitly deferred. Pulling them in here grows scope.
- **The `#[must_use]` list above is the minimum.** If clippy with `must_use_candidate` fires more, you may add them — but only on `pub` functions, only if the return value is genuinely informative, and never on functions that may be called for side effects. Document any additions in `## Codex log`.

## Codex log

### 2026-05-05  codex

Started implementation. Working tree was clean before edits.

### 2026-05-05 14:14  codex

Implemented CODEX-B and submitted for review.

- Converted `BatchError` to derive `thiserror::Error` with the same display strings as the prior manual implementation.
- Removed the direct `async-trait` dependency from `Cargo.toml`; the active dependency tree no longer contains `async-trait`.
- Updated only the two requested `src/lib.rs` head-doc release-line references from `0.7.0` to `0.8.0`.
- Removed `src/tag_subscription.rs`, removed `pub mod tag_subscription`, and re-exported the crate-root `RealTimeSubscription*` aliases directly from `subscription`.
- Removed `_connection_id`, `_connected`, and `_session_timeout` from `EipClient`, both constructors, and the `Debug` impl.
- Added the requested `#[must_use]` attributes.
- Added `ForceRuntimeInitErrorGuard` around the forced runtime-init test flag and documented the `ffi_block_on!` calling-context contract.
- Updated `CHANGELOG.md` with the cleanup notes and the literal-version release-prep reminder.

Verification:

- `cargo fmt --check` passed.
- `cargo clippy --all-features -- -D warnings` passed.
- `cargo clippy --no-default-features -- -D warnings` passed.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` passed with localhost bind permission.
- `cargo test --test plc_sim_tests` passed with localhost bind permission.
- `cargo test --all-features --lib` passed, including `ffi::tests::forced_runtime_init_error_returns_documented_code`.
- `dotnet build` passed in `csharp/RustEtherNetIp`.
- `dotnet test` passed in `csharp/RustEtherNetIp.Tests`: 42/42 tests. NuGet vulnerability metadata lookup warned because `api.nuget.org` was unavailable, but restore/build/test completed.
- `cargo tree -i async-trait -e all` returned "package ID specification `async-trait` did not match any packages"; `cargo tree -e all | rg "async-trait|rust-ethernet-ip v"` showed only the root crate line and no `async-trait`.
- `rg -n "\b_(connection_id|connected|session_timeout)\b" src/` returned zero matches.
- `rg -n "RealTimeSubscription" src/ tests/ csharp/ python/ examples/ --glob '!target/**'` returned only the crate-root re-export site in `src/lib.rs`.

Note: the exact broad grep pattern from the brief, `grep -rn "_connection_id\b\|_connected\b\|_session_timeout\b" src/`, also matches unrelated `o_to_t_connection_id` / `t_to_o_connection_id` fields. Those are active connected-session fields and were not changed.

## Claude review

### 2026-05-05 15:10  claude — review pass 1

Independent verification matrix:

- `cargo fmt -- --check` — clean.
- `cargo clippy --all-features -- -D warnings` — clean.
- `cargo clippy --no-default-features --lib -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — all suites green; `--all-features --lib` reports 56 passing (one more than CODEX-A, accounting for the new `RAII` guard test path indirectly).
- `cargo test --test plc_sim_tests` — 10/10.
- `cargo build --release --features ffi` — produces a cdylib with `56` `_eip_` symbols. Symbol parity with the post-CODEX-A baseline preserved.
- `cargo tree -i async-trait` — "did not match any packages". Confirmed `async-trait` is not in the active build graph; the lockfile entry persists as a transitive of `cargo-tarpaulin` on Linux platforms (legitimate, not stale).
- `grep` audit confirms no orphan references for `_connection_id` / `_connected` / `_session_timeout`, no in-tree users of the deleted `tag_subscription` path, and `RealTimeSubscription*` aliases live exactly at the new re-export site.
- C# wrapper not re-run locally; accepting Codex's reported 42/42 dotnet test result. The NuGet metadata-lookup warning Codex noted is environmental (api.nuget.org unavailability during a sandboxed run) and does not affect test outcomes.

**Strong points:**

- ✅ **Format strings on `BatchError` are byte-identical to the prior manual `Display`.** Confirmed variant-by-variant: `TagNotFound`, `DataTypeMismatch`, `NetworkError`, `CipError` (with `0x{status:02X}` casing preserved), `TagPathError`, `SerializationError`, `Timeout`, `Other`. No semantic drift; existing tests that match on formatted strings continue to pass.
- ✅ **The `AtomicBool` import was caught and removed in the cascade.** Deleting `_connected: Arc<AtomicBool>` orphaned the `use std::sync::atomic::AtomicBool;` import; Codex removed it. Easy to miss.
- ✅ **Shim removal preserves crate-root names.** `RealTimeSubscription{,Manager,Options}` still resolve to the same types via the new direct re-export from `subscription`. `cargo build` and `cargo doc` produce no warnings about orphaned re-exports.
- ✅ **RAII guard is panic-safe.** `ForceRuntimeInitErrorGuard` resets the flag on `Drop`, so a panic mid-test no longer pollutes subsequent tests. The pattern is short (one struct + one `Drop`) and the test now reads as a single line of intent.
- ✅ **`ffi_block_on!` doc comment is the right size.** Three lines pinning the calling contract; no rustdoc bloat.
- ✅ **`#[must_use]` placement is exactly the requested set.** Builders (`RoutePath::*`), pure getters (`version::*`), and the predicate getter (`EtherNetIpError::is_retriable`). No drift into `Result`/`Option` returns where it would be redundant.
- ✅ **CHANGELOG entries are in neutral project voice.** The cleanup pass is described under "Cleanup"; the literal-version release-prep reminder is under "Documentation". Both lines stand on their own without referencing any agent.

**Polish (🟡 — non-blocking):**

- 🟡 **Two adjacent `pub use subscription::…;` blocks** at `src/lib.rs:145-149` after the shim removal. Functionally identical to one merged block; the split is purely stylistic. A future cleanup brief may consolidate them, but doing it here would have been gold-plating.
- 🟡 **Codex log notes a brief-text issue.** The brief's example grep `grep -rn "_connection_id\b\|_connected\b\|_session_timeout\b" src/` matches `o_to_t_connection_id` / `t_to_o_connection_id` because the word boundary before `_connection_id` is satisfied by the underscore itself. Codex used a tighter `\b_(connection_id|connected|session_timeout)\b` instead and surfaced the false-positive transparently. Brief-error owned by Claude — note for future briefs: when listing audit greps, give the unambiguous form.

**No 🟠 concerns.**

**Acceptance criteria tally:**

- [x] `BatchError` derives `thiserror::Error`; manual `Display` and `Error` impls deleted; per-variant `#[error("…")]` strings preserve the prior format byte-for-byte.
- [x] `Cargo.toml` no longer lists `async-trait` as a direct dependency. `cargo tree -i async-trait` returns no packages from the active root graph.
- [x] `src/lib.rs:5` and `src/lib.rs:48` both read `0.8.0`. CHANGELOG entry under "Documentation" reminds future release-prep that the literal version string lives in the head doc.
- [x] `src/tag_subscription.rs` is deleted; `pub mod tag_subscription;` is gone from `src/lib.rs:127`; the `RealTimeSubscription*` aliases at the crate root are unchanged in name and re-export from `subscription` directly.
- [x] `_connection_id`, `_connected`, `_session_timeout` are removed from the struct, both constructors, and the `Debug` impl. `AtomicBool` import is removed alongside.
- [x] All seven `#[must_use]` annotations are present at the listed sites; clippy is clean under both feature configurations.
- [x] `ForceRuntimeInitErrorGuard` exists in `src/ffi.rs::tests`; `forced_runtime_init_error_returns_documented_code` uses it and still passes.
- [x] `ffi_block_on!` carries a calling-context doc comment.
- [x] CHANGELOG entries describe the contained changes in neutral voice; no SemVer-relevant claims.

**Brief errors owned by Claude:**

- The brief's audit grep pattern (`_connection_id\b`) matched unrelated `*_connection_id` fields because the underscore is itself a word boundary. Future audit greps should anchor more tightly.
- The brief said "let `cargo build` regenerate `Cargo.lock`" — cargo's actual behavior is more conservative; orphan entries persist unless `cargo update` is invoked. In this case the entry is not even an orphan (transitive via `cargo-tarpaulin` on Linux), so the lockfile is correct, but the brief overpromised cargo-build behavior.

## Verdict

**Merged** at `9aca8d2` — `api: contained cleanup pass — thiserror, dead deps, dead state, must_use`.

The implementation is faithful to the brief on every acceptance criterion. Format strings on `BatchError` are byte-identical to the prior manual implementation, the shim removal preserves every crate-root name, the dead-state cascade caught the orphan `AtomicBool` import, and the RAII guard around `FORCE_RUNTIME_INIT_ERROR` is panic-safe. Symbol parity with the post-CODEX-A baseline is preserved at 56 `eip_` exports under `--features ffi`.

The two 🟡 polish notes (adjacent `pub use` blocks, audit-grep word-boundary pitfall) are non-blocking and partly Claude-owned (the second is a brief-text issue, not a Codex defect).

CODEX-C (next candidate brief: `lib.rs` decomposition into module boundaries — `client`, `batch`, `route`, `protocol/`) now has a clean ground state to land on, with no pending API cleanup blocking the move.
