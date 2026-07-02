---
id: CODEX-AL
title: Transport & session hardening — timeout desync, sender-context correlation, shared session handle
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
---

## Brief

### Goal

Close the transport-layer integrity gaps from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §2 and §3). These are the highest-severity *runtime* findings in the Rust core: they can attribute PLC response N to request N+1 on an industrial write path.

1. **Stream desync after timeout** (`src/client.rs:3610-3679`, `send_rr_data_item`). On the 10 s timeout the function returns `EtherNetIpError::Timeout` but leaves the connection open with an unread response in flight; the next request's `read_exact` consumes the stale response. `is_retriable()` actively encourages retrying on the same client. Fix: a timeout (or any mid-transaction failure after the write) must poison the stream — mark the connection dead so every subsequent call fails fast with `ConnectionLost` until an explicit reconnect re-establishes a clean stream + fresh session.
2. **No response correlation** (`crates/protocol` encap header). The encapsulation `sender_context` is a constant and never checked on receive, so a desync is undetectable. Fix: stamp each request with a per-client monotonic counter in `sender_context`, verify it on the response, and treat a mismatch as a poison-the-stream protocol error (this is the backstop that turns silent misattribution into a loud failure).
3. **`session_handle` diverges across clones** (`src/client/diagnostics.rs:32-41` vs the documented invariant at `src/client.rs:234-235`). `check_health_detailed` re-registers on keep-alive failure, updating only its own clone's copied `u32` while every other clone (subscription pollers, FFI registry entries, tag-group tasks) keeps the old handle on the shared stream. Fix: move `session_handle` into the Arc-shared state (`Arc<AtomicU32>`), restoring the "one handle per stream" invariant. This also lets the FFI drop its clone→await→`store_client` re-insert pattern (`src/ffi.rs:115-119`, `:1447-1454`, `:1486`), which currently races `eip_disconnect` and can resurrect a removed client as a permanent leak.
4. **`register_session` framing** (`src/client.rs:368-405`): takes the stream lock separately for write and read (a concurrent request from another clone can interleave into the registration exchange), and reads the 28-byte reply with a single `read()` (fragmented replies fail spuriously). Fix: one lock guard across the whole transaction (match `send_rr_data_item`), `read_exact` on the 24-byte header then length-driven body read (match the framing logic in `send_rr_data_item`).

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §2, §3 (Rust FFI).
- `src/client.rs:230-256` — the per-field SHARED/COPIED clone contract. Item 3 changes one line of it; update the comments to match.
- `src/client.rs:3610-3679` — `send_rr_data_item`, the one place request/response framing is done right today. Items 1, 2, 4 all center here.
- `crates/protocol/src/encap.rs` — `EncapsulationHeader` encode/decode; where `sender_context` lives.
- `docs/agents/notes/ffi-safety.md` — the FFI invariants; the `store_client` pattern and why clones exist.
- `tests/plc_sim.rs` failure-injection hooks (drop-response timeout, mid-stream disconnect) — the test infrastructure for item 1 already exists.
- `tests/ffi_state_consistency.rs` — the registry-visibility regression tests that must keep passing after the `store_client` removal.

### Files to create or modify

- `src/client.rs` — connection-poisoned flag in the Arc-shared state (e.g. `Arc<AtomicBool>`); `send_rr_data_item` sets it on timeout/partial-transaction failure and checks it on entry; `register_session` single-guard + proper framing; `session_handle: Arc<AtomicU32>` + clone-contract comment update.
- `crates/protocol/src/encap.rs` (+ `src/client.rs` call sites) — sender_context stamping/verification. Keep the wire shape identical (8 bytes, already present); only the value/checking policy changes.
- `src/client/diagnostics.rs` — re-registration updates the shared handle; document that other in-flight users see the new handle atomically.
- `src/ffi.rs` — remove `store_client` re-insert sites now that health/diagnostics mutations are Arc-visible; `eip_disconnect` remains the only remover.
- `src/error.rs` — if a distinct variant helps (`EtherNetIpError::ResponseMismatch` or reuse `Protocol`), keep `is_retriable()` semantics coherent: a poisoned connection is NOT retriable-in-place; it requires reconnect.
- `tests/plc_sim_tests.rs` — new tests (below).
- `CHANGELOG.md`.

### Behavior

- After a timeout, the *same* client returns `ConnectionLost` (or equivalent non-retriable-in-place error) on the next call instead of reading the stale response; after an explicit reconnect (or actor/`RetryClient` reconnect policy), operation resumes cleanly.
- A response whose sender_context doesn't match the in-flight request never reaches a caller as data.
- Re-registration on one clone is immediately visible to all clones; the FFI registry no longer needs post-await re-insertion.
- No public API signature changes. `EipClient` remains `Clone`.

### Test requirements

Simulator-backed (all run in CI):

- `timeout_poisons_connection`: inject drop-response for tag A; `read_tag(A)` times out; assert next `read_tag(B)` on the same client fails fast with the poisoned/connection error, *not* B's data and *not* A's stale data.
- `late_response_not_misattributed`: inject a delayed (not dropped) response beyond the timeout if the sim's injection supports it — otherwise cover via the sender-context unit test: craft a response with a stale context and assert `ResponseMismatch`.
- `reregistration_visible_across_clones`: clone the client, force re-register via one clone (the sim accepts any handle today — assert on the client-side shared value), assert the other clone observes the new handle.
- `register_session_fragmented_reply`: if the sim can write the 28-byte reply in two chunks, cover it; otherwise unit-test the framing parse with a split buffer.
- FFI: `tests/ffi_state_consistency.rs` extended — connect, spawn a thread calling `eip_check_health_detailed` in a loop, concurrently `eip_disconnect`; assert the registry is empty afterwards (the resurrection-race regression test).
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, C# `dotnet build` + `dotnet test` (FFI-touching task).

### Acceptance criteria

- All four fixes implemented with the tests above green.
- `store_client` (the re-insert helper) is gone from `src/ffi.rs`; grep-clean.
- The clone-contract comment block at `src/client.rs:230-256` accurately describes the new state.
- No new `unwrap`/`panic` paths; atomics use documented orderings (Relaxed is fine for the handle and poison flag — say so in a comment).
- CHANGELOG `[Unreleased]` `### Fixed` entries. Wire format unchanged except sender_context values — call this out for the maintainer's hardware smoke (protocol-touching task ⇒ maintainer hardware validation before release per review lifecycle).

### Out of scope

- `send_connected_cip_request`'s missing timeout (`src/client/string.rs:603`) — the connected-messaging subsystem is being removed/quarantined by [[codex-ap-string-udt-graveyard]]; don't harden dead code.
- Automatic transparent reconnect inside `EipClient` — reconnect policy belongs to `RetryClient`/the actor layer; this brief only makes failure states explicit and safe.
- The C# keep-alive concurrency fix — [[codex-aj-csharp-wrapper-critical-fixes]].
- Subscription task lifecycle — [[codex-ar-subscription-fleet-lifecycle]].

### Risks and gotchas

- **Cancellation safety**: poisoning must happen even if the caller drops the future between write and read. Setting the poison flag *before* the write and clearing it only after a complete successful read ("assume poisoned unless proven clean") is the robust shape; evaluate against the actor path which may rely on sequential completion.
- Some controllers echo sender_context imperfectly on error replies — verify against the simulator first and keep the mismatch check tolerant on encap-error replies (status ≠ 0) if the maintainer's hardware smoke shows echo quirks. Flag any tolerance you add in the log.
- `Arc<AtomicU32>` for the handle changes `Clone` cost trivially but changes *semantics* deliberately — audit every `session_handle` read site (grep) so none caches the value across an await.
- The FFI `_by_id` wrappers rely on clone-then-operate; after removing `store_client`, confirm no path mutated only the clone's non-shared state expecting re-insertion (that was the original reason for the pattern — CODEX-M context).

## Codex log

## Claude review

## Verdict
