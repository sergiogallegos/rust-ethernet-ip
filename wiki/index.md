# Wiki Index

This file catalogs the current wiki pages.

Status values:

- `seed`
- `active`
- `historical`
- `needs-review`

## Core

- [README.md](README.md) — What the wiki is for and how it differs from the main docs. `seed`

## Releases

- [releases/0.7.0-validation-synthesis.md](releases/0.7.0-validation-synthesis.md) — Consolidated real-hardware and gate-level view of `0.7.0` validation. `active`
- [releases/0.8.0-validation-synthesis.md](releases/0.8.0-validation-synthesis.md) — Follow-up ControlLogix validation view for the current draft `0.8.0` line. `active`

## Controllers

- [controllers/firmware-behavior.md](controllers/firmware-behavior.md) — Current controller-family and firmware-specific behavior synthesis from validated targets. `active`

## Limitations

- [limitations/string-and-udt-write-behavior.md](limitations/string-and-udt-write-behavior.md) — Current synthesis of direct-write limitations and recommended workarounds. `active`

## Protocol

- [protocol/route-path-behavior.md](protocol/route-path-behavior.md) — Current route-path behavior, validation status, and implementation guidance. `active`
- [protocol/abi-contract.md](protocol/abi-contract.md) — FFI ABI version, capability bitmap, and wrapper load-time compatibility policy. `active`
- [protocol/cip-path-validation.md](protocol/cip-path-validation.md) — CIP request path-size validation rules and current empty-path policy. `confirmed`

## Wrapper Parity

- [wrapper-parity/rust-vs-csharp.md](wrapper-parity/rust-vs-csharp.md) — Current parity picture between the Rust core and the C# wrapper. `active`
- [wrapper-parity/ffi-registry-clone-audit.md](wrapper-parity/ffi-registry-clone-audit.md) — FFI registry clone semantics, copied-field risks, and CODEX-M Phase A recommendation. `needs-review`

## Investigations

- [investigations/llm-knowledge-base-pattern.md](investigations/llm-knowledge-base-pattern.md) — Repo-specific view of the file-backed LLM knowledge-base pattern and the maintenance risks it introduces. `active`
- [investigations/macbook-dashboard-demo-strategy.md](investigations/macbook-dashboard-demo-strategy.md) — Recommended MacBook-hosted demo architecture for manager-facing PLC dashboards using the currently validated stacks. `active`
- [investigations/ecosystem-platform-patterns-2026-04-19.md](investigations/ecosystem-platform-patterns-2026-04-19.md) — External ecosystem patterns from Rockwell, Ignition community, and Design Group that inform the Python/data-platform roadmap. `active`
- [investigations/python-wrapper-strategy-2026-04-19.md](investigations/python-wrapper-strategy-2026-04-19.md) — Current recommendation to build Python on the stable Rust FFI boundary, plus notes on stale historical wrapper docs. `active`
- [investigations/python-mvp-surface-2026-04-19.md](investigations/python-mvp-surface-2026-04-19.md) — Concrete Python MVP API, chosen FFI calls, and current native ergonomics gaps. `active`
- [investigations/research-papers-industrial-platform-roadmap-2026-04-19.md](investigations/research-papers-industrial-platform-roadmap-2026-04-19.md) — Curated industrial research papers and what they can realistically improve in this repo. `active`
- [investigations/research-feature-map-2026-04-19.md](investigations/research-feature-map-2026-04-19.md) — Paper-to-feature mapping for Python, metadata, collector, and adapter work, with timing guidance. `active`
- [investigations/metadata-schema-export-design-2026-04-19.md](investigations/metadata-schema-export-design-2026-04-19.md) — Proposed Rust-first schema export contract built on current discovery APIs. `active`
- [investigations/collector-service-mvp-design-2026-04-19.md](investigations/collector-service-mvp-design-2026-04-19.md) — Proposed Python-first collector service shape using batch polling and simple local sinks. `active`
- [investigations/rest-mqtt-adapter-boundaries-2026-04-19.md](investigations/rest-mqtt-adapter-boundaries-2026-04-19.md) — Recommended ownership boundaries for future REST and MQTT adapters above the Rust core and wrappers. `active`
- [investigations/monitoring-diagnostics-plan-2026-04-19.md](investigations/monitoring-diagnostics-plan-2026-04-19.md) — Current monitoring and diagnostics gaps plus the recommended Rust-first improvement order. `active`
- [investigations/docker-example-stacks-2026-04-19.md](investigations/docker-example-stacks-2026-04-19.md) — First local Docker packaging layer for the Python API, collector, and optional MQTT example services. `active`
- [investigations/rockwell-official-docs-2026-04-16.md](investigations/rockwell-official-docs-2026-04-16.md) — 2026-04-16 check of current official Rockwell EtherNet/IP and Logix data-access publications. `active`
- [investigations/rust-toolchain-baseline-2026-04-19.md](investigations/rust-toolchain-baseline-2026-04-19.md) — Current Rust baseline, Rust 2024 migration outcome, and MSRV posture after the 2026-04-19 refresh. `active`
- [investigations/software-architecture-map.md](investigations/software-architecture-map.md) — Current architecture ownership map and links to the maintainer-facing architecture document. `active`
- [investigations/architecture-review-2026-05-18.md](investigations/architecture-review-2026-05-18.md) — Post-books architecture synthesis and reconciled roadmap from Claude and Codex review passes. `active`
- [investigations/documentation-state-2026-04-20.md](investigations/documentation-state-2026-04-20.md) — Current documentation-health assessment, including active docs that are healthy and older docs that need clearer historical framing. `active`
- [investigations/test-coverage-strength-2026-05-18.md](investigations/test-coverage-strength-2026-05-18.md) — Current assessment of Rust, C#, and Python test strength, local command results, and prioritized coverage gaps. `active`

## Planned High-Value Pages

- `investigations/native-vs-wrapper-error-surfaces.md` — Mapping of Rust/native error detail to C# wrapper exceptions and messages. `seed`

## Notes

- Add pages here when they become durable reference points.
- Prefer updating an existing page over creating near-duplicates.
