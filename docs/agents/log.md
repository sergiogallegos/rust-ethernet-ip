# Agent Activity Log

Append-only chronological transcript of cross-agent activity. One line per event. Newest at bottom.

Format: `YYYY-MM-DD HH:MM  <author>  <task-id-or-->  <event>`

Use `--` for task-id when the event is project-wide (protocol bootstrap, etc).

---

2026-05-01  claude  --        Cross-agent collaboration protocol bootstrapped at docs/agents/. Templated from OpenWebHMI HEAD; rust-ethernet-ip CLAUDE.md gained an "Agent collaboration" appendix pointing at this directory.
2026-05-05  claude  CODEX-A   Brief authored: FFI safety, runtime hardening, and lint baseline. Gates `pub mod ffi;` on the existing `ffi` Cargo feature, surfaces `Runtime::new()` failure via FFI return code, converts `RwLock`/`HashMap` unwraps at FFI-reachable sites, and adds a conservative crate-level `#![deny]` / `#![warn]` baseline. Status: open.
2026-05-05  codex   CODEX-A   Implementation submitted: gated FFI behind `--features ffi`, added fallible runtime initialization return code, converted FFI-reachable poison/unwrap paths, added lint baseline, and verified Rust/C# build and tests. Status: submitted.
2026-05-05  claude  CODEX-A   Merged at `3d98abf`. Independent verification: cargo fmt clean, both clippy feature variants clean, full test matrix green, symbol parity verified (56 eip_ exports under --features ffi, 0 in default rlib). Four polish notes documented as non-blocking candidates for CODEX-B. Status: merged.
2026-05-05  claude  CODEX-B   Brief authored: contained API cleanup. BatchError → thiserror, drop async-trait direct dep, fix 0.7.0 → 0.8.0 docstring rot, delete tag_subscription.rs shim, remove three unused EipClient fields, add #[must_use] to selected builders/getters, plus two cheapest polish items carried over from CODEX-A (RAII guard for FORCE_RUNTIME_INIT_ERROR, ffi_block_on! contract doc). Excludes any SemVer-major change. Status: open.
2026-05-05  codex   CODEX-B   Implementation started. Status: in-progress.
2026-05-05  codex   CODEX-B   Implementation submitted: converted BatchError to thiserror, removed unused async-trait edge and tag_subscription shim, removed unused EipClient fields, added selected must_use annotations, and applied CODEX-A test/macro polish. Rust and C# verification passed. Status: submitted.
2026-05-05  claude  CODEX-B   Merged at `9aca8d2`. Independent verification: cargo fmt clean, both clippy feature variants clean, full test matrix green, BatchError Display strings byte-identical to prior manual impl, FFI symbol parity preserved at 56 eip_ exports, async-trait absent from active dep graph. Two non-blocking polish notes recorded; one brief-text error owned by Claude (audit-grep pattern). Status: merged.
2026-05-05  claude  CODEX-C   Brief authored: pure-mechanical decomposition of the 8.3k-line lib.rs into route.rs, batch.rs, types.rs, and client.rs across four phases. No behavior change, no signature change. Gating step for CODEX-D. Status: open.
2026-05-05  claude  CODEX-D   Brief authored: extract pub(crate) Encode/Decode traits for the wire protocol; move PlcValue, EncapsulationHeader, and CIP framing into src/protocol/. Round-trip codec tests required (≥30); benchmarks must show no >5% regression. Depends on CODEX-C. Status: open.
2026-05-05  claude  CODEX-E   Brief authored: four small polish items — Once-based runtime-init log dedupe, merge adjacent pub use subscription blocks, LazyLock the tag-name regex, drop the unused cargo-tarpaulin dev-dep. Independent of CODEX-C/D. Status: open.
