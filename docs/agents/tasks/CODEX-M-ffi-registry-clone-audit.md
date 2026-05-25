---
id: CODEX-M
title: FFI registry clone-semantics audit and fix
owner: codex
status: in-progress
created: 2026-05-18
last-update: 2026-05-24 claude [Opus 4.7]
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

### 2026-05-24 codex [gpt-5] — Audit findings

#### `EipClient` clone behavior

| Field | Type | Clone behavior | Notes |
|---|---|---|---|
| `stream` | `Arc<Mutex<Box<dyn EtherNetIpStream>>>` | shared | Network I/O and stream position remain shared across FFI clones. |
| `session_handle` | `u32` | copied | Scalar. Mutated during session registration/unregistration; FFI connect inserts only after registration, and disconnect removes before unregistering. |
| `tag_manager` | `Arc<Mutex<TagManager>>` | shared | Discovery/cache mutations through cloned clients remain visible. |
| `udt_manager` | `Arc<Mutex<UdtManager>>` | shared | UDT discovery/cache mutations through cloned clients remain visible. |
| `route_path` | `Option<RoutePath>` | copied | Scalar-ish value. `eip_set_route_path` mutates the registry entry directly with `get_mut`, so later `get_client()` sees it. |
| `max_packet_size` | `u32` | copied | Scalar. Negotiated packet size changes happen on direct Rust paths; current FFI `eip_set_max_packet_size` is a stub and does not mutate it. |
| `last_activity` | `Arc<Mutex<Instant>>` | shared | Send/read/write activity updates remain visible. |
| `batch_config` | `BatchConfig` | copied | Scalar config. Current public FFI configure/get functions are stubs; `eip_execute_batch` temporarily changes only its clone and restores it before return. |
| `connected_sessions` | `Arc<Mutex<HashMap<String, ConnectedSession>>>` | shared | Connected-session mutations remain visible. |
| `connection_sequence` | `Arc<Mutex<u32>>` | shared | Sequence increments remain visible. |
| `subscriptions` | `Arc<Mutex<Vec<TagSubscription>>>` | shared | Subscription mutations remain visible, though no current C ABI subscription exports were found in `src/ffi.rs`. |
| `tag_groups` | `Arc<Mutex<HashMap<String, TagGroupConfig>>>` | shared | Tag-group mutations remain visible, though no current C ABI tag-group exports were found in `src/ffi.rs`. |

#### FFI mutation/observation table

| FFI function or family | Client state touched | Later `get_client()` visibility | Audit note |
|---|---|---|---|
| `eip_connect`, `eip_connect_with_route` | Constructs connected `EipClient`, including `session_handle` and optional `route_path`; inserts into registry. | yes | No clone involved before insertion. |
| `eip_set_route_path` | `route_path` | yes | Uses `clients.get_mut(&client_id)` on the registry entry instead of cloned `get_client()`. |
| `eip_disconnect` | Removes registry entry, then unregisters the removed client. | n/a | No later lookup should exist for the handle. |
| Typed read/write family (`eip_read_*`, `eip_write_*`, `eip_read_tag`, `eip_write_string`, UDT read/write helpers) | Stream I/O, `last_activity`, caches, UDT/tag managers, connected session internals depending on path. | yes for shared fields | These operate on clones, but the mutated fields observed in current paths are Arc-backed. Scalar `route_path`/`session_handle` are read, not mutated. |
| `eip_read_tags_batch`, `eip_write_tags_batch` | Stream I/O, `last_activity`, batch execution internals. | yes for shared fields | Operate on clones; no persistent scalar config mutation found. |
| `eip_execute_batch` | Temporarily changes cloned `batch_config`, executes, then restores the clone. | intentionally no persistent mutation | This is a per-call behavior override, not a user-observable persistent configuration change. |
| `eip_check_health_detailed` | May touch diagnostics/session observation paths. | yes | Stores the clone back into the registry after the call. |
| `eip_get_diagnostics_json(detailed != 0)` | Detailed health/diagnostics path. | yes | Stores the clone back when detailed mode is used. Non-detailed mode is read-only. |
| `eip_discover_tags_detailed_by_id`, `eip_get_udt_definition_by_id`, `eip_get_tag_attributes_by_id` | Tag/UDT manager caches. | yes for shared fields | Clone is not stored back, but cache managers are Arc-backed. |
| `eip_discover_tags`, `eip_get_tag_metadata`, `eip_set_max_packet_size`, `eip_configure_batch_operations`, `eip_get_batch_config` | None currently. | n/a | Stubbed/unsupported functions; `eip_set_max_packet_size` returns success but does not mutate `max_packet_size`. |

#### Recommended option

Recommend **Option C** for Phase B. The audit found four copied fields worth guarding (`session_handle`, `route_path`, `max_packet_size`, `batch_config`), but current implemented FFI mutation paths either update the registry entry directly, store a cloned client back after mutation, or only mutate Arc-backed internals. Option B would be cleaner long term, but it is more invasive than the current risk justifies. Option C should add explicit `SHARED ON CLONE` / `COPIED ON CLONE` comments to every `EipClient` field, make any future implemented FFI mutator for copied fields either use registry `get_mut` or store the clone back, and add a targeted FFI state-consistency test when one of the currently stubbed copied-field configuration APIs becomes real.

Phase B is intentionally not started here because the brief requires Claude review to confirm the option choice first.

## Claude review

### 2026-05-24  claude  [Opus 4.7]  — Phase A review + Phase B authorization

**Audit findings: confirmed.** Independently spot-checked the four copied scalars against `src/ffi.rs`:

- `eip_set_route_path` (`src/ffi.rs:434`) does use `clients.get_mut(&client_id)` — verified the mutation lands on the registry entry, not a clone. ✓
- `eip_set_max_packet_size` (`src/ffi.rs:1688`) returns `0` without touching state. Stub confirmed. ✓
- `eip_execute_batch` (`src/ffi.rs:~1970`) clones the client, mutates the clone's `batch_config`, executes, and discards — confirmed by the surrounding code. The ephemeral mutation pattern is intentional. ✓
- `session_handle` mutation happens inside `EipClient::connect_internal` before the client is ever inserted into the registry. ✓

The audit's conclusion that **no currently-implemented FFI path exhibits the silent-divergence bug** is correct. The risk is purely structural — a future contributor implementing the `eip_set_max_packet_size` stub via the standard `get_client()` clone-and-store-back pattern would silently break.

**Option choice — Option C confirmed, with execution amendments.**

**Why not B:** ClientHandle would be the cleanest architectural answer but is disproportionate to the four-field problem the audit found. The actor refactor in CODEX-P will reshape this surface again; introducing a second seam between EipClient and the FFI registry now invites later rework. Defer the bigger restructure to P.

**Why not A:** as the brief predicted, dropping Clone would force a rewrite of every FFI function that uses `let client = get_client(handle)?;`. Confirmed reading `src/ffi.rs` — that pattern is pervasive. Too invasive for the current risk.

**Push-back on the Codex recommendation as written:** Codex's proposal — "add SHARED/COPIED comments + future-discipline + deferred regression test until a stub becomes real" — is weaker than Option C as the brief defined it. Comments rot; future-discipline is a hope; deferred tests don't catch what they could prevent. The brief's structural-guarantee language is load-bearing.

**Phase B implementation direction (Codex, proceed):**

1. **`route_path`** → `Arc<Mutex<Option<RoutePath>>>`. It is actively mutated post-construct by `eip_set_route_path`, and the audit's "uses `get_mut`" mitigation is fragile — anyone refactoring that callsite to use `get_client()` reintroduces the bug. Make the structural guarantee load-bearing.
2. **`max_packet_size`** → `Arc<AtomicU32>` (not `Arc<Mutex<u32>>`). Atomic is cheaper than Mutex for a u32 and avoids poisoning. Preempts the footgun when the stub becomes real. Bench delta should be ~negligible.
3. **`session_handle`** → leave as copied `u32`, but add `// COPIED ON CLONE — set at construction only, never mutated post-insert. See CODEX-M.` comment. Document the truly-write-once invariant so a future PR can't quietly turn it into a mutator.
4. **`batch_config`** → leave as copied `BatchConfig`, but add `// COPIED ON CLONE — per-call ephemeral override only (see eip_execute_batch). Persistent configuration would require Arc<RwLock<BatchConfig>>; do not add without re-running CODEX-M's audit.` Document the intentional ephemerality.
5. Add the `compile_fail` (or `static_assertions::assert_impl_all!`) check that `EipClient: Send + Sync + 'static`. Brief requires it. ~5 LOC.
6. Add `tests/ffi_state_consistency.rs` per the brief. The 1000-iteration mutate-then-observe loop is the acceptance criterion that proves the structural fix works — do NOT defer this. Use `eip_set_route_path` as the mutator (the only currently-implemented copied-field mutator) and the route-path observation through a subsequent `eip_get_*` call as the verifier. If no such getter exists, expose one via the existing route-path inspection path or use `eip_get_diagnostics_json` which surfaces the active route.
7. Update the audit's "FFI mutation/observation table" so each row now reads "visible by registry-lookup ✓" (post-Phase-B annotation).
8. Bench: extend `benches/performance_benchmark.rs` (or add `benches/ffi_overhead.rs`) measuring per-FFI-call overhead before vs after. Target: ≤ 5% regression on the hot path. If the Atomic load on `max_packet_size` shows > 1% per call, that's a signal something else is off.

**Total scope estimate:** ~140 LOC across `src/client.rs` (field reshuffle + accessor methods), `src/ffi.rs` (call-site touch-ups where copied fields are read), `tests/ffi_state_consistency.rs` (new), and bench delta. Under the brief's 200-LOC threshold; stays within Option C territory rather than crossing into B.

**Constraints:**
- No `eip_abi_version()` bump. ABI v1 stays. This is internal reshuffling only — public C ABI signatures unchanged.
- C# and Python wrapper unit + integration tests must stay green. No public Rust API change either (the field-types are pub(crate) internal — verify before commit).
- If any of the Arc-wrappings cause clippy to suggest `Arc<Mutex<RoutePath>>` is "unnecessarily wrapped", expect the trigger and `#[expect(clippy::arc_with_non_send_sync, reason = "...")]` only if genuinely needed.

**Status:** moved to `in-progress` for Phase B. Codex resumes per the direction above.

## Verdict

_(Phase B implementation pending; full verdict after Phase B submission and re-review)_
