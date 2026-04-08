# Official Sources and Standards Traceability

This page tracks the **official technical documentation** used to design, validate, and maintain `rust-ethernet-ip`.

- Verified on: `2026-04-07`
- Stable release line: `0.7.0`
- Previous stable line: `0.6.3`

## Official Documentation Matrix

| Publication / Spec | Official Source | In-Repo Artifact | How We Use It |
|---|---|---|---|
| **1756-PM020I-EN-P** (Logix 5000 Controllers Data Access, Sep 2025) | https://literature.rockwellautomation.com/idc/groups/literature/documents/pm/1756-pm020_-en-p.pdf | [1756-pm020_-en-p.pdf](1756-pm020_-en-p.pdf) and derived notes ([CIP_PROTOCOL_REFERENCE_1756-PM020.md](CIP_PROTOCOL_REFERENCE_1756-PM020.md), [ARRAY_ELEMENT_ADDRESSING_GUIDE.md](ARRAY_ELEMENT_ADDRESSING_GUIDE.md), [AB_String_UDT_Write_Limitations.md](AB_String_UDT_Write_Limitations.md), [UDT_IMPLEMENTATION_REVIEW.md](UDT_IMPLEMENTATION_REVIEW.md)) | Primary reference for CIP services, symbolic path encoding, tag/UDT access, and known STRING/UDT behavior constraints. |
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
  - `ENET-RM002`
  - `1756-PM012`
  - `TypeEncode_CIPRW.pdf`

## Recommendation for 0.7.0 Hardening

For full traceability and reproducibility, keep this document updated when:

1. A new Rockwell publication revision is adopted.
2. A new ODVA publication is used for implementation decisions.
3. Any external (non-mirrored) reference is promoted to an in-repo artifact or intentionally left external.

## Licensing / Redistribution Note

Rockwell and ODVA publications can have distribution and usage restrictions depending on document type and license terms.  
Before mirroring additional official PDFs in-repo, confirm redistribution is allowed.
