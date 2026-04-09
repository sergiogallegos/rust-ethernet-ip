# STRING And UDT Write Behavior

## Summary

The main direct-write failures observed in this project are controller-imposed limitations around standalone `STRING` writes, `STRING` members inside UDTs, and UDT array element member writes. The stable workaround is whole-structure read-modify-write.

## Current Understanding

- `confirmed`: Standalone `STRING` writes can fail with CIP extended error `0x2107`.
- `confirmed`: Direct writes to `STRING` members inside UDTs can fail with the same limitation profile.
- `confirmed`: Direct writes to UDT array element members can fail with CIP extended error `0x2107`.
- `confirmed`: Writing the entire UDT array element is supported and is the preferred workaround.
- `confirmed`: These behaviors were observed on both the CompactLogix `5069-L320ERMS3` firmware `35` target and the ControlLogix `1756-L81ES` firmware `37` target validated for `0.7.0`.

## Recommended Patterns

- For standalone `STRING` cases:
  - Prefer a PLC-side intermediary buffer and ladder logic copy when direct external writes are required.
- For `STRING` members inside a UDT:
  - Read the containing UDT, modify in memory, and write the whole UDT back.
- For UDT array element members:
  - Read the full element, modify the member in memory, and write the full element back.

## What This Is Not

- `confirmed`: This is not currently treated as a library protocol bug.
- `confirmed`: The behavior is documented against Rockwell guidance and is reflected in real-hardware validation evidence.
- `needs-care`: Historical notes may use stronger wording about universal PLC behavior than the current evidence strictly proves. In the wiki, keep claims tied to documented sources and tested targets.

## Evidence

- [docs/AB_String_UDT_Write_Limitations.md](../../docs/AB_String_UDT_Write_Limitations.md)
- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Open Questions

- Whether there are specific controller or firmware combinations where `.LEN` and `.DATA` member-level workarounds succeed reliably.
- Whether a future library abstraction should formalize PLC-side intermediary patterns instead of leaving them as documentation guidance.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
