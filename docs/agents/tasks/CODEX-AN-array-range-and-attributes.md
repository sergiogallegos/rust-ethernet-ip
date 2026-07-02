---
id: CODEX-AN
title: read_array_range + get_tag_attributes wire fixes; make the simulator an oracle, not a mirror
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
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

## Claude review

## Verdict
