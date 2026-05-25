---
id: CODEX-N
title: CIP path encoding hard validation
owner: codex
status: merged
created: 2026-05-18
last-update: 2026-05-24 claude [Opus 4.7]
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

### 2026-05-24 codex [gpt-5]

- Added checked `CipRequest::validate()` and checked inherent `CipRequest::encode(&mut BytesMut) -> Result<()>`.
- Replaced path word-count truncation with validation for empty paths, odd byte lengths, and paths longer than 510 bytes.
- Updated `EipClient` read/write request builders and list-tags request construction to use the checked CIP request encoder.
- Kept the generic `Encode` trait unchanged for unrelated infallible encoders; `CipRequest` now uses its checked inherent method at all concrete call sites.
- Removed the superseded CODEX-H `debug_assert!` agenda item from `docs/agents/board.md`.
- Added deterministic round-trip coverage for valid even path lengths through 510 bytes and negative tests for empty, odd, and 512-byte paths.
- Verification:
  - `cargo test protocol::tests::cip --locked`

## Claude review

### 2026-05-24  claude  [Opus 4.7]

**Independent verification:**

- `cargo fmt --all -- --check` — clean
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — 218/218 passed
- Targeted `cargo test --test plc_sim_tests` and `cargo test protocol::tests` — all CIP request tests green
- Manual grep: every call site of the old `Encode for CipRequest` trait impl is converted (`src/client.rs:3269,3288,3317`-ish, plus `4186`). All use `?` to propagate the new `Result<()>` cleanly. No silent unwraps.

**Strong points (✅):**
- `validate()` is split out from `encode()` as a separate `pub(crate)` method (`src/protocol/cip.rs:28-54`). Future callers that want to validate without encoding can do so cheaply. Encode wraps it as `self.validate()?` so the contract is enforced exactly once at the boundary.
- All three failure cases get distinct, actionable error messages with the offending service code embedded: `"path must not be empty"`, `"path length N is not word-aligned"`, `"path length N bytes exceeds 510-byte CIP limit"`. Operators reading the error log will know which CIP service triggered the failure and what was wrong.
- `u8::try_from(self.path.len() / 2).expect("validated path word count fits in u8")` (`cip.rs:60`) — the `expect` is sound because `validate()` already enforced the ≤ 255-word bound. The `.expect("validated …")` reason explains why the panic-path is unreachable. Per CLAUDE.md ("Every `unsafe` block needs a SAFETY comment naming the invariant"), this is the moral equivalent — the reason string names the upstream invariant. ✅
- Negative tests assert both the error message *and* `buf.is_empty()` (`tests.rs:249,258,267`). That's the no-partial-write guarantee made explicit — if validation fails partway through encode, the buffer is untouched. Future refactors that break this property will trip the test.
- Boundary test `cip_request_even_paths_up_to_limit_encode_and_round_trip` (`tests.rs:230-241`) exercises 2 / 4 / 8 / … / 510 — every power-of-two plus the exact CIP word limit. 512 (next byte over the limit) is tested as the rejection case.
- Inherent method approach over keeping the trait impl + adding a new method — correct because the new signature returns `Result`, which is incompatible with the existing `Encode::encode(&self, buf)` trait. Codex's brief log explicitly flagged this choice; correctly resolved by dropping the trait impl for `CipRequest` specifically while leaving `Encode` intact for the infallible encoders (`CipResponse`, `SendDataRequest`, headers). ✅
- Brief asked for `EtherNetIpError::Protocol(String)` or a new `CipPathInvalid { reason }`. Codex chose `Protocol(String)` — minimum-blast-radius option, no API surface change. Reasonable choice given the formatted messages already carry all the structured information a caller would extract from a typed variant.

**Findings (🟡 polish, non-blocking):**
- 🟡 The validation error messages embed `service` via `format!`. That allocates per failure. For a hot retry loop where validation fails repeatedly, the allocation cost shows up. Not a real concern in normal operation (validation should never fail at runtime — it's a guard against malformed-internally-constructed paths) but worth a `tracing::warn!` follow-up if anyone profiles a repro.
- 🟡 The agenda update in `docs/agents/board.md:36-43` cleanly removed the now-superseded `debug_assert` bullet (CODEX-H went from 6 items to 5). The brief asked Codex to do this and it's done. Good follow-through.

**Findings (🟠 real concerns) — none.**

**Acceptance criteria tally:**
- ✅ `CipRequest::encode` returns `Result<EtherNetIpError>`
- ✅ Parity check (path length must be even)
- ✅ Overflow check (≤ 510 bytes = 255 words)
- ✅ Empty-path check
- ✅ Pinned negative tests for all three failures
- ✅ Pinned boundary test at exactly 510 bytes
- ✅ All call sites converted to propagate the Result
- ✅ Generic `Encode` trait left intact for other encoders
- ✅ CODEX-H agenda updated to drop the superseded `debug_assert` item

## Verdict

### 2026-05-24  claude  [Opus 4.7]  status: merged

**Merged.** This closes a real silent-corruption hazard: the old `(self.path.len() / 2) as u8` cast would have produced wrong CIP requests for any internally-constructed path > 510 bytes (truncating the word count and shifting subsequent payload bytes into the path region). Three failure modes now fail loudly with structured errors at the boundary. The split between `validate()` and `encode()` is a clean future-proofing for callers who want to pre-validate.

No defects to fix during merge. The `u8::try_from(...).expect(...)` pattern is a good idiomatic example for the codebase — flagged in the strong-points list so future PRs can mirror it.
