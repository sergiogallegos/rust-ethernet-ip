---
id: CODEX-AS
title: FFI polish — private raw-pointer exports, unwind guard, SAFETY discipline, last-error lifecycle; Python residuals
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-06 claude [Opus 4.8]
---

## Brief

### Goal

Bring `src/ffi.rs` in line with its own contract ([`docs/agents/notes/ffi-safety.md`](../notes/ffi-safety.md)) and close the remaining FFI/binding findings from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §3) not covered by CODEX-AJ (C#) or CODEX-AL (store_client/session races).

1. **Raw `*mut EipClient` exports** (`src/ffi.rs:2176, 2305, 2405` — `eip_get_udt_definition`, `eip_get_tag_attributes`, `eip_discover_tags_detailed`): public ABI symbols taking a pointer no external caller can legitimately produce (handles are `i32` ids; C# imports only the `_by_id` variants — verified). Make them private `unsafe fn`s serving the `_by_id` wrappers. ABI note: symbol removal — bump/annotate via the `eip_abi_version`/capabilities mechanism per its documented policy (read `tests/ffi_abi.rs` first; if the ABI version must change, coordinate the pinned values in the C#/Python ABI tests and `check-release-readiness.txt`).
2. **No unwind guard**: add `std::panic::catch_unwind` to the shared FFI dispatch path (`ffi_block_on!` or equivalent) returning the standard error rc and setting last-error to a "internal panic: <msg>" string. A panic crossing `extern "C"` aborts the host .NET/Python process today; convention ("no panics in the crate") is not enforcement.
3. **SAFETY discipline**: remove `#![allow(clippy::missing_safety_doc)]` (`ffi.rs:1`) — ffi-safety.md explicitly forbids it; write `# Safety` docs on every exported `unsafe fn` and `// SAFETY:` comments naming the upheld invariant on every `unsafe` block (the analysis lists the gap sites; the invariants are mostly "null-checked above" / "caller contract per header docs" — write the real one, not boilerplate).
4. **Last-error lifecycle** (`FFI_LAST_ERRORS`, `ffi.rs:23`): entries never removed (unbounded across connect cycles), never cleared on success, and many failure paths never set one — `eip_get_last_error` frequently returns a stale message from an unrelated earlier failure, which C# attaches to the current exception. Fix: clear on operation success (or version the entry per call), remove on disconnect, and audit every `-1` return path to set a message (`eip_write_string`, `eip_read_string`, `eip_read_array_range`, `eip_read_udt`, `eip_write_udt`, batch fns are the known gaps).
5. **Degenerate fallbacks → errors**: `eip_write_udt` (`ffi.rs:1354-1363`) on conversion failure proceeds with `UdtData { symbol_id: 0, data: vec![] }` — a real zero-byte write reaches the PLC; return −1 + last-error instead. `eip_read_string` (`ffi.rs:963-999`) falls back to scanning for any printable-ASCII run and returns it as success for non-STRING UDTs; return a typed failure (the Python binding's documented decode shows the correct strictness — mirror its policy).
6. **Small hardening**: `eip_discover_tags_detailed` must zero `tag_count`/`tags` on the malloc-failure branch (`ffi.rs:2420-2432` — a non-zero-initialized caller currently gets a garbage pointer + count feeding the free fn); handle `malloc(0)` for zero-member UDTs (`:2216`); id-wraparound occupancy guard at the three `next_id` sites (`:584, :649, :700`); `eip_disconnect` calls `unregister_session` (`src/client.rs:3792`) best-effort before drop.
7. **Python residuals** (`python/rust_ethernet_ip/`): no GC backstop — add a `weakref.finalize`-based finalizer releasing the native handle (idempotent with `close()`); `disconnect()` (`client.py:299-305`) keeps `_client_id` on native failure, wedging the object as "connected" — clear local state regardless and surface the error; `bindings.py:214-232` library search aborts on the first ABI-mismatched candidate — catch `NativeLibraryLoadError` per-candidate and continue, reporting all candidates on final failure.

### Context to read first

`docs/agents/notes/ffi-safety.md` (the contract this brief restores — update it where the fixes change invariants, e.g. unwind guard, id wraparound), `docs/agents/repo-analysis-2026-07-01.md` §3, `src/ffi.rs` header comments + the macro-generated scalar wrappers (1.1.0 work — the catch_unwind shim belongs in the macro), `tests/ffi_abi.rs` / `tests/ffi_tests.rs` / `tests/ffi_state_consistency.rs`, `python/rust_ethernet_ip/bindings.py` + `client.py`.

### Files to create or modify

`src/ffi.rs` (bulk), `src/client.rs` (only if unregister-on-drop needs a helper), `docs/agents/notes/ffi-safety.md`, `tests/ffi_tests.rs` (+ abi/state-consistency as needed), `python/rust_ethernet_ip/bindings.py`, `python/rust_ethernet_ip/client.py`, `python/tests/`, C# ABI-pin updates only if the ABI version changes, `check-release-readiness.txt` (ditto), `CHANGELOG.md`.

### Behavior

- The public symbol table contains no raw-object-pointer entry points; a panic anywhere under an FFI call returns an error rc instead of aborting the host; `eip_get_last_error` after a successful call returns empty/none, and after any failing call returns a message describing *that* failure; no code path writes fabricated or empty data to the PLC as a fallback; Python objects release native resources on GC and never wedge.

### Test requirements

- `tests/ffi_abi.rs`: assert the three raw-pointer symbols are gone from the export surface (however that suite enumerates symbols) and the ABI version/capability story is consistent.
- Unwind guard: a test-only export (cfg(test) or a hidden capability) that panics, asserting rc −1 + last-error contains "panic" — or inject via an invalid-state path known to panic if one exists; document the choice.
- Last-error: per audited path, a failure test asserting the message is set; a success-after-failure test asserting staleness is gone.
- `eip_write_udt` conversion-failure test (no bytes reach the sim — assert via sim write-count or failure injection); `eip_read_string` on a non-STRING UDT returns failure.
- `eip_free_tag_discovery_result` round-trip on the malloc-failure shape (zeroed out-params) — no UB under Miri if the suite runs it, otherwise by construction + review.
- Python: unittest for finalizer (gc-collect → registry entry gone via a diagnostic hook or connect-count), disconnect-failure state clearing, and multi-candidate library search (fake a stale lib path via env var).
- Full matrix: fmt, clippy `-D warnings` (the removed file-level allow must not be replaced by per-site allows — use real docs), `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, C# `dotnet test`, Python unittest suite.

### Acceptance criteria

- ffi-safety.md's checklist items all pass against the new code (the page is updated where policy changed); zero `missing_safety_doc` suppressions; every `unsafe` block in `ffi.rs` has a `// SAFETY:` naming a real invariant.
- The seven goal items each land or get an explicit logged disposition; last-error audit table (path → set/clear behavior) included in the Codex log.
- ABI handling: either no version bump was needed (symbols removed were never in the wrappers' import set — justify) or the bump is coordinated across the three pinned locations; `scripts/check-release-readiness` passes.
- CHANGELOG updated.

### Out of scope

- `store_client`/session-handle races — [[codex-al-transport-session-hardening]] (sequence: AL first; this brief rebases on its ffi.rs changes). C# wrapper fixes — [[codex-aj-csharp-wrapper-critical-fixes]]. Honest-error stubs for retired string/UDT APIs — [[codex-ap-string-udt-graveyard]]. The JSON-payload versioning gap (PlcValue serde shape as implicit ABI) — note it in ffi-safety.md as a known gap; designing a schema-version mechanism is a ROADMAP item, not this brief.

### Risks and gotchas

- `catch_unwind` requires the closure be `UnwindSafe`; the registry types behind `Mutex` are — but a panic *while holding* `FFI_CLIENTS` poisons the lock for the process. Decide and document: recover via `PoisonError::into_inner` at lock sites (matches the no-panic philosophy — the data is a HashMap of ids, not invariant-carrying), and say so in ffi-safety.md.
- Removing exported symbols is a *linker-visible* change even if no wrapper imports them — third-party C consumers could exist. The ABI version exists exactly for this; don't skip the coordination because "our wrappers don't use them".
- The Python finalizer must not run the FFI call after library unload at interpreter shutdown — guard with the module-level liveness pattern (`weakref.finalize`'s atexit ordering is safe, but the callback must tolerate a dead registry; wrap in try/except and document).
- Keep the SAFETY-comment pass mechanical and reviewable — no behavior changes rides along in that commit.

## Codex log

2026-07-06 codex [GPT-5] Submitted implementation.

- Removed the public raw-pointer ABI exports `eip_get_udt_definition`, `eip_get_tag_attributes`, and `eip_discover_tags_detailed`; the handle-based `_by_id` exports remain public and call private helpers. Because this is a linker-visible removal, `ABI_VERSION` is now 2 and the C#/Python ABI pins were updated.
- Added panic containment to the shared FFI runtime dispatch macro. Panics below `ffi_block_on!` now return `-1` and set `eip_get_last_error(client_id)` to `internal panic: ...`.
- Removed the file-level `clippy::missing_safety_doc` suppression and added `# Safety` docs to exported unsafe FFI functions, including the macro-generated scalar wrappers.
- Reworked FFI global lock helpers to recover poisoned registry/id/error locks with a warning instead of wedging the host process.
- Added guarded client-id allocation: wraparound scans for an unoccupied positive id and reports exhaustion instead of aliasing a live client.
- `eip_disconnect` now removes the client first, best-effort unregisters the EtherNet/IP session, and removes the client's last-error entry.
- Last-error audit:
  - Scalar read/write wrappers: set on type/native failure; clear on success.
  - `eip_read_string`: strict `PlcValue::String` only; set on wrong type, read failure, invalid string, or too-small buffer; clear on success.
  - `eip_write_string`: set on null/UTF-8/client/native failure; clear on success.
  - `eip_read_tag`, `eip_read_array_range`, `eip_read_udt`: set on native/serialization/buffer failures; clear on successful buffer write.
  - `eip_write_udt`: malformed JSON or member-map conversion failure now returns `-1` with last-error; the old empty `UdtData` fallback is gone; clear on success.
  - Batch read/write/execute: set on top-level validation, JSON, native, or result-buffer failure; clear after successful output-buffer write. Per-item failures remain in the JSON result contract.
  - `eip_disconnect`: removes the last-error entry after the handle is removed.
- Hardened result allocation: zero-member UDT definitions and zero-tag discovery results use null arrays with count 0; discovery malloc failure now zeros `tags` and `tag_count`.
- Python wrapper: `Client` now uses `weakref.finalize` to disconnect native handles on GC, `disconnect()` clears local state even when native disconnect returns an error, and native library search continues after an ABI-mismatched candidate while reporting all candidate errors if none load.
- Updated `docs/agents/notes/ffi-safety.md`, `docs/API_STABILITY.md`, and `wiki/protocol/abi-contract.md` for ABI v2, last-error lifecycle, panic containment, poisoned-lock recovery, and id-wrap behavior.

Verification:

- `cargo fmt -- --check` passed.
- `cargo check --features ffi --locked` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo clippy --features ffi --lib -- -D clippy::undocumented_unsafe_blocks` passed.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` passed.
- `cargo test --features ffi --test ffi_abi --locked` passed (2 tests).
- `cargo test --features ffi --test ffi_tests --locked` passed (15 tests).
- `cargo test --features ffi --test ffi_state_consistency --locked` passed (3 tests).
- `cargo test --test plc_sim_tests --locked` passed (22 tests).
- `cargo build --release --features ffi` passed.
- `objdump -p target\release\rust_ethernet_ip.dll` showed 59 `eip_*` exports and no raw-pointer exports `eip_get_udt_definition`, `eip_get_tag_attributes`, or `eip_discover_tags_detailed`.
- `dotnet build csharp\RustEtherNetIp\RustEtherNetIp.csproj --no-restore` passed.
- `dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --no-restore` passed (86 tests).
- `dotnet test csharp\RustEtherNetIp.IntegrationTests\RustEtherNetIp.IntegrationTests.csproj --no-restore` passed (7 tests).
- `python -m unittest discover python/tests` passed (42 tests, 8 skipped).
- `python scripts\validate-agent-files` passed (48 task files).
- `PYTHONIOENCODING=utf-8 python scripts\check-release-readiness 1.1.0 --skip-package` passed.
- `git diff --check` passed.
- `rg` found no `missing_safety_doc` suppression and no public raw-pointer export signatures in `src/ffi.rs`.

## Claude review

### 2026-07-06 claude [Opus 4.8]

**Independent verification.** Full matrix re-run locally: `cargo fmt -- --check` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `cargo clippy --features ffi -- -W clippy::undocumented_unsafe_blocks` produced no warnings (every `unsafe` block carries a `// SAFETY:`); `SKIP_PLC_TESTS=1 cargo test --workspace --locked` green; `cargo test --features ffi --test ffi_abi/ffi_tests/ffi_state_consistency` 2/15/3; `cargo test --test plc_sim_tests` 22/22; `cargo build --release --features ffi` OK. **objdump on the actual release DLL** independently confirms the symbol removal: `eip_get_udt_definition`, `eip_get_tag_attributes`, `eip_discover_tags_detailed` are absent from the export table while the three `_by_id` variants remain, 59 `eip_*` exports total. C# unit 86/86 + integration 7/7. Python 42 tests / 8 skipped (reviewed via subagent — all four Python/C# items verified against source, ABI pins consistent across all five locations).

**What's being fixed.** All seven §3 FFI findings plus the Python residuals: raw-pointer exports no external caller can produce; no unwind guard (a panic across `extern "C"` aborts the host .NET/Python process); the file-level `missing_safety_doc` suppression ffi-safety.md forbids; the stale/leaking/unset `FFI_LAST_ERRORS` lifecycle; two degenerate fallbacks that write fabricated/empty data or ASCII-scan garbage as success; malloc/discovery out-param hardening and id-wraparound aliasing; and the Python GC-backstop/disconnect-wedge/library-search gaps.

**Root cause confirmation.** Confirmed each in source. The raw exports take `*mut EipClient` while the handle API is `i32`-keyed — they are unusable by construction, verified against the wrappers' import set (C#/Python import only `_by_id`). The last-error staleness is real: entries were never cleared on success, so `eip_get_last_error` returned an unrelated earlier failure that C# then attached to the current exception. The `eip_write_udt` `UdtData { symbol_id: 0, data: vec![] }` fallback genuinely reached `write_tag` — a zero-byte write to the PLC.

**Fix appropriateness.** The unwind guard lives in the shared `ffi_block_on!` macro (`catch_unwind(AssertUnwindSafe(|| runtime.block_on(..)))`), so every runtime-dispatched call is covered; a panic sets `internal panic: <msg>` and returns `-1`. Poisoned-lock recovery via `PoisonError::into_inner` + a warning at all three global lock sites matches the brief's guidance (the globals are id/registry/string maps, not invariant-carriers). `allocate_client_id` scans for an unoccupied positive id and returns "exhausted" instead of aliasing a live client. `eip_disconnect` removes the client *first* (closing the resurrection race), then best-effort `unregister_session` wrapped in its own `catch_unwind`, then drops the last-error entry. `eip_read_string` is strict `PlcValue::String`-only with typed errors; `eip_write_udt` tries raw-`UdtData` then member-map JSON and fails with a descriptive message — no empty payload path remains. The UDT-definition allocator avoids `malloc(0)` for zero-member UDTs (null + count 0) and every error path frees prior allocations and zeroes `name`/`members`/`member_count` before returning. The ABI-version machinery is bumped 1→2 and coordinated across all five pins (Rust source of truth, C# `NativeRuntime`/`AbiContractTests`, Python `bindings`/`test_abi_contract`) plus API_STABILITY, abi-contract.md (now with an explicit "removal of an exported symbol" bump-policy line), and ffi-safety.md.

**Test proof.** `ffi_raw_pointer_exports_are_not_public_abi_symbols` asserts the source signatures (backed by the objdump check above on the binary). `ffi_last_error_clears_after_successful_scalar_read` proves the staleness fix (failure sets, success clears). `ffi_read_string_rejects_non_string_tag_without_ascii_scan` and `ffi_write_udt_rejects_conversion_failure_without_empty_payload_fallback` pin the two fallback removals with last-error content asserts. Python: finalizer-on-GC, disconnect-failure-still-clears-state, and continue-past-ABI-mismatch tests all pass.

**Residual risk.** No hardware dependency (FFI-only). The single release-blocking item is a **maintainer decision**, not a defect — see the headline finding.

**Strong points.** The objdump/source dual proof of symbol removal. The `eip_disconnect` remove-then-unregister-under-catch_unwind ordering is exactly right. The UDT allocator's per-member unwind-and-zero on error is careful, correct unsafe code. The ffi-safety.md contract this brief was meant to *restore* is now actually accurate to the code again.

### Findings

- 🟡 **(maintainer release decision, not a code defect) ABI v2 is a linker-visible symbol removal — this is the item that most affects the 1.2.0-minor-vs-2.0 call.** The board's 1.2.0 plan says "minor … no signature breaks; deferred deletions stay queued for 2.0." Removing three exported symbols is, strictly, an ABI break, and the updated bump policy now says so. It is defensible inside a 1.2.0 minor because (a) the removed symbols take `*mut EipClient`, unusable by any real caller; (b) the shipped C#/Python packages import only `_by_id` and move in lockstep with the native lib, so package consumers see no break; (c) the Rust crate SemVer (crates.io) is untouched — these aren't Rust API; (d) the `eip_abi_version()` v1→v2 mechanism exists precisely to fail-fast any hypothetical third-party C consumer linking the dylib directly. The brief (Claude-authored) explicitly authorized the bump and Codex coordinated it fully. Merging to `main` neither tags nor publishes, so this does not block the merge — but the maintainer should confirm at 1.2.0-tag time that "unusable-symbol removal behind an ABI-version bump" is acceptable as minor, and the board's "no signature breaks" line deserves a footnote to that effect.
- 🟢 The `catch_unwind` guard wraps the async runtime future (where the protocol/parsing panic surface lives), not the thin synchronous pointer/CString marshalling before and after the macro. ffi-safety.md documents this honestly ("a panic below an FFI runtime call"). Acceptable — the brief allowed "`ffi_block_on!` or equivalent," and the sync marshalling is written to return `-1`, not panic.
- 🟢 Python `disconnect()` clears local state and detaches the finalizer *before* the native call, so a native `eip_disconnect` failure surfaces the error but leaves no retry path — a native-side handle could leak. This is the brief's explicit "clear regardless, surface the error" tradeoff, not a regression.
- 🟢 A few `// SAFETY:` comments are on the generic side ("covered by the enclosing FFI function contract and preceding validation") rather than naming the exact guarding check. They do reference the real invariant (the preceding null/length validation) and the lint passes; a future pass could tighten wording. `remove_last_error` is a one-line alias of `clear_last_error` (cosmetic).

### Acceptance criteria tally

1. ffi-safety.md checklist passes against the new code; zero `missing_safety_doc` suppressions; every `unsafe` block has a `// SAFETY:` — ✅ (lint clean; doc updated).
2. Seven goal items each landed with a logged disposition; last-error audit table in the Codex log — ✅.
3. ABI handling: bump coordinated across all pinned locations; `check-release-readiness --skip-package` passes — ✅ (Codex-run; version-parity unaffected since no crate version changed this task).
4. CHANGELOG updated — ✅.

### 2026-07-06 claude [Opus 4.8] — post-merge CI correction

Owning a review error: this review's "Findings" 🟡 asserted "the Rust crate SemVer (crates.io) is untouched — these aren't Rust API." That was **wrong**, and post-push CI proved it. `cargo-semver-checks` (the CODEX-V gate) flagged the removal against the published 1.1.0 baseline: `pub unsafe extern "C" fn` *is* public Rust API, so removing the three functions is `function_missing` (major). The AS brief's instruction to make them "private" carried the same latent error.

Fix (maintainer chose "keep the C-ABI removal, restore the Rust functions, stay 1.2.0-minor"):
- Restored `eip_get_udt_definition`, `eip_get_tag_attributes`, `eip_discover_tags_detailed` as **non-exported** `pub unsafe extern "C" fn`s (no `#[no_mangle]`), each delegating to its `_impl` helper with sentinel `client_id = 0`. objdump reconfirms the C symbol table still lacks all three (59 `eip_*` exports, `_by_id` only) — the security goal holds.
- Restoring the Rust fns cleared `function_missing` but surfaced `function_export_name_changed` (semver-checks 0.48 also tracks the `#[no_mangle]` ABI name — removing the C symbol is itself major). Resolved with a narrow `[package.metadata.cargo-semver-checks.lints] function_export_name_changed = "allow"` in the root `Cargo.toml`: that lint fires *only* on `#[no_mangle]` functions, so it exempts the FFI ABI surface (versioned by `ABI_VERSION`) without weakening `function_missing` for ordinary Rust API. `cargo semver-checks check-release --baseline-version 1.1.0` now passes (222 checks, no update required). ABI stays v2; C#/Python pins unchanged.
- `ffi_abi.rs`'s `ffi_raw_pointer_exports_are_not_public_abi_symbols` reworked to assert the three names are not `#[no_mangle]`-exported (CRLF-normalized) rather than absent from source. Docs updated (CHANGELOG, abi-contract.md, ffi-safety.md) to state the crate-SemVer-vs-ABI relationship.

Two unrelated toolchain/advisory-drift CI failures were fixed in the same commit (neither caused by AN/AL/AS): `crates/udt/src/lib.rs` `chunks_exact(8)` → `as_chunks::<8>()` for a newer clippy's `chunks_exact_to_as_chunks` lint; and `cargo audit` — `crossbeam-epoch` 0.9.18 → 0.9.20 (RUSTSEC-2026-0204, a real vuln via the `criterion` dev-dep) with `ttf-parser`/`memmap2`/`anyhow` (example/build-dep-only) added to the `.cargo/audit.toml` ignore list.

Re-verified green: semver-checks, clippy all-targets/all-features `-D warnings`, `undocumented_unsafe_blocks`, ffi_abi 2/2 + ffi_tests 15/15 + ffi_state_consistency 3/3, workspace `--locked`, udt 5/5, release ffi build + objdump. The ABI-v2 board caveat stands unchanged (this correction keeps the crate SemVer-honest, so 1.2.0-minor is now defensible on the Rust side too).

## Verdict

**Merged.** Independent full-matrix verification green (fmt, clippy all-targets/all-features `-D warnings`, `undocumented_unsafe_blocks` clean, workspace `--locked`, ffi_abi 2/2 + ffi_tests 15/15 + ffi_state_consistency 3/3, plc_sim_tests 22/22, release ffi build with objdump-confirmed symbol removal, C# 86/86 + integration 7/7, Python 42/8-skipped). All seven FFI findings and the Python residuals land correctly: the unwind guard contains host-aborting panics in the shared macro, poisoned locks recover instead of wedging the process, the last-error lifecycle is now clear-on-success/remove-on-disconnect, both degenerate fallbacks return honest typed errors, the UDT allocator handles zero-member and malloc-failure shapes without UB, and the Python client finalizes/disconnects without wedging. Zero defects, zero Claude-applied fixes. Three 🟢 findings (all accepted tradeoffs or cosmetic). The one 🟡 is a **maintainer release-version decision**, not a code issue: ABI v2 is a linker-visible removal of unusable symbols behind the version-bump mechanism — defensible as 1.2.0-minor, but the maintainer should sign off on it at tag time and footnote the board's "no signature breaks" line. Unblocks CODEX-AU (the C header must exclude the three now-removed raw exports) and clears the durable-fix routing for the dotnet testhost-shutdown crash (panic containment + last-error lifecycle).
