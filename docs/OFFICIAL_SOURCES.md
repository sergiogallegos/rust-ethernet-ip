# Official Sources and Standards Traceability

This page tracks the **official technical documentation** used to design, validate, and maintain `rust-ethernet-ip`.

- Verified on: `2026-04-16`
- Working draft line: `1.0.0`
- Last published stable line: `0.7.0`
- Previous stable line: `0.6.3`

## Official Documentation Matrix

| Publication / Spec | Official Source | In-Repo Artifact | How We Use It |
|---|---|---|---|
| **1756-PM020I-EN-P** (Logix 5000 Controllers Data Access, Sep 2025) | https://literature.rockwellautomation.com/idc/groups/literature/documents/pm/1756-pm020_-en-p.pdf | [1756-pm020_-en-p.pdf](1756-pm020_-en-p.pdf) and derived notes ([CIP_PROTOCOL_REFERENCE_1756-PM020.md](CIP_PROTOCOL_REFERENCE_1756-PM020.md), [ARRAY_ELEMENT_ADDRESSING_GUIDE.md](ARRAY_ELEMENT_ADDRESSING_GUIDE.md), [AB_String_UDT_Write_Limitations.md](AB_String_UDT_Write_Limitations.md), [UDT_IMPLEMENTATION_REVIEW.md](UDT_IMPLEMENTATION_REVIEW.md)) | Primary reference for CIP services, symbolic path encoding, tag/UDT access, and known STRING/UDT behavior constraints. |
| **ENET-UM006C-EN-P** (EtherNet/IP Network Devices User Manual, Sep 2025) | https://literature.rockwellautomation.com/idc/groups/literature/documents/um/enet-um006_-en-p.pdf | Referenced externally (not currently mirrored in repo) | Network-device reference for EtherNet/IP terminology, TCP/CIP connection layering, explicit/implicit messaging, and UCMM context. |
| **1756-RM094N-EN-P** (Logix 5000 Controllers Design Considerations Reference Manual, Sep 2025) | https://literature.rockwellautomation.com/idc/groups/literature/documents/rm/1756-rm094_-en-p.pdf | Referenced externally (not currently mirrored in repo) | Secondary controller-family and network-design context; not a primary tag data-access implementation source. |
| **ENET-WP001A-EN-P** (EtherNet/IP Industrial Protocol White Paper) | https://literature.rockwellautomation.com/idc/groups/literature/documents/wp/enet-wp001_-en-p.pdf | [enet-wp001_-en-p.pdf](enet-wp001_-en-p.pdf) | Background context on EtherNet/IP architecture and CIP-over-Ethernet principles used for protocol-level framing decisions. |
| **PUB00213R0** (EtherNet/IP Quick Start for Vendors Handbook, ODVA) | ODVA publication catalog / member resources (ODVA managed) | [PUB00213R0_EtherNetIP_Developers_Guide.pdf](PUB00213R0_EtherNetIP_Developers_Guide.pdf) | Vendor-oriented implementation reference and interoperability guidance used during protocol behavior cross-checks. |
| **ENET-RM002D-EN-P** (Ethernet Reference Manual) | https://literature.rockwellautomation.com/idc/groups/literature/documents/rm/enet-rm002_-en-p.pdf | Referenced externally (not currently mirrored in repo) | Operational/network behavior reference for Ethernet-level integration and troubleshooting guidance. |
| **1756-PM012** (Logix 5000 Controller Messages Programming Manual) | https://literature.rockwellautomation.com/idc/groups/literature/documents/pm/1756-pm012_-en-p.pdf | Referenced externally (not currently mirrored in repo) | Message and controller communication patterns used as secondary guidance for route-path and messaging behavior. |
| **TypeEncode_CIPRW.pdf** (Rockwell type encoding note) | https://www.rockwellautomation.com/content/dam/rockwell-automation/sites/downloads/pdf/TypeEncode_CIPRW.pdf | Referenced externally (not currently mirrored in repo) | Extra detail for structure/tag type encoding analysis and edge-case behavior. |

## Coverage Notes

- The repository already includes the two most heavily used references:
  - `1756-pm020_-en-p.pdf`
  - `enet-wp001_-en-p.pdf`
- ODVA vendor guide (`PUB00213R0`) is also present in `docs/`.
- Some secondary references are currently link-only (external), not mirrored:
  - `ENET-UM006`
  - `1756-RM094`
  - `ENET-RM002`
  - `1756-PM012`
  - `TypeEncode_CIPRW.pdf`

## Feature Traceability Notes

The official-source set is considered sufficient for the current `1.0.0` release-candidate scope. The primary implementation authority is Rockwell `1756-PM020I-EN-P` for Logix tag data access, supported by ODVA and Rockwell EtherNet/IP networking references for protocol and routing context.

| Library Area | Primary Source | Repo Surface | Validation Evidence |
|---|---|---|---|
| Scalar tag read/write | `1756-PM020I-EN-P` | `src/client.rs`, `src/protocol/`, `src/types.rs` | Rust unit tests, simulator tests, real PLC validation |
| Symbolic tag paths, arrays, bits | `1756-PM020I-EN-P` | `src/tag_path.rs`, `src/protocol/`, `src/client.rs` | Codec/tag-path tests, simulator tests, real PLC validation |
| UDT discovery and UDT payload behavior | `1756-PM020I-EN-P`, `TypeEncode_CIPRW.pdf` | `src/udt.rs`, `src/types.rs`, `src/client.rs` | UDT tests, wrapper tests, real PLC validation |
| STRING and restricted write behavior | `1756-PM020I-EN-P`, `TypeEncode_CIPRW.pdf` | [`docs/AB_String_UDT_Write_Limitations.md`](AB_String_UDT_Write_Limitations.md), `src/client.rs` | CompactLogix and ControlLogix validation records |
| Multiple Service Packet / batch reads | `1756-PM020I-EN-P`, ODVA EtherNet/IP guide | `src/batch.rs`, `src/protocol/`, `src/client.rs` | Rust tests, simulator tests, C# and Python wrapper tests |
| ControlLogix routing / route paths | Rockwell EtherNet/IP networking docs, ODVA guide | `src/route.rs`, [`docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md`](CONTROLLOGIX_ROUTING_IMPLEMENTATION.md) | Routed ControlLogix validation records |
| FFI and wrapper behavior | Project ABI contract, not Rockwell-defined | `src/ffi.rs`, `csharp/`, `python/` | C# and Python contract tests |

The FFI, C#, and Python wrapper surfaces are project-defined compatibility contracts. They are validated against the Rust implementation and real PLC behavior, but they are not specified by Rockwell documentation.

## 2026-04-16 Official Publication Check

Rockwell's currently discoverable official material relevant to this library still points to `1756-PM020I-EN-P` (September 2025) as the primary Logix data-access reference. That revision is already tracked in-repo and remains the correct basis for CIP services, symbolic path encoding, tag reads/writes, Multiple Service Packet behavior, UDT access, and BOOL handling.

`ENET-UM006C-EN-P` (September 2025) is a relevant newer network-device source for EtherNet/IP connection terminology and message-type context. It does not supersede `1756-PM020I-EN-P` for tag data access, but it should be kept in the traceability matrix for routing, connection, explicit-message, and UCMM discussions.

`1756-RM094N-EN-P` (September 2025) is useful controller-family/network-design context. It is not currently an implementation reference for this library's tag data-access encoding.

No immediate protocol implementation change was identified from this source check.

## Recommendation for 1.0.0

For full traceability and reproducibility, keep this document updated when:

1. A new Rockwell publication revision is adopted.
2. A new ODVA publication is used for implementation decisions.
3. Any external (non-mirrored) reference is promoted to an in-repo artifact or intentionally left external.

## Licensing / Redistribution Note

Rockwell and ODVA publications can have distribution and usage restrictions depending on document type and license terms.  
Before mirroring additional official PDFs in-repo, confirm redistribution is allowed.
