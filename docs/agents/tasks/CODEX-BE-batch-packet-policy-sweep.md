---
id: CODEX-BE
title: Batch packet-policy characterization and safe tuning
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Post-1.2.1 performance follow-up; not a release blocker. Depends on the
CODEX-BA…BD safety sequence for final live runs.**

CODEX-AW already made batch grouping honor both operation count and encoded
packet bytes. This task must not reimplement that work. Characterize whether
the conservative default of 20 operations/504 bytes should remain universal or
whether negotiated/controller-specific policies can safely reduce round trips.

### Required work

1. Add benchmark parameters for maximum operations and batch packet bytes.
2. Sweep operation limits 10, 20, 30, and 50 and packet limits 504, 1,000,
   2,000, and negotiated maximum where the route accepts them.
3. Measure logical sizes 20, 50, 100, 200, and 500, read-only first and writes
   only with explicit opt-in.
4. Record raw and Tukey-filtered latency, p50/p95/p99/max, tags/s, calls/s,
   packet count, bytes, failures, and controller/host impact when available.
5. Test routed ControlLogix and direct CompactLogix before recommending any
   default change. Rejecting larger packets is evidence, not a test failure.
6. If adaptive policy is justified, make it opt-in or capability-driven and
   preserve 504 bytes as the safe fallback.

### Acceptance criteria

- A dated, reproducible characterization record preserves all tested and
  rejected combinations.
- No default changes from one controller result alone.
- Any code change has simulator/unit coverage and full cross-binding gates.
- Documentation separates negotiated connection size from safe routed MSP
  payload policy.

### Out of scope

- Parallel requests through one client.
- Bypassing CODEX-AW byte accounting.
- Publishing a universal throughput promise.

## Codex log

## Claude review

## Verdict
