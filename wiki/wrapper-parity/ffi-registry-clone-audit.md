# FFI Registry Clone Audit

## Summary

`active` as of 2026-05-24: the C FFI registry stores `EipClient` values and most FFI functions retrieve cloned clients. The clone model is currently acceptable for implemented mutation paths because persistent mutations either touch shared `Arc` fields, mutate the registry entry directly, or store the clone back. Copied scalar fields still need explicit guardrails before more FFI configuration APIs are implemented.

## Current Understanding

- `EipClient` mixes shared clone state (`stream`, managers, `last_activity`, connected sessions, subscriptions, tag groups) with copied scalar state (`session_handle`, `route_path`, `max_packet_size`, `batch_config`).
- Current FFI route-path mutation uses `get_mut` on the registry entry, so later lookups observe the new route.
- Current FFI batch execution temporarily modifies the cloned `batch_config` and restores it; this is intentionally per-call and not persistent.
- Unsupported configuration exports such as `eip_set_max_packet_size` and `eip_configure_batch_operations` do not currently mutate copied fields.
- CODEX-M Phase A recommends Option C: annotate clone behavior and require copied-field FFI mutators to either mutate the registry entry directly or store the clone back.

## Evidence

- [`src/client.rs`](../../src/client.rs) defines the mixed shared/copied `EipClient` fields.
- [`src/ffi.rs`](../../src/ffi.rs) defines the `FFI_CLIENTS` registry, `get_client()` cloning behavior, and current FFI mutation paths.
- [`docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md`](../../docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md) records the field-by-field and function-by-function audit.

## Open Questions

- `needs-review`: Claude must confirm or counter-propose the CODEX-M Phase B option before implementation.
- `unclear`: whether future public FFI batch/max-packet configuration should make copied fields shared or use registry-entry mutation only.

## Related Pages

- [../protocol/abi-contract.md](../protocol/abi-contract.md)
