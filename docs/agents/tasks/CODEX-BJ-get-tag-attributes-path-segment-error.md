---
id: CODEX-BJ
title: Investigate get_tag_attributes/get_udt_definition path-segment error on a live controller-scope UDT tag
owner: claude
status: merged
created: 2026-08-22
last-update: 2026-08-22 claude [Sonnet 5]
---

## Brief

### Priority and dependency

Flagged for maintainer triage — severity and scope are not yet known.
Not blocking 1.2.1: the schema-refresh/drift-recovery mechanism CODEX-BA
through CODEX-BD validate does not depend on this code path (the live BD
UDT-section gate was rewritten to route around it). No task dependency.

### Context

During the CODEX-BD live UDT-layout gate session (2026-08-22, ControlLogix
1756-L75 firmware 33, `1756-EN2T` bridge slot 1, host macOS 26.5.2/`25F84`,
commit `95fc581`), `EipClient::get_udt_definition("gSchemaUdt")` — which
calls `get_tag_attributes()` first — failed on a real, controller-scope,
freshly created UDT-typed tag (`gSchemaUdt`, instance of a two-member UDT:
`Marker` DINT + `Flags` BOOL[64]):

```
Protocol error: Get Attribute List for 'gSchemaUdt' failed: Path segment error
```

Isolated with a throwaway probe (`examples/_tmp_udt_probe.rs`, not
committed) before building the live gate tool, with these results against
the same tag on the same connection:

| Call | Result |
|---|---|
| `client.read_tag("gSchemaUdt")` | OK — `Udt(UdtData { symbol_id: 0, data: [...14 bytes...] })` |
| `client.read_tag("gSchemaUdt.Marker")` | OK — `Dint(0)` |
| `client.read_tag("gSchemaUdt.Flags")` | OK — `Udint(0)` |
| `client.read_tag("gSchemaUdt.Flags[0]")` | OK — `Bool(false)` |
| `client.get_tag_attributes("gSchemaUdt")` | **FAIL** — path segment error |
| `client.discover_tags_detailed()` | OK — includes `TagAttributes { name: "gSchemaUdt", data_type: 160, data_type_name: "UDT", dimensions: [], permissions: ReadWrite, scope: Controller, template_instance_id: Some(2970), size: 0 }` |

Two things stand out:

1. `read_tag` on the whole UDT returns `symbol_id: 0` in the `UdtData` —
   worth checking whether that's expected for this tag/path or itself a
   symptom.
2. The bulk `discover_tags_detailed()` sweep (which internally uses a
   different CIP mechanism to enumerate all symbols) already carries the
   `template_instance_id` that `get_tag_attributes`'s single-tag
   `Get_Attribute_List` (service `0x03`, built by
   `build_get_attributes_request` at `src/client.rs:2725`) fails to
   retrieve directly.

The CODEX-BD live UDT gate (`examples/schema_udt_gate_live.rs`) was written
to use `read_tag()` (whole-UDT payload length) plus `discover_tags_detailed()`
(`template_instance_id`) as its layout-change signal specifically to route
around this failure, so it did not block validating the session-survival /
refresh / rediscovery mechanism that section exists to test. See the
"Rust — UDT layout edit + download + rediscovery detail" section of
[`docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md`](../../validation/2026-08-22_1756-L75_fw33_schema-change-gate.md).

### Required investigation

1. Reproduce against the same or an equivalent controller-scope UDT tag.
   Determine whether the failure is:
   - specific to freshly created/downloaded tags (e.g. a symbol-database
     staleness window before some other operation "warms" it — note
     `read_tag` and `discover_tags_detailed` both succeeded on the same
     tag in the same session, so if this is warmth-related the warming
     event is something other than a plain read or bulk discovery);
   - specific to this firmware/processor family (1756-L75 fw33);
   - specific to controller scope vs program scope;
   - a general bug in `build_get_attributes_request`'s path encoding for
     bare (non-array, non-member) UDT tag names; or
   - something else entirely.
2. Compare `build_get_attributes_request`'s CIP path construction
   (`src/client.rs:2725`, uses `self.build_tag_path(tag_name)`) against the
   working `read_tag` path for the same tag name to find the actual wire
   difference. A packet capture on the reproducing hardware would settle
   this quickly if available.
3. Determine blast radius: `get_udt_definition`, `get_tag_attributes`, and
   anything downstream of them (UDT discovery/parsing workflows,
   `discover_udt_members`) are public API — if this affects real UDT-typed
   tags broadly (not just this one test fixture), it's a correctness bug
   worth prioritizing; if it's narrow/tag-specific, it may just need a
   documented workaround note in `docs/agents/notes/ab-firmware-quirks.md`.

### Test requirements

- A reproducing simulator test if the root cause is protocol-encoding
  related (add a `SimulatedPlc` behavior that rejects this specific request
  shape, matching the real controller's response).
- If a code fix lands, a regression test proving `get_tag_attributes`/
  `get_udt_definition` succeed against the previously-failing shape.

### Acceptance criteria

- Root cause identified and documented (in this task file at minimum; in
  `docs/agents/notes/ab-firmware-quirks.md` if it's a firmware-specific
  quirk worth other implementers knowing about).
- Either a fix with regression coverage, or an explicit maintainer decision
  to leave it as a documented limitation with a workaround (as CODEX-BD's
  live gate tool already demonstrates: `read_tag()` +
  `discover_tags_detailed()` in place of `get_udt_definition()`).

### Out of scope

- Live-hardware re-validation beyond what's needed to confirm a fix — that
  stays maintainer-controlled per `docs/agents/notes/release-hardware-validation.md`.
- Changing `discover_tags_detailed()`'s behavior; it already works.

## Codex log

_(none — Codex hit its usage limit before picking this up; implemented directly by
Claude at maintainer direction, per the CODEX-AY precedent, using the still-live
1756-L75 connection from the CODEX-BD session.)_

## Claude review

### 2026-08-22 19:15  claude [Sonnet 5]

**Investigation**

Reproduced against the live 1756-L75/B fw33.011 (via 1756-EN2T/D fw10.007,
backplane routed, slot 0) with a throwaway probe before touching any code.
Result: `get_tag_attributes()` fails with `Protocol error: Get Attribute List
for '<tag>' failed: Path segment error` for **every** tag shape tested —
`gTestUDT` (long-established controller-scope UDT), `gTestUDT_Array[0]`
(array element), `Program:TestProgram.gTestUDT` (program scope),
`gSchemaUdt` (freshly created that session), and `gTestArray_DINT` (a
non-UDT atomic array). 100% failure, ruling out every hypothesis in the
brief's "specific to X" list except a general library-or-firmware
incompatibility with this exact request shape on this route.

Path encoding was ruled out directly: `build_get_attributes_request`
(`src/client.rs:2725` at brief-authoring time) and `build_read_request`
both call the identical `self.build_tag_path(tag_name)` — confirmed by
reading both functions side by side. Since reads against the exact same
tag names succeed on the exact same connection, the wire path bytes are
proven byte-identical between a working request and a failing one; only
the service byte (`0x03` vs `0x4C`) and request-data payload differ.

`discover_tags_detailed()` (CIP service `0x55`, a bulk Symbol-Object sweep)
succeeds against the same tags on the same connection and already reports
`template_instance_id` — the field both of `get_tag_attributes`'s internal
callers (`get_udt_definition`, `write_tag`'s zero-`symbol_id` UDT-write
fallback) actually need. Grepped the repo for prior real-hardware evidence
of `get_tag_attributes`/`get_udt_definition`: none found — every historical
UDT validation record goes through `read_udt_chunked`/`read_tag` instead,
so this path had apparently never been hardware-tested before.

**Fix**

`get_tag_attributes` (`src/client.rs`) now tries the existing direct
per-tag request first — unchanged fast path, split out unchanged into
`get_tag_attributes_direct` — and only on failure falls back to
`get_tag_attributes_via_discovery`, a new helper that parses the tag name
with `TagPath::parse` (already used elsewhere in this file), resolves the
base tag name and program scope, calls `discover_tags_detailed()` or
`discover_program_tags()` accordingly, and matches by name. Both discovery
methods already return `Vec<TagAttributes>` — the same type — so the
fallback is a straight lookup, not a conversion.

**Test proof**

- Live re-verification (same connection style as the investigation probe):
  all five previously-failing cases now succeed, including through the
  array-element and program-scoped paths (base-name/scope resolution
  confirmed correct); `get_udt_definition("gSchemaUdt")` now succeeds
  end-to-end and correctly reports both members (`Marker`, `Flags`).
- Added `simulated_plc_get_tag_attributes_unknown_tag_still_fails_after_discovery_fallback`
  to `tests/plc_sim_tests.rs`, plus a new `SimBehavior.reject_get_attribute_list`
  toggle (and the simulator plumbing to honor it) that reproduces the
  100%-failure shape observed on real hardware. Confirmed the two
  pre-existing `get_tag_attributes` tests (`known_tag`, `unknown_tag_returns_error`)
  still pass unchanged.
- **Did not** add a simulator-level positive-fallback test (proving recovery
  succeeds, not just fails cleanly): the simulator has no CIP service `0x55`
  handler at all, so `discover_tags_detailed`/`discover_program_tags` cannot
  succeed against `SimulatedPlc` yet — attempted this first, it failed with
  a transport-level error unrelated to this fix, confirmed by inspection
  that service `0x55` isn't in the simulator's request dispatch match arm.
  This is a pre-existing simulator gap, out of this fix's scope per the
  brief's "Out of scope: Changing discover_tags_detailed()'s behavior."
  Documented as a known test gap in `ab-firmware-quirks.md` rather than
  silently leaving it uncovered.
- Full matrix: `cargo fmt -- --check`, `cargo clippy --workspace
  --all-targets --all-features -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test
  --workspace --locked` (0 failures), `cargo test --test plc_sim_tests
  --locked` (26/26, was 25/25), `scripts/schema-change-gate` (all 7 steps
  PASS) — all clean.

**Residual risk**

- The true wire-level reason real ControlLogix rejects the symbolic-path
  Get Attribute List request is not conclusively identified — no packet
  capture was taken. The leading hypothesis (real hardware wants Class
  `0x6B` numeric-instance addressing for this specific service, inferred
  from the discovery service's success) is documented as a hypothesis, not
  fact, in `ab-firmware-quirks.md`.
- The discovery-sourced `TagAttributes.size` field is always `0`. Neither
  internal caller needs it, but a future public caller relying on
  `get_tag_attributes(...).size` after hitting this fallback would get an
  inaccurate value — documented, not silently hidden.
- Not tested against a *directly-connected* (unrouted) controller, so
  whether this is specific to routed/Unconnected-Send framing or general to
  this firmware regardless of route remains open.

**Acceptance criteria tally**
- ✅ Root cause identified and documented (this file + `ab-firmware-quirks.md`,
  with the "not conclusively identified at the wire level" caveat stated
  honestly rather than overclaimed).
- ✅ Fix with regression coverage (live-hardware-verified fix; simulator
  covers the failure-propagation shape; positive-path simulator coverage
  blocked by a documented pre-existing gap, not silently skipped).

## Verdict

Root-caused and fixed. The failure is universal (any tag, any shape, any
scope) on this controller/route, proven unrelated to path encoding (the
identical path succeeds for reads), and the fix restores correct behavior
using a mechanism (`discover_tags_detailed`/`discover_program_tags`) this
library already had and already trusted. Live-hardware-verified for every
case the original investigation found failing. Merging — the CIP-level
"why does real hardware reject this specific request" question stays open
as documented residual risk, not as a blocker.
