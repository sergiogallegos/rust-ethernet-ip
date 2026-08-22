---
id: CODEX-BH
title: Tag-shape and scope performance matrix
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Post-1.2.1 characterization follow-up; not a release blocker. Run after
CODEX-BE establishes the packet policy used by the matrix.**

The current 3,305 tags/s result is a repeated controller-scoped DINT-array
element workload. Characterize which parts generalize to other tag shapes,
paths, scopes, and payload sizes.

### Required work

1. Define a shared manifest covering:
   - independent atomic scalars versus elements of one array;
   - controller versus program scope;
   - short versus long symbolic paths;
   - DINT, REAL, packed BOOL, built-in/custom STRING;
   - UDT members, whole UDT reads, and fragmented structures.
2. Measure cold-cache and warm-cache phases separately.
3. Measure single and batch read/write behavior where semantically supported.
4. Report latency distributions, throughput, packet/byte counts, failures,
   host CPU/RSS, and cache diagnostics for Rust, C#, Python, and C/C++ using one
   release native artifact.
5. Preserve exact controller/firmware/route context and do not combine unlike
   workloads into one headline throughput number.

### Acceptance criteria

- Every matrix row has a precise workload definition and support status.
- Cold/warm cache and native/sequential wrapper paths are visibly distinct.
- Results are reproducible from committed runner options and manifest data.
- Website and README may summarize only evidence linked to the full dated
  validation record.

### Out of scope

- Declaring universal performance from one processor.
- Changing packet defaults during this task.
- Whole-UDT write support beyond currently safe APIs.

## Codex log

## Claude review

## Verdict
