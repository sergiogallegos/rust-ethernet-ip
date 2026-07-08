---
id: CODEX-AR
title: Subscription, fleet, and event lifecycle — stop() stops, no blocked poll tasks, lag-tolerant forwarding
owner: codex
status: merged
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

2026-07-07 codex [GPT-5] — Implementation submitted.

- Fixed single-tag subscription lifecycle: poll loops now check
  `TagSubscription::is_active()`, `stop()` halts polling within the configured
  interval, `EipClient::unsubscribe()` stops and evicts matching subscriptions,
  and `update_subscription()` prunes inactive registry entries.
- Replaced subscription/tag-group notification `send().await` with bounded
  `try_send` + drop-oldest delivery so abandoned consumers cannot permanently
  block poll tasks. Kept the existing value stream API and added
  `TagSubscriptionEvent` / `wait_for_event()` / `into_event_stream()` for
  value-or-error observation.
- Single-tag polling now emits nonterminal error events for retriable failures,
  backs off at least 250 ms on retriable failures, and stops after 3 consecutive
  `ConnectionLost` failures to avoid hot-looping against AL-poisoned streams.
  CIP general statuses `0x01` and `0x07` now map to `Connection` /
  `ConnectionLost` so connection-failure replies participate in the existing
  retriable policy.
- Fixed fleet forwarding: `Lagged` is tolerated, per-id forwarder handles are
  owned and aborted on replacement/drop, replacement no longer leaves old
  forwarders ghost-emitting under the same PLC id, and `Client::events()` is
  observation-only.
- Confirmed `src/monitoring.rs` item 7 remains AQ-resolved:
  `start_monitoring()` logs a deprecation warning and contains no spawn.
- FFI/wrapper check: `rg eip_subscribe` found no native subscription FFI export,
  so no C/Python/C# subscription surface needed updating.
- Verification passed:
  `cargo fmt -- --check`;
  `cargo clippy --workspace --all-targets --all-features -- -D warnings`;
  `cargo test --test subscription_tests --test fleet_tests --test client_actor_tests --locked`;
  `cargo test fleet::tests::forward_events_loop_continues_after_lagged_source --locked`;
  `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked`;
  `cargo test --test plc_sim_tests --locked`;
  `cargo semver-checks check-release --baseline-version 1.1.0`.

## Claude review

### 2026-07-07 21:10  claude [Opus 4.8]

**Independent verification**
- `cargo fmt -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `cargo test --test subscription_tests --test fleet_tests --test client_actor_tests --locked` — 7 / 5 / 9 passed (0 failed).
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 0 failed across the workspace.
- `cargo test --test plc_sim_tests --locked` — 24/24.
- `cargo semver-checks check-release --baseline-version 1.1.0` — 220 pass, "no semver update required" (new `unsubscribe`/event API is additive).
- Not FFI-touching (`ffi.rs` untouched, `rg eip_subscribe` empty, no C#/Python wrapper change) — C#/Python suites not required.

**What's being fixed**
- Task-lifecycle defects in the live subscription/fleet/event paths: `stop()` that didn't stop, poll tasks an abandoned consumer could deadlock, subscriptions killed by the first transient error, an ever-growing registry, fleet forwarding that died on `Lagged` and ghost-emitted after client replacement, and a side-effecting `events()`.

**Root cause confirmation**
- Confirmed per item. (1) Single-tag loop now `while subscription_task.is_active()` with a guard before the inter-poll sleep (`client/subscriptions.rs:47,77`) — stop halts within one interval. (2) Both the value channel and event channel use `try_send_drop_oldest` (`subscription.rs:273`), and `tag_group::publish_event` (`:160`) too — grep confirms no blocking `send().await` on any notification channel in the touched files; the subscription owns its receiver so a dropped consumer never closes the channel and the loop keeps running. (3) On read error the loop publishes an error event, tracks consecutive `ConnectionLost`, and goes terminal on `!is_retriable() || consecutive >= 3` with a ≥250 ms backoff otherwise (`subscriptions.rs:57-75`) — this closes the poison hot-loop I flagged in the brief refresh (`ConnectionLost` is retriable, so the cap is what prevents the spin). (4) `unsubscribe` (public) + `retain(is_active)` in `unsubscribe`/`subscription_count`/`update_subscription` (`subscriptions.rs:89,256`). (5) `Fleet` owns `forwarders: HashMap<PlcId, JoinHandle>`, aborts the old handle on `insert_client` replace (`fleet.rs:69`) and all handles on `Drop`; `forward_events_loop` (`:131`) does `Lagged => continue`, `Closed => break`. (6) `events()` is now just `self.events.subscribe()` (`actor.rs:153`).

**Fix appropriateness**
- Right layer, additive surface, SemVer-clean. The fleet `Connected` handling is a deliberate, coherent design: `insert_client` emits exactly one deterministic synthetic `Connected` FleetEvent per insertion (`fleet.rs:74`), and the forward loop suppresses the actor's own one-time startup `Connected` (`:140`) to avoid a duplicate — terminal `Disconnected`/`WorkerStopped` still forward, which is what a fleet consumer needs. The single-shot actor has no reconnect path, so no meaningful `Connected` is dropped. The CIP `0x01`/`0x07` → `Connection`/`ConnectionLost` mapping is an in-scope addition that lets connection-failure replies feed the item-3 retriable policy.

**Test proof**
- Deterministic, sim-backed, bounded-wait tests for every item: `abandoned_subscription_consumer_does_not_block_polling` (subscribes at `update_rate:1`, never consumes, asserts `sim.read_count > 110` within a 2 s timeout — past the 100-slot capacity where the pre-fix code deadlocks), `stop_halts_single_tag_polling`, `transient_single_tag_error_is_reported_and_recovers`, `unsubscribe_stops_and_evicts_subscription`, `direct_subscription_updates_drop_oldest_under_backpressure`, `subscribing_to_events_has_no_side_effect_on_existing_subscribers`, `forward_events_loop_continues_after_lagged_source`, `replacing_client_aborts_old_forwarder` (asserts no terminal event under the old id after replacement). The sim gained a `read_count` hook to make polling progress observable without sleep-as-sync.

**Residual risk**
- Sim/unit-level only; no hardware run (none needed — these are task-lifecycle fixes, not wire changes).
- The narrow concurrent-last-drop race in the event-channel map (see Findings) can leak a single bounded map entry; not a per-poll leak.

**Strong points (✅)**
- The poison hot-loop is handled exactly where the brief warned: retriable `ConnectionLost` is bounded by a consecutive-failure cap, not treated as "retry forever."
- Fleet forwarders are owned and cancelled on both replace and `Drop` — no orphan tasks; the replace-abort is proven by an assert-absence test.
- `try_send_drop_oldest` is a single reused primitive across value, event, and tag-group channels — one place to reason about backpressure.
- Delivery-semantics change (block→drop-oldest) is documented on the public methods, honestly framed as "the old behavior was deadlock, not delivery."

**Findings**
- 🟡 The per-subscription event channels live in a global `TAG_SUBSCRIPTION_EVENTS: HashMap<usize, …>` keyed by `Arc::as_ptr(&is_active)` (`subscription.rs:68,258`), evicted in `Drop` when `strong_count == 1`. It works and is bounded by *active* subscriptions, but it's heavier than adding `event_sender`/`event_receiver` `Arc<Mutex<…>>` fields to the struct exactly like the existing value channels — and it carries a narrow race: two clones of one subscription dropped concurrently on different threads can each read `strong_count > 1` (before the other's Arc field-drop decrements) and both skip removal, leaking one stale entry. Non-blocking (bounded, single entry); recommend a follow-up to move the channels onto the struct or evict explicitly on `stop()`/`unsubscribe`. Not a merge-time fix — it's an architectural change, not ≤5 lines.
- 🟢 `try_send_drop_oldest` under receiver-lock contention drops the *new* value (drop-newest in that window) rather than strictly drop-oldest — matches the "drop where possible" wording.
- 🟢 `events()` delivers no current-state snapshot on subscribe, so a late subscriber won't observe the one-time startup `Connected` (broadcast semantics) — within brief scope (snapshot was optional).
- 🟢 Fleet emits one deterministic synthetic `Connected` per insert and dedups the actor's own — coherent, verified against `replacing_client_aborts_old_forwarder`.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Fixes 1–6 with the tests above (item 7 pre-resolved by AQ — confirmed `start_monitoring` has no `spawn`).
- ✅ Abandoned-consumer test demonstrates progress past channel capacity under a bounded wait (pre-fix deadlock avoided).
- ✅ No `send().await` on a notification channel from any poll/forwarding task in the touched files (grep clean).
- ✅ No public signature breaks — `unsubscribe` + event API additive; `cargo semver-checks` green.
- ✅ CHANGELOG entries present; drop-oldest semantics documented on the public types.

## Verdict

### 2026-07-07 21:10  claude [Opus 4.8]

**Merged.** Full independent matrix green (fmt, clippy `--all-targets --all-features -D warnings`, subscription/fleet/actor suites 7/5/9, workspace `--all-features --locked`, plc_sim 24/24, semver-checks 220-pass/no-update). All six live-path items are fixed with deterministic, bounded-wait, sim-backed tests — including the abandoned-consumer case that deadlocks pre-fix. The item-3 error policy correctly bounds retriable `ConnectionLost` with a consecutive-failure cap, closing the poison hot-loop the refreshed brief warned about, and the fleet forwarder ownership/abort model is proven by assert-absence tests. Item 7 was pre-resolved by AQ and confirmed untouched. One 🟡 (the global `Arc`-pointer-keyed event-channel map is heavier than struct fields and has a narrow concurrent-last-drop leak race) is non-blocking and left as a documented follow-up — bounded to a single stale entry, and refactoring it is an architectural change out of scope for a merge-time fix. Zero defects, zero Claude-applied fixes. Merged at `f5c895c`.
