---
id: CODEX-AW
title: Batch grouping ignores max_packet_size — large batch reads fail with EIP 0x65 (Invalid Length)
owner: codex
status: open
created: 2026-07-08
last-update: 2026-07-08 claude [Opus 4.8]
---

## Brief

### Goal

Hardware validation on 2026-07-08 (5069-L330ERM fw38, all bindings) found that a batch **read**
of ≥~20 tags with realistic path lengths fails wholesale with EtherNet/IP encapsulation status
`0x65` (Invalid Length). Reproduced with `gTestArray_DINT[0..N]`: `N=5` and `N=10` succeed;
`N=20, 25, 40, 50` fail and **every tag in the batch reports the error** (the controller
rejects the entire Multiple Service Packet, not individual services). See
[`docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`](../../validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md)
finding 1.

Root cause: `optimize_operation_groups` and `sequential_operation_groups`
(`src/client/batch_exec.rs:343` and `:370`) chunk operations **only** by
`BatchConfig::max_operations_per_packet` (default 20) and completely ignore
`BatchConfig::max_packet_size` (default 504 bytes). Twenty read services for `gTestArray_DINT[N]`
paths exceed 504 bytes in one MSP, so the request overruns what this controller accepts and it
rejects the whole packet. Batch **writes** happen to stay under the limit for the tested tags,
but they share the same unbounded grouping and are equally exposed with longer paths or larger
values.

The library advertises `max_packet_size` in its public `BatchConfig` (with a doc comment about
"PLC packet size limits") but never enforces it. Fix that: group by a byte budget as well as an
operation count, and **split** oversized batches into more packets rather than emitting one
oversized packet that fails.

### Context to read first

- `src/client/batch_exec.rs` — `optimize_operation_groups` (`:343`), `sequential_operation_groups`
  (`:370`), and `execute_operation_group` (`:381`, the MSP builder) — the per-service request
  encoding there is the ground truth for the byte estimate. Reuse it; do not invent a second
  size formula that can drift from the encoder.
- `src/batch.rs` — `BatchConfig` (`max_operations_per_packet`, `max_packet_size`) and their doc
  comments; the defaults are 20 / 504.
- [`docs/agents/notes/cip-framing.md`](../notes/cip-framing.md) and
  [`docs/agents/notes/unconnected-send.md`](../notes/unconnected-send.md) — MSP framing and the
  Unconnected Send envelope the group is wrapped in; the byte budget must leave room for the MSP
  header (service `0x0A`, Message Router path), the service-count + offset table (`2 + 2*n`
  bytes), and the surrounding envelope, not just the sum of per-service request bytes.
- The 2026-07-02 STRING probe note in `docs/validation/` records a related MSP attribution gap
  (a nonzero MSP-level status being blamed on the whole batch) — relevant background, not this
  task's fix.

### Files to create or modify

`src/client/batch_exec.rs` (grouping logic; likely a shared `estimated_request_len(op) ->
usize` helper derived from the existing encoder), and a test file — extend
`tests/batch_operations_tests.rs` or the simulator-backed batch tests. `src/batch.rs` only if a
config/doc tweak is warranted. No public API signature change.

### Behavior

- Grouping accumulates operations into a packet until **either** `max_operations_per_packet`
  **or** `max_packet_size` (measured as the full request the group will emit, including MSP
  header + offset table + envelope headroom) would be exceeded, then starts a new packet. Both
  the optimized (reads/writes separated) and sequential paths respect the byte budget.
- A batch of any size succeeds by splitting into as many packets as needed; no single packet
  exceeds `max_packet_size`. A single operation that alone exceeds the budget still goes out in
  its own packet (don't drop it, don't infinite-loop).
- If a group is still rejected (genuinely too large for the controller despite the budget), the
  error is surfaced per the existing `continue_on_error` contract — but the default `504`-byte
  budget must make the validated `gTestArray_DINT[0..50]` read succeed.

### Test requirements

- A simulator-backed test that issues a batch read of ≥40 long-named tags and asserts (a) it
  succeeds and returns all values, and (b) it was split into >1 packet (assert on group count
  via a unit test of the grouping function, which is the deterministic, hardware-free surface).
- A unit test on the grouping function directly: feed N operations with known encoded sizes and
  assert the resulting groups each fit `max_packet_size` and honor `max_operations_per_packet`,
  including the single-oversized-operation edge case.
- Full matrix: `cargo fmt -- --check`, `cargo clippy -- -D warnings`,
  `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`.
- Note in the Codex log the byte-budget accounting you chose (what headroom you reserved for MSP
  header + offset table + Unconnected Send envelope) so review can check it against the encoder.

### Acceptance criteria

- `optimize_operation_groups` and `sequential_operation_groups` enforce both the op-count and
  the byte budget; oversized batches split instead of failing.
- New grouping unit test + simulator batch-read test pass; existing batch tests stay green.
- Hardware re-validation (maintainer, next PLC session): `gTestArray_DINT[0..50]` batch read
  succeeds across Rust/Python/C#/C++. Record it against this task.

### Out of scope

- The read-vs-write batching **throughput** asymmetry (batch read ~1.1 ms/tag vs batch write
  ~89 µs/tag) — investigate separately if it persists after correct splitting; note it in the
  log but do not chase it here.
- Connected (Class 3) messaging or Large Forward Open to raise the packet ceiling — a
  larger, separate capability.
- Per-service MSP error attribution (the `0x1E`/embedded-status gap) — CODEX-AN territory.

### Risks / gotchas

- The byte estimate must come from the same encoder `execute_operation_group` uses, or it will
  drift and either over-split (slow) or under-split (0x65 returns). Prefer building the request
  bytes and measuring, or a helper both call.
- Watch the off-by-one at exactly 20 ops / 504 bytes: the validated boundary is `N=10` ok /
  `N=20` fail for `gTestArray_DINT[N]`; a correct fix makes `N=20` split and succeed.
- Don't regress the small-batch fast path — a 3-tag batch must still go out as one packet.

## Codex log

## Claude review

## Verdict
