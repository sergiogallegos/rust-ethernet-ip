# Cross-Binding Hardware Feature Gate

Last updated for the `1.2.1` development line.

This companion gate closes the coverage boundary left by the full-tag runners.
It tests batch operations, whole-UDT reads, and discovery through each public
language surface. It does **not** replace the full-tag read/write gate.

## What It Tests

| Capability | Rust | C# | Python | C/C++ |
|---|---:|---:|---:|---:|
| Mixed controller/program batch read | Yes | Yes | Yes | Yes |
| Native batch write | Yes | Yes | Yes, safe atomic subset | Yes |
| Grouped write API | Yes | Yes | Yes, native + typed fallback | Yes |
| Whole controller/program UDT reads | Yes | Yes | Yes | Yes |
| Whole UDT-array-element reads | Yes | Yes | Yes | Yes |
| Controller tag discovery | Yes | Yes | N/A | Yes |
| Program tag discovery | Yes | N/A | N/A | N/A |
| Restore verification | Yes | Yes | Yes | Yes |

`N/A` means the binding does not expose that capability in the 1.2.x line. It
is not counted as a failed hardware result. Known program-scoped paths remain
readable in every binding.

The C# runner calls both `ReadTagsBatch` and a read-only `ExecuteBatch`. This
prevents the consumer-friendly sequential fallback in `ReadTagsBatch` from
masking a failure in the native multi-service path.

Python `write_tags()` native-batches unique atomic scalar and numeric-array
writes. STRING/UDT, member/bit, packed-BOOL-array, and duplicate-name writes
retain typed sequential behavior; see the
[1756-L75 native-write validation](2026-08-22_1756-L75_fw33_python-native-batch-writes.md).

## Dedicated Write Targets

Only these four DINT elements are modified:

- `gTestArray_DINT[50]`
- `gTestArray_DINT[51]`
- `Program:TestProgram.gTestArray_DINT[50]`
- `Program:TestProgram.gTestArray_DINT[51]`

Each runner reads the original values before any mutation, writes temporary
exercise values, verifies them, restores the originals, and verifies the
restore. Live mode refuses to start without `--allow-writes`.

An operating-system termination, power loss, or network loss during the write
window can still prevent restoration. Use only dedicated non-production tags
and monitor the controller during the run.

## Preconditions

- The controller is not controlling production equipment.
- The `gTest*` controller and `Program:TestProgram.gTest*` tags from
  `docs/PLC_TEST_TAG_DEFINITIONS.md` are loaded.
- The Ethernet module address and CPU slot are known.
- Rust, Python, .NET 10, CMake, and a C++17 compiler are installed.

Run offline checks first:

```bash
cargo run --example hardware_feature_gate --locked -- --dry-run
PYTHONPATH=python python3 python/examples/hardware_feature_gate.py --dry-run
dotnet run --project examples/CSharpHardwareFeatureGate/CSharpHardwareFeatureGate.csproj -c Release -- --dry-run
cargo build --release --features ffi --locked
cmake -S examples/cpp -B target/cpp -DRUST_ETHERNET_IP_NATIVE_LIB="$PWD/target/release/librust_ethernet_ip.dylib"
cmake --build target/cpp --target cpp_hardware_feature_gate
target/cpp/cpp_hardware_feature_gate --dry-run
```

Use `.so` instead of `.dylib` for the CMake native-library path on Linux.

## One-Command Live Run on macOS or Linux

For a ControlLogix CPU in slot 0 behind an EtherNet/IP bridge:

```bash
scripts/run-cross-binding-feature-gate.sh \
  192.168.0.10:44818 \
  0 \
  TestProgram \
  --allow-writes
```

The script runs Rust, Python, C#, and C/C++ serially. A failure stops later
bindings. The active runner still attempts its restore before returning a
failure.

## Pass Criteria

- Every supported discovery operation returns the expected `gTestUDT` tag.
- All four whole-UDT paths return a non-empty structure value.
- All ten mixed batch-read results succeed; C# also passes the independent
  native `ExecuteBatch` read check.
- Every applicable write result succeeds and reads back correctly.
- All four original DINT values are restored and verified in every binding.
- Every runner prints a final `PASS` line.

Do not add a hardware compatibility-matrix checkmark until the console output,
controller model, firmware, topology, commands, and restore outcome have been
captured in a dated file under `docs/validation/`.
