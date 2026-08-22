---
id: CODEX-BG
title: Cross-binding one-hour and 24-hour endurance soak
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Post-1.2.1 validation follow-up; not a release blocker unless the maintainer
promotes it. Depends on CODEX-BD so schema diagnostics are available.**

Build a resumable endurance runner and record the first long-running physical
PLC evidence without reducing the result to “still connected.”

### Required work

1. Add one-hour shakedown profiles for all four bindings using the same release
   native artifact.
2. Add a 24-hour native-core read soak with 100 ms, 500 ms, and 1 s tag groups;
   make 24-hour wrapper runs selectable for later contributions rather than
   requiring four consecutive days for the first record.
3. Record operation/success/failure counts, maximum consecutive failures,
   data gaps, reconnect count/duration, p50/p95/p99/max, cache diagnostics,
   process CPU/RSS, and controller communication/task observations.
4. Make output incremental and resumable so an interrupted run retains valid
   evidence. Include monotonic timestamps and periodic checkpoints.
5. Provide read-only default plus a separately authorized read/write profile
   with dedicated tags and restoration.

### Acceptance criteria

- One-hour all-binding shakedown and one 24-hour read result are traceably
  recorded for an exact controller/firmware/topology.
- No silent data-gap interval is omitted from the report.
- Resource trends and reconnects are quantified.
- Hardware matrix distinguishes endurance evidence from functional `Done`.
- Contributor instructions explain how to submit additional wrapper/PLC soaks.

### Out of scope

- Production equipment or safety/output tags.
- Treating one 24-hour run as proof for a controller family.
- Load generation that exceeds an agreed controller utilization ceiling.

## Codex log

## Claude review

## Verdict
