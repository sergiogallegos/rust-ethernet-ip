# 2026-07-08 Cross-binding full-coverage + STRING-write validation — CompactLogix 5069-L330ERM fw38

Date: 2026-07-08
Tester: Claude [Opus 4.8] + Sergio Gallegos (maintainer-owned hardware)
Library version: `1.1.0` line, `main` at `dc4eb16` (post CODEX-AJ…AU remediation set; pre-1.2.0)

## Scope

Pre-1.2.0 hardware validation pass against a real controller, across **all four** language
bindings (Rust native, Python, C#, C/C++), plus dedicated STRING-write and latency/batch
benchmarking. First run to exercise every binding since the CODEX-AT STRING-write fix and
the CODEX-AU C/C++ header landed.

- Controller: CompactLogix **5069-L330ERM**, firmware **38**, at `192.168.0.101:44818`, slot 0, direct connect.
- Native library: `target/release/rust_ethernet_ip.dll` built from `main` at `dc4eb16` with `--features ffi` (C ABI v2, `eip_abi_version() == 2`). All four bindings back-ended by this one artifact.
- Manifest: `examples/full_coverage_tags.json` (AV-corrected: 2304 total / 2268 writeable / 17 expected-blocked / 19 read-only).

## Commands executed

```bash
# Native cdylib (once; do NOT run non-ffi cargo example commands afterward — they clobber the cdylib)
cargo build --release --features ffi

# Full coverage — Rust / Python / C#
TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 cargo run --release --example test_plc_full_coverage
TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 PYTHONPATH=python PYTHONUTF8=1 python python/examples/test_plc_full_coverage.py
TEST_PLC_ADDRESS=192.168.0.101:44818 TEST_PLC_SLOT=0 dotnet run --project examples/CSharpFullCoverage/CSharpFullCoverage.csproj -c Release

# Dedicated STRING round-trip + latency/batch probes (temporary, per binding; deleted after)
```

## Full-coverage result

All four bindings PASSED. The Rust/Python/C# runners are byte-identical; the **C/C++ runner
is the corrected one** — it exercises the full manifest intent including STRING writes.

| Binding | Reads | Writes | Verify | blocked_as_expected | Anomalies | Result | Artifact |
|---|---|---|---|---|---|---|---|
| Rust   | 2304/2304 | 2266/2266 | 2266/2266 | 0  | 0 | PASS | `rust_1783526957.json` |
| Python | 2304/2304 | 2266/2266 | 2266/2266 | 0  | 0 | PASS | `python_20260708T161839Z.json` |
| C#     | 2304/2304 | 2266/2266 | 2266/2266 | 0  | 0 | PASS | `csharp_20260708T162422Z.json` |
| C/C++  | 2304/2304 | **2268/2268** | **2268/2268** | **17** | 0 | PASS | `cpp_1783530025.json` |

`examples/cpp/full_coverage.cpp` (new this session) parses the same
`examples/full_coverage_tags.json`, expands it with the same rules as the Rust runner, and
drives all six phases through the C ABI. Because it **does** write+verify the 2 standalone
STRINGs and blocked-probe the 17 UDT STRING members, it reports the manifest's full labels:
**2304 read / 2268 write / 2268 verify / 17 blocked / 19 read-only, 0 anomalies**. This is what
the Rust/Python/C# runners will report once the harness STRING gap (finding 2) is closed.

### Count reconciliation — the harness does NOT exercise STRING writes

The manifest labels **2268 writeable / 17 expected-blocked / 19 read-only**, but each runner
**wrote 2266 and probed 0 as blocked**. This is not a regression; it is a blind spot in the
harness itself:

- `examples/test_plc_full_coverage.rs` `rand_value()` / `nines()` return `None` for
  `Kind::String` (and `Kind::Udt`). Phase 2 (write) and Phase 4 (blocked-probe) both
  `continue` past every String tag.
- Consequence: the **2 standalone STRING tags** (`ctrl.STRING`, `prog.STRING`) — labeled
  `writeable` — are silently skipped (2268 − 2 = **2266** written), and the **17
  `encoding_blocked_udt_string_member` tags** are never probed (17 − 17 = **0** blocked).
- The C# and Python runners mirror the same skip.

Net: **a STRING-write regression would pass this gate undetected.** STRING coverage was
obtained out-of-band (next section). Closing this gap is filed as a follow-up (see Follow-up).

## STRING-write validation (dedicated probes — the headline result)

Standalone (top-level) STRING writes now **work on every binding**, controller- and
program-scoped. UDT-member STRINGs remain current-encoding-blocked with CIP `0xFF/0x2107`,
exactly as documented in `docs/agents/notes/ab-firmware-quirks.md`. Every probe restored the
original tag value.

| Binding | `gTest_STRING` (ctrl) | `Program:TestProgram.gTest_STRING` (prog) | `gTestUDT.Member5_String` | `gTestUDT_Array[0].Member5_String` |
|---|---|---|---|---|
| Rust   | write+readback PASS | write+readback PASS | rejected 0x2107 (expected) | rejected 0x2107 (expected) |
| Python | write+readback PASS | write+readback PASS | rejected (expected) | rejected (expected) |
| C#     | write+readback PASS | write+readback PASS | rejected (expected) | rejected (expected) |
| C/C++  | write+readback PASS | write+readback PASS | rejected 0x2107 (expected) | rejected 0x2107 (expected) |

This confirms, across the full binding matrix, the 2026-07-02 single-controller finding that
the "firmware blocks direct STRING writes" claim was a misdiagnosis — the fix (structure type
`0x02A0`, handle `0x0FCE`, 88-byte payload) round-trips on all four bindings.

## Benchmarks (live PLC, 5069-L330ERM fw38)

Single-tag latency (200 iterations, `gTestArray_DINT[0]`, value restored):

| Binding | Single read | Single write | Connect path |
|---|---|---|---|
| Rust   | ~2031 µs/op (492 ops/s) | ~2507 µs/op (399 ops/s) | routed (`add_slot(0)`) |
| Python | ~1841 µs/op (543 ops/s) | ~2051 µs/op (487 ops/s) | routed |
| C#     | ~1911 µs/op (523 ops/s) | ~2019 µs/op (495 ops/s) | routed |
| C/C++  | ~3340 µs/op (299 ops/s) | ~3410 µs/op (293 ops/s) | plain `eip_connect` (no route) |

Batch (50 × DINT array elements per call unless noted):

| Operation | Result |
|---|---|
| Batch **write**, 50 tags (Rust `execute_batch`) | ~89 µs/tag, ~11 235 tags/s — ~28× faster than single writes |
| Batch **read**, 10 tags (within byte budget) | ~1095 µs/tag, ~913 tags/s — only ~1.7× faster than single reads |
| Batch **read**, ≥20 tags | **FAILS** — EIP encapsulation status `0x65` (Invalid Length); see Findings |

Note: the C/C++ probe used plain `eip_connect` (the only connect the `eip_client.hpp` RAII
wrapper exposes), which is measurably slower than the routed connect the other three used;
functional correctness was identical.

## Findings

1. **Batch READ of ≥~20 long-path tags fails with EIP `0x65` (Invalid Length).** The batch
   grouper (`optimize_operation_groups`) enforces `max_operations_per_packet` (default 20) but
   **not** `max_packet_size` (default 504 bytes). Twenty `gTestArray_DINT[N]` read services in
   one Multiple Service Packet exceed 504 bytes, so the controller rejects the entire
   encapsulation. Reproduced: 5/10 tags OK; 20/25/40/50 all fail. Shared Rust core → affects
   all bindings; Python raises `BatchReadError`, Rust returns per-tag `Err`s that callers
   easily swallow. Batch **writes** pack fewer bytes and are unaffected. **Not fixed here** —
   needs a brief to enforce a byte budget (and to split, not fail) on the read path.

2. **Full-coverage harness never exercises STRING writes or blocked-probes** (see reconciliation
   above). The release gate cannot catch a STRING-write regression as written.

3. **Python full-coverage runner crashes on Windows under stdout redirection.** It prints
   `✓`/`✗` glyphs; when stdout is redirected the interpreter defaults to cp1252 and raises
   `UnicodeEncodeError`. Worked around with `PYTHONUTF8=1`. Cosmetic/portability, example-only.

4. **Read-batching is far less effective than write-batching** (~1.7× vs ~28× per-tag speedup).
   Worth a look alongside finding 1, since both point at the read batch path.

## Restore / state

Every probe read the original value first and wrote it back at the end
(`gTest_STRING` → "STRING FROM CONTROLLER TAG", program STRING → "TEST STRING PROGRAM",
`gTestArray_DINT[0]` and the batch DINT range restored). The full-coverage runners settle
writeable tags to their terminal state (999999 / 9999 / 99.99 / true) by design.

## Validation coverage matrix update

| Controller family | Specific model | Firmware | Validated date | Notes |
|---|---|---|---|---|
| CompactLogix | 5069-L330ERM | 38 | 2026-07-02, **2026-07-08** | STRING-write fix + C/C++ header; 4-binding pass |
| ControlLogix | 1756-L75 | v33 | 2026-05-24/25 | Two-controller cross-binding run |
| CompactLogix | 1769-L18ER-BB1B | v33 | 2026-05-25 | |
| CompactLogix | 5069-L320ERMS3 | v35 | 2026-04-07 | |
| ControlLogix | 1756-L81ES | — | 2026-04-16 | |

## Library version snapshot

- Native: `rust_ethernet_ip.dll` C ABI v2, built from `main` @ `dc4eb16`, `--features ffi`.
- Rust / C# / Python / C++ bindings all loaded this same artifact.

## Follow-up

- Close the harness STRING gap so the release gate writes+verifies standalone STRINGs and
  probes the 17 blocked members (would have turned this into a single-command validation).
- Brief and fix the batch-read `0x65` byte-budget bug (finding 1).
- Fix the Python runner's Windows-redirect Unicode crash (finding 3).
- CODEX-AO Phase 2 packet captures for the UDT-member STRING encoding remain outstanding;
  this run re-confirmed those members reject `0x2107` on fw38.
