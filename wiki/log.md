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
