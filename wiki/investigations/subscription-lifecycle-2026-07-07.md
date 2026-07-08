# Subscription Lifecycle

## Summary

CODEX-AR makes live tag subscriptions cooperative and nonblocking. `stop()` and
`unsubscribe()` now stop network polling, slow consumers drop oldest queued
notifications instead of wedging producer tasks, and single-tag poll errors are
observable through a new value/error event stream.

## Current Understanding

- `confirmed`: `TagSubscription::stop()` is now checked by the single-tag poll
  loop in [`../../src/client/subscriptions.rs`](../../src/client/subscriptions.rs).
- `confirmed`: `EipClient::unsubscribe(tag_path)` stops matching subscriptions
  and prunes them from the client's internal subscription registry.
- `confirmed`: value notifications and tag-group events use bounded
  drop-oldest delivery under backpressure. This is intentionally lossy; it
  replaces the old behavior where an abandoned consumer could block the poll
  task permanently.
- `confirmed`: `TagSubscriptionEvent` reports value updates and polling errors.
  Retriable errors produce nonterminal events and keep polling; repeated
  `ConnectionLost` errors stop after a small cap to avoid hot-looping on an
  AL-poisoned stream.
- `confirmed`: CIP general statuses `0x01` and `0x07` map to connection-class
  errors, so they participate in the existing retriable-error policy.

## Evidence

- Implementation: [`../../src/subscription.rs`](../../src/subscription.rs),
  [`../../src/client/subscriptions.rs`](../../src/client/subscriptions.rs),
  [`../../src/tag_group.rs`](../../src/tag_group.rs),
  [`../../src/client.rs`](../../src/client.rs)
- Simulator coverage: [`../../tests/subscription_tests.rs`](../../tests/subscription_tests.rs),
  [`../../tests/plc_sim.rs`](../../tests/plc_sim.rs)
- Task record: [`../../docs/agents/tasks/CODEX-AR-subscription-fleet-lifecycle.md`](../../docs/agents/tasks/CODEX-AR-subscription-fleet-lifecycle.md)

## Open Questions

- Should future wrapper surfaces expose subscription events, or should this stay
  Rust-only until there is a wrapper-level subscription contract?
- Should drop-oldest counters be surfaced in diagnostics if users need to audit
  missed subscription updates?

## Related Pages

- [`client-actor-service-retry-2026-05-24.md`](client-actor-service-retry-2026-05-24.md)
- [`fleet-api-2026-05-24.md`](fleet-api-2026-05-24.md)
