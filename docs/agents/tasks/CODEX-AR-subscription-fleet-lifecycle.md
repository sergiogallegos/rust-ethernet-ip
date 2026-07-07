---
id: CODEX-AR
title: Subscription, fleet, and event lifecycle — stop() stops, no blocked poll tasks, lag-tolerant forwarding
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-07 claude [Opus 4.8]
---

## Brief

### Goal

Fix the task-lifecycle defects in the live subscription/fleet/event paths from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §2):

1. **`stop()` doesn't stop** (`src/client/subscriptions.rs:39-56`): the single-tag poll loop (`tokio::spawn` at `:39`, `loop` at `:41`) never checks `subscription.is_active()` — `TagSubscription::stop()` only silences notifications while the task keeps issuing a TCP read every `update_rate` ms forever. The tag-group loop (`:176`, `while subscription_task.is_active()`) checks; mirror it.
2. **Abandoned subscriptions deadlock their poll task permanently** (`src/subscription.rs:45,53,94` + `client/subscriptions.rs:44`): the subscription holds its own `Receiver` behind `Arc<Mutex>` (`subscription.rs:45`) and the client's Vec retains a clone, so the channel can never disconnect; the 100-slot buffer (`mpsc::channel(100)` at `:53`) fills when the consumer stops calling `wait_for_update()`, and `sender.send().await` (`:94`) blocks forever — the loop's only exit (send error) is unreachable. Same pattern in `TagGroupSubscription::publish_event` (`src/tag_group.rs:155-161`, `sender.send(event).await` at `:160`) which isn't guarded by the group loop's `is_active()` check while blocked. Fix: replace the blocking `send().await` with `try_send` + drop-oldest (or `send_timeout` + treat-as-lagged) so a slow/dead consumer can never wedge the producer; on channel-closed, exit the task.
3. **First transient read error permanently kills a subscription** (`client/subscriptions.rs:49-52`): `break` on any `read_tag` error with only a log line — no error event reaches the consumer. The tag-group path publishes `ReadFailure` events and keeps going; give the single-tag path equivalent semantics (publish an error notification; continue on `is_retriable()` errors; stop with a terminal event otherwise).
4. **Subscription registry grows forever** (`src/client.rs:529` + `client/subscriptions.rs:256` `update_subscription`): every subscribe pushes into `Arc<Mutex<Vec<TagSubscription>>>`; nothing evicts. Add `unsubscribe` (public) and prune stopped entries in `update_subscription`.
5. **Fleet forwarding dies on `Lagged`** (`src/fleet.rs:102-113`): `while let Ok(event) = client_events.recv().await` exits on *any* error, but `broadcast::RecvError::Lagged(n)` is recoverable — `continue` (optionally surfacing a lag event). Also: `insert_client` replacing a PLC id leaves the old forwarding task alive emitting under the same id — shut it down (abort handle or a shutdown watch).
6. **`Client::events()` side effects** (`src/client/actor.rs:153-157`): subscribing *sends* a spurious `Connected` to all existing subscribers, even when the actor is dead. Make subscription observation-only; if a "current state" snapshot on subscribe is wanted, deliver it to the new subscriber only.
7. **`start_monitoring` leak — RESOLVED by CODEX-AQ (merged `272e0ae`), no action required.** AQ landed first and made `ProductionMonitor::start_monitoring` (`src/monitoring.rs:526`) a no-op that logs a deprecation warning and no longer spawns the 30 s-interval task; `ProductionMonitor` itself is now `#[deprecated]`. Confirm at implementation time that `monitoring.rs` still has no `tokio::spawn` in `start_monitoring` and leave it alone — this item exists only so the analysis §2 count stays traceable.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §2; `src/subscription.rs`, `src/client/subscriptions.rs`, `src/tag_group.rs` in full (the group path is the better sibling — reuse its patterns); `src/fleet.rs`; `src/client/actor.rs` (event vocabulary and who consumes it — `RetryClient`, C# wrapper?); `tests/subscription_tests.rs` + `tests/fleet_tests.rs` (existing conventions to extend).
- **Dependency state (both merged, refresh vs the original Fable-5 brief):** CODEX-AL (`253706e`) landed the shared-session-handle + stream-poison model — see its effect on item 3's error policy in the gotchas. CODEX-AQ (`272e0ae`) already fixed item 7 (`start_monitoring` leak) and deprecated `SubscriptionManager`/`ProductionMonitor` as types, so this task touches only the *live* subscription/fleet/event paths, never those deprecated types. AQ also added `Arc`-shared per-client `DiagnosticCounters` on the CIP send path; the new `unsubscribe`/prune work in item 4 is independent of it. Line references in this brief were refreshed against `main` at `537ccf1`; re-verify each before touching.

### Files to create or modify

`src/subscription.rs`, `src/client/subscriptions.rs`, `src/tag_group.rs`, `src/fleet.rs`, `src/client/actor.rs`, `src/client.rs` (unsubscribe + prune), FFI/wrapper surface only if a subscription FFI export exists (grep `eip_subscribe`; if none, note it), `tests/subscription_tests.rs`, `tests/fleet_tests.rs`, `CHANGELOG.md`. (`src/monitoring.rs` is *not* modified — item 7 is pre-resolved by CODEX-AQ; only confirm no `spawn` remains.)

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

- Fixes 1–6 with the tests above (item 7 pre-resolved by CODEX-AQ — verify-only); the abandoned-consumer test demonstrated deadlocking pre-fix (bounded-wait failure recorded in the log).
- No `send().await` on a notification channel from a poll/forwarding task anywhere in the touched files (grep).
- No public signature breaks (`unsubscribe` and the monitoring guard are additive; the guard's old return type — if `start_monitoring` returned `()` — additive-in-type via new method if needed to stay semver-clean; check with semver-checks).
- CHANGELOG entries; doc comments state the drop-oldest semantics.

### Out of scope

- Deprecating `SubscriptionManager`/`ProductionMonitor` as types — [[codex-aq-dead-stratum-deprecation]]. Reconnect policy in the actor (event vocabulary only gets the side-effect fix, not a health loop — that's a ROADMAP design question). Deadband/`change_threshold` semantics (documented in 1.1.0; unchanged).

### Risks and gotchas

- Drop-oldest on a `mpsc` channel isn't native — either wrap with a `try_send` + on-full `try_recv`-then-`try_send` dance (document the race window) or switch the notification channel to `tokio::sync::broadcast`/`watch` where lag semantics are built-in. `watch` (latest-value) may actually match subscription semantics best — evaluate and justify the choice in the log; don't silently change delivery guarantees beyond what the docs you write state.
- **CODEX-AL is merged (`253706e`), so this is live behavior now:** the poll task holds an `EipClient` clone; once the shared stream is poisoned, `ensure_stream_usable()` makes every subsequent poll fail fast with `ConnectionLost` (not a slow timeout). The transient-error policy (item 3) must therefore not hot-loop at `update_rate` against a dead connection — back off or emit a terminal event after N consecutive `ConnectionLost`s. Match on `is_retriable()`; note `ConnectionLost` *is* retriable per `error.rs`, so "retriable → continue" alone would hot-loop — add a consecutive-failure cap or backoff specifically for the poisoned-stream case.
- `insert_client` shutdown: the old task may be mid-`recv`; abort is safe there, but don't abort mid-send to consumers — prefer a cooperative shutdown flag checked per iteration, falling back to `abort()` after a grace timeout.
- Existing `subscription_tests.rs` may encode the old blocking-delivery behavior — rewrite those tests deliberately and say so in the log, don't contort the fix to keep them green.

## Codex log

## Claude review

## Verdict
