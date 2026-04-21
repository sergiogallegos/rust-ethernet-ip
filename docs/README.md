# Documentation Index

This folder contains project documentation for `rust-ethernet-ip`.

Current release state:
- Stable published line: `0.7.0`
- Working draft line: `0.8.0` (not published)
- Previous stable line: `0.6.3`

## Start Here

- [../README.md](../README.md): Project overview and quick start
- [programmer_manual.md](programmer_manual.md): Programmer manual (Rust + C# integration tracks)
- [INTEGRATION_AND_DEPLOYMENT.md](INTEGRATION_AND_DEPLOYMENT.md): Step-by-step integration and deployment guide for Rust, C#, and Python users
- [SOFTWARE_ARCHITECTURE.md](SOFTWARE_ARCHITECTURE.md): Current architecture map, design boundaries, and refactor guidance
- [PLATFORM_EXPANSION_BACKLOG.md](PLATFORM_EXPANSION_BACKLOG.md): Planned Python/data-platform expansion backlog and rationale
- [CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md](CODEX_PYTHON_PLATFORM_EXPANSION_PROMPT.md): Detailed Codex prompt for planning and implementing the Python/data-platform path
- [PYTHON_WRAPPER_STRATEGY.md](PYTHON_WRAPPER_STRATEGY.md): Current recommendation and plan shape for the future Python wrapper
- [PYTHON_MVP_API_AND_FFI_MAPPING.md](PYTHON_MVP_API_AND_FFI_MAPPING.md): Concrete Python MVP API, FFI mapping, and identified native-layer gaps
- [research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md](research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md): Curated papers and technical references for future platform/data-layer improvements
- [RESEARCH_FEATURE_MAP.md](RESEARCH_FEATURE_MAP.md): Mapping from the curated research papers to concrete repo components, priorities, and recommended implementation order
- [METADATA_SCHEMA_EXPORT_DESIGN.md](METADATA_SCHEMA_EXPORT_DESIGN.md): Proposed stable export contract for tag and UDT metadata built on the current Rust discovery APIs
- [COLLECTOR_SERVICE_MVP_DESIGN.md](COLLECTOR_SERVICE_MVP_DESIGN.md): Proposed first collector-service shape using the Python wrapper, batch polling, and CSV/SQLite sinks
- [REST_MQTT_ADAPTER_BOUNDARIES.md](REST_MQTT_ADAPTER_BOUNDARIES.md): Recommended ownership boundaries and transport shape for future REST and MQTT adapters
- [MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md](MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md): Recommended next diagnostics and health-surface improvements for the Rust core and thin wrappers
- [DOCKER_EXAMPLE_STACKS.md](DOCKER_EXAMPLE_STACKS.md): Local Docker packaging for the Python API, collector, and optional MQTT example services
- [OFFICIAL_SOURCES.md](OFFICIAL_SOURCES.md): Official Rockwell/ODVA references used by this library
- [../CHANGELOG.md](../CHANGELOG.md): Release history and ongoing changes
- [../CONTRIBUTING.md](../CONTRIBUTING.md): Contribution workflow and required checks
- [../wiki/README.md](../wiki/README.md): Maintainer-oriented knowledge wiki for synthesized engineering context

## Release and Quality Gates

- [release/0.8.0_RELEASE_NOTES_DRAFT.md](release/0.8.0_RELEASE_NOTES_DRAFT.md)
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

- [INTEGRATION_AND_DEPLOYMENT.md](INTEGRATION_AND_DEPLOYMENT.md)
- [TROUBLESHOOTING.md](TROUBLESHOOTING.md)
- [VERSION_MANAGEMENT.md](VERSION_MANAGEMENT.md)
- [DLL_DEPLOYMENT.md](DLL_DEPLOYMENT.md)

## Historical / Deep-Dive Notes

Some files in this folder are historical analyses from earlier release phases and may include old planning timelines.
Some older files now include an explicit `Historical reference` banner near the top; when present, treat that file as traceability context rather than the current source of truth.

Use them for context, but treat the following as authoritative for current behavior:
- `../README.md`
- `programmer_manual.md`
- `release/0.8.0_RELEASE_NOTES_DRAFT.md`
- current records in `validation/`
- the maintainer wiki in `../wiki/`
