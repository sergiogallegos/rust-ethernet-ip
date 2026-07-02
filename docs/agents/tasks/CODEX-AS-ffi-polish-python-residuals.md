---
id: CODEX-AS
title: FFI polish — private raw-pointer exports, unwind guard, SAFETY discipline, last-error lifecycle; Python residuals
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
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

## Claude review

## Verdict
