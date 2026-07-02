---
id: CODEX-AR
title: Subscription, fleet, and event lifecycle — stop() stops, no blocked poll tasks, lag-tolerant forwarding
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
---

## Brief

### Goal

Fix the task-lifecycle defects in the live subscription/fleet/event paths from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §2):

1. **`stop()` doesn't stop** (`src/client/subscriptions.rs:38-56`): the single-tag poll loop never checks `subscription.is_active()` — `TagSubscription::stop()` only silences notifications while the task keeps issuing a TCP read every `update_rate` ms forever. The tag-group loop (`:176`) checks; mirror it.
2. **Abandoned subscriptions deadlock their poll task permanently** (`src/subscription.rs:41-59` + `client/subscriptions.rs:44`): the subscription holds its own `Receiver` behind `Arc<Mutex>` and the client's Vec retains a clone, so the channel can never disconnect; when the consumer stops calling `wait_for_update()`, the 100-slot buffer fills and `sender.send().await` blocks forever — the loop's only exit (send error) is unreachable. Same pattern in `TagGroupSubscription::publish_event` (`src/tag_group.rs:154-161`, 64-slot buffer) which isn't guarded by the group loop's `is_active()` check while blocked. Fix: replace the blocking `send().await` with `try_send` + drop-oldest (or `send_timeout` + treat-as-lagged) so a slow/dead consumer can never wedge the producer; on channel-closed, exit the task.
3. **First transient read error permanently kills a subscription** (`client/subscriptions.rs:47-52`): `break` on any `read_tag` error with only a log line — no error event reaches the consumer. The tag-group path publishes `ReadFailure` events and keeps going; give the single-tag path equivalent semantics (publish an error notification; continue on `is_retriable()` errors; stop with a terminal event otherwise).
4. **Subscription registry grows forever** (`src/client.rs:253` + `client/subscriptions.rs:32-35`): every subscribe pushes into `Arc<Mutex<Vec<TagSubscription>>>`; nothing evicts. Add `unsubscribe` (public) and prune stopped entries in `update_subscription`.
5. **Fleet forwarding dies on `Lagged`** (`src/fleet.rs:102-113`): `while let Ok(event) = client_events.recv().await` exits on *any* error, but `broadcast::RecvError::Lagged(n)` is recoverable — `continue` (optionally surfacing a lag event). Also: `insert_client` replacing a PLC id leaves the old forwarding task alive emitting under the same id — shut it down (abort handle or a shutdown watch).
6. **`Client::events()` side effects** (`src/client/actor.rs:153-157`): subscribing *sends* a spurious `Connected` to all existing subscribers, even when the actor is dead. Make subscription observation-only; if a "current state" snapshot on subscribe is wanted, deliver it to the new subscriber only.
7. **`start_monitoring` leak** (`src/monitoring.rs:506-515`): each call spawns an unstoppable 30 s-interval task pinning the metrics `Arc`. Return a guard/handle whose drop stops the task. (If CODEX-AQ deprecates the module first, apply the minimal guard fix anyway — deprecated ≠ leaking.)

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §2; `src/subscription.rs`, `src/client/subscriptions.rs`, `src/tag_group.rs` in full (the group path is the better sibling — reuse its patterns); `src/fleet.rs`; `src/client/actor.rs` (event vocabulary and who consumes it — `RetryClient`, C# wrapper?); `tests/subscription_tests.rs` + `tests/fleet_tests.rs` (existing conventions to extend).

### Files to create or modify

`src/subscription.rs`, `src/client/subscriptions.rs`, `src/tag_group.rs`, `src/fleet.rs`, `src/client/actor.rs`, `src/monitoring.rs`, `src/client.rs` (unsubscribe + prune), FFI/wrapper surface only if a subscription FFI export exists (grep `eip_subscribe`; if none, note it), `tests/subscription_tests.rs`, `tests/fleet_tests.rs`, `CHANGELOG.md`.

### Behavior

- `stop()`/`unsubscribe` halt network polling within one poll interval; dropped consumers never block a producer; a subscription outlives transient errors and reports them as events; fleet forwarding survives lag bursts; replaced fleet clients don't ghost-emit; subscribing to events has no side effects on other subscribers; every spawned task has an owner that can stop it.
- Notification-delivery semantics change from "block until consumed" to "drop-oldest under backpressure" — document this on the public types (it was never a real guarantee; the old behavior was deadlock, not delivery).

### Test requirements

Simulator-backed, deterministic (no sleeps-as-synchronization; use event assertions with timeouts):

- stop-halts-polling: subscribe against the sim with a read-counter (sim exposes per-tag read counts, or count via failure-injection hooks), stop, assert the count stabilizes.
- abandoned-consumer: subscribe, never consume, let >100 updates elapse (fast `update_rate`), assert the poll task still makes progress (e.g. counter keeps rising) — the pre-fix code deadlocks here, so this test must use a bounded wait.
- transient-error-recovery: inject a per-tag read failure for one poll, assert an error event is delivered and a subsequent value event follows.
- unsubscribe-evicts: subscribe N, unsubscribe N, assert registry length (test-visible via a `#[doc(hidden)]` len accessor or debug snapshot).
- fleet-lagged: overflow a small broadcast buffer, assert forwarding continues after the burst; replace-client: assert no events under the old registration after `insert_client` replacement.
- events-no-side-effect: two subscribers; the second subscribing must not deliver anything to the first.
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`.

### Acceptance criteria

- All seven fixes with the tests above; the abandoned-consumer test demonstrated deadlocking pre-fix (bounded-wait failure recorded in the log).
- No `send().await` on a notification channel from a poll/forwarding task anywhere in the touched files (grep).
- No public signature breaks (`unsubscribe` and the monitoring guard are additive; the guard's old return type — if `start_monitoring` returned `()` — additive-in-type via new method if needed to stay semver-clean; check with semver-checks).
- CHANGELOG entries; doc comments state the drop-oldest semantics.

### Out of scope

- Deprecating `SubscriptionManager`/`ProductionMonitor` as types — [[codex-aq-dead-stratum-deprecation]]. Reconnect policy in the actor (event vocabulary only gets the side-effect fix, not a health loop — that's a ROADMAP design question). Deadband/`change_threshold` semantics (documented in 1.1.0; unchanged).

### Risks and gotchas

- Drop-oldest on a `mpsc` channel isn't native — either wrap with a `try_send` + on-full `try_recv`-then-`try_send` dance (document the race window) or switch the notification channel to `tokio::sync::broadcast`/`watch` where lag semantics are built-in. `watch` (latest-value) may actually match subscription semantics best — evaluate and justify the choice in the log; don't silently change delivery guarantees beyond what the docs you write state.
- The poll task holds an `EipClient` clone; after CODEX-AL, a poisoned connection makes every poll fail fast — the transient-error policy must not hot-loop at `update_rate` against a dead connection forever; back off or emit terminal after N consecutive `ConnectionLost`s. If AL hasn't landed, design for it (match on `is_retriable()`).
- `insert_client` shutdown: the old task may be mid-`recv`; abort is safe there, but don't abort mid-send to consumers — prefer a cooperative shutdown flag checked per iteration, falling back to `abort()` after a grace timeout.
- Existing `subscription_tests.rs` may encode the old blocking-delivery behavior — rewrite those tests deliberately and say so in the log, don't contort the fix to keep them green.

## Codex log

## Claude review

## Verdict
