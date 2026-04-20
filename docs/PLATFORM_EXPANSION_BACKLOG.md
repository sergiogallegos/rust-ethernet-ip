# Platform Expansion Backlog

This document captures the next major expansion direction for `rust-ethernet-ip` beyond the current Rust core and C# wrapper.

The guiding idea is:

- keep Rust as the protocol and performance core
- use thin language wrappers to increase adoption
- position the project as an industrial data-access layer for modern software systems

## Core Vision Guardrail

This repo must remain:

- the core EtherNet/IP library written in Rust
- strong, safe, fast, documented, tested, and current
- the protocol and semantics source of truth

Wrappers and ecosystem pieces are important, but they are support layers for users building their own projects. They must not dilute the core objective of the repository.

## Summary

The strongest opportunity is not to compete with Rockwell, Ignition, or system integrators on full platforms.

The repo can instead own the missing layer:

- direct PLC data access from modern software stacks
- clean semantics for reads, writes, metadata, batching, routing, and health
- reusable bridges into .NET, Python, data pipelines, and service backends

## Session Checkpoint

Last updated: 2026-04-19

Last completed work:

- exposed the Rust diagnostics snapshot through FFI and thin C# and Python wrapper types
- verified Rust checks/tests, Python tests, C# build, and C# tests

Best next step when a real PLC is available:

1. run real-PLC validation for:
   - schema export
   - Python wrapper
   - collector service
   - diagnostics snapshots and wrapper health surfaces

If a real PLC is still not available:

1. work on Docker-based example stacks

## External Patterns Reviewed

These public sources were reviewed on 2026-04-19:

- Rockwell Automation GitHub: <https://github.com/RockwellAutomation>
- `ra-logix-cicd`: <https://github.com/RockwellAutomation/ra-logix-cicd>
- Ignition Module Development Community:
  - `ignition-extensions`: <https://github.com/IgnitionModuleDevelopmentCommunity/ignition-extensions>
  - `IgnitionNode-RED`: <https://github.com/IgnitionModuleDevelopmentCommunity/IgnitionNode-RED>
- Barry-Wehmiller Design Group GitHub: <https://github.com/design-group>
- `ignition-docker`: <https://github.com/design-group/ignition-docker>
- `ignition-tag-cicd-module`: <https://github.com/design-group/ignition-tag-cicd-module>

## What Those Repos Suggest

## Rockwell Automation

Observed pattern:

- public repos focus on Logix CI/CD, VCS-friendly transforms, and vendor tooling
- they do not publish a modern open data-access SDK for enterprise software stacks

Implication for this repo:

- there is room for a developer-friendly access layer that makes live PLC data easier to integrate into software engineering workflows
- schema export, deterministic metadata access, and automation-friendly surfaces are valuable

## Ignition Module Development Community

Observed pattern:

- strong modular gateway/plugin architecture
- scripting and integration surfaces matter as much as protocol access
- gateway-facing APIs are used to connect to Node-RED and other systems

Implication for this repo:

- the Rust core should be treated as a reusable driver layer
- Python is a strong fit as the scripting and data-science access layer
- future adapters and services should be built as thin extensions around the Rust core, not as parallel implementations

## Design Group / BW Design Group

Observed pattern:

- practical industrial stack templates matter: Docker, Ignition, databases, CI/CD, utility scripts
- the value is often in deployable workflows, not only reusable libraries
- Python appears in real integration and gateway-utility work

Implication for this repo:

- examples and service templates are strategic, not optional
- the project should grow beyond a library into a small ecosystem of data-collection and integration components

## Recommendations

## Positioning

Evolve the project narrative from:

- `Rust EtherNet/IP library`

to:

- `Industrial Data Access Layer for Modern Software Systems`

This better matches:

- ERP and MES connectivity
- OEE and historian-style data collection
- analytics and AI pipelines
- REST, MQTT, and service-based integrations

## Technical Direction

Prefer this long-term shape:

1. Rust core
2. stable external FFI boundary
3. thin wrappers for C# and Python
4. higher-level services and templates built on the wrappers

This preserves one source of truth for protocol behavior while allowing broader adoption.

## Backlog

## Immediate

- [x] Finish current C# wrapper structural cleanup without changing public behavior
- [x] Add a first-class Python wrapper architecture review and MVP implementation plan
- [x] Decide whether the Python wrapper should sit on the existing C ABI or a newly formalized FFI surface
- [x] Document wrapper layering and compatibility expectations for Rust, C#, and Python

## Near-Term

- [x] Execute Python MVP validation against simulator-backed flows:
  - connect/disconnect
  - read one tag
  - write one tag
  - batch read
  - health check
- [x] Add Python packaging skeleton and local dev instructions
- [x] Add Python examples for:
  - CSV logging
  - SQLite logging
  - batch reads
- [x] Add Python integration-test skeleton for simulator-backed flows
- [x] Extend Python examples for:
  - pandas dataset generation
  - simple API service
- [x] Verify C# wrapper parity is maintained while Python support is introduced
- [ ] Execute Python MVP validation against a real PLC path
- [ ] Implement Python MVP production-hardening:
  - connect/disconnect
  - read one tag
  - write one tag
  - batch read
  - health check
- [x] Investigate and fix Python simulator-path issues found during MVP validation:
  - batch `STRING` read now handles Allen-Bradley `0x00CE`
  - Python `float` now defaults to PLC `REAL`, with explicit `LREAL` override when needed

## Paper-Driven Backlog

- [x] Define a metadata and schema-export design informed by research papers 2 and 11
- [x] Implement Rust-side schema export structs and `export_schema()` based on the approved design
- [ ] Add real-PLC validation coverage for schema export
- [x] Add batch-first Python examples for analytics and low-latency collection informed by papers 8, 9, and 13
- [x] Design a collector-service MVP informed by papers 3, 12, and 13
- [x] Implement the collector-service MVP example
- [x] Design REST and MQTT adapter boundaries informed by papers 4, 14, and 15
- [x] Define a monitoring and diagnostics improvement plan informed by paper 10
- [x] Implement a stable Rust-side diagnostics snapshot and stronger error classification
- [x] Expose the Rust diagnostics snapshot through FFI and thin wrapper types
- [ ] Add real-PLC validation coverage for diagnostics snapshots and wrapper health surfaces

## Data Platform Components

- [x] Add a configurable data-collector service
- [x] Add an MQTT publisher example or service
- [x] Add a small REST API service example
- [x] Add Docker-based example stacks for local development

## Higher-Level Industrial Features

- [ ] Explore schema export / metadata export APIs
- [ ] Explore deterministic PLC snapshot/export formats
- [ ] Explore historian-ready collection patterns
- [ ] Explore OEE / MES starter patterns
- [ ] Explore AI and analytics-ready dataset generation flows

## Design Constraints

- keep Rust as the semantic source of truth
- do not duplicate protocol logic in wrappers
- do not break existing C# wrapper behavior
- prefer thin adapters and service templates over a second implementation stack
- keep Python dependencies optional where possible outside the core package

## Recommended Execution Order

1. stabilize wrapper boundaries and current architecture docs
2. design Python wrapper strategy against the real repo state
3. implement a narrow Python MVP
4. add data examples and service templates
5. iterate toward platform components like collectors, MQTT, and API layers

## Notes

- The public material reviewed suggests this project complements existing industrial platforms rather than competing with them directly.
- The strongest product opportunity is to become the open, modern bridge between Rockwell PLC data and enterprise/data/AI software stacks.
- The paper-to-feature map is tracked in `docs/RESEARCH_FEATURE_MAP.md`.
