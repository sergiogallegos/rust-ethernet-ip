# Documentation Index

This folder contains project documentation for `rust-ethernet-ip`.

Current release state:
- Stable published line: `0.7.0`
- Working draft line: `0.7.1` (not published)
- Previous stable line: `0.6.3`

## Start Here

- [../README.md](../README.md): Project overview and quick start
- [programmer_manual.md](programmer_manual.md): Programmer manual (Rust + C# integration tracks)
- [OFFICIAL_SOURCES.md](OFFICIAL_SOURCES.md): Official Rockwell/ODVA references used by this library
- [../CHANGELOG.md](../CHANGELOG.md): Release history and ongoing changes
- [../CONTRIBUTING.md](../CONTRIBUTING.md): Contribution workflow and required checks
- [../wiki/README.md](../wiki/README.md): Maintainer-oriented knowledge wiki for synthesized engineering context

## Release and Quality Gates

- [release/0.7.1_RELEASE_NOTES_DRAFT.md](release/0.7.1_RELEASE_NOTES_DRAFT.md)
- [0.7.0_HARDENING_GATE.md](0.7.0_HARDENING_GATE.md)
- [validation/REAL_PLC_TESTING.md](validation/REAL_PLC_TESTING.md)
- [validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [audit/0.7.0_docs_api_audit.md](audit/0.7.0_docs_api_audit.md)
- [audit/0.7.0_open_issues_triage.md](audit/0.7.0_open_issues_triage.md)
- [compat/0.7.0_route_udt_compatibility.md](compat/0.7.0_route_udt_compatibility.md)
- [compat/0.7.0_plc_simulator_compatibility_matrix.md](compat/0.7.0_plc_simulator_compatibility_matrix.md)
- [perf/0.7.0_baseline_vs_0.6.3.md](perf/0.7.0_baseline_vs_0.6.3.md)

## Core Technical References

- [CIP_PROTOCOL_REFERENCE_1756-PM020.md](CIP_PROTOCOL_REFERENCE_1756-PM020.md)
- [EtherNetIP_Connection_Paths_and_Routing.md](EtherNetIP_Connection_Paths_and_Routing.md)
- [CONTROLLOGIX_ROUTING_IMPLEMENTATION.md](CONTROLLOGIX_ROUTING_IMPLEMENTATION.md)
- [ARRAY_ELEMENT_ADDRESSING_GUIDE.md](ARRAY_ELEMENT_ADDRESSING_GUIDE.md)
- [tag_introspection.md](tag_introspection.md)
- [AB_String_UDT_Write_Limitations.md](AB_String_UDT_Write_Limitations.md)

## Troubleshooting and Operations

- [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)
- [DLL_DEPLOYMENT.md](DLL_DEPLOYMENT.md)

## Historical / Deep-Dive Notes

Some files in this folder are historical analyses from earlier release phases and may include old planning timelines.

Use them for context, but treat the following as authoritative for current behavior:
- `README.md`
- `CHANGELOG.md`
- `0.7.0_HARDENING_GATE.md`
- `audit/0.7.0_docs_api_audit.md`
