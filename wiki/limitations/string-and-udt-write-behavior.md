# STRING And UDT Write Behavior

## Summary

Current mainline separates formerly conflated `0x2107` and packet-size cases:

- standalone standard Logix `STRING` writes are confirmed writeable with the correct structure encoding;
- custom Logix string types and UDT string members are writeable when the request uses the target's real structure handle;
- scalar UDT array element member writes are confirmed writeable on the 2026-07-03 validation target when the full member path is preserved;
- large custom strings now have simulator-covered CIP fragmented read/write support, with hardware re-validation still pending for `Str500+`.

## Current Understanding

- `confirmed`: On 2026-07-02, a 5069-L330ERM fw38 accepted direct standalone `STRING` writes when encoded as structure type `0x02A0`, standard STRING handle `0x0FCE`, element count `1`, and an 88-byte payload.
- `confirmed`: Older standalone `STRING` failures were library encoding failures, not proof of a firmware prohibition.
- `confirmed`: On 2026-07-03, the CODEX-AV matrix showed all 60 scalar UDT-array-element-member targets (DINT, REAL, BOOL, INT; controller and program scopes) wrote successfully on 5069-L330ERM fw38.
- `confirmed`: On 2026-07-08, `STRING` members inside UDTs and UDT array elements were shown to be custom string types whose earlier `0x2107` failures came from sending the built-in `STRING` handle `0x0FCE` instead of the target's real structure handle.
- `confirmed`: CODEX-AY made public string writes handle-aware for built-in and custom string types; CODEX-AX relabeled the shared full-coverage manifest so all 17 former `encoding_blocked_udt_string_member` targets are now `writeable`.
- `sim-confirmed`: CODEX-AZ added CIP Read Tag Fragmented (`0x52`) and Write Tag Fragmented (`0x53`) support and simulator coverage for a 600-byte custom string. Real-hardware `Str500+` confirmation remains pending.
- `confirmed`: CODEX-AP retired old exploratory STRING writers and offset-based UDT member APIs as unsupported compatibility stubs; maintained writes use `write_tag`, `write_string_tag`, direct member tag writes, or service-layer helpers.
- `confirmed`: CODEX-AO Phase 1 closed a read-modify-write safety gap in
  `crates/udt`: truncated UDT bytes and missing member-map values now error
  instead of silently skipping members or zero-filling bytes. This does not
  resolve the capture-gated UDT wire-format question.

## Recommended Patterns

- For standalone standard `STRING` tags:
  - Use `write_tag(..., PlcValue::String(...))`, `write_string_tag`, or wrapper `WriteString`/`write_tag` calls that route to the maintained structure encoding.
- For built-in or custom `STRING` members inside a UDT:
  - Use the string write/read APIs (`write_tag(..., PlcValue::String(...))`,
    `write_string_tag`, wrapper `WriteString` / `write_tag`, and typed string
    reads). Current mainline discovers the target's real structure handle.
- For scalar UDT array element members:
  - Use the direct member path; service-layer helpers fall back to whole-UDT read-modify-write only on the `0x2107` data-type mismatch shape.
- For UDT array element `STRING` members:
  - Use the direct member path through the string APIs; the old RMW-only
    recommendation is superseded for validated custom string members.

## What This Is Not

- `superseded`: Blanket claims that firmware blocks standalone `STRING` writes or scalar UDT-array-element-member writes are obsolete for current mainline.
- `confirmed`: `0x2107` is a Read/Write Tag data-type mismatch. It can indicate malformed library encoding, stale UDT symbol IDs, or a structure-handle mismatch; it should not be flattened into a generic firmware ban.
- `needs-care`: Historical notes may use stronger wording about universal PLC behavior than the current evidence strictly proves. Keep current claims tied to dated validation sources and tested targets.

## Evidence

- [docs/AB_String_UDT_Write_Limitations.md](../../docs/AB_String_UDT_Write_Limitations.md)
- [docs/agents/notes/ab-firmware-quirks.md](../../docs/agents/notes/ab-firmware-quirks.md)
- [docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md](../../docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md)
- [docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md](../../docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md)
- [docs/STRING_HANDLING.md](../../docs/STRING_HANDLING.md)
- [docs/validation/2026-07-03_blocked_write_label_probe_plan.md](../../docs/validation/2026-07-03_blocked_write_label_probe_plan.md)
- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Open Questions

- Whether CODEX-AO packet captures confirm the current collapsed
  `0x02A0 + symbol_id` structure write type or require marker-plus-handle
  encoding.
- Whether scalar UDT-array-element-member direct writes hold across older ControlLogix and CompactLogix firmware, beyond the 5069-L330ERM fw38 CODEX-AV matrix.
- Whether CODEX-AZ fragmented large-string reads/writes validate unchanged on
  real `Str500+` controller and program-scope tags.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
