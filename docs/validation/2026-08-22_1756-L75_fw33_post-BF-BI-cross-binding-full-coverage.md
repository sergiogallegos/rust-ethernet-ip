# 1756-L75 Firmware 33 Post-BF/BI Cross-Binding Full Coverage

Date: 2026-08-22 (America/Denver)  
Result: **PASS**  
Repository state: CODEX-BF/BI merged through `4a5f1ca`; current `3711d37`
adds documentation-only README/website changes

## Scope

Maintainer-requested post-merge regression run against the shared 2,304-tag
manifest through the Rust, C#, Python, and C/C++ public surfaces. This run
checks broad single-tag read/write behavior after CODEX-BF and CODEX-BI; the
Python native grouped-write path has separate batch evidence in
[the CODEX-BF validation](2026-08-22_1756-L75_fw33_python-native-batch-writes.md).

## Target

- Controller: ControlLogix `1756-L75/B`, firmware `33.011`
- Route: `1756-EN2T/D` firmware `10.007` in chassis slot 1 to processor in
  backplane slot 0
- Slot: `0`
- Manifest: [`full_coverage_tags.json`](../../examples/full_coverage_tags.json),
  2,304 paths: 2,285 writeable and 19 read-only
- Build: optimized `1.2.1` development Rust/FFI artifact

The test address is intentionally omitted from this retained record. Every
runner used explicit `--allow-writes`, wrote randomized exercise values,
verified them, then settled all writeable paths to the terminal-value family
(`999999` / `9999` / `99.99` / `true` / `SETTLED`) and verified one sample
from each of 18 categories.

## Commands

Each binding was run serially from the repository root:

```bash
cargo run --release --example test_plc_full_coverage --locked -- \
  --plc-address <PLC_ADDRESS> --plc-slot 0 --allow-writes --out-dir <RESULT_DIR>

dotnet run --project examples/CSharpFullCoverage/CSharpFullCoverage.csproj \
  -c Release -- --plc-address <PLC_ADDRESS> --plc-slot 0 \
  --allow-writes --out-dir <RESULT_DIR>

PYTHONPATH=python PYTHONUTF8=1 python3 python/examples/test_plc_full_coverage.py \
  --plc-address <PLC_ADDRESS> --plc-slot 0 --allow-writes \
  --out-dir <RESULT_DIR>

cmake --build target/cpp --target cpp_full_coverage -j2
target/cpp/cpp_full_coverage --plc-address <PLC_ADDRESS> --plc-slot 0 \
  --allow-writes --out-dir <RESULT_DIR>
```

## Results

| Binding | Preflight | Reads | Writes | Verify | Settle | Settle verify | Anomalies | Result |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| Rust | 2304/2304 | 2304/2304 | 2285/2285 | 2285/2285 | 2285/2285 | 18/18 | 0 | PASS |
| C# | 2304/2304 | 2304/2304 | 2285/2285 | 2285/2285 | 2285/2285 | 18/18 | 0 | PASS |
| Python | 2304/2304 | 2304/2304 | 2285/2285 | 2285/2285 | 2285/2285 | 18/18 | 0 | PASS |
| C/C++ | 2304/2304 | 2304/2304 | 2285/2285 | 2285/2285 | 2285/2285 | 18/18 | 0 | PASS |

All four bindings produced byte-identical phase totals with no failed reads,
writes, read-back verifications, or settle operations. Controller-scope and
program-scope BOOL/numeric arrays, STRINGs, UDT members, nested members, and UDT
array-element members all passed. Whole UDT and whole UDT-array-element paths
remained read-only as defined by the manifest.

Ignored raw artifacts:

- `rust_1787452726.json`
- `csharp_20260823T024003Z.json`
- `python_20260823T024105Z.json`
- `cpp_1787452933.json`

The final PLC state is the expected settled terminal state.
