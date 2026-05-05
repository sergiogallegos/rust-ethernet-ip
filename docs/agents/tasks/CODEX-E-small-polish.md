---
id: CODEX-E
title: Small polish — runtime-init log dedupe, regex caching, re-export merge, dev-dep audit
owner: codex
status: open
created: 2026-05-05
last-update: 2026-05-05 claude
---

## Brief

### Goal

A small basket of independent polish items collected through CODEX-A and CODEX-B reviews. Every item is mechanical, narrow, and independently revertable. Land them as one submission.

This brief is independent of CODEX-C and CODEX-D — the items touch `src/ffi.rs`, `src/lib.rs`, `src/tag_manager.rs`, and `Cargo.toml` / CI. None of those locations conflict with the lib.rs decomposition or the codec extraction in a way that would force ordering. If CODEX-C or CODEX-D are in flight, prefer to land this brief first (it is the smallest) so the bigger briefs rebase against a clean state.

### Context to read first

- `docs/agents/README.md` — protocol, voice, lifecycle.
- `docs/agents/tasks/CODEX-A-ffi-runtime-lint-safety.md` — Claude review polish items #3 (runtime-init log dedupe).
- `docs/agents/tasks/CODEX-B-contained-api-cleanup.md` — Claude review polish item #1 (adjacent `pub use subscription::…;` blocks).
- `src/ffi.rs:31-37` — current `runtime_init_error_code` helper logs unconditionally on every failure.
- `src/lib.rs:145-149` — two adjacent `pub use subscription::…;` blocks introduced by CODEX-B's shim removal.
- `src/tag_manager.rs:473` — inline `regex::Regex::new(...).unwrap()` runs on every call to whichever method holds it.
- `Cargo.toml:65` — `cargo-tarpaulin = "0.27"` listed in `[dev-dependencies]`. Verify whether anything actually uses it as a dev-dependency vs. a CI-installed binary.
- `.github/workflows/ci.yml:58-63` — the tarpaulin step uses `actions-rs/cargo@v1` with `command: tarpaulin`. The action installs the binary independently; the dev-dep entry only inflates the lock graph.

### Behavior

Four contained changes. Land in one submission.

**1. Deduplicate the runtime-init failure log.**

`src/ffi.rs:31-37`:

```rust
fn runtime_init_error_code(error: &std::io::Error) -> c_int {
    tracing::error!("[FFI] Failed to initialize Tokio runtime: {}", error);
    EIP_ERROR_RUNTIME_INIT
}
```

After a failed `Runtime::new()`, every FFI call hits this path and emits a duplicate `tracing::error!` line. Wrap the log emission in a `std::sync::Once`:

```rust
static RUNTIME_INIT_LOG: std::sync::Once = std::sync::Once::new();

fn runtime_init_error_code(error: &std::io::Error) -> c_int {
    RUNTIME_INIT_LOG.call_once(|| {
        tracing::error!("[FFI] Failed to initialize Tokio runtime: {}", error);
    });
    EIP_ERROR_RUNTIME_INIT
}
```

The first failed call emits the diagnostic; subsequent calls return the error code silently. This matches the operational reality (a single host-level failure manifests as N FFI errors but only one underlying cause). Note: the `error` argument is consumed by the closure; if the borrow checker complains, capture `error.to_string()` outside the `call_once` and move the `String` in.

**2. Merge the adjacent `pub use subscription::…;` blocks.**

`src/lib.rs` after CODEX-B has:

```rust
pub use subscription::{SubscriptionManager, SubscriptionOptions, TagSubscription};
pub use subscription::{
    SubscriptionManager as RealTimeSubscriptionManager,
    SubscriptionOptions as RealTimeSubscriptionOptions,
    TagSubscription as RealTimeSubscription,
};
```

Merge into one block:

```rust
pub use subscription::{
    SubscriptionManager,
    SubscriptionManager as RealTimeSubscriptionManager,
    SubscriptionOptions,
    SubscriptionOptions as RealTimeSubscriptionOptions,
    TagSubscription,
    TagSubscription as RealTimeSubscription,
};
```

(Or the equivalent in a single line if rustfmt prefers; let `cargo fmt` decide the final layout.)

**3. Lift the tag-name regex to a `LazyLock`.**

`src/tag_manager.rs:473` currently constructs the regex on every call:

```rust
regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9]*(?:[._][a-zA-Z0-9]+)*$").unwrap();
```

Replace with a module-level `LazyLock`:

```rust
use std::sync::LazyLock;

static TAG_NAME_RE: LazyLock<regex::Regex> = LazyLock::new(|| {
    regex::Regex::new(r"^[a-zA-Z][a-zA-Z0-9]*(?:[._][a-zA-Z0-9]+)*$")
        .expect("tag name regex pattern is a valid literal")
});
```

Replace the call site with `TAG_NAME_RE.is_match(...)` (or whatever method the current code calls). The `expect` message replaces the bare `unwrap` and gives an actionable diagnostic if the literal pattern is ever malformed (it won't be — it's a constant string — but the message documents the assumption).

**4. Drop the `cargo-tarpaulin` dev-dependency.**

`Cargo.toml:65` lists `cargo-tarpaulin = "0.27"` in `[dev-dependencies]`. `cargo-tarpaulin` is a binary tool, not a library; nothing in `src/` or `tests/` does `use cargo_tarpaulin`. The CI workflow at `.github/workflows/ci.yml:58-63` invokes it via `actions-rs/cargo@v1` which installs the binary independently and does not consult `Cargo.toml`.

**Verify before deleting:**

```
grep -rn "cargo_tarpaulin\|cargo-tarpaulin" src/ tests/ benches/ examples/
```

Should return zero matches inside the Rust code. If grep finds a matching `use cargo_tarpaulin;` or similar, **stop and ask** in `## Codex log` — the assumption is wrong.

If the grep is clean, delete the `cargo-tarpaulin = "0.27"` line. Then run `cargo update --workspace` and commit the resulting `Cargo.lock` shrinkage (the lockfile loses tarpaulin and its transitive closure: `zbus`, `xml-rs`, `procfs`, etc., on the order of 30-40 entries removed).

After the delete, verify CI still works by reading `.github/workflows/ci.yml:58-63`. The `actions-rs/cargo@v1` action with `command: tarpaulin` installs the binary on demand — nothing in the workflow references `Cargo.toml` for tarpaulin. CI behavior should not change.

### Test requirements

- `cargo fmt -- --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass.
- `cargo clippy --no-default-features --lib -- -D warnings` — must pass.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass.
- `cargo test --test plc_sim_tests` — must pass.
- `cd csharp/RustEtherNetIp && dotnet build && cd ../RustEtherNetIp.Tests && dotnet test` — must pass.
- `cargo build --release --features ffi` — produces a cdylib with exactly `56` `_eip_` exports.
- New unit test in `src/ffi.rs::tests`: exercise the runtime-init failure path twice in sequence (two calls to the forced-error path) and assert via a tracing-capture helper that only one error log line was emitted. If the tracing-capture mechanism is not already available in the test harness, document the gap in `## Codex log` and rely on visual inspection of the log output instead — the dedupe correctness is observable but not necessarily auto-verifiable.
- New unit test in `src/tag_manager.rs::tests` (or wherever the regex is exercised): verify the regex is the same pattern by exercising it on a known-good tag name and a known-bad tag name. If a test like this already exists, no new test is needed.

### Acceptance criteria

- [ ] `RUNTIME_INIT_LOG: Once` exists in `src/ffi.rs` and `runtime_init_error_code` emits at most one log line per process lifetime.
- [ ] `src/lib.rs` has one merged `pub use subscription::…;` block; `cargo fmt` is clean.
- [ ] `TAG_NAME_RE: LazyLock<Regex>` exists at module scope in `src/tag_manager.rs`; the call site uses the cached regex.
- [ ] `Cargo.toml` no longer lists `cargo-tarpaulin` in `[dev-dependencies]`; `Cargo.lock` reflects the shrinkage; CI's `tarpaulin` step still runs (verified by reading the workflow file, not by triggering CI in this brief).
- [ ] FFI symbol parity preserved: 56 `_eip_` exports.
- [ ] CHANGELOG entry under "Cleanup" describing the four polish items in neutral voice; no SemVer-relevant claims.

### Out of scope

- Converting `get_tag_metadata` to return `Result<Option<TagMetadata>>` (CODEX-A polish #4). That is a public API change and belongs in a SemVer-major release brief.
- Any change to `try_init_tracing`'s `Box<dyn Error>` signature (also a public API change).
- `#[non_exhaustive]` on any enum.
- Adding new lints (`missing_docs`, `missing_debug_implementations`).
- Replacing `regex` with `regex-lite` or any other dependency change.
- Replacing `Once` with `OnceLock` for the log dedupe — `Once` is the right primitive when there is no value to cache; `OnceLock` would be over-engineered.
- The `ffi_block_on!` early-return contract doc was added in CODEX-B; do not touch it.
- Workflow modernization (`actions-rs/cargo@v1` is deprecated; that's a separate brief).

### Risks and gotchas

- **`Once::call_once` panics propagate.** If the closure panics, subsequent `call_once` invocations panic too. The closure here is a `tracing::error!` macro call, which does not panic under normal conditions. If the tracing backend is misconfigured to a panicking subscriber, this could cascade — flag in `## Codex log` if the test suite surfaces it. Otherwise this is a non-issue.
- **Borrow lifetime in the `Once` closure.** The `error: &std::io::Error` argument is borrowed; the closure captures it. The `call_once` closure must outlive the borrow, which it does because `call_once` runs synchronously. If the borrow checker disagrees, capture `error.to_string()` before the `call_once` and move the `String` in.
- **`cargo update` lockfile diff.** Removing `cargo-tarpaulin` will drop ~30-40 entries from `Cargo.lock` (transitive closure: `zbus`, `xml-rs`, `procfs`, `nix`, `quote`, `proc-macro2`, etc., at versions specific to tarpaulin's needs). Some of those crates appear at *different* versions for other deps; only the tarpaulin-specific versions disappear. Commit the lockfile shrinkage as part of this brief.
- **CI verification is read-only.** The brief does not require triggering CI in this submission. The reviewer will verify by reading the workflow file and confirming `actions-rs/cargo@v1` with `command: tarpaulin` does not require a `Cargo.toml` entry. The maintainer can validate end-to-end on the next CI run.
- **The tracing-capture test in step 1 may not be feasible.** The standard `tracing-subscriber` testing helpers require additional setup. If the test harness in this repository does not already have a capture helper, the brief explicitly permits documenting the gap in `## Codex log` and relying on the dedupe correctness being observable but not auto-verifiable. Do not pull in a new dev-dependency just to test this one log line.

## Codex log

*(empty — codex appends entries on starting work)*

## Claude review

*(empty — claude appends after submission)*

## Verdict

*(empty — claude writes on merge or rejection)*
