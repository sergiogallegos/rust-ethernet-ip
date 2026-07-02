---
id: CODEX-AP
title: Retire the string/UDT strategy graveyard — never-worked public paths return honest errors
owner: codex
status: open
created: 2026-07-01
last-update: 2026-07-01 claude [Fable 5]
---

## Brief

### Goal

The 2026-07-01 repository analysis ([`docs/agents/repo-analysis-2026-07-01.md`](../repo-analysis-2026-07-01.md), §1 "Public APIs that cannot ever have worked") identified a cluster of exploratory protocol code in `src/client.rs` / `src/client/string.rs` that is publicly exported but provably non-functional or hazardous. Removing public API is SemVer-major, so on the 1.x line this brief follows the established `eip_get_tag_metadata` precedent: each retired path returns an explicit `Unsupported`-style error (and gains `#[deprecated]`), with actual deletion queued for 2.0 (add to `docs/ROADMAP.md`'s 2.0 section). Internal dead code is deleted outright.

The retirement list (verification in the analysis doc; re-verify each before touching):

1. `write_string` (`src/client/string.rs:1003-1040`) — request missing the path-size byte; status read from the service-reply byte, so success reports as `WriteError { status: 0xCD }`. Two independent defects; never worked. The working path (`write_string_tag` → `write_tag(PlcValue::String)`) is untouched and becomes the documented redirect.
2. `write_ab_string_udt` (`string.rs:121-133`) — checks byte 2 of the raw CPF envelope (always 0), returns `Ok(())` unconditionally; payload is not a valid struct write. Silent false success — the most dangerous shape.
3. Connected messaging / Forward Open subsystem (`string.rs:190-452, 603-632` — `establish_connected_session`, `parse_forward_open_response`, `write_string_connected`, `send_connected_cip_request`) — response parsed at the wrong layer so every Forward Open fails after 6×100 ms; layered status checks read reserved bytes; connected reads have no timeout. Dead on arrival end-to-end.
4. `write_string_unconnected` (`string.rs:822-951`) — reverse-engineered payload ("Structure appears to be…", type `0x0FCE`, no element count) matching no documented Write Tag shape.
5. `write_ab_string_components` (`string.rs:13-68`) — one SINT per round trip (82 RTTs worst case), non-atomic `.LEN`-first sequence, and depends on the malformed `.DATA[i]` segment being fixed by CODEX-AM. **Decision point:** if, after CODEX-AM lands, this path works against the sim and provides a real quirk workaround (check `docs/agents/notes/ab-firmware-quirks.md` — STRING writes are CIP `0x2107` territory), keep it as *the* documented fallback with a warning about atomicity; otherwise retire it with the rest. Record the decision + evidence in the log.
6. `read_udt_chunked` strategies B–D (`src/client.rs:1719-2021`) — `read_udt_chunk_advanced` emits protocol-invalid requests and returns error bytes as data; two loops don't terminate against a peer answering full-size chunks (unbounded growth); strategy D converts total failure into `Ok(Udt { data: vec![] })`. Keep strategy A only if it is the plain fragmented-read fallback that real code paths rely on — trace callers first; the dispatch trigger (`msg.contains("Partial transfer")`) must become a typed error match on whatever remains.
7. `read_udt_member_by_offset` / `write_udt_member_by_offset` (`src/client.rs:2045-2152`) — index into the CIP reply envelope, not the payload; the write path round-trips envelope bytes to the PLC as data.
8. Internal dead code, delete outright: `_build_ab_string_write_request`, `_get_connected_session` (`src/client.rs:3007-3084, 4084-4104`), the two `unreachable!` in `batch_exec.rs:186/263` (restructure to make the states unrepresentable or return a typed internal error), and whatever items 1–7 orphan.

Also correct `parse_extended_error` (`src/client.rs:3180-3319`) while in the area: extended status is signaled by the additional-status size field, not by general status `0xFF`; collapse the duplicated both-endianness match.

### Context to read first

- `docs/agents/repo-analysis-2026-07-01.md` §1; `docs/agents/notes/ab-firmware-quirks.md` (which workarounds are load-bearing — the brief retires *strategies*, never the documented quirk handling); `docs/agents/notes/unconnected-send.md`; the FFI exports in `src/ffi.rs` that front any retired method (grep each name) — their behavior must change to the honest-error contract too, following the `eip_configure_batch_operations` "explicitly unsupported" pattern and its `ffi_batch_config_apis_are_explicitly_unsupported` test.

### Files to create or modify

`src/client/string.rs` (bulk), `src/client.rs`, `src/client/batch_exec.rs`, `src/ffi.rs` (matching honest-error stubs + capability note if applicable), `src/error.rs` (an `Unsupported { api, reason }`-style variant if none fits), C#/Python wrapper doc comments for any wrapped retired API (verify with grep; wrappers should surface the error, not pre-empt it), `docs/ROADMAP.md` (2.0 removal list), `CHANGELOG.md` (`### Deprecated` section), `tests/plc_sim_tests.rs` / `tests/ffi_tests.rs`.

### Behavior

- Every retired public API: `#[deprecated(note = "...never functioned; use X; removal in 2.0")]` + returns the typed unsupported error immediately (no network I/O). No silent false success remains anywhere in the string/UDT surface.
- Line count of `src/client/string.rs` drops substantially; every remaining line traces to a documented quirk or the working `write_tag` path.

### Test requirements

- Per retired API: a test asserting the immediate typed error (and, for the FFI stubs, the rc/last-error contract).
- Existing suites prove the *working* paths still work: STRING round-trip via `write_tag(PlcValue::String)` sim tests, UDT RMW via service layer, batch tests.
- `cargo doc` builds without broken intra-doc links to removed items; deprecation warnings do not trip `-D warnings` in-crate (use `#[expect(deprecated)]` at the few internal call sites that must remain until 2.0, with reasons).
- Full matrix: fmt, clippy `-D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, `cargo test --test plc_sim_tests`, C# + Python suites (FFI-touching).

### Acceptance criteria

- Items 1–8 each either retired-with-honest-error, deleted (internal), or kept-with-evidence (item 5's decision point, strategy A tracing) — an explicit disposition per item in the Codex log.
- No `msg.contains("Partial transfer")`-style stringly dispatch remains.
- `cargo semver-checks` (baseline 1.1.0, post-CODEX-AK) passes — deprecation + behavior change of never-working paths is not a signature break; if semver-checks disagrees on any item, stop and report.
- ROADMAP 2.0 section lists every deferred deletion. CHANGELOG updated.

### Out of scope

- Fixing any of these paths to actually work — that's what the retirement replaces; capture-gated UDT work is [[codex-ao-udt-wire-format-investigation]], attributes/array-range is [[codex-an-array-range-and-attributes]].
- The transport hardening in `send_rr_data_item` — [[codex-al-transport-session-hardening]] (note: retiring connected messaging removes `send_connected_cip_request`, which is why AL scoped it out).
- `TagManager`/monitoring dead strata — [[codex-aq-dead-stratum-deprecation]].

### Risks and gotchas

- **Verify "never worked" independently per item before retiring** — the analysis was verified by reading, not by hardware, and this repo has one precedent (CODEX-O / analysis conflict) of a read-based conclusion colliding with hardware evidence. For each item: reproduce the defect with a focused sim test or byte-level assertion first, record it, then retire. If any item works against the sim, stop and re-scope.
- The full-coverage manifest (2299 tags) may exercise some of these names — check `examples/full_coverage_tags.json` runners for calls into retired APIs; the exerciser must keep passing (it should — it uses the working paths — but verify).
- `read_udt_chunked` strategy A tracing: `read_tag` on large UDTs may genuinely depend on the fragmented-read fallback. If real fragmentation support is needed, that's the documented CIP service `0x52` Read Tag Fragmented — a *new* correctly-built path belongs to CODEX-AO's phase 2 or a follow-up, not a rescue of strategy B.
- Keep the diff reviewable: mechanical retirement first, `parse_extended_error` cleanup as a separate commit within the task.

## Codex log

## Claude review

## Verdict
