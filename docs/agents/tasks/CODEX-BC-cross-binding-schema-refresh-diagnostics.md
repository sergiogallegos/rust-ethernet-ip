---
id: CODEX-BC
title: Cross-binding schema refresh API and cache diagnostics
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1. Depends on CODEX-BA and CODEX-BB.**

Expose the comprehensive schema refresh and its diagnostics consistently
through the C ABI, C#, Python, and C/C++ surfaces.

### Required implementation

1. Add a handle-based C export such as `eip_refresh_schema(client_id)`.
2. Add thin wrapper methods:
   - C#: `RefreshSchema()`;
   - Python: `refresh_schema()`;
   - C++ convenience layer: `refreshSchema()`;
   - document the C function in the public header.
3. Update the FFI header parity gate and ABI capability bitmap. Determine and
   document whether the additive export requires an ABI version change; do not
   change the ABI number without coordinating every pin.
4. Extend diagnostics with:
   - schema generation;
   - refresh count;
   - array-classification hits, misses, and evictions;
   - datatype contradictions;
   - successful and failed read recoveries.
5. Ensure diagnostic JSON additions remain backward-compatible for existing
   wrapper parsers.
6. Document the maintenance workflow: pause writes, edit/download, refresh,
   optionally rediscover/verify, then resume writes.

### Test requirements

- FFI success, invalid-client, last-error, and clone-visibility tests.
- Header/export parity and ABI/capability tests.
- C#, Python, and C++ wrapper tests proving the same native generation changes.
- Diagnostics values increment for hits, misses, refresh, eviction,
  contradiction, and recovery without exposing proprietary tag values.
- Full Rust, C#, Python, and C++ offline gates pass against one release FFI
  artifact.

### Acceptance criteria

- Every supported language can explicitly refresh the same native schema
  state without reconnecting.
- Diagnostics make cache behavior measurable and are documented accurately.
- Existing consumers that ignore new diagnostics fields continue working.
- Public wrapper guides contain the schema-maintenance example.
- CHANGELOG and draft 1.2.1 release notes are updated.

### Out of scope

- UI or automatic Studio 5000 integration.
- Polling an undocumented controller project revision.
- Hardware editing and download execution (CODEX-BD).

## Codex log

## Claude review

## Verdict
