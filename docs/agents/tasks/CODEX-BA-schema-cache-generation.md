---
id: CODEX-BA
title: Comprehensive schema refresh and shared cache generation
owner: codex
status: merged
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1.** First task in the schema-cache sequence. No task dependency.

The array-classification cache added on the 1.2.1 development line materially
improves repeated batch reads, but cache lifetime is currently tied mainly to a
client/route rather than the controller schema. An online delete/recreate/rename
workflow can reuse a symbolic path for a different datatype without breaking
the EtherNet/IP session. Offline UDT edits can also replace definitions behind
stable tag paths after a download.

Implement one comprehensive schema refresh primitive and a shared schema
generation that every schema-derived cache participates in.

### Context to read first

- `AGENTS.md`
- `wiki/investigations/array-type-cache-lifecycle.md`
- `wiki/wrapper-parity/ffi-registry-clone-audit.md`
- `src/client.rs`
- `src/tag_manager.rs`
- `src/udt.rs`

### Required implementation

1. Add a shared, monotonic schema generation to `EipClient`. It must remain
   shared across clones used by the FFI registry.
2. Add a Rust `refresh_schema()` operation that advances the generation and
   clears every live schema-derived cache:
   - packed-BOOL/non-BOOL array classification;
   - `TagManager` metadata;
   - `TagManager`'s separate UDT-definition map;
   - `UdtManager` definitions and per-tag attributes.
3. Keep `clear_caches()` source-compatible, either as the implementation or as
   a documented alias of `refresh_schema()`.
4. Route changes must advance/clear the same schema state.
5. Prevent an in-flight operation from repopulating an entry created under an
   older generation after a refresh. Cache entries may be generation-stamped,
   or insertion may reject an obsolete captured epoch.
6. Add internal counters for generation, refreshes, array-cache hits, misses,
   and evictions. Public/wrapper diagnostic exposure belongs to CODEX-BC.

### Test requirements

- Positive and negative array classifications are generation-scoped.
- `refresh_schema()` clears both TagManager cache families, UdtManager state,
  and array classification.
- Cloned clients observe the same generation and refresh.
- A deterministic stale-in-flight insertion cannot repopulate the new epoch.
- Route changes advance the generation.
- Existing cache and FFI clone-consistency tests remain green.
- Full offline Rust matrix: fmt, all-feature Clippy with warnings denied, and
  locked workspace tests with physical-PLC tests skipped.

### Acceptance criteria

- There is one authoritative Rust schema-refresh operation.
- No known schema-derived cache survives that operation.
- Clone races cannot reintroduce entries from an earlier generation.
- No public API is removed or behaviorally broadened beyond invalidation.
- CHANGELOG, Rust API docs, and the cache-lifecycle wiki are updated.

### Out of scope

- Automatic response contradiction recovery (CODEX-BB).
- FFI and wrapper APIs (CODEX-BC).
- Live PLC schema-change validation (CODEX-BD).
- Time-based cache expiration as the primary safety mechanism.

## Codex log

### 2026-08-22 10:32  codex [GPT-5]

Implementation started per maintainer direction. The stale project-context
summary in `docs/agents/board.md` was corrected to recognize the eight active
CODEX-BA through CODEX-BH tasks. CODEX-BA begins the ordered BA through BD
release-blocking sequence.

### 2026-08-22 10:40  codex [GPT-5]

Submitted implementation. `EipClient` now owns a clone-shared monotonic schema
generation; `refresh_schema()` advances it and clears array classifications,
TagManager metadata and UDT definitions, plus all UdtManager state.
`clear_caches()` remains a source-compatible alias. Route changes advance the
same generation. Array entries are generation-stamped, and all post-I/O tag,
UDT, attribute, and array cache fills reject an obsolete captured generation.
Internal refresh/hit/miss/eviction counters were added for CODEX-BC exposure.

Verification passed: `cargo fmt -- --check`, all-workspace/all-target/all-feature
Clippy with warnings denied, seven focused cache-generation tests, and
`SKIP_PLC_TESTS=1 cargo test --workspace --locked` including 25/25 simulator
tests and all doctests. CHANGELOG, draft 1.2.1 notes, and the cache-lifecycle
wiki were updated.

## Claude review

### 2026-08-22 16:35  claude [Sonnet 5]

**Independent verification**
- `cargo fmt -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — all pass (full suite rerun this session; targeted reruns of `client::array_type_cache_tests::refresh_schema_clears_every_schema_cache_and_is_clone_visible`, `::stale_generation_cannot_repopulate_array_classification`, `::route_changes_advance_the_shared_schema_generation` individually confirmed passing).
- `cargo test --test plc_sim_tests` — 25/25.

**What's being fixed**
- Cache lifetime was tied to a client/route rather than controller schema; an online delete/recreate/rename or offline UDT redefinition could leave stale packed-BOOL/UDT/tag-metadata state behind a stable symbolic path. This adds one authoritative, clone-shared schema generation and a comprehensive `refresh_schema()`.

**Root cause confirmation**
- Confirmed: prior `clear_caches()` cleared tag metadata, `UdtManager`, and array classification, but not `TagManager`'s separate `udt_definitions` map (per `wiki/investigations/array-type-cache-lifecycle.md`'s prior `confirmed` note, now corrected by this change).

**Fix appropriateness**
- `schema_generation`/`tag_manager_generation`/`udt_manager_generation` are `Arc<AtomicU64>` fields on `EipClient` (`src/client.rs:537-544`), so clones observe the same counters — the right layer for a value that must be clone-visible.
- `refresh_schema()` (`src/client.rs:1115`) advances the generation via `advance_schema_generation()` (`src/client.rs:1101`, which also zeroes `tag_manager_generation`/`udt_manager_generation` and clears the array cache), clears `TagManager`'s metadata cache and its separate `udt_definitions` map, and clears `UdtManager` in full — closing exactly the gap noted above.
- `clear_caches()` (`src/client.rs:1141`) is now a one-line alias calling `refresh_schema()` — source-compatible as the brief required.
- Route changes (`set_route_path` / `clear_route_path`, `src/client.rs:1191`/`1205`) call `advance_schema_generation()` directly (not the full `refresh_schema()`), which is a deliberate and correct distinction: route changes invalidate schema-derived state but are not "explicit refreshes" for diagnostics purposes — proven by `route_changes_advance_the_shared_schema_generation` asserting `schema_refreshes` stays at 0 across two route mutations.
- Stale-fill rejection is real, not cosmetic: `cache_array_is_packed_bool_at_generation` (`src/client.rs:1251`) re-checks `self.schema_generation() != generation` under the lock before inserting and returns `false` (not silently succeeding) if the epoch moved; the equivalent generation-check-before-insert pattern is repeated for `TagManager` metadata/UDT fills (`discover_tags`, `discover_udt_members`) and `UdtManager` fills (`get_udt_definition`, `get_tag_attributes`), all guarded by `if self.schema_generation() == generation`.

**Test proof**
- `refresh_schema_clears_every_schema_cache_and_is_clone_visible` populates all four cache families (array, tag metadata, TagManager UDT map, UdtManager definitions+attributes) then calls `refresh_schema()` on a *clone* and asserts every family is empty on the original — real clone-visibility proof, not just same-instance.
- `stale_generation_cannot_repopulate_array_classification` captures a pre-refresh generation, refreshes, then attempts an insert at the stale generation and asserts it's rejected — closes the brief's "clone races cannot reintroduce entries from an earlier generation" criterion directly.
- Not independently re-derived: the exact interleaving timing window a real concurrent clone race would hit (the test is deterministic/sequential, which is the correct and sufficient way to prove the *guard exists*, but doesn't prove there's no unguarded window elsewhere — every insertion site listed above was read directly and applies the same guard pattern consistently, so this is `not proven` by fuzzing/loom but is proven by code inspection across every insertion site).

**Residual risk**
- `src/client.rs` and `src/client/batch_exec.rs` are physically interleaved with CODEX-BB's changes in the same hunks — this review evaluates BA's contract on the merged tree, since a clean pre-BB diff doesn't exist to review in isolation. This is a review-boundary artifact, not a code defect; the two tasks are merged together in the same commit for exactly this reason.
- No live-hardware proof that a real Studio 5000 online replacement triggers exactly this generation path end-to-end — that's CODEX-BD's live session, still pending.

**Strong points (✅)**
- Distinguishing "advance generation" (route changes) from "explicit refresh" (counted in diagnostics) is a genuinely careful call that avoids polluting the `schema_refreshes` counter with implicit invalidations.
- Generation-stamped cache entries plus a reject-on-stale-insert guard is the textbook fix for the clone-race window the brief called out — implemented at every insertion site, not just the array cache.
- `clear_caches()` staying a real (not deprecated-and-forgotten) alias avoids a breaking change for existing callers.

**Findings**
- 🟢 No public API removed; `refresh_schema()` is additive, `clear_caches()` unchanged in signature.
- 🟡 Findings shared with CODEX-BB (see that task's review) concern the interleaved code, not BA specifically.

**Acceptance criteria tally**
- ✅ One authoritative Rust schema-refresh operation — `refresh_schema()`.
- ✅ No known schema-derived cache survives that operation — verified by test citing all four families.
- ✅ Clone races cannot reintroduce entries from an earlier generation — verified by `stale_generation_cannot_repopulate_array_classification` plus code inspection of every insertion site.
- ✅ No public API removed or behaviorally broadened beyond invalidation.
- ✅ CHANGELOG, Rust API docs, and the cache-lifecycle wiki are updated (`CHANGELOG.md`, `wiki/investigations/array-type-cache-lifecycle.md` — the latter documents CODEX-BA/BB/BC/BD together).

## Verdict

Merged (`src/client.rs` + `src/client/batch_exec.rs`, combined with CODEX-BB in one commit — the two are physically interleaved at the hunk level and cannot be reviewed or landed as clean standalone diffs). BA's generation infrastructure is sound, clone-visible, and closes the exact cache-family gap the prior lifecycle doc flagged.
