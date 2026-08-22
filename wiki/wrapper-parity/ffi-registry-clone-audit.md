# FFI Registry Clone Audit

## Summary

`active` as of 2026-08-21: the C FFI registry stores `EipClient` values and most FFI functions retrieve cloned clients. Route, max-packet-size, and array-type-cache state that must survive registry cloning is structurally shared.

## Current Understanding

- `EipClient` mixes shared clone state (`stream`, managers, `route_path`, `max_packet_size`, `array_type_cache`, `last_activity`, connected sessions, subscriptions, tag groups) with copied scalar state (`session_handle`, `batch_config`).
- Current FFI route-path mutation may operate on a cloned client; `route_path` is shared on clone, so later registry lookups observe the new route.
- Current FFI max-packet-size mutation operates on a cloned client; `max_packet_size` is an `Arc<AtomicU32>`, so later registry lookups observe the new value.
- Current FFI batch execution temporarily modifies the cloned `batch_config` and restores it; this is intentionally per-call and not persistent.
- Packed-BOOL array classification is shared across clones so repeated wrapper
  batch reads reuse both positive and negative results. Route mutation clears
  that shared cache before installing or removing the route, and the public
  cache-clear operation clears it explicitly.
- `eip_configure_batch_operations` remains unsupported; persistent batch configuration would need a new audit or an explicitly shared config field.
- CODEX-M uses Option C: annotate clone behavior and move mutation-bearing scalar fields to shared state.

## Evidence

- [`src/client.rs`](../../src/client.rs) defines the mixed shared/copied `EipClient` fields.
- [`src/ffi.rs`](../../src/ffi.rs) defines the `FFI_CLIENTS` registry, `get_client()` cloning behavior, and current FFI mutation paths.
- [`docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md`](../../docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md) records the field-by-field and function-by-function audit.

## Open Questions

- `unclear`: whether future public FFI batch configuration should keep the current per-call override model or make `batch_config` shared behind a lock.

## Related Pages

- [../protocol/abi-contract.md](../protocol/abi-contract.md)
