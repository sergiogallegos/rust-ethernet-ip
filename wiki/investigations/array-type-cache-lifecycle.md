# Array-Type Cache Lifecycle

## Summary

`active` as of 2026-08-22: cached packed-BOOL classifications materially
improve repeated batch reads, but a controller project download is not itself
an observable invalidation event in the current client.

## Current Understanding

- The cache stores only `base array path -> packed BOOL or ordinary array`; it
  is not a general cache of every PLC tag datatype.
- A new `EipClient` starts empty. C#, Python, and C/C++ disconnect/reconnect
  paths create a new native client ID, so their native cache starts empty.
- Rust callers that discard the old client and call `EipClient::connect()` also
  start empty. `RetryClient` retries operations on the same actor/client and is
  not an automatic transport reconnection mechanism.
- Route changes and Rust `clear_caches()` explicitly clear the array cache.
- `confirmed`: the current Rust `clear_caches()` clears tag metadata, the
  `UdtManager`, and array classification, but does not call `TagManager`'s
  separate `clear_udt_cache()`. It is not yet a complete schema refresh.
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

## Recommended Hardening

- Add an explicit native cache-clear export and thin C#, Python, and C/C++
  wrapper methods so applications can react to a known controller download.
- Make batch reads self-healing: carry the prepared array classification into
  response parsing, detect a response-type contradiction, evict the affected
  entry, rebuild, and retry the read batch once.
- Evict affected schema entries on Symbol Not Found/path errors so an online
  delete-and-recreate sequence is reclassified after the tag reappears.
- Prefer a cross-language `clear_caches`/schema-refresh operation that clears
  array, tag metadata, STRING-handle, and UDT-definition state together after
  a known online schema edit or project download.
- Protect invalidation with a shared schema generation. Without an epoch, an
  in-flight request through a cloned client could repopulate an old
  classification immediately after another caller clears the cache.
- Treat write retry separately. Do not automatically replay an ambiguous write
  after a transport failure; packed-BOOL read-modify-write can reclassify before
  sending the write.
- Add simulator tests for BOOL-to-DINT and DINT-to-BOOL transitions without
  reconnecting, plus a real-controller download/reconnect validation.
- A TTL may limit stale duration but is not sufficient as the primary safety
  mechanism because an incorrect operation can occur before expiry.

## Evidence

- [`src/client.rs`](../../src/client.rs) owns cache lookup and invalidation.
- [`src/client/batch_exec.rs`](../../src/client/batch_exec.rs) selects packed-
  BOOL addressing during request preparation and decodes response types.
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
- Does a 1756-L75 firmware 33 download always break the current encapsulation
  session through the tested 1756-EN2T route?
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
