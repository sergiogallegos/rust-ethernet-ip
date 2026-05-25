# 2026-05-25 Cross-binding full-coverage validation — two real controllers

Date: 2026-05-25
Tester: Codex + Claude + Sergio Gallegos
Library version: `1.0.0` (`main` at `9de4915` at start, `c02ff6c`-onward post-CODEX-AE merge `59a2176`)

## Scope

First full-coverage hardware validation against **two** real Allen-Bradley controllers in the same day, using the unified `examples/full_coverage_tags.json` manifest (landed in CODEX-AE / `59a2176`) consumed by all three language bindings.

Validates that the 1.0.0 library line behaves identically on:

- **ControlLogix `1756-L75`** firmware v33, slot 0, routed through a `1756-EN2T` ethernet bridge at `10.136.15.20:44818`
- **CompactLogix `1769-L18ER-BB1B`** firmware v33, slot 0, direct (integrated CPU + ethernet), at `192.168.0.1:44818`

Both PLCs carry the same tag set per `docs/PLC_TEST_TAG_DEFINITIONS.md` (2299 distinct tag targets after manifest expansion).

## Commands Executed

For each binding, run from repo root:

```bash
# ControlLogix L75
TEST_PLC_ADDRESS=10.136.15.20:44818 TEST_PLC_SLOT=0 cargo run --release --example test_plc_full_coverage
TEST_PLC_ADDRESS=10.136.15.20:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpFullCoverage -c Release
TEST_PLC_ADDRESS=10.136.15.20:44818 TEST_PLC_SLOT=0 python3 python/examples/test_plc_full_coverage.py

# CompactLogix L18ER
TEST_PLC_ADDRESS=192.168.0.1:44818  TEST_PLC_SLOT=0 cargo run --release --example test_plc_full_coverage
TEST_PLC_ADDRESS=192.168.0.1:44818  TEST_PLC_SLOT=0 dotnet run --project examples/CSharpFullCoverage -c Release
TEST_PLC_ADDRESS=192.168.0.1:44818  TEST_PLC_SLOT=0 python3 python/examples/test_plc_full_coverage.py
```

## Result

All six runs PASSED with byte-identical per-phase counts.

| Controller | Binding | Preflight | Reads | Writes | Verify | Phase 6 | Blocked | Anomalies | Result |
|---|---|---|---|---|---|---|---|---|---|
| 1756-L75 (CtlLgx) | Rust | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |
| 1756-L75 (CtlLgx) | C# | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |
| 1756-L75 (CtlLgx) | Python | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |
| 1769-L18ER (CmpLgx) | Rust | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |
| 1769-L18ER (CmpLgx) | C# | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |
| 1769-L18ER (CmpLgx) | Python | 2299/2299 | 2299/2299 | 2206/2206 | 2206/2206 | 14/14 | 60 | 0 | PASS |

JSON result artifacts (gitignored, kept locally under `examples/full_coverage_results/`):

- `rust_1779738567.json`, `csharp_20260525T195416Z.json`, `python_20260525T195548Z.json` — ControlLogix L75
- `rust_1779741541.json`, `csharp_20260525T204204Z.json`, `python_20260525T204437Z.json` — CompactLogix L18ER

## Observations

- **Cross-platform parity confirmed.** CompactLogix L18ER (integrated 1769 chassis, no backplane bridge) behaves identically to ControlLogix L75 (routed through 1756-EN2T) for the entire test surface — reads, writes, verifies, firmware-blocked rejections, Phase 6 settle-verify all match to the count.
- **Same firmware-blocked behavior on both controllers.** 60 expected CIP `0x2107` rejections per binding (top-level STRING tags, UDT STRING members, UDT-array-element member writes) — matches the documented AB firmware quirks in `docs/agents/notes/ab-firmware-quirks.md`.
- **Phase 6 settle-verify is closing the loop correctly.** 14 sample tags per category read back at terminal state (`999999` / `9999` / `99.99` / `true`) on both controllers.
- **Usability finding closed by CODEX-AF:** the initial CODEX-AE runners resolved the manifest path relative to the current working directory, so running from `examples/CSharpFullCoverage/` or `/tmp` could fail with a missing `examples/full_coverage_tags.json`. CODEX-AF made default manifest resolution cwd-independent across Rust, C#, and Python while keeping `--manifest <path>` as an explicit override. This was pure example-harness ergonomics, not a library bug.

## Validation Coverage Matrix Update

Adding L18ER (CompactLogix) and L75 (ControlLogix) to the validated controller family list:

| Controller family | Specific model | Firmware | Validated date | Notes |
|---|---|---|---|---|
| ControlLogix | 1756-L81ES | (unspecified) | 2026-04-16 | Earlier validation |
| ControlLogix | 1756-L75 | v33 | 2026-05-24, 2026-05-25 | This validation |
| CompactLogix | 5069-L320ERMS3 | v35 | 2026-04-07 | Earlier validation |
| CompactLogix | 1769-L18ER-BB1B | v33 | 2026-05-25 | This validation |

`wiki/protocol/route-path-behavior.md` keeps ethernet-hop encoding at `likely` because a true multi-chassis bench is still pending; direct-connect routing (both CompactLogix integrated and ControlLogix-via-EN2T-slot-0) is fully validated by this run.

## Library version snapshot

- Rust: `rust-ethernet-ip 1.0.0` (crates.io)
- C#: `RustEtherNetIp 1.0.0` (NuGet — built locally from current `main`)
- Python: `rust-ethernet-ip 1.0.0` (in-repo wrapper)

All three back-ended by the same `target/release/librust_ethernet_ip.dylib` built from the `main` head at validation time.
