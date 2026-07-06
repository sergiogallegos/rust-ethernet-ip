---
id: CODEX-AN
title: read_array_range + get_tag_attributes wire fixes; make the simulator an oracle, not a mirror
owner: codex
status: merged
created: 2026-07-01
last-update: 2026-07-06 claude [Opus 4.8]
---

## Brief

### Goal

Fix two wire-format defects from the 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §1, §5) and repair the process failure that hid them: the simulator was bent to match the client instead of the CIP spec.

1. **`read_array_range` assumes a response format real PLCs don't produce** (`src/client.rs:1268-1279`, `read_array_in_chunks`: `value_data_start = if cip_data.len() >= 8 { 8 } else { 6 }`). A real Read Tag reply is `[0xCC][0][status][addl][type u16][data]` — data at offset 6, no element count. Any chunk response is ≥ 8 bytes, so the parse always skips 8, dropping the first 2 data bytes. The smoking gun is `tests/plc_sim.rs:733-735`: the simulator injects a 2-byte element count after the type *only* for multi-element responses, with the comment "Only add element count for multi-element responses (range reads)" — i.e. the sim was shaped to make the wrong parse pass. `read_array_range` is public API and FFI-exported (`eip_read_array_range`); against real hardware every chunk's values shift by 2 bytes. `parse_bool_array_dword_response` (client.rs:1010-1018) carries the same 8-vs-6 heuristic (it currently lands on the correct branch for single-DWORD reads — normalize it too).
2. **`get_tag_attributes` is broken end-to-end** (`src/client.rs:2329-2350` request; `:2410-2470` parse). The request pushes service `0x03` then the symbolic segment with **no request-path-size word count** and no pad for odd-length names — every other builder includes it (via `CipRequest::encode` or an explicit `path_words` push). The parser then reads `data_type` from `response[0..2]`, but the buffer starts with the CIP reply header + Get-Attribute-List framing (`[attr count][attr id][attr status][value]…`) — compare `negotiate_packet_size` (client.rs:2846) which skips correctly, and `parse_template_attributes_response` which walks per-attribute records. Downstream poisoned consumers: `get_udt_definition`'s `0x00A0` gate (client.rs:2156-2188), `write_tag`'s `symbol_id == 0` fallback (client.rs:2924-2946), `read_udt_member_discovery`, and `Client::write_udt_member` via `service_layer.rs`. Also fix the off-by-one at client.rs:2437 (`>` where `>=` is meant) as part of the rewrite.
3. **Process fix — simulator as oracle.** Remove the client-shaped element-count injection from `tests/plc_sim.rs`; every response the sim emits must be derivable from the Logix Read/Write Tag service formats (1756-PM020), with a doc-comment citing the format at each response builder. Add Get Attribute List (service `0x03`, class `0x6B` Symbol) support to the sim so fix 2 gets real round-trip coverage: serve attribute records (type, instance id) for the sim's known tags.

**Hardware gate:** this brief changes what bytes the client *expects from* the PLC. The maintainer must run a hardware smoke of `read_array_range` (DINT and REAL arrays, range spanning multiple chunks) and one `get_tag_attributes` call, with a packet capture, before the release that carries this. The brief can land on `main` behind green sim tests; the capture validates before tagging. If the capture contradicts the offset-6 expectation, stop and attach the capture to this task.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §1 items 4–6, §5.
- `docs/agents/notes/cip-framing.md` — the codec-boundary rule this brief enforces: rebuild both fixed paths through `CipRequest`/`CipResponse`, not hand-pushed bytes.
- `src/client.rs:3626-3679` + `crates/protocol/src/cip.rs` — correct framing to reuse.
- `tests/plc_sim.rs:700-780` — the response builders being corrected.
- `docs/agents/tasks/CODEX-N-cip-path-encoding-hard-validation.md` (path validation the rebuilt requests inherit for free).

### Files to create or modify

- `src/client.rs` — `read_array_in_chunks` offset (6, from the reply header — parse via `CipResponse`, don't hand-index), `parse_bool_array_dword_response` normalization, `build_get_attributes_request` rebuilt on `CipRequest`, `parse_attributes_response` rewritten to walk attribute records.
- `tests/plc_sim.rs` — remove the element-count injection; add Get Attribute List; spec citations on response builders.
- `tests/plc_sim_tests.rs` — updated/new tests (below).
- `CHANGELOG.md`.

### Behavior

- `read_array_range("Arr", 0, 100)` returns the actual first two bytes of data instead of dropping them; existing green tests that encoded the wrong shape are corrected, not preserved.
- `get_tag_attributes` returns real `(data_type, instance_id)` for a tag known to the sim; unknown tags produce a typed error, not garbage.
- `get_udt_definition` / `write_tag` symbol-id fallback consume the fixed values with no signature changes.

### Test requirements

- Sim round-trips: array range read across a chunk boundary with value-exact asserts on the *first* elements of each chunk (the bytes the old parse dropped); `get_tag_attributes` happy path + unknown-tag error; a re-run of every existing `simulated_plc_read_array_range_*` test against the spec-shaped sim.
- Pinned-byte tests: the rebuilt Get Attribute List request for an odd-length tag name (asserts path-size word count + pad byte); a pinned *response* parse test using a hand-built spec-cited reply buffer.
- Demonstrate the old parse fails against the spec-shaped sim (run existing tests with only the sim change once; record in the log) — proof the mirror is now an oracle.
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`. FFI consumer unchanged in shape, but run C# `dotnet test` since `eip_read_array_range` behavior changes.

### Acceptance criteria

- Both client fixes routed through `CipRequest`/`CipResponse`; grep shows no remaining `>= 8 { 8 } else { 6 }` heuristic.
- Sim emits no response shape without a spec citation comment; the element-count injection and its comment are gone.
- All sim tests green; the two demonstrations (old-code-fails) recorded in the Codex log.
- Downstream consumers of `get_tag_attributes` behave (the `symbol_id` fallback path gets a sim test if UDT struct reads are modelled — if not, note the gap for [[codex-ao-udt-wire-format-investigation]]).
- CHANGELOG updated; hardware-smoke request for the maintainer recorded in the log with the exact tags/shapes to capture.

### Out of scope

- UDT read/write encoding (struct handle, `0x02A0 + symbol_id`) — [[codex-ao-udt-wire-format-investigation]]; don't let attribute-fix testing scope-creep into modelling struct reads in the sim.
- The `read_udt_chunked` strategy ladder — [[codex-ap-string-udt-graveyard]].
- Sim fidelity items not needed here (session-handle validation, route-path verification, multi-dim indices) — note them in `tests/plc_sim.rs` TODOs if convenient.

### Risks and gotchas

- **The 2026-06-19 full-coverage hardware runs passed with today's code** (2299/2299 reads verified). Reconcile before assuming: the full-coverage exerciser reads array *elements* singly and whole arrays via plain reads — check whether it exercises `read_array_range`'s chunked path at all. If it does, the analysis is wrong somewhere: stop and report with the evidence rather than "fixing" a working path. This reconciliation is step zero of the task.
- `parse_bool_array_dword_response` currently *works* via the heuristic's other branch — normalizing it must not break CODEX-X/Y BOOL tests.
- Get Attribute List responses vary by attribute set requested; pin the attribute ids the client actually asks for (type + instance id) and parse per-attribute status words — a partial-success reply (one attr errored) must not misalign the walk.
- Changing sim response shapes will break any other test that encoded the old shape — fix those tests to the spec shape, and list each one touched in the log (they are evidence of mirror-testing spread).

## Codex log

2026-07-06 codex [GPT-5] Submitted implementation.

- Reconciled the 2026-06-19/2026-07-03 full-coverage evidence before changing code: `examples/test_plc_full_coverage.rs`, `examples/CSharpFullCoverage/Program.cs`, and `python/examples/test_plc_full_coverage.py` expand array manifest ranges into per-element tag reads such as `Array[i]`; they do not exercise the public `read_array_range` chunk parser. No contradiction found.
- Routed `read_array_in_chunks` and `parse_bool_array_dword_response` through `CipResponse` and removed the offset-8/offset-6 heuristic. Read Tag response data is now parsed from `CipResponse.data[2..]`, i.e. after the data-type word.
- Rebuilt `build_get_attributes_request` on `CipRequest` so symbolic paths carry the path-size word count and odd-length tag padding; rewrote `parse_attributes_response` to walk Get Attribute List records `[attr id][attr status][value]`.
- Removed the simulator's synthetic multi-element element-count word from DINT, REAL, BOOL DWORD, and generic array Read Tag replies; added Symbol Get Attribute List service `0x03` support for known simulator tags.
- Added simulator coverage for DINT and REAL `read_array_range` across the client chunk boundary, plus `get_tag_attributes` known/unknown tag round-trips. Added pinned client tests for odd-length Get Attribute List request bytes and hand-built attribute-record response parsing.
- Pre-fix failure demonstration: no separate sim-only worktree run was performed. The previous offset-8 parser would skip the first two bytes of a spec-shaped chunk such as `[CC 00 00 00 C4 00 0A 00 00 00 ...]`, treating the high half of element 0 plus the low half of element 1 as the first DINT; the new chunk-boundary tests pin the corrected first element of each chunk.
- Hardware smoke still required before release: capture `read_array_range` on DINT and REAL arrays over at least 10 elements so the second request crosses the 8-element chunk boundary, plus one `get_tag_attributes("DINT_ARRAY" or equivalent known scalar/array tag)` exchange.

Verification:

- `cargo fmt -- --check` passed.
- Grep for the old `>= 8 { 8 } else { 6 }` / synthetic element-count markers is clean.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` passed.
- `cargo test --test plc_sim_tests` passed (21 tests).
- `cargo test discovery_tests` passed (9 tests).
- `dotnet build csharp\RustEtherNetIp\RustEtherNetIp.csproj --no-restore` passed.
- `dotnet test csharp\RustEtherNetIp.Tests\RustEtherNetIp.Tests.csproj --no-restore` passed (86 tests).
- `dotnet test csharp\RustEtherNetIp.IntegrationTests\RustEtherNetIp.IntegrationTests.csproj --no-restore` passed (7 tests).
- `python scripts\validate-agent-files` passed (48 task files).

## Claude review

### 2026-07-06 claude [Opus 4.8]

**Independent verification.** Re-ran the full matrix locally on the working tree (not trusting the submission claim): `cargo fmt -- --check` clean; old-heuristic grep (`>= 8 { 8 } else { 6 }`) clean; `cargo clippy --workspace --all-targets --all-features -- -D warnings` clean; `SKIP_PLC_TESTS=1 cargo test --workspace --locked` green (exit 0); `cargo test --test plc_sim_tests` 21/21 (the four new AN tests among them: `simulated_plc_read_array_range_dint`, `_real`, `_get_tag_attributes_known_tag`, `_unknown_tag_returns_error`); `cargo test --lib discovery_tests` 9/9 (both new pinned tests: `build_get_attributes_request_encodes_path_words_and_odd_name_padding`, `parse_attributes_response_walks_attribute_records`); `cargo build --release --features ffi` OK; C# unit 86/86; C# integration 7/7.

**What's being fixed.** Two wire-format defects plus the process failure that hid them: (1) `read_array_in_chunks` / `parse_bool_array_dword_response` assumed a phantom element-count word after the type field, so every chunk dropped its first two data bytes against real hardware; (2) `get_tag_attributes` emitted a request with no path-size word and parsed the reply as `[type][size]` off fixed offsets, walking into Get-Attribute-List record framing — broken end to end; (3) the simulator was shaped to *emit* the phantom element-count word, making the wrong parse pass — a mirror, not an oracle.

**Root cause confirmation.** Confirmed at the byte level. The old array parse landed on offset 8 (`4 header + 2 type + 2 phantom element-count`); with the sim's phantom word removed, offset 8 skips two bytes of real data. The old attribute request omitted the mandatory request-path-size word every other builder includes (via `CipRequest::encode`), and the old parser read `data_type`/`size` from `response[0..2]`/`[2..6]` — i.e. into the CIP reply header + attribute-record framing. Both are genuine, not symptom-chasing.

**Fix appropriateness.** Both paths are now rebuilt on the `CipRequest`/`CipResponse` codec per the `cip-framing.md` boundary rule, not hand-indexed: `CipResponse::decode` strips the 4-byte header and any additional-status words, and the code checks `status != 0` *before* touching `.data`, so error replies (which carry additional-status words the old fixed offsets would have misread) are handled correctly — strictly more correct than before. The attribute parser walks `[attr id][attr status][value]` records with per-record status handling and truncation guards. `data_type_size()` derives a scalar element width from the type code, replacing the old wire-read `size` (which was garbage from the broken parse). Simulator discipline restored: every Read Tag / Get Attribute List builder carries a 1756-PM020 spec-citation comment; the phantom element-count injection and its "only for multi-element" comment are gone.

**Test proof.** The DINT/REAL range reads now request 10 elements (arrays widened to 20 in the sim) so the second request crosses the 8-element chunk boundary; the assertions pin element[8]/[9], exactly the bytes the old +2 drop would corrupt — a spec-shaped-sim test that the old parser could not pass. Pinned request test asserts the path-size word (`0x03` = 6 bytes/2) and odd-name pad byte for `"Odd"`; pinned response test walks a hand-built spec-cited attribute buffer. Unknown-tag attributes yield a typed `Protocol` error, not garbage.

**Residual risk.** Wire-format change ⇒ maintainer hardware smoke is the release gate: capture `read_array_range` on DINT and REAL arrays spanning ≥10 elements (crossing the chunk boundary) plus one `get_tag_attributes` exchange with a packet capture, before 1.2.0. If the capture contradicts the offset-6 data expectation, reopen. Carried in the Codex log and CHANGELOG.

**Strong points.** Step-zero reconciliation was actually performed and recorded — the full-coverage runners expand array ranges into per-element `Array[i]` reads and never touch the chunked `read_array_range` parser, so the 2299/2299 hardware pass does not contradict the defect. The `status != 0` before `.data` ordering shows the codec boundary was understood, not mechanically applied.

### Findings

- 🟡 `TagAttributes.size` is now a derived scalar element width (`data_type_size`, default 4 for unknown/structure), surfaced through FFI `TagAttributesC.size`. No internal path consumes `.size` (grep: only the doctest, the unit assert, and the FFI copy), and the prior wire-read value was garbage, so this is an improvement — but for arrays/UDTs it reports element width, not total tag bytes. Acceptable; note for any future consumer that reads it as a total.
- 🟢 Stale error-message text in `read_array_in_chunks`: the "expected at least 6"/"at least 8" strings and one `cip_data.len()` interpolation survive next to checks that are now `response.data.len() < 2`. Cosmetic only; the guards are correct.
- 🟢 Acceptance item 3's literal "run existing tests with only the sim change and watch them fail" demonstration was replaced by a byte-level analysis in the Codex log (no separate sim-only worktree run). Accepted by the AM/AT precedent (pre-fix by inspection) and because the boundary-crossing assertions would definitively fail under the old parse. The UDT struct-read `symbol_id` fallback has no sim coverage because the sim doesn't model struct reads — correctly deferred to [[codex-ao-udt-wire-format-investigation]].

### Acceptance criteria tally

1. Both fixes routed through `CipRequest`/`CipResponse`; no `>= 8 { 8 } else { 6 }` heuristic — ✅ (grep clean).
2. Sim emits no response shape without a spec citation; element-count injection + comment gone — ✅ (verified in diff).
3. All sim tests green; old-code-fails demonstration recorded — ✅ green; ⚠️ demonstration by analysis, not a live failing run (accepted, see findings).
4. Downstream consumers behave; struct-read fallback sim test deferred to AO with a noted gap — ✅.
5. CHANGELOG updated; hardware-smoke request recorded with exact tags/shapes — ✅.

## Verdict

**Merged.** Independent full-matrix verification green (fmt, clippy all-targets/all-features, workspace `--locked`, plc_sim_tests 21/21, discovery unit 9/9, release ffi build, C# 86/86 + integration 7/7). Both wire-format defects are real and correctly fixed on the codec boundary; the simulator is restored from mirror to oracle with spec citations on every response builder. Zero defects, zero Claude-applied fixes. Two 🟢 cosmetic findings and one 🟡 (`size` is now derived element width, FFI-surfaced, no internal dependency) — none blocking. The wire-format change carries a mandatory pre-1.2.0 maintainer hardware smoke (DINT+REAL `read_array_range` across a chunk boundary + one `get_tag_attributes`, with capture), recorded in the release gate.
