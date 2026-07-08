# Release Hardware-Validation Maintainer Decisions

Read this before tagging or publishing any release (crates.io, NuGet, PyPI). A green CI
matrix (`SKIP_PLC_TESTS=1` + simulator) is necessary but **not sufficient**: CI never touches
a real controller, and several wire-format behaviors (STRING structure encoding, UDT
`symbol_id` handling, batch packet sizing, CIP `0x2107` shapes) only surface against firmware.

Verified against CompactLogix 5069-L330ERM fw38, 2026-07-08.

## Rule: no release without a live cross-binding hardware pass

- Before any version bump + publish, run the full-coverage manifest on real hardware through
  **every shipped binding** — Rust native, Python, C#, and (smoke-level) C/C++. Reason: all
  four load the same cdylib but exercise different marshalling; a wrapper-only regression
  passes CI and the other three bindings.
- Record the run under `docs/validation/YYYY-MM-DD_*.md` (model + firmware in the filename)
  before tagging. Reason: releases are audited against the controller/firmware they were
  proven on; an unrecorded run does not count.
- Treat any unexpected anomaly, or any per-binding count that differs from the others, as a
  release blocker until explained. Reason: byte-identical cross-binding counts are the whole
  point of the shared manifest.

## Procedure

```bash
cargo build --release --features ffi          # build the C-ABI cdylib ONCE
# Do NOT run a non-ffi `cargo run --example …` afterward — it recompiles and clobbers the
# cdylib without FFI exports, and the Python/C#/C++ bindings then fail to load it (ABI/export
# mismatch). Run all Rust example probes BEFORE this build, or rebuild --features ffi after.

TEST_PLC_ADDRESS=<ip>:44818 TEST_PLC_SLOT=0 cargo run --release --example test_plc_full_coverage
TEST_PLC_ADDRESS=<ip>:44818 TEST_PLC_SLOT=0 PYTHONPATH=python PYTHONUTF8=1 \
  python python/examples/test_plc_full_coverage.py
TEST_PLC_ADDRESS=<ip>:44818 TEST_PLC_SLOT=0 \
  dotnet run --project examples/CSharpFullCoverage/CSharpFullCoverage.csproj -c Release
```

On Windows, set `PYTHONUTF8=1` for the Python runner or it crashes printing `✓`/`✗` under
redirected stdout (cp1252).

## Gate criteria

- All three manifest runners: `reads`, `writes`, `verify` full, `anomalies=0`, `RESULT=PASS`,
  and identical per-category counts across bindings.
- C/C++: at minimum a STRING + DINT + batch smoke through `include/rust_ethernet_ip.h` /
  `examples/cpp/eip_client.hpp` against the same controller.

## Known gaps to cover out-of-band (until closed)

- **The manifest harness does not write or blocked-probe STRING tags.** `rand_value()` /
  `nines()` in `test_plc_full_coverage.rs` return `None` for `Kind::String`, so the 2
  standalone STRINGs and 17 `encoding_blocked_udt_string_member` entries are skipped — the run
  shows `writes=2266` and `blocked_as_expected=0`, not `2268`/`17`. A STRING-write regression
  passes the gate. Until the harness is fixed, run a dedicated STRING round-trip
  (controller + program scope) per binding and confirm UDT-member STRINGs still reject
  `0x2107`. See [ab-firmware-quirks](ab-firmware-quirks.md).
- **Batch reads of ≥~20 long-path tags fail** with EIP `0x65` (Invalid Length) because the
  grouper ignores `max_packet_size`. Keep hardware batch-read checks under the byte budget or
  they will report a false failure unrelated to the release.

## Restore policy

Every hardware probe must read a tag's original value first and write it back at the end (or
print that it is intentionally stateful). The full-coverage runners settle writeable tags to a
known terminal state by design. Reason: the test tags are shared across every future run.
