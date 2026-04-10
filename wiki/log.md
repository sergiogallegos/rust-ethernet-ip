# Wiki Log

Append-only record of wiki maintenance activity.

## [2026-04-09] reframe | initialize repository wiki

- Added the repo-specific wiki schema in `AGENTS.md`.
- Created the initial `wiki/` scaffold with `README.md`, `index.md`, and `log.md`.
- Positioned the wiki as a maintainer knowledge layer, not a replacement for user-facing docs.

Sources used:

- `README.md`
- `CLAUDE.md`
- `docs/README.md`
- `CONTRIBUTING.md`

## [2026-04-09] ingest | seed initial synthesis pages

- Added a `0.7.0` validation synthesis page covering real-hardware conclusions and release-gate context.
- Added a limitations page for direct `STRING` and UDT-member write behavior.
- Added a route-path behavior page tying together implementation notes, routing guidance, and validation evidence.
- Updated `wiki/index.md` to register the new active pages.

Sources used:

- `docs/0.7.0_HARDENING_GATE.md`
- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/AB_String_UDT_Write_Limitations.md`
- `docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md`
- `docs/EtherNetIP_Connection_Paths_and_Routing.md`

## [2026-04-09] ingest | add parity and controller behavior synthesis

- Added a wrapper-parity page covering validated alignment between the Rust core and the C# wrapper.
- Added a controller/firmware behavior page capturing what is shared across the validated CompactLogix and ControlLogix targets and what differs.
- Updated `wiki/index.md` to register the new active pages and the next planned investigation page.

Sources used:

- `csharp/RustEtherNetIp/README.md`
- `csharp/RustEtherNetIp/IEtherNetIpClient.cs`
- `csharp/RustEtherNetIp.Tests/BatchConfigContractTests.cs`
- `csharp/RustEtherNetIp.Tests/TagGroupApiTests.cs`
- `csharp/RustEtherNetIp.Tests/TagGroupEventDiagnosticsTests.cs`
- `docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/audit/0.7.0_docs_api_audit.md`
- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`

## [2026-04-09] ingest | add llm knowledge-base pattern investigation

- Added an investigation page describing how the broader file-backed "wiki LLM" pattern maps onto this repository's wiki.
- Recorded the main benefits of the pattern for this repo and the failure modes that require schema, source hierarchy, and linting discipline.
- Updated `wiki/index.md` to register the new investigation page.

Sources used:

- `AGENTS.md`
- `README.md`
- `wiki/README.md`
- User-provided excerpts on the file-backed "wiki LLM" and "LLM knowledge bases" workflow

## [2026-04-10] query | recommend macbook dashboard demo strategy

- Added a MacBook dashboard demo strategy page tying the recommendation to the validated Rust, C# wrapper, and example-app paths.
- Updated `wiki/index.md` to register the new investigation page.
- Captured the main recommendation that a web app is the best current demo shape for manager-facing access, with backend choice depending on whether the goal is pure Rust proof or .NET integration proof.

Sources used:

- `wiki/releases/0.7.0-validation-synthesis.md`
- `wiki/wrapper-parity/rust-vs-csharp.md`
- `wiki/limitations/string-and-udt-write-behavior.md`
- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `examples/web_app/README.md`
- `examples/AspNetExample/README.md`
- `docs/PLC_TEST_TAG_DEFINITIONS.md`

## [2026-04-10] ingest | implement rust macbook dashboard demo

- Updated `examples/web_app/` from a simple form demo into a manager-facing dashboard with route-path connect, controller identity display, live batch snapshot cards, a speed benchmark panel, and local traceability-event persistence.
- Expanded the same dashboard to include production-style trend charts and stronger use of the seeded `Program:TestProgram.*` tags for manufacturing-focused views.
- Updated the MacBook dashboard strategy page to reflect the implemented Rust web path and the current file-backed persistence choice.
- Updated `examples/web_app/README.md` so the example documentation matches the implemented API surface and demo story.

Sources used:

- `examples/web_app/backend/src/main.rs`
- `examples/web_app/frontend/src/App.tsx`
- `examples/web_app/frontend/src/types.ts`
- `examples/web_app/README.md`
- `wiki/investigations/macbook-dashboard-demo-strategy.md`
