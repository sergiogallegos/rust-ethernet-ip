---
id: CODEX-AL
title: Transport & session hardening — timeout desync, sender-context correlation, shared session handle
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-06 claude [Opus 4.8]
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

2026-07-06 codex [GPT-5] Submitted implementation.

- Moved `EipClient::session_handle` to `Arc<AtomicU32>` and updated diagnostics/legacy connected-message call sites to load the shared handle. `check_health_detailed` re-registration now updates the handle visible to all clones using the same stream.
- Added shared `stream_poisoned: Arc<AtomicBool>` and `sender_context_counter: Arc<AtomicU64>`. Atomics use `Relaxed` because they are scalar fail-fast/visibility markers; the stream mutex still serializes I/O.
- Reworked `register_session` to hold one stream lock across write + response read, use `read_exact` for the 24-byte header and length-driven body, and leave the stream poisoned on incomplete transaction failures.
- Reworked `send_rr_data_item` to fail fast on a poisoned stream, mark the stream poisoned for the duration of the transaction, stamp each SendRRData with a monotonic sender context, verify the response context, and clear poison only after a complete framed response.
- Updated the test and standalone simulators to echo encapsulation sender contexts; the test simulator can also corrupt them for the mismatch regression.
- Removed the FFI `store_client` helper and both clone reinsert call sites; registry removal by `eip_disconnect` is no longer vulnerable to health/diagnostics resurrection.
- Added regressions for timeout poisoning, sender-context mismatch poisoning, fragmented RegisterSession replies, clone-visible re-registration, and FFI disconnect-vs-detailed-health resurrection.

Verification:

- `cargo test transport_tests` passed (2 tests).
- `cargo test --test plc_sim_tests simulated_plc_timeout_failure_mode` passed.
- `cargo test --test plc_sim_tests simulated_plc_sender_context_mismatch_poisons_connection` passed.
- `cargo test --features ffi --test ffi_state_consistency` passed (3 tests).
- `cargo fmt -- --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` passed.
- `cargo test --test plc_sim_tests` passed (22 tests).
- `cargo build --release --features ffi` passed.
- `dotnet build csharp\RustEtherNetIp\RustEtherNetIp.csproj --no-restore` passed.
- `dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --no-restore` passed (86 tests).
- `dotnet test csharp\RustEtherNetIp.IntegrationTests\RustEtherNetIp.IntegrationTests.csproj --no-restore` passed (7 tests).

Hardware smoke still required before 1.2.0: SendRRData sender_context values are no longer constant, so the maintainer should include a normal read/write packet capture in the pre-release hardware pass and confirm the controller echoes the request context.

## Claude review

### 2026-07-06 claude [Opus 4.8]

**Independent verification.** Full matrix re-run locally on the working tree: `cargo fmt -- --check` clean; `grep store_client src/ffi.rs` empty; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `SKIP_PLC_TESTS=1 cargo test --workspace --locked` green (exit 0); `cargo test transport_tests` 2/2; `cargo test --features ffi --test ffi_state_consistency` 3/3 (incl. the resurrection regression); `cargo test --test plc_sim_tests` 22/22 (incl. `simulated_plc_timeout_failure_mode` and `simulated_plc_sender_context_mismatch_poisons_connection`); `cargo build --release --features ffi` OK; C# unit 86/86; C# integration 7/7.

**What's being fixed.** The four §2/§3 transport-integrity findings: (1) a post-timeout stream left an unread response in flight so the next request read stale bytes — silent response-N-to-request-N+1 misattribution, made worse by `is_retriable()` inviting in-place retry; (2) the constant, never-checked `sender_context` made desync undetectable; (3) `session_handle` was `COPIED ON CLONE`, so `check_health_detailed` re-registration updated only the caller's clone while every other clone (pollers, FFI registry, tag-group tasks) kept the stale handle; (4) `register_session` took the stream lock separately for write and read and read the reply with a single `read()`.

**Root cause confirmation.** All four confirmed in the source, not symptom-chasing. The clone-copied `session_handle: u32` is the literal root of both the handle-divergence bug and the FFI `store_client` resurrection race — the re-insert existed *only* because mutations didn't propagate through the clone. Moving it to `Arc<AtomicU32>` fixes the divergence and makes the re-insert redundant in one stroke.

**Fix appropriateness.** `session_handle`, `stream_poisoned`, and `sender_context_counter` all move into `Arc` shared state with `Relaxed` ordering documented (correct — they are scalar fail-fast/visibility markers; the stream `Mutex` serializes the actual I/O ordering). The poison discipline matches the brief's "assume poisoned unless proven clean": `send_rr_data_item` sets `stream_poisoned = true` *before* the write (under the stream lock) and clears it *only* after a complete framed response — so a future dropped mid-transaction, a write error, a header/body timeout, or a `sender_context` mismatch all leave the stream poisoned and the next call fails fast with `ConnectionLost`. The error-status branch correctly drains the response body *then* clears poison (the stream is clean, only the app-level op failed). `register_session` now holds one lock across write+read and frames the reply with `read_exact` on the 24-byte header then a length-driven body read. The FFI `store_client` helper and both re-insert sites are gone; `get_client` returns a clone and every field the health/register path mutates is now `Arc`-shared, so `eip_disconnect` (the sole remover) can no longer be undone by a concurrent detailed-health call. Clone-contract comments at the struct are accurately rewritten; the `Send + Sync + 'static` assertion still holds.

**Test proof.** `simulated_plc_timeout_failure_mode` extends to assert the *next* read after a dropped-response timeout returns `ConnectionLost`, not stale data — the core scenario. `simulated_plc_sender_context_mismatch_poisons_connection` drives the sim's new `corrupt_sender_context_after` injection and asserts both the `sender_context mismatch` protocol error and the follow-on fast-fail. `ffi_detailed_health_cannot_resurrect_disconnected_client` hammers `eip_check_health_detailed` on a thread across a concurrent `eip_disconnect` and asserts the registry is empty afterward. `register_session_accepts_fragmented_reply` splits the 28-byte reply across two writes; `reregistration_updates_session_handle_across_clones` proves a re-register on one clone is visible on another. Both simulators echo `header[12..20]`.

**Residual risk.** Wire-format-observable change (`sender_context` is no longer constant) ⇒ maintainer hardware smoke is the release gate: a normal read/write packet capture confirming the controller echoes the per-request context. Recorded in the Codex log and CHANGELOG.

**Strong points.** The coupling of the `Arc<AtomicU32>` handle move with the `store_client` removal is exactly right — the resurrection race could not have been closed without the shared handle, and both landed together with a dedicated stress regression. The poison shape is genuinely cancellation-safe, not just timeout-safe.

### Findings

- 🟡 The `sender_context` mismatch check runs before the status check and applies to *all* replies, including encapsulation-error replies (`status != 0`). The happy path and CIP-level errors are safe — the encapsulation layer echoes `sender_context` per spec whenever it processes the frame. The narrow risk is a genuine *encapsulation-layer* error (bad session handle, unsupported command) from a controller that doesn't echo context on such replies, which would be misclassified as a desync and poison the stream. The brief anticipated exactly this and said to keep the check tolerant on `status != 0` replies *if* the hardware smoke shows echo quirks. Accepted as-is pending that evidence; the relaxation is a ~2-line change (skip the context check when `status != 0`). This is the #1 hardware-smoke decision point.
- 🟢 `EncapsulationHeader::send_rr_data` (the constant `[0x01..0x08]` context builder) now has zero `src/` call sites — every SendRRData goes through `send_rr_data_with_context`. It remains `pub` in the protocol crate (public API, harmless), but is dead weight on the live path; a candidate for the 2.0 API sweep, not this task.

### Acceptance criteria tally

1. All four fixes implemented with the specified tests green — ✅.
2. `store_client` gone from `src/ffi.rs`; grep-clean — ✅.
3. Clone-contract comment block accurately describes the new state — ✅ (verified; `session_handle`/`stream_poisoned`/`sender_context_counter` documented SHARED ON CLONE, `batch_config` honestly kept COPIED).
4. No new `unwrap`/`panic`; atomics use documented `Relaxed` orderings with a comment — ✅.
5. CHANGELOG `[Unreleased]` `### Fixed` entry; `sender_context` wire call-out for the hardware smoke — ✅.

## Verdict

**Merged.** Independent full-matrix verification green (fmt, clippy all-targets/all-features `-D warnings`, workspace `--locked`, transport_tests 2/2, ffi_state_consistency 3/3, plc_sim_tests 22/22, release ffi build, C# 86/86 + integration 7/7). All four transport-integrity findings — the highest-severity runtime defects in the Rust core — are correctly and coherently fixed: the `Arc<AtomicU32>` session handle simultaneously restores the one-handle-per-stream invariant and removes the FFI resurrection race; the poison flag is cancellation-safe (set-before-write, clear-after-complete-read); `sender_context` correlation turns silent misattribution into a loud fast-fail; `register_session` is single-locked and properly framed. Zero defects, zero Claude-applied fixes. One 🟡 (uniform `sender_context` check on error replies — brief-anticipated, gated on the hardware smoke, ~2-line relaxation if needed) and one 🟢 (now-dead constant-context builder). The wire-observable `sender_context` change carries a mandatory pre-1.2.0 maintainer hardware smoke (normal read/write capture confirming the controller echoes the per-request context), recorded in the release gate.
