# 1756-L75 Firmware 33 Schema-Change Gate

Status: **offline gate passed; live Studio 5000 execution pending**

This record is deliberately not a hardware PASS yet. It prepares the exact,
maintainer-controlled validation required before the 1.2.1 release and will be
completed during the live session.

## Target

- Processor: ControlLogix `1756-L75`
- Processor firmware: major revision 33 (full revision to capture live)
- Chassis: four slots; CPU in slot 0, `1756-EN2T` bridge in slot 1
- Route: bridge TCP endpoint to backplane slot 0
- Bridge firmware: pending live capture
- Host: Apple MacBook Pro, Apple M2; macOS version pending live capture
- Library: `1.2.1` development line, C ABI v3
- Commit/build identity: pending final live run
- PLC address: intentionally omitted

## Offline Result — 2026-08-22

Command:

```bash
scripts/schema-change-gate
```

One `cargo build --release --features ffi --locked` artifact backed every
wrapper. Results:

| Surface | Result | Evidence exercised |
|---|---|---|
| Rust | PASS | 7 dynamic simulator tests: DINT/BOOL/REAL transitions, delete/recreate, controller/program scope, indices 5/40, batch correlation, no write replay |
| C ABI | PASS | refresh success, clone-visible generation, invalid handle and last-error |
| C# | PASS | refresh advances diagnostics generation and refresh count through the simulator |
| Python | PASS | refresh advances diagnostics generation and refresh count through the simulator |
| C/C++ | PASS | header/export parity (60 symbols) and C++ refresh-generation smoke |

No proprietary tag names or values are emitted by schema-cache diagnostics.

## Live Checklist

Follow [SCHEMA_CHANGE_GATE.md](SCHEMA_CHANGE_GATE.md). Complete one row only
after capturing the starting values, session behavior, counters, retry count,
write count, and restoration result.

| Scenario | Rust | C# | Python | C/C++ | Session survived? | Restored? |
|---|---|---|---|---|---|---|
| Controller DINT[64] -> BOOL[64], indices 5/40 | pending | pending | pending | pending | pending | pending |
| Controller BOOL[64] -> DINT[64], indices 5/40 | pending | pending | pending | pending | pending | pending |
| Program DINT[64] -> BOOL[64], indices 5/40 | pending | pending | pending | pending | pending | pending |
| Program BOOL[64] -> DINT[64], indices 5/40 | pending | pending | pending | pending | pending | pending |
| UDT layout edit + download + rediscovery | pending | pending | pending | pending | pending | pending |
| Post-schema full coverage and batch baseline | pending | pending | pending | pending | n/a | pending |

## Final Controller State

Pending live execution. The live session must record either successful restore
to the backed-up fixture or an explicitly accepted final test-only schema and
its values. Until then, this file must not be cited as hardware compatibility
evidence for schema-change recovery.

