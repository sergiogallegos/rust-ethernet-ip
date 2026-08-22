---
id: CODEX-BD
title: Schema-change simulator and real-hardware validation gate
owner: codex
status: open
created: 2026-08-22
last-update: 2026-08-22 codex [GPT-5]
---

## Brief

### Priority and dependency

**Blocks 1.2.1 validation. Depends on CODEX-BA, CODEX-BB, and CODEX-BC.**

Create a repeatable validation gate for schema changes that occur without an
application restart, then execute the hardware portion with maintainer control
of Studio 5000 and the dedicated test controller.

### Required implementation

1. Extend the simulator or a deterministic test stream so a tag can disappear
   and reappear under the same symbolic name with a different datatype/shape.
2. Add an offline cross-binding runner that proves explicit refresh behavior
   in Rust, C#, Python, and C/C++.
3. Add an opt-in live procedure for the 1756-L75 firmware 33 through the
   1756-EN2T route covering:
   - online temporary-tag replacement under the original name;
   - ordinary array to packed BOOL and the reverse;
   - indices below and above 32;
   - program and controller scope;
   - offline UDT member/layout update and download;
   - whether the encapsulation session survives each change.
4. Require explicit write opt-in, dedicated tags, starting-value capture, and
   restoration. The runner must never edit PLC schema itself.
5. Record automatic read recovery, explicit refresh in all bindings, UDT
   rediscovery, errors, retries, and final controller state.

### Test requirements

- Deterministic simulator tests pass without PLC access.
- All four bindings use one release native artifact.
- Live validation records exact processor, firmware, bridge/route, host, build,
  and controller edit sequence without publishing the PLC address.
- No write is duplicated or sent using stale packed-BOOL addressing.
- Existing full-coverage and batch baselines remain green after the schema test.

### Acceptance criteria

- A dated validation record exists under `docs/validation/`.
- Hardware matrix and release notes link the result without generalizing beyond
  the exact controller/firmware/topology.
- The cache-lifecycle wiki and append-only wiki log are updated.
- The controller is restored or its intentional final test state is recorded.

### Out of scope

- Automated Studio 5000 project manipulation.
- Production-controller writes.
- Performance packet tuning (CODEX-BE).

## Codex log

## Claude review

## Verdict
