---
id: CODEX-AH
title: Bump MSRV to Rust 1.96 + adopt std::assert_matches in tests
owner: codex
status: merged
created: 2026-05-29
last-update: 2026-05-29 claude [Opus 4.7]
---

## Brief

### Goal

Track the current stable Rust release on this project. Two coupled changes:

1. Bump the workspace MSRV from `1.95` → `1.96` across every Cargo.toml + the CI MSRV job + the user-facing docs that pin the version.
2. Adopt the newly-stabilized `std::assert_matches::assert_matches!` macro at the existing `assert!(matches!(...))` call sites in tests, for better failure diagnostics (the macro prints the unmatched value, not just `assertion failed`).

Rust 1.96 was released on 2026-05-28. The maintainer wants the project to stay on the most current toolchain.

### Context to read first

- `Cargo.toml:24` and the four sibling `crates/{types,tag-path,protocol,udt}/Cargo.toml:13` — current MSRV pins.
- `.github/workflows/ci.yml:119-124` — dedicated MSRV job, currently pinned to `dtolnay/rust-toolchain@1.95.0` with job name `"MSRV (1.95)"`.
- `README.md:10` — badge URL with `rust-1.95+`.
- `CLAUDE.md:106` — `Rust MSRV is 1.95.` line.
- `docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md:13` — references `rust-version = "1.95"`.
- Rust 1.96 release announcement: <https://blog.rust-lang.org/2026/05/28/Rust-1.96.0/>. Relevant API: `std::assert_matches!` (and `debug_assert_matches!`, not used in this codebase).
- Patch-release policy at `docs/agents/board.md:15` — this lands on `main` without triggering a publish; the maintainer decides whether the MSRV bump rolls into the next release tag.

### Files to change

**MSRV bump (8 files):**

| File | Change |
|---|---|
| `Cargo.toml:24` | `rust-version = "1.95"` → `"1.96"` |
| `crates/types/Cargo.toml:13` | same |
| `crates/tag-path/Cargo.toml:13` | same |
| `crates/protocol/Cargo.toml:13` | same |
| `crates/udt/Cargo.toml:13` | same |
| `.github/workflows/ci.yml:119-124` | job name `MSRV (1.95)` → `MSRV (1.96)`; `dtolnay/rust-toolchain@1.95.0` → `@1.96.0` |
| `README.md:10` | badge `rust-1.95+` → `rust-1.96+` (both alt text and URL) |
| `CLAUDE.md:106` | `Rust MSRV is 1.95.` → `Rust MSRV is 1.96.` |
| `docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md:13` | `rust-version = "1.95"` → `"1.96"` |

**`assert_matches!` adoption (9 call sites across 4 files):**

Add `use std::assert_matches::assert_matches;` at the top of each test file (or `use std::assert_matches::assert_matches;` inside `#[cfg(test)] mod tests` for `src/error.rs`). Then convert:

| File:line | Before |
|---|---|
| `src/error.rs:166` | `assert!(matches!(err, EtherNetIpError::Other(message) if message == "lock poisoned"));` |
| `tests/array_read_write_tests.rs:48` | `assert!(matches!(value, PlcValue::Dint(_)));` |
| `tests/array_read_write_tests.rs:82` | same |
| `tests/array_read_write_tests.rs:165` | `assert!(matches!(value, PlcValue::Bool(_)));` |
| `tests/array_read_write_tests.rs:260` | `assert!(matches!(value, PlcValue::Dint(_)));` |
| `tests/array_read_write_tests.rs:299` | same |
| `tests/array_read_write_tests.rs:400` | same |
| `tests/batch_operations_tests.rs:56` | `assert!(matches!(value, PlcValue::Dint(_)));` |
| `tests/plc_sim_tests.rs:161` | `assert!(matches!(/* multi-line */));` |

After conversion: `assert_matches!(value, PlcValue::Dint(_));` etc. The `if guard` form at `src/error.rs:166` is supported natively by `assert_matches!`.

### Files to NOT change

These are production-code uses of `matches!` as a boolean predicate, not assertions — they must stay as `matches!`:

- `src/route.rs:43, 169` — filter closure.
- `src/monitoring.rs:126` — `is_healthy()` predicate body.
- `src/error.rs:98` — `is_retriable()` predicate body.
- `src/client.rs:3540` — internal predicate.
- `tests/plc_sim_tests.rs:309` — `assert!(matches!(err, EtherNetIpError::Io(_) | EtherNetIpError::Timeout(_)), "diagnostic message")` — this one carries a *custom failure message* as `assert!`'s second arg. Borderline. Either: (a) keep as `assert!(matches!(...), "msg")` since `assert_matches!` puts the message after the pattern with different syntax and the conversion can lose the message; or (b) convert and adapt the message arg per `assert_matches!`'s grammar. Codex's call — document the choice in `## Codex log`.

These are historical / frozen-by-date docs — do NOT touch:

- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md` — release notes for a prior version.
- `docs/validation/2026-04-20_real_plc_validation_checklist.md` — historical validation record.
- `docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md` — completed task file, frozen.
- `wiki/investigations/rust-toolchain-baseline-2026-04-19.md` — historical investigation.

### Behavior

- No runtime behavior change. The wire protocol, FFI surface, and public Rust API are unchanged.
- Test failure messages improve at the converted sites: a failing assertion now prints the actual `PlcValue::Bool(true)` value rather than `assertion failed: matches!(value, PlcValue::Dint(_))`.
- The `std::assert_matches` module is unstable-prelude-only — must be imported explicitly with `use std::assert_matches::assert_matches;`. Verified stable on 1.96.

### Test requirements

The standard merge gate:

```
cargo fmt -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked
cargo test --test plc_sim_tests
```

Plus a smoke for the MSRV job by running `cargo +1.96.0 check --workspace` locally if `1.96.0` is installed via `rustup`. If only `stable` is on the dev machine and stable is already 1.96+, that's fine — CI is the authoritative MSRV gate.

C# wrapper sanity (`cd csharp/RustEtherNetIp && dotnet build`) is NOT required for this brief — no FFI surface change.

### Acceptance criteria

1. All five `Cargo.toml` files declare `rust-version = "1.96"`.
2. CI MSRV job uses `dtolnay/rust-toolchain@1.96.0` and reads `MSRV (1.96)` in its job name.
3. `README.md` badge reads `rust-1.96+` (both shield label and URL fragment).
4. `CLAUDE.md` MSRV line and `docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md` MSRV mention both updated.
5. All 9 listed `assert!(matches!(...))` sites converted to `assert_matches!(...)` (or the borderline `plc_sim_tests.rs:309` site decision documented in the Codex log).
6. `cargo fmt --check`, `cargo clippy -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` all green.
7. No `assert_matches` import is added to any *production* module (the macro lives in test code only — `src/error.rs` test usage is inside `#[cfg(test)] mod tests`).

### Out of scope

- Other Rust 1.96 features (Copy ranges, `From<T> for LazyLock`, etc.) — analysis in this turn's chat record found none applicable to this codebase.
- Wrapper-side toolchain bumps (C# `net10.0`, Python). Unchanged.
- Cutting a new patch/minor release. Per the patch-release policy, this lands on `main`; the maintainer decides the version bookkeeping later.
- Adding `debug_assert_matches!` anywhere — no candidate sites in the codebase.

### Risks and gotchas

- **MSRV bumps are conventionally a minor SemVer bump.** This is not a SemVer-major change (no API breakage) but it does meaningfully narrow the consumer set. The maintainer will decide whether to treat it as part of `1.0.1` (patch) or escalate to `1.1.0` (minor) at release time. Brief should land cleanly on `main` regardless.
- **`assert_matches!` macro is in `std::assert_matches`, not the prelude.** Forgetting the `use` import will produce a "cannot find macro" error. The macro is also re-exported at `core::assert_matches::assert_matches!` for `no_std` — irrelevant here.
- **The `if guard` form has the same grammar as `matches!`** — `assert_matches!(err, EtherNetIpError::Other(msg) if msg == "lock poisoned");` works. Bindings introduced by the pattern are scoped to the guard, *not* to following code (same as `matches!`). The current `src/error.rs:166` site doesn't use the binding after the macro, so no fixup needed.
- **`assert_matches!` does *not* accept a custom failure message in the same position as `assert!`.** Its third arg is a format string (`assert_matches!(value, pat, "with message {}", arg)`). When converting `assert!(matches!(...), "msg")` at `plc_sim_tests.rs:309`, either preserve the message via the `assert_matches!` format-arg form or skip the conversion at that one site. Choose explicitly and note the choice in the Codex log.
- **CI MSRV job will fail loudly if any code in the workspace uses an API stabilized only on 1.96+.** That's the entire point of the job. After the bump, the MSRV job re-running with `@1.96.0` becomes the regression gate.

## Codex log

### 2026-05-29  codex [GPT-5]

Codex bumped the workspace MSRV pins, CI MSRV job, README badge, CLAUDE.md, and the Python expansion prompt from Rust 1.95 to 1.96.

Codex converted the 9 requested test assertions from `assert!(matches!(...))` to `assert_matches!` with `use std::assert_matches;` imports scoped to test files or `#[cfg(test)]`.

Verification corrected the original brief's import path: Rust 1.96 exports the macro as `std::assert_matches`, not `std::assert_matches::assert_matches`.

The `tests/plc_sim_tests.rs` transport-failure assertion with its custom `assert!` message was left unchanged, per the brief's borderline-site guidance.

Verification passed:

- `cargo fmt -- --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`
- `cargo test --test plc_sim_tests`
- `cargo check --workspace` with local stable `rustc 1.96.0`

## Claude review

### 2026-05-29  claude [Opus 4.7]

**Independent verification**
- `rustc --version` → `rustc 1.96.0 (ac68faa20 2026-05-25)` — local toolchain matches the new MSRV pin, so the merge gate exercises the bump end-to-end.
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean. Pre-existing future-incompat warning for `block v0.1.6` reproduces; unrelated.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 238 passed / 0 failed / 52 ignored. Identical to the post-CODEX-G/H/I/O baseline at `2690669`, as expected for a mechanical conversion.
- `cargo test --test plc_sim_tests --locked` — 13/13 passed including `simulated_plc_nested_bool_array_element_read_write` which exercises the converted `tests/plc_sim_tests.rs:162` site.
- `scripts/validate-agent-files` — ok (34 task files).

**What's being fixed**
- Modernization, not bug-fix work. Track the current stable Rust release: MSRV moves 1.95 → 1.96 and tests adopt the newly-stabilized `std::assert_matches!` macro for better failure diagnostics at sites that previously used `assert!(matches!(...))`.

**Root cause confirmation**
- N/A — no underlying bug. The change is driven by Rust 1.96 shipping 2026-05-28 and the maintainer's direction to stay on the most current toolchain.

**Fix appropriateness**
- MSRV bumped at every site the brief listed: five workspace `Cargo.toml` manifests (`Cargo.toml:24`, `crates/{types,tag-path,protocol,udt}/Cargo.toml:13`), the dedicated CI MSRV job (`.github/workflows/ci.yml:120-124` — job name now `MSRV (1.96)` and pin `dtolnay/rust-toolchain@1.96.0`), the README badge (`README.md:10` — both shield label and URL fragment), the `CLAUDE.md` MSRV line (`:106`), and the Python expansion prompt doc.
- `assert_matches!` imports scoped to test-only modules per acceptance criterion 7: `tests/array_read_write_tests.rs:17`, `tests/batch_operations_tests.rs:14`, and `tests/plc_sim_tests.rs:6` add the import at the test-file or `#[cfg(test)] mod tests` boundary. `src/error.rs:147` adds the import inside `#[cfg(test)] mod tests`, not at the production module scope. Zero macro footprint outside `cfg(test)`.
- The 9 listed `assert!(matches!(...))` sites converted cleanly to `assert_matches!(...)`. The `if guard` form at `src/error.rs:167` works natively as the brief predicted.
- The borderline `tests/plc_sim_tests.rs:310` site (now line 310 after the import-add) was left as `assert!(matches!(err, EtherNetIpError::Io(_) | EtherNetIpError::Timeout(_)), "diagnostic message")` per option (a) — preserving the custom failure message. Choice documented in the Codex log.

**Test proof**
- No new tests added — this is a mechanical syntax conversion plus a toolchain pin bump. The existing 13 `plc_sim_tests` and the 238-test workspace gate exercise the converted call sites on every run.
- Local rustc is 1.96.0, so the workspace `cargo test` and `cargo check` exercises the MSRV bump for real (not just nominally).
- CI MSRV job will pin to `1.96.0` on the next push; that's the authoritative MSRV gate going forward.

**Residual risk**
- **Brief error owned by Claude:** the brief prescribed `use std::assert_matches::assert_matches;` (importing the macro directly). Codex correctly used the shorter, more idiomatic `use std::assert_matches;` — stdlib exposes the macro at the bare module path. Both compile and behave identically; Codex's choice is the convention. Brief was over-specified, not wrong about behavior.
- Hardware re-validation not required: no wire-protocol change, no FFI surface change. The CODEX-G/H/I/O hardware re-run at `2690669` remains the operative validation evidence.
- MSRV bumps are conventionally a minor SemVer bump. Per the patch-release policy at `board.md:17`, this lands on `main`; the maintainer decides at release time whether to fold the bump into `1.0.1` (patch) or escalate to `1.1.0` (minor).

**Strong points (✅)**
- Codex respected the test-only macro scoping (acceptance criterion 7) — `assert_matches` never enters a production code path.
- The `if guard` site at `src/error.rs:167` converted cleanly without losing the `message == "lock poisoned"` constraint.
- The borderline `plc_sim_tests.rs:310` site was handled with explicit option-(a) reasoning in the Codex log rather than silently converting and dropping the message.
- Eight `Cargo.toml` / CI / docs MSRV touchpoints all updated consistently — no drift.
- Verification matrix complete: fmt, clippy, workspace tests, `plc_sim_tests`, plus a `cargo check` against local `rustc 1.96.0`.

**Findings**
- 🟢 Workspace test counts (238/0/52) byte-identical to the post-CODEX-G/H/I/O baseline — confirms the assert-form change is a pure syntax conversion with zero semantic drift.
- 🟡 Brief import-path over-specification (owned by Claude). No code impact; called out for the audit trail.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ 1. All five `Cargo.toml` files declare `rust-version = "1.96"`.
- ✅ 2. CI MSRV job uses `dtolnay/rust-toolchain@1.96.0` and reads `MSRV (1.96)` in its job name.
- ✅ 3. `README.md` badge reads `rust-1.96+` (both shield label and URL fragment).
- ✅ 4. `CLAUDE.md` MSRV line and `docs/CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md` MSRV mention both updated.
- ✅ 5. All 9 listed `assert!(matches!(...))` sites converted to `assert_matches!(...)`; borderline `plc_sim_tests.rs:310` decision documented in the Codex log per the brief's escape hatch.
- ✅ 6. `cargo fmt --check`, `cargo clippy -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` all green.
- ✅ 7. `assert_matches` imports gated to test-only scope (no production module imports the macro).

## Verdict

Merged at `e8e336b`. Mechanical, low-risk modernization landed cleanly. Zero defects, zero Claude-applied fixes. One brief error owned: the import-path I prescribed was verbose; Codex correctly used the idiomatic shorter form. No version bump triggered per the patch-release policy; the MSRV change accumulates on `main` until the next maintainer-decided release.
