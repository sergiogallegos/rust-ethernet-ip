# STRING And UDT Write Behavior

## Summary

Current mainline separates three formerly conflated `0x2107` cases:

- standalone standard Logix `STRING` writes are confirmed writeable with the correct structure encoding;
- scalar UDT array element member writes are confirmed writeable on the 2026-07-03 validation target when the full member path is preserved;
- `STRING` members inside UDTs and UDT array elements still reject with `0xFF/0x2107` under the current member encoding, so the stable workaround remains whole-structure read-modify-write.

## Current Understanding

- `confirmed`: On 2026-07-02, a 5069-L330ERM fw38 accepted direct standalone `STRING` writes when encoded as structure type `0x02A0`, standard STRING handle `0x0FCE`, element count `1`, and an 88-byte payload.
- `confirmed`: Older standalone `STRING` failures were library encoding failures, not proof of a firmware prohibition.
- `confirmed`: On 2026-07-03, the CODEX-AV matrix showed all 60 scalar UDT-array-element-member targets (DINT, REAL, BOOL, INT; controller and program scopes) wrote successfully on 5069-L330ERM fw38.
- `confirmed`: `STRING` members inside UDTs and UDT array elements still rejected with `0xFF/0x2107` under the current member encoding on 5069-L330ERM fw38.
- `confirmed`: CODEX-AP retired old exploratory STRING writers and offset-based UDT member APIs as unsupported compatibility stubs; maintained writes use `write_tag`, `write_string_tag`, direct member tag writes, or service-layer helpers.

## Recommended Patterns

- For standalone standard `STRING` tags:
  - Use `write_tag(..., PlcValue::String(...))`, `write_string_tag`, or wrapper `WriteString`/`write_tag` calls that route to the maintained structure encoding.
- For `STRING` members inside a UDT:
  - Read the containing UDT, modify in memory, and write the whole UDT back.
- For scalar UDT array element members:
  - Use the direct member path; service-layer helpers fall back to whole-UDT read-modify-write only on the `0x2107` data-type mismatch shape.
- For UDT array element `STRING` members:
  - Read the full element, modify the member in memory, and write the full element back.

## What This Is Not

- `superseded`: Blanket claims that firmware blocks standalone `STRING` writes or scalar UDT-array-element-member writes are obsolete for current mainline.
- `confirmed`: `0x2107` is a Read/Write Tag data-type mismatch. It can indicate malformed library encoding, stale UDT symbol IDs, or current-encoding `STRING` member rejection; it should not be flattened into a generic firmware ban.
- `needs-care`: Historical notes may use stronger wording about universal PLC behavior than the current evidence strictly proves. Keep current claims tied to dated validation sources and tested targets.

## Evidence

- [docs/AB_String_UDT_Write_Limitations.md](../../docs/AB_String_UDT_Write_Limitations.md)
- [docs/agents/notes/ab-firmware-quirks.md](../../docs/agents/notes/ab-firmware-quirks.md)
- [docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md](../../docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md)
- [docs/validation/2026-07-03_blocked_write_label_probe_plan.md](../../docs/validation/2026-07-03_blocked_write_label_probe_plan.md)
- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Open Questions

- Whether CODEX-AO packet captures identify a member-specific direct encoding for `STRING` members inside UDTs.
- Whether scalar UDT-array-element-member direct writes hold across older ControlLogix and CompactLogix firmware, beyond the 5069-L330ERM fw38 CODEX-AV matrix.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
