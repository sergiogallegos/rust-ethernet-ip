---
id: CODEX-M
title: FFI registry clone-semantics audit and fix
owner: codex
status: open
created: 2026-05-18
last-update: 2026-05-18 claude [Opus 4.7]
---

## Brief

### Goal

Audit and fix the `EipClient` clone semantics that the FFI surface relies on via the global `FFI_CLIENTS` registry in `src/ffi.rs`. The current design returns a `Clone`'d copy of `EipClient` from `get_client()` (`src/ffi.rs:63,74`). Because the struct derives `#[derive(Clone)]` and *mixes* shared interior (`Arc<Mutex<_>>`, `Arc<TagManager>`, etc.) with scalar fields that are copied on clone, mutations to scalar fields applied through one FFI call are invisible to subsequent FFI calls that re-fetch through `get_client()`. State silently drifts depending on which field a code path touches.

This brief turns that silent drift into either an enforced invariant (every mutation-bearing field is shared) or a hard structural change (the FFI handle is a distinct, all-shared type). Either way, after this brief lands, no FFI function silently mutates a field that another FFI function cannot observe.

Driven by the architecture review at [`wiki/investigations/architecture-review-2026-05-18.md`](../../../wiki/investigations/architecture-review-2026-05-18.md), Phase 0 item 2. Runs *after* CODEX-L (so the ABI baseline pin protects this restructuring) and *before* CODEX-J (so the mechanical `client.rs` split happens on a correctly-shared struct).

### Context to read first

- `src/client.rs:225-260` — the `EipClient` struct definition, every field, and the `Clone` derive. **Read every field's type.**
- `src/ffi.rs` end-to-end (2,843 lines) — every `#[no_mangle] extern "C"` function. Focus on the ones that take or return a client handle, and any that mutate per-connection state (session timeout, route path, batch config, subscriptions).
- `src/ffi.rs:15-100` — the `FFI_CLIENTS` registry, the `get_client()` / `insert_client()` helpers, and the global runtime singleton.
- `wiki/investigations/architecture-review-2026-05-18.md` — the parent synthesis document, specifically the "missed issues" section that surfaced this bug.

### Files to create or modify

Phase A (investigation, before any code change):

- New section in this task's `## Codex log` titled **Audit findings** containing:
  - A table of every `EipClient` field, its type, whether it is shared on clone (Arc/Mutex/etc.) or copied on clone (scalar/Box).
  - A table of every FFI function that mutates client state, what field it touches, and whether the mutation is visible to a later `get_client()` call.
  - A recommended option (A / B / C below) with one-paragraph justification.

Phase B (implementation, after Claude reviews the findings and confirms the option):

- `src/client.rs` — field reshuffling per chosen option.
- `src/ffi.rs` — registry-handle wiring per chosen option.
- New `tests/ffi_state_consistency.rs` (gated on `cfg(feature = "ffi")`) — see Test requirements.

### Behavior

Pick one of three options based on the audit findings:

- **Option A — Remove `Clone` from `EipClient`, registry returns `&EipClient` (or `MutexGuard`)**.
  Likely too restrictive across the C ABI; document if so in the audit and skip.

- **Option B — Introduce `ClientHandle` (cheap-clone, all-Arc) as the FFI registry value type**.
  `EipClient` stays public for direct Rust consumers; `ClientHandle` is the type stored in `FFI_CLIENTS` and returned by `get_client()`. All FFI mutations go through the handle. This is the cleanest separation and the most defensible long-term but is the most code change.

- **Option C — Audit each scalar field on `EipClient`, move every field that is mutated by any FFI path into `Arc<Mutex<_>>` (or `Arc<AtomicX>` where appropriate). Keep `Clone` derive. Add a `// SHARED ON CLONE` / `// COPIED ON CLONE` comment on every field.**
  Smallest code change. Risk: future contributors add a new field and forget which kind it should be; mitigation = a `compile_fail` test asserting `EipClient: Send + Sync + 'static` plus a doc-comment review checklist.

The audit (Phase A) recommends one; Claude review confirms or counter-proposes; Codex implements (Phase B). **Do not skip Phase A — the option choice depends on what the audit finds, not on guessing.**

### Test requirements

- `tests/ffi_state_consistency.rs` (new, `cfg(feature = "ffi")` gated):
  - Connect via FFI (use a simulator address from `tests/plc_sim.rs`).
  - Mutate a piece of state via one FFI call (whichever path the audit identifies as most-likely-to-drift; e.g., set a session timeout).
  - Observe that state via a different FFI call.
  - Assert the mutation is visible.
  - Repeat 1,000 times alternating mutate / observe; assert no drift across iterations.
- Run the existing C# `dotnet test` matrix and Python `unittest` matrix — both must stay green after the fix.
- Add a benchmark (or extend `benches/performance_benchmark.rs`) comparing per-FFI-call overhead before vs after, to catch unexpected slow-down from the handle restructure. Acceptable budget: ≤ 5 % regression in FFI hot path.

### Acceptance criteria

- **Audit findings table** committed in `## Codex log` *before* any code change.
- Chosen option implemented; the audit's mutation/observation table now annotates each row "visible by registry-lookup ✓".
- `tests/ffi_state_consistency.rs` passes locally and in CI.
- C# `dotnet test` and Python `unittest` matrices stay green; no wrapper-level regression.
- `cargo bench` shows ≤ 5 % FFI overhead regression (or none).
- `cargo audit`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features --locked --verbose` all green.
- If Option B is chosen: `wiki/protocol/abi-contract.md` (created by CODEX-L) is updated to document `ClientHandle` as the FFI registry type and the rationale for the split.
- No change to the `eip_abi_version()` value (still `1`) — this brief is intentionally non-ABI-breaking. If Option B requires a renamed FFI symbol, that's a brief failure; restructure internally only.

### Out of scope

- The actor refactor (CODEX-P). This brief does *not* introduce request-correlator semantics; it makes the *current* Clone story honest. CODEX-P can run after this without rework.
- Changing the public Rust API of `EipClient`. Direct Rust consumers see the same surface after this brief.
- Restructuring `client.rs` for size (CODEX-J). The submodule split happens after this brief on a now-correctly-shared struct.
- Adding new FFI exports. This brief touches only the registry plumbing and the underlying field-sharing model.

### Risks and gotchas

- Option B is the most invasive but the safest long-term. If the audit shows Option C is achievable in < 200 LOC of changes, prefer C; if more, prefer B.
- Whichever option is chosen, *every* FFI function that currently calls `let client = get_client(handle)?;` must be re-examined: does it depend on the returned value being a snapshot, or on it being a live shared reference? An audit table per FFI function (in Phase A) catches this.
- The 1,000-iteration stress test should run against the existing simulator, not against a mock. Network-level interleavings are the kind of bug this brief is trying to prevent.
- Be careful with `tokio::sync::Mutex` vs `std::sync::Mutex` inside a registry value held across FFI calls. The runtime is global; if a future FFI helper is `extern "C"` and synchronous, it cannot `.await` a tokio mutex — it must use `blocking_lock()` or hold its own runtime entry. The audit should note any current cross-boundary mutex usage.

## Codex log

_(append work entries here — Phase A audit findings BEFORE any code change)_

## Claude review

_(append review entries here — confirm option choice before Phase B begins)_

## Verdict

_(final disposition)_
