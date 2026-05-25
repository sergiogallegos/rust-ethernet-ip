# Claude Review Template

Use this template for new `## Claude review` entries. Keep the review concise, cite files or symbols for every code claim, and write `not proven` when a claim was not independently verified.

## Skeleton

```md
### YYYY-MM-DD HH:MM  claude [model]

**Independent verification**
- `<command>` — pass/fail and one-line result

**What's being fixed**
- One line restating the bug or feature.

**Root cause confirmation**
- Confirmed/not investigated, with file:line or symbol citations.

**Fix appropriateness**
- Judgment on whether the change lands at the right layer, with citations.

**Test proof**
- Tests added or rerun, plus uncovered edge cases.

**Residual risk**
- Known limitations, not-proven claims, hardware gaps, or future follow-ups.

**Strong points (✅)**
- Citation-anchored positives worth preserving.

**Findings**
- 🟢 factual note
- 🟡 polish, non-blocking
- 🟠 real concern, blocks merge unless fixed
- 🔴 defect, rejects

**Acceptance criteria tally**
- ✅ Criterion copied from the brief — result
- 🟡 partially Criterion copied from the brief — missing piece
- ❌ Criterion copied from the brief — failed
- (deferred) Criterion copied from the brief — explicit owner/timing
```

## Worked Example

### 2026-05-24 13:00  claude [Opus 4.7]

**Independent verification**
- `cargo fmt --all -- --check` — clean after unrelated example files were formatted separately.
- `cargo clippy --workspace --all-targets -- -D warnings` — clean.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — 208 passed, 0 failed.
- `cargo test --test plc_sim_tests --locked` — 10/10 passed.
- `PYTHONPATH=python python3 -m unittest python.tests.test_client_value_mapping python.tests.test_client_contract python.tests.test_bindings` — 23/23 passed.
- `PYTHONPATH=python RUST_ETHERNET_IP_START_SIM=1 python3 -m unittest python.tests.test_integration` — 8/8 passed.
- Hardware re-run against ControlLogix 1756-L75 at `10.136.15.20` — 2299 tags exercised, 2206 writeable; zero CIP `0x1E` errors.

**What's being fixed**
- Python single-tag writes now dispatch through typed FFI exports instead of the batch endpoint, restoring BOOL and numeric write parity with C#.

**Root cause confirmation**
- Confirmed: `Client.write_tag()` dispatch in `python/rust_ethernet_ip/client.py` now routes concrete scalar types before falling back to UDT batch behavior.
- Confirmed: typed FFI signatures in `python/rust_ethernet_ip/bindings.py` cover BOOL, integer widths, REAL/LREAL, and STRING.

**Fix appropriateness**
- Appropriate layer: the bug was in Python wrapper dispatch, so the fix stays in the wrapper and does not change Rust core wire behavior.
- The UDT fallback remains intentional because UDT typed-write parity is a separate surface.

**Test proof**
- Unit tests cover typed dispatch, STRING dispatch, UDT fallback, and pre-FFI validation errors.
- Simulator integration tests round-trip the updated write path.
- Real hardware confirmed all plain and program-scoped BOOL writes that previously failed.

**Residual risk**
- The review did not prove future UDT typed-write behavior; UDT remains on the batch path by design.
- Float validation accepts integer literals for REAL writes, which is probably user-friendly but should stay intentional.

**Strong points (✅)**
- Typed FFI registration uses `c_int` for BOOL values and `c_double` for REAL/LREAL, matching the Rust FFI signatures and C# wrapper shape.
- Dispatch checks BOOL before integer handling, avoiding Python's `bool`-is-`int` trap.
- Integer range and ctypes lookup tables keep future scalar expansion localized.
- UDT fallback is explicitly preserved and documented.
- Tests cover both direct wrapper mapping and simulator-backed behavior.

**Findings**
- 🟡 Float validation casts through `float(value)`, so `write_tag("REAL_TAG", 42, value_type="REAL")` succeeds. That is likely correct, but the intent should remain documented if the validator is tightened later.
- 🟢 Full hardware validation was performed by Claude because Codex did not have PLC access during implementation.
- 🟠 Real concerns — none.
- 🔴 Defects — none.

**Acceptance criteria tally**
- ✅ All typed single-tag write FFI exports registered with correct argtypes/restype.
- ✅ Read exports registered as infrastructure for future read-path work.
- ✅ `Client.write_tag()` dispatches to typed exports for all non-UDT scalar/string types.
- ✅ New unit tests pass without simulator.
- ✅ New integration tests pass against simulator.
- ✅ Rust fmt, clippy, workspace tests, and simulator tests pass.
- ✅ Changelog entry present.
- ✅ Hardware re-run output captured by the reviewer.
