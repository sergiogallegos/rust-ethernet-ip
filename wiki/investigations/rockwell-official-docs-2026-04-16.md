# Rockwell Official Documentation Check 2026-04-16

## Summary

The repository remains aligned with Rockwell's current official Logix data-access publication found during the 2026-04-16 check. `1756-PM020I-EN-P` from September 2025 is still the primary implementation reference for Logix tag/CIP data access.

The main new synthesis item is traceability, not code: `ENET-UM006C-EN-P` from September 2025 should be tracked as a relevant EtherNet/IP network-device reference for connection and messaging terminology.

## Current Understanding

- `confirmed`: `1756-PM020I-EN-P` is already listed in [../../docs/OFFICIAL_SOURCES.md](../../docs/OFFICIAL_SOURCES.md) and remains the primary source for CIP services, symbolic paths, tag reads/writes, Multiple Service Packet behavior, UDT access, and BOOL handling.
- `confirmed`: `ENET-UM006C-EN-P` is relevant for EtherNet/IP terminology, TCP/CIP connection layering, explicit/implicit messaging, and UCMM context.
- `likely`: `1756-RM094N-EN-P` is useful background for controller families and network planning, but it is not a direct implementation source for tag data-access encoding.
- `confirmed`: No immediate protocol implementation change was identified from this publication check.

## Evidence

- [docs/OFFICIAL_SOURCES.md](../../docs/OFFICIAL_SOURCES.md)
- [docs/release/0.8.0_RELEASE_NOTES_DRAFT.md](../../docs/release/0.8.0_RELEASE_NOTES_DRAFT.md)
- Rockwell `1756-PM020I-EN-P`, Logix 5000 Controllers Data Access, September 2025
- Rockwell `ENET-UM006C-EN-P`, EtherNet/IP Network Devices User Manual, September 2025
- Rockwell `1756-RM094N-EN-P`, Logix 5000 Controllers Design Considerations Reference Manual, September 2025

## Open Questions

- Should `ENET-UM006C-EN-P` be mirrored locally, or kept as an external link to avoid redistribution uncertainty?
- Should route-path and connection-limit documentation cite `ENET-UM006C-EN-P` directly where the current docs rely mostly on older local notes?

## Related Pages

- [../protocol/route-path-behavior.md](../protocol/route-path-behavior.md)
- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
