# Array-Type Cache Lifecycle

## Summary

`active` as of 2026-08-22: CODEX-BA added a clone-shared schema generation and
comprehensive Rust refresh operation. CODEX-BB added response-validated,
one-time read recovery for packed-BOOL classification drift. A controller
project download is not directly observable, so an explicit refresh remains
the deterministic application hook for known project changes.

## Current Understanding

- The cache stores only `base array path -> packed BOOL or ordinary array`; it
  is not a general cache of every PLC tag datatype.
- A new `EipClient` starts empty. C#, Python, and C/C++ disconnect/reconnect
  paths create a new native client ID, so their native cache starts empty.
- Rust callers that discard the old client and call `EipClient::connect()` also
  start empty. `RetryClient` retries operations on the same actor/client and is
  not an automatic transport reconnection mechanism.
- `EipClient::refresh_schema()` advances a clone-shared monotonic generation
  and clears array classification, TagManager metadata, TagManager's separate
  UDT-definition map, and all UdtManager definitions/templates/tag attributes.
- `clear_caches()` remains source-compatible as an alias of
  `refresh_schema()`.
- Route changes advance the same generation. TagManager/UdtManager contents
  become immediately ineligible for cache hits and are cleared eagerly by an
  explicit refresh or before the next valid generation-owned insertion.
- Array classifications carry their generation, and post-I/O insertion checks
  reject a captured generation that became stale while the request was in
  flight. Tag and UDT cache fills use the same before-insert generation check.
- `unclear`: Studio 5000 downloads may interrupt a given connection, but the
  library cannot rely on every download doing so. If the session survives, the
  cache currently remains populated.
- `practitioner-confirmed` on the project test hardware: an online Logix edit
  can replace a tag's effective schema without a download by creating a
  temporary tag, moving logic references, deleting the unused original, and
  renaming the replacement to the original symbolic name. The application
  connection can therefore survive while one cache key changes meaning.
- Offline UDT edits followed by a download can similarly change member layout
  and structure handles behind existing tag paths; this affects the broader
  tag/UDT caches, not only packed-BOOL classification.
- Ordinary-to-ordinary changes such as DINT array to REAL array retain the
  same non-BOOL classification. Transitions to or from packed BOOL are the
  safety-relevant stale-cache case because addressing semantics change.
- Single and native-batch array reads validate returned datatypes against the
  generation-stamped classification. A contradiction or symbolic-path failure
  evicts only that array path, rebuilds it, and retries the logical read once.
- Batch recovery replaces only the failed read result at its original input
  position. Unrelated results are not reordered or replayed.
- Packed-BOOL writes may reclassify during the pre-write DWORD read. Once a
  write request has been sent, the library does not replay it after an error or
  ambiguous transport outcome.
- Confirmed by the dynamic simulator for controller/program scope, indices on
  both sides of the 32-bit DWORD boundary, DINT[]/BOOL[] transitions,
  DINT[]/REAL[] compatibility, temporary deletion/recreation, batch result
  correlation, and fail-closed writes.

## Remaining Hardening

- Add an explicit native cache-clear export and thin C#, Python, and C/C++
  wrapper methods so applications can react to a known controller download.
- Expose the native schema refresh and cache diagnostics consistently across
  C, C#, Python, and C++ (CODEX-BC).
- Add the real-controller online-replacement and download validation record.
- A TTL may limit stale duration but is not sufficient as the primary safety
  mechanism because an incorrect operation can occur before expiry.

## Evidence

- [`src/client.rs`](../../src/client.rs) owns cache lookup and invalidation.
- [`src/client/batch_exec.rs`](../../src/client/batch_exec.rs) selects packed-
  BOOL addressing during request preparation and decodes response types.
- [`tests/schema_drift_recovery_tests.rs`](../../tests/schema_drift_recovery_tests.rs)
  exercises dynamic same-name mutations and bounded recovery.
- [`docs/validation/SCHEMA_CHANGE_GATE.md`](../../docs/validation/SCHEMA_CHANGE_GATE.md)
  defines the maintainer-controlled live edit/download and restoration gate.
- [`docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md`](../../docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md)
  records both the offline PASS and the completed live 1756-L75 firmware-33
  hardware PASS (array schema-swap both directions/scopes on all four
  bindings, UDT layout-edit/download with session-survival confirmed, and
  the post-schema full-coverage/batch regression).
- [`src/ffi.rs`](../../src/ffi.rs) creates and removes native client registry
  entries; no cache-clear export currently exists.
- [`csharp/RustEtherNetIp/EthernetNetIpClient.Connection.cs`](../../csharp/RustEtherNetIp/EthernetNetIpClient.Connection.cs)
  creates a new native client on reconnect.
- [`python/rust_ethernet_ip/client.py`](../../python/rust_ethernet_ip/client.py)
  creates a new native client ID after disconnect/connect.
- [Rockwell Tag Editor documentation](https://www.rockwellautomation.com/en-us/docs/studio-5000-logix-designer/37-00/contents-ditamap/studio-5000-logix-designer/tag-editor-and-data-monitor.html)
  describes creating, deleting, and modifying tag properties.
- [Rockwell download options documentation](https://www.rockwellautomation.com/en-us/docs/studio-5000-logix-designer/38-02/contents-ditamap/studio-5000-logix-designer/about-the-controller-project-status-tray/download-dialog-box---options-tab-parameters.html)
  explicitly treats offline UDT structure updates as schema-affecting downloads.

## Open Questions

- Which controller status or identity attribute, if any, can reliably expose a
  downloaded project revision without adding material polling overhead?
- ~~Does a 1756-L75 firmware 33 download always break the current
  encapsulation session through the tested 1756-EN2T route?~~ Answered for
  this controller/route/session shape (2026-08-22): no. The live UDT-gate
  session's Rust connection survived two consecutive offline
  member-add-then-restore downloads without a reconnect. "Always" is not
  proven — this is one session's evidence, not exhaustive — but it is
  direct data, not speculation.
- Which existing cache owners can be consolidated without changing public API,
  and which must remain separate but participate in one schema epoch?

## Related Pages

- [../controllers/hardware-validation-program.md](../controllers/hardware-validation-program.md)
- [../wrapper-parity/ffi-registry-clone-audit.md](../wrapper-parity/ffi-registry-clone-audit.md)

## Tracked Work

- Release-blocking sequence: [CODEX-BA](../../docs/agents/tasks/CODEX-BA-schema-cache-generation.md)
  → [CODEX-BB](../../docs/agents/tasks/CODEX-BB-schema-drift-self-healing.md)
  → [CODEX-BC](../../docs/agents/tasks/CODEX-BC-cross-binding-schema-refresh-diagnostics.md)
  → [CODEX-BD](../../docs/agents/tasks/CODEX-BD-schema-change-validation-gate.md).
- Retained follow-ups: [CODEX-BE](../../docs/agents/tasks/CODEX-BE-batch-packet-policy-sweep.md),
  [CODEX-BF](../../docs/agents/tasks/CODEX-BF-python-native-batch-writes.md),
  [CODEX-BG](../../docs/agents/tasks/CODEX-BG-cross-binding-endurance-soak.md),
  and [CODEX-BH](../../docs/agents/tasks/CODEX-BH-tag-shape-performance-matrix.md).
