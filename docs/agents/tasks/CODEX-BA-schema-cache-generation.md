---
id: CODEX-BA
title: Comprehensive schema refresh and shared cache generation
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
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

## Claude review

## Verdict
