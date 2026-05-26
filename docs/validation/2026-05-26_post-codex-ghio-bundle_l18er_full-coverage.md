# 2026-05-26 Post-CODEX-G/H/I/O hardware re-validation — CompactLogix L18ER

Date: 2026-05-26
Tester: Claude + Sergio Gallegos (maintainer-owned hardware)
Library version: `1.0.0` (`main` at `d0e6f20`; library code at `2690669`)
Trigger: re-validate after the CODEX-G/H/I/O bundle merged at `2690669` to clear the hardware gate before any 1.0.1 publish — specifically the CODEX-O wire change (type-prefixed UDT writes now emit `0x02A0 + symbol_id` instead of the `0x00A0` placeholder).

## Scope

Single-controller hardware re-run against the **CompactLogix `1769-L18ER-BB1B`** firmware v33, slot 0, direct (integrated CPU + ethernet), at `192.168.0.1:44818`. The ControlLogix L75 at `10.136.15.20` was not on the network at run time; the L18ER repeats the same 2299-tag manifest that previously matched the L75 byte-for-byte (per [`2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md`](2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md)), so L18ER alone is sufficient for the CODEX-O regression-gate purpose. Cross-controller re-validation against the L75 stays queued for the next time both controllers are bench-available.

## Commands Executed

From repo root:

```bash
cargo build --release --features ffi --locked

TEST_PLC_ADDRESS=192.168.0.1:44818 TEST_PLC_SLOT=0 \
  cargo run --release --example test_plc_full_coverage

TEST_PLC_ADDRESS=192.168.0.1:44818 TEST_PLC_SLOT=0 \
  dotnet run --project examples/CSharpFullCoverage -c Release

TEST_PLC_ADDRESS=192.168.0.1:44818 TEST_PLC_SLOT=0 PYTHONPATH=python \
  python3 python/examples/test_plc_full_coverage.py
```

## Result

All three bindings PASSED with byte-identical per-phase counts. Numbers match the 2026-05-25 baseline exactly.

| Binding | Reads | Writes | Verify | Blocked | Anomalies | Result |
|---|---|---|---|---|---|---|
| Rust | 2299/2299 | 2206/2206 | 2206/2206 | 60 | 0 | PASS |
| C# | 2299/2299 | 2206/2206 | 2206/2206 | 60 | 0 | PASS |
| Python | 2299/2299 | 2206/2206 | 2206/2206 | 60 | 0 | PASS |

JSON result artifacts (gitignored, kept locally under `examples/full_coverage_results/`):

- `rust_1779763774.json`
- `csharp_20260526T025400Z.json`
- `python_20260526T025629Z.json`

## What this gate confirms about the CODEX-O wire change

`crates/protocol/src/values.rs::write_data_type` and `encode_type_prefixed` now route UDT type-prefixes through `PlcValue::known_data_type()`, emitting `0x02A0 + symbol_id` instead of the `0x00A0` placeholder. The exerciser exercises every UDT write path the manifest defines — controller-scope UDT members, program-scope UDT members, nested UDT arrays, top-level UDT writes (firmware-blocked), UDT-array element member writes (firmware-blocked). Identical pass counts on this re-run confirm:

- Common UDT RMW (read → mutate → write back with captured `symbol_id`): unchanged on the wire, still accepted (4/4 UDT_members per scope, 35/35 UDT_nested per scope, 350/50 UDTarr_elem_nested per scope).
- The 60 firmware-blocked rejections still come back as `0x2107` — the wire change did not move any path from "blocked" to "writeable" or vice versa.
- Zero unexpected anomalies across 2299 read targets and 2206 write targets per binding.

## Patch-release gate status

Hardware re-validation gate for the CODEX-O wire change is **cleared on the L18ER**. The 1.0.1 publish can proceed once:

- The maintainer accumulates whatever else they want to roll into 1.0.1 (per user direction: accumulate more before cutting).
- A second controller is re-validated if available (L75 was not on the network for this run).

## Library version snapshot

- Rust: `rust-ethernet-ip 1.0.0` (workspace, `Cargo.toml` at `1.0.0`)
- C#: `RustEtherNetIp 1.0.0` (built locally from current `main`)
- Python: `rust-ethernet-ip 1.0.0` (in-repo wrapper)

All three back-ended by the same `target/release/librust_ethernet_ip.dylib` built from `main` head `d0e6f20` (library code is at `2690669`).
