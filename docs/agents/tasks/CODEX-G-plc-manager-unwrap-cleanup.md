---
id: CODEX-G
title: plc_manager.rs unwrap cleanup
owner: codex
status: merged
created: 2026-05-26
last-update: 2026-05-26 claude [Opus 4.7]
---

## Brief

Backlog-tracked polish item (no standalone brief was authored — board "Post-1.0.0 polish" item #1 served as the contract). Replace the soundness-adjacent `.unwrap()` calls on live paths in `src/plc_manager.rs`:

- `src/plc_manager.rs:25` — parse in `Default` (`"127.0.0.1:44818".parse().unwrap()`)
- Lines 135, 139, 163, 173, 183 — `HashMap::get_mut` / `last_mut` on connection pool lookups

A panic from any of these is a real failure mode when reached from the FFI side. Convert to `Result` propagation via `EtherNetIpError::Connection(...)`.

## Codex log

### 2026-05-26  codex [gpt-5]

Replaced `.unwrap()` on the default-address parse with `SocketAddr::from(([127, 0, 0, 1], 44818))`. Replaced the four connection-pool `get_mut`/`last_mut` unwraps with `ok_or_else` returning `EtherNetIpError::Connection(...)` with descriptive messages. Restructured `get_connection` to compute `max_connections` and `max_packet_size` upfront so the borrow checker doesn't fight the pool lookup. Bundled with CODEX-H partial, CODEX-I, and CODEX-O at commit `2690669`.

## Claude review

### 2026-05-26  claude [Opus 4.7]

Reviewed as part of the consolidated `2690669` bundle. Full structured review entry in [`docs/agents/log.md`](../log.md) (2026-05-26 entry). Independent verification: `cargo fmt --all -- --check` clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` 238/0/52. All 5 unwrap sites from the brief audit are gone; restructured pool access compiles cleanly with the simultaneous borrow of `config`.

## Verdict

### 2026-05-26  claude [Opus 4.7]

**Merged at `2690669`** (bundled commit "post-1.0 polish and stabilize dotnet CI"). Zero defects, zero Claude-applied fixes. No individual brief file was authored before submission — board entry served as the contract. Patch-eligible library change; rides into the next 1.0.1 publish.
