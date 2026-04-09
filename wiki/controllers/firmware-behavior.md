# Controller And Firmware Behavior

## Summary

The current `0.7.0` evidence suggests the validated CompactLogix and ControlLogix targets share the same major direct-write limitation profile, while differing more in seeded data state and topology than in core supported behavior.

## Current Understanding

- `confirmed`: The CompactLogix `5069-L320ERMS3` firmware `35` target and the ControlLogix `1756-L81ES` firmware `37` target both passed the main exercised feature set for `0.7.0`.
- `confirmed`: Both validated targets show the same main direct-write restriction pattern:
  - standalone `STRING` writes fail
  - `STRING` members inside UDTs fail when written directly
  - UDT array element members fail when written directly
- `confirmed`: Both targets supported the exercised read/write, batch, route-path, health-check, tag discovery, and UDT/nested access flows.
- `confirmed`: The ControlLogix validation specifically adds routed-path confidence via `1756-EN3TR` slot `0`.

## Notable Differences

### Topology

- CompactLogix validation was performed using a direct connection to `192.168.0.1:44818`.
- ControlLogix validation was performed using a routed connection to `192.168.0.101:44818` through `1756-EN3TR` slot `0`.

### Seeded Data Profile

- `confirmed`: On the validated ControlLogix target, controller-scoped test tags were mostly readable but seeded to zero or blank values.
- `confirmed`: Program-scoped tags under `Program:TestProgram.*` retained the richer seeded dataset on the same ControlLogix target.
- `confirmed`: The limitation profile still matched the CompactLogix target despite those seed-value differences.

### Observed Validation Notes

- `confirmed`: On the CompactLogix pass, native Multiple Service BOOL-array decoding was fixed during validation for packed `0x00D3` responses.
- `confirmed`: On the CompactLogix pass, complete-structure UDT reads returned `symbol_id = 0` in the exercised examples, while member access and nested access still worked for the tested scenarios.
- `confirmed`: No ControlLogix-only regression class was called out in the corresponding real-hardware validation record.

## Practical Guidance

- Treat both validated controller families as supported for the exercised `0.7.0` scenarios.
- Do not assume that seeded tag values or controller-scoped fixture state will match across hardware families.
- Keep firmware-specific claims tied to exact model and firmware references until more hardware evidence exists.

## Evidence

- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Open Questions

- Whether additional firmware revisions preserve the same direct-write limitation profile.
- Whether future pages should split controller-family behavior from firmware-specific deltas once more validation records exist.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
