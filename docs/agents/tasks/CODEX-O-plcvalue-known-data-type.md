---
id: CODEX-O
title: PlcValue::Udt::get_data_type() placeholder honesty + type-prefixed UDT encoding
owner: codex
status: merged
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
---

## Brief

Backlog-tracked polish item (no standalone brief was authored — board "Post-1.0.0 polish" item #6 served as the contract). `PlcValue::Udt::get_data_type()` returned the `0x00A0` placeholder for all UDTs regardless of whether a real symbol type code was known (`crates/types/src/lib.rs:78`). Either:
- (a) return `Option<u16>` / `Result` instead of synthesizing a fake type code, or
- (b) capture the real type code in `UdtData` at parse time and propagate it.

Verify via test that the placeholder never escapes through the FFI as a misleading real CIP type. Contained brief; can run any time after CODEX-L.

## Codex log

### 2026-05-26  codex [gpt-5]

Chose hybrid: kept `get_data_type() -> u16` for backward compat (unwraps to `0x00A0` for unknown UDT) and added `known_data_type() -> Option<u16>` for honest reporting. `PlcValue::Udt` now returns `Some(0x02A0 + symbol_id)` when `symbol_id > 0`, `None` otherwise. Routed `crates/protocol/src/values.rs::write_data_type` and `encode_type_prefixed` through `known_data_type` so type-prefixed UDT writes emit the symbol-derived AB type code instead of the generic `0x00A0` marker. Added two pinned-bytes tests: `unknown_udt_type_remains_generic_structure_marker` (symbol_id == 0 fallback) and `udt_type_prefixed_encode_uses_symbol_derived_type` (symbol_id == 0x1234 → 0x14D4 on the wire). Existing `write_data_type_uses_udt_symbol_id` test still asserts `0x14D4` — backward compat for the common UDT RMW path preserved. Bundled with CODEX-G, CODEX-H partial, and CODEX-I at commit `2690669`.

## Claude review

### 2026-05-26  claude [Opus 4.7]

Reviewed as part of the consolidated `2690669` bundle. Full structured review entry in [`docs/agents/log.md`](../log.md) (2026-05-26 entry). **Wire-protocol change**: type-prefixed UDT writes now send `0x02A0 + symbol_id` (the documented AB convention) instead of the `0x00A0` placeholder. Previously, sending `0x00A0` as a type-prefix for a UDT write would have been firmware-rejected — so this is correcting a latent bug, not breaking a working path. The `symbol_id == 0` edge case (UDT constructed without a prior read) falls back to `0x00A0` and is explicitly tested. Existing pinned-bytes test for `symbol_id == 0x1234 → 0x14D4` still passes — the common UDT RMW workflow (read first, capture symbol_id, write back) is unchanged on the wire.

**Residual risk**: hardware re-validation for this wire change has not been performed. The 2026-05-25 cross-binding validation predates this commit. Maintainer-owned hardware re-run is queued before any 1.0.1 publish (per user direction: accumulate more before cutting). API addition (`known_data_type`) is SemVer-compatible (purely additive).

## Verdict

### 2026-05-26  claude [Opus 4.7]

**Merged at `2690669`** (bundled commit). Zero defects, zero Claude-applied fixes. Patch-eligible library change with wire-protocol implications — hardware re-validation gate must clear before this lands in 1.0.1.
