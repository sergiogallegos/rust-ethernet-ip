---
id: CODEX-BJ
title: Investigate get_tag_attributes/get_udt_definition path-segment error on a live controller-scope UDT tag
owner: codex
status: open
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

## Claude review

## Verdict
