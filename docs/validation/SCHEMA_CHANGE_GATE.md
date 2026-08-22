# Controller Schema-Change Validation Gate

Use this gate on a dedicated development controller when validating a release
that changes tag metadata, packed-BOOL classification, STRING handles, or UDT
definition caching. The software never edits controller schema; the maintainer
performs every Studio 5000 action and confirms each transition.

## Offline Gate

Run before touching hardware:

```bash
scripts/schema-change-gate
```

It builds one release FFI artifact and verifies Rust dynamic schema recovery,
C ABI generation visibility, C# and Python refresh diagnostics, C header/export
parity, and C++ refresh-generation behavior. It does not contact a real PLC.

## Live Gate Companion (Rust)

`examples/schema_change_gate_live.rs` automates the repeatable, non-editing
steps of the "Per-Binding Online Replacement" procedure below for the Rust
binding: connect, capture a baseline, warm-read both scopes/indices, an
optional restore-safe pre-edit write smoke check, a pause for the maintainer's
Studio 5000 action, post-edit reads with automatic-recovery counters, explicit
`refresh_schema()`, rediscovery, post-refresh reads, and an optional
restore-safe post-refresh write/verify. It never edits controller schema and
prints a result block to paste into the dated validation record.

```bash
cargo run --release --example schema_change_gate_live -- --dry-run
cargo run --release --example schema_change_gate_live -- --allow-writes
```

Defaults come from `TEST_PLC_ADDRESS`, `TEST_PLC_SLOT`, and `TEST_PLC_PROGRAM`
(see the table in `CLAUDE.md`); override with `--plc-address`, `--plc-slot`,
`--program`, or `--tag` (defaults to `gSchemaSwap`). Run it once per Studio
5000 edit — twice per scope pair to cover both shape directions — and run the
UDT layout/download section by hand, since that section changes the
connection's own session lifecycle and is not yet automated. C#, Python, and
C++ companions remain manual per the steps below.

## Safety Preconditions

- Use only a development controller or an approved maintenance window.
- Back up the controller project first.
- Do not use safety, motion, I/O, recipe, production, or externally referenced
  tags.
- Require an explicit `--allow-writes` decision before any value write.
- Capture every starting value and restore it after each binding. A schema edit
  itself is always manual in Studio 5000.
- Keep Rust, C#, Python, and C/C++ processes on the same release native build.
- Record the processor, full firmware, bridge and firmware, chassis route,
  host/architecture, commit, and native ABI. Do not publish the PLC address.

## Dedicated Fixture

Create equivalent controller- and program-scoped fixtures in `TestProgram`:

- `gSchemaSwap`: 64-element array, initially `DINT[64]`;
- `gSchemaSwapReplacement`: 64-element `BOOL[64]`, used only during the
  maintainer-controlled online replacement;
- `gSchemaUdt`: instance of a dedicated `SchemaGateUdt` containing a DINT
  marker and a BOOL array of at least 64 elements.

Use distinctive values at indices 5 and 40. Confirm that no controller logic
references either swap tag before deletion/rename. Repeat the gate with the
types reversed so both `DINT[] -> BOOL[]` and `BOOL[] -> DINT[]` are observed.

## Per-Binding Online Replacement

Repeat the complete sequence independently for Rust, C#, Python, and C/C++ so
each binding holds an open session and warm classification cache:

1. Connect through the intended route and record the encapsulation session
   state plus schema generation.
2. Read controller and program `gSchemaSwap[5]` and `[40]` twice. Record cache
   hit/miss counters and values.
3. If writes are explicitly enabled, capture both starting values, write only
   the dedicated elements, verify once, and restore before continuing.
4. Pause all application writes. Keep the client process and connection alive.
5. In Studio 5000, perform the normal online replacement: move any test-only
   references away, delete the unused original, and rename the replacement to
   `gSchemaSwap`. Do not let the runner manipulate schema.
6. Read indices 5 and 40 without calling refresh first. Record the first error,
   contradiction counter, recovery count, final value, and whether the same
   encapsulation session survived. Automatic recovery must perform at most one
   logical retry.
7. Call the binding's explicit refresh operation:
   - Rust: `client.refresh_schema().await`;
   - C: `eip_refresh_schema(client_id)`;
   - C#: `client.RefreshSchema()`;
   - Python: `client.refresh_schema()`;
   - C++: `client.refreshSchema()`.
8. Confirm generation and refresh count advance by one. Rediscover tags where
   that binding exposes discovery, then read both scopes/indices again.
9. With explicit write approval, write and read back only the dedicated
   elements. Verify the request uses the new addressing shape and that exactly
   one write was sent. Restore captured values.
10. Reverse the tag shapes and repeat steps 4–9.

## Offline UDT Edit and Download

1. Warm `gSchemaUdt` definition/template caches in every binding, then pause
   writes while leaving clients open.
2. Go offline in Studio 5000, add or reorder a dedicated non-I/O member in
   `SchemaGateUdt`, and download the project.
3. Record whether each TCP/encapsulation session was closed, remained usable,
   or required reconnecting. Do not infer one binding's result for another.
4. Call explicit schema refresh after the download/reconnect, rediscover the
   UDT, and verify that the new layout/handle is observed.
5. Restore the backed-up UDT definition or record the intentional final test
   fixture. Run refresh and rediscovery again after restoration.

## Pass Criteria

- Both shape directions pass in controller and program scope at indices 5 and
  40 in all four bindings.
- Automatic read recovery is bounded and observable; no unrelated batch result
  moves position.
- Explicit refresh advances the same native generation exposed to each wrapper.
- No write is duplicated, replayed after ambiguity, or sent using stale
  packed-BOOL DWORD addressing.
- UDT rediscovery matches the downloaded layout after refresh.
- Starting values and the intended controller schema are restored and verified.
- The ordinary full-coverage and batch feature gates still pass afterward.

