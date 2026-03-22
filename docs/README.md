# Documentation Index

This folder contains project documentation for `rust-ethernet-ip`.

Current release state:
- Stable published line: `0.6.3`
- Active hardening line on `main`: `0.7.0` (unreleased)

## Start Here

- [../README.md](../README.md): Project overview and quick start
- [programmer_manual.md](programmer_manual.md): Programmer manual (Rust + C# integration tracks)
- [OFFICIAL_SOURCES.md](OFFICIAL_SOURCES.md): Official Rockwell/ODVA references used by this library
- [../CHANGELOG.md](../CHANGELOG.md): Release history and unreleased changes
- [../CONTRIBUTING.md](../CONTRIBUTING.md): Contribution workflow and required checks

## Release and Quality Gates

- [0.7.0_HARDENING_GATE.md](0.7.0_HARDENING_GATE.md)
- [audit/0.7.0_docs_api_audit.md](audit/0.7.0_docs_api_audit.md)
- [compat/0.7.0_route_udt_compatibility.md](compat/0.7.0_route_udt_compatibility.md)
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
