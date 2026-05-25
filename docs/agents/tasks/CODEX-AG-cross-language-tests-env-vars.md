---
id: CODEX-AG
title: cross_language_compatibility_tests — honor SKIP_PLC_TESTS + TEST_PLC_ADDRESS, migrate to gTest* tag set
owner: codex
status: submitted
created: 2026-05-25
last-update: 2026-05-25 codex [gpt-5]
---

## Brief

### Goal

`tests/cross_language_compatibility_tests.rs` predates the `test_helpers.rs` convention every other PLC-dependent integration test in this repo uses. It hardcodes `TEST_PLC_IP = "192.168.0.1:44818"` (`tests/cross_language_compatibility_tests.rs:7`), ignores `SKIP_PLC_TESTS`, reads legacy tag names (`TestTagController`) that don't exist on the current `gTest*` PLC project layout, and only skips on `"Connection"` / `"timeout"` error substrings — not on CIP `0x04` "Path segment error" (tag-not-found).

The result: the file has been silently passing in CI for months because nothing answers at `192.168.0.1` in CI, but it failed loudly the moment the maintainer connected a real PLC at that exact IP for the 2026-05-25 CompactLogix L18ER validation:

```
test cross_language_compatibility_tests::test_ffi_compatibility ... FAILED
test cross_language_compatibility_tests::test_memory_safety ... FAILED
  → panicked: "FFI compatibility test failed: Protocol error: CIP Error 0x04: Path segment error"
```

Bring the file into line with the established convention: env-var-addressing, `SKIP_PLC_TESTS` honored, gracefully skip on tag-not-found (with a clear log line), use the `gTest*` tag set the rest of the integration suite uses.

### Context to read first

- `tests/cross_language_compatibility_tests.rs` — the test file (5 tests, ~225 lines, 2 of which read legacy tag names)
- `tests/test_helpers.rs` — the established helper module. `get_test_plc_address()` (line 36), `get_test_plc_slot()` (48), `should_skip_plc_tests()` (69), `connect_to_plc()` (87) — pattern other integration tests follow
- Any other integration test file as a worked example, e.g. `tests/program_tag_tests.rs` or `tests/route_path_operations_tests.rs` — both use `mod test_helpers;` and call the helpers
- `docs/PLC_TEST_TAG_DEFINITIONS.md` — the current `gTest*` tag layout the live PLC carries
- `examples/full_coverage_tags.json` — the manifest with the canonical writeable + read-only tag set for picking representative tags
- 2026-05-25 log entry recording the failure and noting "worth a future brief" — the immediate motivator
- 2026-05-25 CODEX-AF review (`docs/agents/tasks/CODEX-AF-cwd-independent-manifest-resolution.md` § Claude review) — flagged this as a pre-existing issue surfaced by CODEX-AF's environment

### Files to create or modify

- `tests/cross_language_compatibility_tests.rs` — primary surface:
  - Add `mod test_helpers;` at the top per the established pattern.
  - Remove the hardcoded `TEST_PLC_IP` constant.
  - At the top of each `#[tokio::test]`, call `should_skip_plc_tests()` and early-return if true.
  - Replace direct `EipClient::connect(TEST_PLC_IP)` with `connect_to_plc(&get_test_plc_address(), 10).await`. If the helper returns `None`, early-return (the helper already prints a skip message).
  - Replace legacy tag names (`TestTagController` and any other `TestTag*` strings) with `gTest*` equivalents — see "Tag-name migration" below.
  - Extend the skip-on-error pattern so CIP `0x04` "Path segment error" (or any "Path destination unknown" / tag-not-found shape) triggers a graceful skip with `tracing::debug!("Skipping test - tag not available on PLC: {}", e)`, not a panic.
- (optional) `tests/test_helpers.rs` — only if a new helper is genuinely useful (e.g. `is_tag_not_found_error(&e) -> bool`). Don't add it speculatively.

### Behavior

After the brief lands, this matrix holds:

| Environment | Expected behavior |
|---|---|
| `SKIP_PLC_TESTS=1` set | All 5 tests skip cleanly with the standard `tracing::debug!` message |
| No env vars set, no PLC at `192.168.0.1` | All 5 tests skip cleanly via connection refused (existing behavior; preserved) |
| `TEST_PLC_ADDRESS=10.136.15.20:44818 TEST_PLC_SLOT=0` set, real PLC reachable, `gTest*` tags present | All 5 tests run and pass against the real PLC |
| `TEST_PLC_ADDRESS=192.168.0.1:44818 TEST_PLC_SLOT=0` set, real PLC reachable, **but tags missing** | All 5 tests skip with a clear `tag not available` message, exit 0 |
| Real PLC reachable, tags present, but the test triggers a real library bug | Test panics with the library error message (existing behavior preserved for real failures) |

The fifth row is the load-bearing distinction: tag-not-found is a setup issue (skip), but other CIP error codes that indicate library bugs (timeout in the middle of a read, partial response, etc.) should still surface as test failures.

### Tag-name migration

Map legacy `TestTag*` names to current `gTest*` equivalents. Suggested mapping:

| Legacy name (current code) | Replacement (`gTest*` set) |
|---|---|
| `TestTagController` | `gTestArray_DINT[0]` (controller-scoped DINT, always present on the validated PLCs) |
| Any other `TestTag*` reference | Closest equivalent from `examples/full_coverage_tags.json` |

If `test_ffi_compatibility` or `test_memory_safety` are iterating across "a list of tag names" rather than testing a single tag, pick a representative subset from the manifest (e.g. `gTestArray_DINT[0]`, `gTestArray_REAL[0]`, `gTestArray_BOOL[0]`, `gTestUDT.Member1_DINT`) so the test exercises multiple types without re-implementing the full-coverage exerciser.

Do NOT include firmware-blocked write paths (top-level STRING, UDT-array-element members) in the new tag list — those would correctly fail with CIP `0x2107` and confuse the skip logic.

### Test requirements

- `SKIP_PLC_TESTS=1 cargo test --test cross_language_compatibility_tests --locked` exits 0 with all 5 tests reported as `ok` (Rust prints `ok` for skipped tests that return early, which is the convention the rest of the suite uses).
- Without env vars and without a PLC at the default address, `cargo test --test cross_language_compatibility_tests --locked` still passes (connection-refused skip path).
- With a real PLC at the env-var address carrying the `gTest*` tags, all 5 tests run and pass.
- The repo's existing CI matrix continues to pass (`SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` is the CI gate; the new code should let that gate stay at 236 / 0 — actually it should *rise* by 0 because these tests already counted as "passed via skip" in CI).
- `scripts/validate-agent-files` passes.

### Acceptance criteria

- Hardcoded `TEST_PLC_IP` constant is gone.
- `mod test_helpers;` is declared at the top of the file.
- Every `#[tokio::test]` calls `should_skip_plc_tests()` at the top and early-returns on true.
- Every connection uses `connect_to_plc(&get_test_plc_address(), 10)` or equivalent helper, not a raw `EipClient::connect`.
- Every tag read uses a `gTest*` name from the canonical inventory.
- Tag-not-found errors are caught and trigger a graceful skip with a clear log line, not a panic.
- Real library errors (other CIP error codes, hard failures) still surface as test failures.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` passes at 236 / 0 from any cwd.
- `cargo test --workspace --all-features --locked` (no env vars) passes when no PLC is reachable AND when a PLC is reachable but missing the legacy `TestTag*` tags.
- `scripts/validate-agent-files` passes.

### Out of scope

- Library code changes. This is `tests/` only.
- Adding new tests beyond the existing 5. The brief is conformance to the convention, not coverage expansion.
- Replacing these integration tests with the full-coverage exerciser (`examples/test_plc_full_coverage.rs`). The exerciser is for hardware validation; these are for cross-language API parity. Different surface, different cadence.
- Adding a CI job that exercises the tests against a real PLC. CI continues to run with `SKIP_PLC_TESTS=1`; real-hardware verification stays maintainer-owned.
- Removing or rewriting `test_performance_consistency` / `test_error_handling_consistency` / `test_rust_library_core_functionality` if they already pass — only the two failing tests (`test_ffi_compatibility`, `test_memory_safety`) plus the env-var and SKIP_PLC_TESTS wiring need to change. Touch the other three only to add the standard skip-check header.

### Risks and gotchas

- **The "graceful skip on tag-not-found" pattern is the load-bearing decision.** If you're too generous about what counts as a "missing tag" error, you'll silently mask real library bugs. Stay narrow: match the error string for `CIP Error 0x04` ("Path segment error") and `0x05` ("Path destination unknown") specifically; anything else propagates as a test failure.
- **`mod test_helpers;` declaration matters.** Each test file is a separate crate per Cargo conventions; the `mod test_helpers;` declaration is what lets `test_helpers.rs` be reused. Don't try to import via path.
- **Don't rename the test functions.** Their names appear in CI logs and in any historical comparison. Just change the body.
- **Don't change the `[#tokio::test]` attribute or async-ness.** The existing tests are async per the library's API; preserving that shape avoids touching unrelated machinery.
- **CI is already passing for these tests** (they skip via connection-refused). The brief is about making them ALSO pass on developer machines that have a PLC at the default address but with the current tag set. Not a CI regression risk; an environment-portability fix.
- **Patch-release policy still holds.** This is `tests/` only, no library change. Lands on `main` without triggering a 1.0.1 release per the 2026-05-25 maintainer direction.

## Codex log

### 2026-05-25  codex [gpt-5]

Updated `tests/cross_language_compatibility_tests.rs` to use the established PLC test helper convention: `mod test_helpers`, `SKIP_PLC_TESTS` early returns, `TEST_PLC_ADDRESS` via `get_test_plc_address()`, and `connect_to_plc(..., 10)` instead of the hardcoded `192.168.0.1:44818`. Migrated the test reads from legacy `TestTag*` / `API_Web` tags to representative current `gTest*` tags and added a narrow setup-skip path for CIP `0x04` / `0x05` tag-not-found errors.

While verifying the required workspace skip-gate, two adjacent legacy PLC-dependent test files failed for the same reason with a reachable current PLC: `tests/program_tag_tests.rs` and `tests/udt_enhanced_parsing_tests.rs`. Brought those files onto the same helper/env-var/`gTest*` convention so `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` is honestly green in this environment.

Verification:

- `SKIP_PLC_TESTS=1 cargo test --test cross_language_compatibility_tests --locked` passed.
- `cargo test --test cross_language_compatibility_tests --locked` passed against the reachable default PLC.
- `SKIP_PLC_TESTS=1 cargo test --test program_tag_tests --locked` and `cargo test --test program_tag_tests --locked` passed.
- `SKIP_PLC_TESTS=1 cargo test --test udt_enhanced_parsing_tests --locked` and `cargo test --test udt_enhanced_parsing_tests --locked` passed.
- `cargo fmt -- --check` passed.
- `cargo clippy --test cross_language_compatibility_tests --test program_tag_tests --test udt_enhanced_parsing_tests --locked -- -D warnings` passed.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` passed.

## Claude review

_(append review entries here)_

## Verdict

_(final disposition)_
