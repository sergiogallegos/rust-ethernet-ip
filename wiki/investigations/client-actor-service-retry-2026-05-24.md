# Client Actor, Service Helpers, and Retry Policy

## Summary

The 2026-05-24 CODEX-P/R/Q/S work adds an actor-backed `Client` API beside the existing `EipClient` facade. The current implementation is additive for Rust callers and does not replace FFI, C#, or Python wrapper entry points.

## Current Understanding

- `confirmed`: `Client` is a cheap clone handle that sends requests over an mpsc channel to a worker task owning the underlying `EipClient`.
- `confirmed`: `ConnectionEvent` is currently a lifecycle broadcast with `Connected`, `Disconnected`, and `WorkerStopped` variants.
- `confirmed`: restricted-write helpers remain concrete to documented Logix behavior: STRING writes and UDT member / UDT-array-member writes route through full-value write flows rather than a generic service framework.
- `confirmed`: `RetryPolicy` wraps the actor client and uses `EtherNetIpError::is_retriable()`; writes are not retried unless explicitly enabled with `retry_writes(true)`.
- `unclear`: wrapper-level adoption is not implemented yet. Existing C# and Python APIs still call the FFI surface directly.

## Evidence

- Actor implementation: [`../../src/client/actor.rs`](../../src/client/actor.rs)
- Service-layer helpers: [`../../src/client/service_layer.rs`](../../src/client/service_layer.rs)
- Public re-exports: [`../../src/client.rs`](../../src/client.rs), [`../../src/lib.rs`](../../src/lib.rs)
- Simulator coverage: [`../../tests/client_actor_tests.rs`](../../tests/client_actor_tests.rs)
- Task records: [`../../docs/agents/tasks/CODEX-P-client-actor.md`](../../docs/agents/tasks/CODEX-P-client-actor.md), [`../../docs/agents/tasks/CODEX-R-client-events.md`](../../docs/agents/tasks/CODEX-R-client-events.md), [`../../docs/agents/tasks/CODEX-Q-service-layer.md`](../../docs/agents/tasks/CODEX-Q-service-layer.md), [`../../docs/agents/tasks/CODEX-S-retry-policy.md`](../../docs/agents/tasks/CODEX-S-retry-policy.md)

## Open Questions

- Should wrapper APIs adopt the actor surface directly, or should actor semantics remain Rust-only while wrappers keep the FFI registry model?
- Should `ConnectionEvent` gain reconnect/session-recycled variants after reconnect behavior exists?
- Should service-layer helper coverage be expanded against real ControlLogix hardware before publishing v0.8.0?

## Related Pages

- [`architecture-review-2026-05-18.md`](architecture-review-2026-05-18.md)
- [`../limitations/string-and-udt-write-behavior.md`](../limitations/string-and-udt-write-behavior.md)
- [`../wrapper-parity/ffi-registry-clone-audit.md`](../wrapper-parity/ffi-registry-clone-audit.md)
