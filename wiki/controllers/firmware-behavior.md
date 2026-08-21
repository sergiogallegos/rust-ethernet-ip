# Controller And Firmware Behavior

## Summary

Current evidence spans five exact processor/firmware combinations. The strongest
current-release result is the `1.2.0` four-binding gate on CompactLogix
`5069-L330ERM` firmware `38`; older rows remain useful historical evidence but
must not be treated as `1.2.0` certification.

## Current Understanding

- `confirmed`: CompactLogix `5069-L330ERM` firmware `38` passed the `1.2.0`
  release gate across Rust, C#, Python, and C/C++ with 2,338 reads, 2,319
  writes/read-back verifications, and zero anomalies.
- `confirmed`: On that target, standalone and UDT-member built-in/custom
  strings write through handle-aware and fragmented paths; scalar UDT array
  element members also write when the full path is preserved.
- `confirmed`: Historical physical coverage also includes CompactLogix
  `5069-L320ERMS3` fw35 and `1769-L18ER-BB1B` fw33 plus ControlLogix
  `1756-L75` fw33 and `1756-L81ES` fw37.
- `confirmed`: Routed single-chassis access is proven through `1756-EN2T` and
  `1756-EN3TR`; true multi-hop routing remains unvalidated.
- `superseded`: The `0.7.0` interpretation that direct standalone STRING,
  STRING-member, and scalar UDT-array-member writes were firmware-blocked was
  corrected by the `1.2.0` encoding/path fixes and fw38 hardware probes.

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

- Use [the authoritative hardware matrix](../../docs/HARDWARE_COMPATIBILITY.md)
  for exact model/firmware/binding claims.
- Do not assume that seeded tag values or controller-scoped fixture state will match across hardware families.
- Keep firmware-specific claims tied to exact model and firmware references until more hardware evidence exists.

## Evidence

- [docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md](../../docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md)
- [docs/HARDWARE_COMPATIBILITY.md](../../docs/HARDWARE_COMPATIBILITY.md)

## Open Questions

- Whether additional firmware revisions preserve the same direct-write limitation profile.
- Whether future pages should split controller-family behavior from firmware-specific deltas once more validation records exist.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
