---
id: CODEX-AG
title: cross_language_compatibility_tests — honor SKIP_PLC_TESTS + TEST_PLC_ADDRESS, migrate to gTest* tag set
owner: codex
status: merged
created: 2026-05-25
last-update: 2026-05-25 claude [Opus 4.7]
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

### 2026-05-25  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` — full workspace passes (236+ tests, zero failures, zero unexpected ignored).
- `scripts/validate-agent-files` — ok (29 task files).
- Behavior matrix row 1 (`SKIP_PLC_TESTS=1`): all 5 tests reported `ok` in 0.00s — clean early-return.
- Behavior matrix row 2 (no env, unreachable addr `10.255.255.1:44818`): `ok` after 10.00s — confirms `connect_to_plc` 10s timeout path.
- Behavior matrix row 3 (live PLC at `192.168.0.1:44818`, current `gTest*` tags present): `ok` in 0.07s — wall-clock `time` confirms real connect + reads (~70ms vs 10s for unreachable, against the same compiled binary on the same machine, with PLC reachability proven via `nc -z 192.168.0.1 44818`).
- Behavior matrix row 4 (live PLC, tags missing): not independently provable without a stripped PLC; trusted via code inspection of `is_tag_not_found_error` match arms.
- `cargo test --test program_tag_tests --locked` — 4/4 ok against live PLC (0.05s).
- `cargo test --test udt_enhanced_parsing_tests --locked` — 5/5 ok against live PLC (0.05s).

**What's being fixed**
- `tests/cross_language_compatibility_tests.rs` ignored `SKIP_PLC_TESTS`, hardcoded `192.168.0.1:44818`, and panicked on legacy `TestTagController` reads when run against a live PLC carrying the current `gTest*` layout — surfaced on the 2026-05-25 CompactLogix L18ER validation.

**Root cause confirmation**
- Confirmed: pre-fix code at the original `tests/cross_language_compatibility_tests.rs:7` declared `TEST_PLC_IP` as a const, bypassing the `test_helpers` convention every other PLC-dependent integration test uses. Skip filter only matched `"Connection"` / `"timeout"` substrings; CIP `0x04` slipped through to `panic!`.

**Fix appropriateness**
- Right layer: tests/ only, no library change. Three files (`tests/cross_language_compatibility_tests.rs:1-3`, `tests/program_tag_tests.rs:1-3`, `tests/udt_enhanced_parsing_tests.rs:1-3`) now declare `#[allow(dead_code)] mod test_helpers;` per the established cross-crate convention documented at `tests/test_helpers.rs:11-13`.
- `is_tag_not_found_error` shape (`tests/cross_language_compatibility_tests.rs:18-24`) is narrow as the brief required — matches `CIP Error 0x04` / `0x05` / `Path segment error` / `Path destination unknown` strings, nothing broader. Real library errors still propagate via `panic!` in the catch-all `Err(e)` arm.
- The early-return shape (`should_skip_plc_tests()` → `tracing::debug!` → `return`) is byte-identical to the convention used in `tests/program_tag_tests.rs` after the fix and matches `connect_to_plc`'s own internal logging.

**Test proof**
- All 4 of the brief's behavior-matrix rows verified empirically (row 4 by inspection).
- Full workspace `SKIP_PLC_TESTS=1` gate stays green.
- Live-PLC runs prove tests actually exercise the wire (0.07s vs 10s timeout discriminates skip from execute).
- No new tests added — the brief explicitly excluded coverage expansion.

**Residual risk**
- Row 4 ("tags missing, skip cleanly") was not directly reproduced against hardware with a stripped tag set; the live PLC carries the full `gTest*` inventory. Code inspection of the match arms confirms the path, but a maintainer-driven stripped-PLC run would close the loop.
- Codex extended scope to `tests/program_tag_tests.rs` and `tests/udt_enhanced_parsing_tests.rs` — both carried the identical anti-pattern. Documented in the Codex log; verified both files apply the exact same fix shape as the target file, both pass the workspace gate and live-PLC runs. Acceptable scope creep (delivers more value, same risk profile, no contract change).

**Strong points (✅)**
- `mod test_helpers;` declaration matches the cross-crate-module convention required by Cargo's test-binary model — `tests/test_helpers.rs:11-13` notes this is load-bearing.
- `is_tag_not_found_error` taking `&impl Display` (`tests/cross_language_compatibility_tests.rs:17`) is the right generic boundary — works against `EtherNetIpError` without coupling tests to its variant shape.
- Representative tag set in `REPRESENTATIVE_TAGS` (`tests/cross_language_compatibility_tests.rs:7-16`) covers DINT / REAL / BOOL / UDT-member / UDT-nested-array — five distinct codec paths in six tag entries, no firmware-blocked write paths.
- Scope-creep files use byte-identical helper shape — easy to audit, easy to maintain.
- Codex documented the scope expansion in the log up-front rather than burying it in the diff.

**Findings**
- 🟢 The `is_tag_not_found_error` helper is duplicated across three files. Promoting it into `tests/test_helpers.rs` would deduplicate, but the brief said "Don't add [helpers] speculatively" — the call to keep it inline per-file is defensible until a fourth file needs it.
- 🟡 Polish — none.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ Hardcoded `TEST_PLC_IP` constant is gone (`tests/cross_language_compatibility_tests.rs` no longer declares it).
- ✅ `mod test_helpers;` declared at the top of all three modified files.
- ✅ Every `#[tokio::test]` calls `should_skip_plc_tests()` at the top.
- ✅ Every connection uses `connect_to_plc(&get_test_plc_address(), 10)`.
- ✅ Every tag read uses a `gTest*` name from the canonical inventory.
- ✅ Tag-not-found errors trigger graceful skip; other errors panic.
- ✅ `SKIP_PLC_TESTS=1 cargo test --workspace --all-features --locked` passes from the repo root.
- ✅ `cargo test --workspace --all-features --locked` passes against the live PLC (verified: target file 0.07s; scope-creep files 0.05s each).
- ✅ `scripts/validate-agent-files` passes.

## Verdict

### 2026-05-25  claude [Opus 4.7]

**Merged at `fe5059c`** — `test: align PLC integration tests with env config`.

Codex delivered the brief's contract cleanly and extended scope to two adjacent test files (`tests/program_tag_tests.rs`, `tests/udt_enhanced_parsing_tests.rs`) that carried the identical pre-existing anti-pattern. The expansion was documented in the Codex log up-front and applies the same convention with byte-identical helper shape — accepted as justified scope creep that closes the same class of bug across the test suite, not just the one file the brief named.

All four behavior-matrix rows verified (row 4 by code inspection only — would require a stripped PLC). Full workspace gate, clippy, fmt, validate-agent-files all clean. Live PLC at `192.168.0.1` confirms tests genuinely exercise the wire, not silently skip.

Patch-release policy per 2026-05-25 maintainer direction: tests/ only, no library change → lands on `main` without triggering a 1.0.1 release.

No Claude-applied fixes during merge.
