---
id: CODEX-I
title: Real codec benchmarks — replace placeholder no-ops
owner: codex
status: merged
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
---

## Brief

Backlog-tracked polish item (no standalone brief was authored — board "Post-1.0.0 polish" item #3 served as the contract). Replace the placeholder `benches/performance_benchmark.rs` (three mock functions that don't exercise the codec at all — `black_box(PlcValue::Dint(42))`, `Vec<PlcValue>` push, no-op) with benchmarks that actually call `PlcValue::encode`, `PlcValue::decode`, `EncapsulationHeader::encode`, and a realistic batch-request build via `BytesMut`. Closes the brief-error gap from CODEX-D where the `>5%` regression gate was a sub-nanosecond noise check.

## Codex log

### 2026-05-26  codex [gpt-5]

Rewrote `benches/performance_benchmark.rs`:
- `bench_plc_value_encode`: BOOL / DINT / REAL / STRING / UDT fixtures through `PlcValue::encode`
- `bench_plc_value_decode`: matching 5-type fixtures through `values::decode_payload`
- `bench_encapsulation_header_encode`: `EncapsulationHeader::new(REGISTER_SESSION, ...)` round-trip
- `bench_cip_request_encode`: 5/10/25/50/100-tag batches alternating READ_TAG / WRITE_TAG with realistic payload encoding via `values::write_data_type` + `PlcValue::to_bytes`

Bundled with CODEX-G, CODEX-H partial, and CODEX-O at commit `2690669`.

## Claude review

### 2026-05-26  claude [Opus 4.7]

Reviewed as part of the consolidated `2690669` bundle. Full structured review entry in [`docs/agents/log.md`](../log.md) (2026-05-26 entry). Independent verification: `cargo bench --no-run --locked` compiles cleanly. Benchmarks now exercise the actual codec boundary established by CODEX-D — `>5%` regression gate is no longer a noise check. Realistic batch sizes (5–100 tags) match production batch shapes.

## Verdict

### 2026-05-26  claude [Opus 4.7]

**Merged at `2690669`** (bundled commit). Zero defects, zero Claude-applied fixes. Benchmark-only change; no library behavior impact. Patch-neutral (doesn't trigger 1.0.1 by itself but bundles cleanly with the patch-eligible items).
