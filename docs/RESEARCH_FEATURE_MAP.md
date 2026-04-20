# Research Feature Map

This document maps the curated research papers in [research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md](research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md) to concrete repo components, implementation ideas, and timing.

Date: 2026-04-19

## Summary

The papers are most useful for:

- metadata and schema direction
- Python and data-workflow examples
- collector, API, and MQTT service layers
- performance and topology decisions above the Rust core

They are less useful for:

- low-level EtherNet/IP packet handling
- controller-specific protocol correctness

That split matters. The Rust core should remain grounded in Rockwell behavior, validation records, and the current codebase. The papers are mainly useful for the layers around it.

## Priority Buckets

## 0.8.0 Candidate Scope

These fit the current active direction without diluting the repo's core mission.

- Python MVP wrapper built on the existing FFI boundary
- Python examples for CSV, SQLite, and analytics-friendly collection
- stronger architecture and wrapper-boundary documentation
- backlog definition for metadata/schema export
- backlog definition for collector/API/MQTT adapters

## Post-0.8.0 Near-Term

- metadata and schema export design
- collector service MVP
- REST API example/service
- MQTT publisher example/service
- structured snapshot/export format for controller state

## Longer-Term

- richer structured asset models
- event-streaming adapters
- anomaly and diagnostics hooks
- historian and OEE starter patterns

## Paper-to-Feature Mapping

## Paper 1. OPC UA Performance Analysis

- Repo component:
  - docs and architecture
  - future metadata/schema layer
- Best idea:
  - justify richer structured access without confusing it with the protocol core
- Recommendation:
  - use this as a design reference when formalizing schema export, not as a reason to bloat the read/write API
- Timing:
  - post-0.8.0

## Paper 2. Automatic Configuration of OPC UA for IIoT Environments

- Repo component:
  - tag discovery roadmap
  - environment bootstrap tooling
- Best idea:
  - move toward deterministic discovery/export flows instead of ad hoc introspection
- Recommendation:
  - define a future discovery/export contract that can emit stable JSON for tags, UDTs, and routes where supported
- Timing:
  - design in 0.8.x, implementation later

## Paper 3. IIoT Gateway with OPC UA Based on Sitara AM335X

- Repo component:
  - collector service
  - edge gateway examples
- Best idea:
  - edge acquisition should be a service layer with explicit polling cycles and output sinks
- Recommendation:
  - build `rust-ethernet-ip` collector examples as thin wrappers around the Rust core, not as logic inside the library
- Timing:
  - near-term after Python MVP

## Paper 4. IIoT Protocol Comparative Study

- Repo component:
  - API design
  - service-template roadmap
- Best idea:
  - request/response belongs in the core access layer; pub/sub belongs in adapters
- Recommendation:
  - keep the library focused on direct PLC access and batch reads, then add MQTT or stream adapters as separate layers
- Timing:
  - immediate design guidance

## Paper 5. OPIIoT Open Communication Platform

- Repo component:
  - project framing
  - service adapter roadmap
- Best idea:
  - the repo can become a stable OT-to-software access layer without becoming a monolithic platform
- Recommendation:
  - keep wrappers thin and examples practical; avoid absorbing platform responsibilities into the core crate
- Timing:
  - immediate design guidance

## Paper 6. Digital Twin Survey

- Repo component:
  - long-term schema and object-model direction
- Best idea:
  - structured controller models are useful only if synchronization and ownership boundaries stay explicit
- Recommendation:
  - treat this as support for future typed metadata export, not for vague "digital twin" branding
- Timing:
  - later

## Paper 7. Digital Twin Systematic Review

- Repo component:
  - architecture docs
  - roadmap discipline
- Best idea:
  - keep the repo concrete about data pipelines, schemas, and synchronization instead of using broad digital-twin language
- Recommendation:
  - use in docs as guardrail material, not as a feature trigger by itself
- Timing:
  - immediate documentation guidance

## Paper 8. Machine Learning in Predictive Maintenance

- Repo component:
  - Python wrapper examples
  - collector and export patterns
- Best idea:
  - consistent time-series export matters more than fancy analytics abstractions in the core library
- Recommendation:
  - prioritize CSV, SQLite, and dataframe-friendly examples over in-library analytics features
- Timing:
  - 0.8.0 candidate

## Paper 9. AI + IoT Predictive Maintenance Practical Approach

- Repo component:
  - Python examples
  - future sample apps
- Best idea:
  - show how PLC reads can feed enterprise and analytics flows without embedding ERP or ML concerns into the library
- Recommendation:
  - add examples that combine PLC polling with downstream data handling patterns
- Timing:
  - 0.8.0 candidate for examples, deeper integrations later

## Paper 10. ML for ICS Intrusion Detection

- Repo component:
  - monitoring and diagnostics roadmap
- Best idea:
  - anomaly detection needs reliable operational telemetry before it needs ML
- Recommendation:
  - strengthen connection stats, operation errors, and health surfaces before considering anomaly features
- Timing:
  - later

## Paper 11. Asset Administration Shell Modeling

- Repo component:
  - metadata and schema export
  - architecture docs
- Best idea:
  - structure metadata around stable submodels such as identity, communication, configuration, and condition data
- Recommendation:
  - if schema export is added, organize it around explicit sections instead of raw unstructured dumps
- Timing:
  - near-term design work

## Paper 12. CPPS and Edge Computing Architecture

- Repo component:
  - collector and service architecture
- Best idea:
  - Rust should own deterministic PLC access; Python or service layers should own edge preprocessing and external integrations
- Recommendation:
  - keep the collector/API/MQTT work out of the Rust core crate
- Timing:
  - near-term after Python MVP

## Paper 13. Flat IIoT Architecture for Low-Latency Collection

- Repo component:
  - batch and polling API guidance
  - future performance benchmarks
- Best idea:
  - avoid unnecessary layering and serialization churn in high-frequency collection paths
- Recommendation:
  - add Python examples that use batch reads and interval-driven loops rather than many single-tag calls
- Timing:
  - 0.8.0 examples and later perf work

## Paper 14. OPC UA vs MQTT in a Unified Namespace Context

- Repo component:
  - MQTT roadmap
  - service adapter design
- Best idea:
  - structured access and distribution transport solve different problems and should stay separate
- Recommendation:
  - if MQTT support is added, implement it as an adapter or example service, not as a replacement for library semantics
- Timing:
  - near-term service roadmap

## Paper 15. Semantic Interconnection with OPC UA Pub/Sub

- Repo component:
  - future event-streaming adapter design
  - metadata-preserving service boundaries
- Best idea:
  - semantics should travel with published data instead of being lost at the first translation boundary
- Recommendation:
  - future stream or MQTT payloads should preserve tag identity, value type, timestamp, and status
- Timing:
  - later, after collector basics

## Recommended Concrete Work Sequence

1. Finish Python MVP and smoke-test it against the current FFI surface.
2. Add practical batch-oriented Python examples for logging and analytics preparation.
3. Formalize the metadata/schema-export backlog, informed by papers 2 and 11.
4. Design a collector-service MVP, informed by papers 3, 12, and 13.
5. Add MQTT or REST adapters only after the collector boundary is clean, informed by papers 4, 14, and 15.

## Guardrail

Use the papers to improve:

- wrapper ergonomics
- service boundaries
- schema direction
- deployment examples

Do not use them to pull the repo away from its primary identity:

- Rust EtherNet/IP core
- validated against real protocol behavior
- thin wrappers for user projects
