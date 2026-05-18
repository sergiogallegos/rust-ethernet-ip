---
id: CODEX-N
title: CIP path encoding hard validation
owner: codex
status: open
created: 2026-05-18
last-update: 2026-05-18 claude [Opus 4.7]
---

## Brief

### Goal

Replace silent truncation / overflow in `CipRequest::encode` (`src/protocol/cip.rs:29,35`) with explicit validation that returns a typed error. The current code computes `(self.path.len() / 2) as u8` and writes that as the CIP path-words field. Three failure modes hide in that expression:

1. **Odd-length paths truncate silently.** CIP requires word-aligned paths; an odd-length byte buffer encodes a wrong length (rounds down) and the next field starts mid-word.
2. **Paths > 510 bytes overflow `u8` silently.** `512 / 2 = 256` becomes `0` after `as u8`. The packet is technically well-formed but addresses nothing.
3. **Empty paths are not validated against the service.** Some CIP services require a non-empty path (e.g. tag reads); accepting an empty path lets a malformed request reach the wire.

This brief makes all three cases return a typed error before the request is encoded. It also **supersedes** the `debug_assert!(self.path.len() % 2 == 0)` item in the existing CODEX-H agenda entry — that mention should be removed when CODEX-H is briefed, because a `debug_assert!` only fires in dev builds and does nothing in the released `cdylib` that wrappers actually load.

Driven by the architecture review at [`wiki/investigations/architecture-review-2026-05-18.md`](../../../wiki/investigations/architecture-review-2026-05-18.md), Phase 0 item 3.

### Context to read first

- `src/protocol/cip.rs` (197 lines, entire file).
- `src/protocol/encap.rs` (79 lines, entire file) — for symmetry on the framing-layer length fields.
- `src/protocol/tests.rs` (282 lines) — existing pinned-byte tests; this brief adds positive *and* negative cases here.
- `src/error.rs` — pick the right variant for the new validation failures (likely `Protocol(String)` or a new `CipPathInvalid { reason }`; choose during implementation, justify in `## Codex log`).
- `src/route.rs` and any caller of `CipRequest::encode` — every call site needs to handle the new `Result` return type.
- `wiki/investigations/architecture-review-2026-05-18.md` — parent synthesis.

### Files to create or modify

- `src/protocol/cip.rs` — change `CipRequest::encode` signature from `fn encode(&self) -> Bytes` to `fn encode(&self) -> Result<Bytes, EtherNetIpError>`. Add validation. Optionally add a `validate(&self) -> Result<(), EtherNetIpError>` helper for symmetry with the rest of the codec.
- Every caller of `CipRequest::encode` — propagate the `Result`. Most live in `src/client.rs`; the codex split (CODEX-J) will move them later but the caller list is finite and `grep` will find them.
- `src/protocol/tests.rs` — add positive round-trip property test and four negative tests (odd length, > 510 bytes, empty path on read service, empty path on a service that allows it).
- `src/error.rs` — add a new error variant if `Protocol(String)` is judged too lossy. If a new variant is added, it must be `#[non_exhaustive]` from the start (the planned `#[non_exhaustive]` sweep in CODEX-K covers the rest of the enum later; new variants added now should already wear it).

### Behavior

- `CipRequest::encode` validates before writing any bytes:
  - `self.path.len() % 2 == 0` — odd-length path is a programmer error; return `Err`.
  - `self.path.len() / 2 <= u8::MAX as usize` — too-long path returns `Err`.
  - `self.path.is_empty()` — return `Err` unless the audit identifies a service that legitimately uses an empty path; document the allowed exception list inline.
- On success: existing byte layout is unchanged (this is wire-format-preserving).
- Error variant carries enough detail for a caller to know which rule failed: `reason` enum or a structured message with the offending length.

### Test requirements

- **Property test** (new, in `protocol/tests.rs` using `proptest` or hand-rolled): for any `path: Vec<u8>` with even length ≤ 510, `CipRequest::encode` succeeds and the encoded bytes round-trip back to a `CipRequest` with the original `path`.
- **Negative test**: odd-length path → `Err` with the expected reason.
- **Negative test**: 512-byte path → `Err` with the expected reason.
- **Negative test**: empty path on a service that requires one → `Err` with the expected reason.
- **Positive test**: empty path on a service that allows one (if any are identified) → `Ok`.
- All existing positive pinned-byte tests in `protocol/tests.rs` must continue to pass.
- Run the full simulator suite (`cargo test --test plc_sim_tests`) and the C# `dotnet test` suite — neither should regress (they construct paths via higher-level helpers that produce well-formed paths).

### Acceptance criteria

- No `as u8` cast on path-derived arithmetic remains in `src/protocol/cip.rs`.
- `CipRequest::encode` returns `Result<Bytes, EtherNetIpError>`; every call site updated.
- New tests pass; existing tests pass.
- `cargo fmt -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test --workspace --all-features --locked --verbose` all green.
- `cargo test --test plc_sim_tests` green.
- C# `dotnet test` and Python `unittest` matrices stay green.
- The CODEX-H agenda entry's `debug_assert!(self.path.len() % 2 == 0)` mention is removed in the same commit (`board.md` update) since this brief supersedes it.
- No change to the wire format — the validation is purely on the Rust-side input.

### Out of scope

- Refactoring the rest of the codec. This brief touches only path-length validation.
- Adding new CIP service constants or path-builder helpers (`TagPath` already does that).
- Hardening the encap-layer length fields in `encap.rs` — separate concern, may justify a follow-up brief if the audit surfaces similar `as u8` patterns there.
- Adding `#[non_exhaustive]` to existing error variants — CODEX-K covers that.

### Risks and gotchas

- The current silent truncation may have been compensating for an off-by-one elsewhere. Tracing every caller of `CipRequest::encode` and re-running the simulator suite catches it. If a caller silently relied on truncation, this brief surfaces the latent bug — that's a feature, not a regression.
- If `EtherNetIpError` gains a new variant, technically every downstream exhaustive `match` is source-breaking. CODEX-K is the planned `#[non_exhaustive]` sweep, but adding a new variant *before* that sweep is the same risk we'd take at 1.0 anyway. Acceptable, but call it out in the changelog entry.
- The property-test crate (`proptest`) is not currently a dev-dependency. Add it under `[dev-dependencies]` in `Cargo.toml`; commit the resulting `Cargo.lock` change. If proptest is judged overkill, hand-roll a deterministic test that covers length 0, 2, 4, 8, 16, 32, 64, 128, 256, 510, plus the negative cases.
- `Bytes` vs `BytesMut` — preserve the current return type. The signature change is `Bytes` → `Result<Bytes, _>`, not `Bytes` → `Result<BytesMut, _>`.

## Codex log

_(append work entries here)_

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
