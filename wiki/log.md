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

## [2026-04-16] ingest | check current Rockwell official docs

- Added a Rockwell official documentation check investigation page.
- Updated `wiki/index.md` to register the new page.
- Recorded that `1756-PM020I-EN-P` remains the primary Logix data-access implementation source and that `ENET-UM006C-EN-P` should be tracked as a relevant EtherNet/IP connection/messaging reference.
- No immediate protocol implementation change was identified.

Sources used:

- `docs/OFFICIAL_SOURCES.md`
- `docs/release/0.7.1_RELEASE_NOTES_DRAFT.md`
- Rockwell `1756-PM020I-EN-P`, Logix 5000 Controllers Data Access, September 2025
- Rockwell `ENET-UM006C-EN-P`, EtherNet/IP Network Devices User Manual, September 2025
- Rockwell `1756-RM094N-EN-P`, Logix 5000 Controllers Design Considerations Reference Manual, September 2025

## [2026-04-16] ingest | record 0.7.1 ControlLogix validation

- Added a `0.7.1` validation synthesis page.
- Updated `wiki/index.md` to register the new page.
- Recorded that Rust and C# wrapper real-PLC validation passed on the exercised `1756-L81ES` routed ControlLogix feature set, with remaining full-matrix failures matching known firmware limitations.
- Recorded the C# validation-example native-library copy fix found during the run.

Sources used:

- `docs/validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/release/0.7.1_RELEASE_NOTES_DRAFT.md`
- `CHANGELOG.md`

## [2026-04-19] query | assess rust toolchain baseline

- Added a Rust toolchain baseline investigation page.
- Updated `wiki/index.md` to register the new page.
- Recorded that local development is already on Rust `1.95.0`, while the crate at that point still declared `rust-version = "1.70"` and `edition = "2021"`.
- Recorded that `cargo check --all-targets` passes on Rust `1.95.0`, but a Rust 2024 migration is not yet a low-risk mechanical bump because `cargo fix --edition` surfaced async temporary drop-order warnings.

Sources used:

- `Cargo.toml`
- `examples/desktop_app/Cargo.toml`
- `examples/web_app/backend/Cargo.toml`
- `README.md`
- `examples/desktop_app/README.md`
- `docs/VERSION_MANAGEMENT.md`
- Rust 1.95.0 release post, `https://blog.rust-lang.org/2026/04/16/Rust-1.95.0/`

## [2026-04-19] reframe | rust 2024 prep pass

- Updated the Rust toolchain baseline investigation after a source refactor pass aimed at Rust 2024 compatibility without changing the edition.
- Recorded that explicit local bindings and guard drops removed the repo-owned async/drop-order warnings in core code, examples, and tests.
- Recorded that the remaining repo-owned Rust 2024 warnings are concentrated in `src/ffi.rs`, where exported `#[no_mangle]` entry points would want Rust 2024 unsafe attributes that conflicted with the then-current `rust-version = "1.70"` baseline.

## [2026-04-19] reframe | finalize rust 2024 baseline

- Raised the crate compiler baseline to Rust `1.95` and updated workspace manifests to `edition = "2024"`.
- Converted exported FFI entry points in `src/ffi.rs` from `#[no_mangle]` to `#[unsafe(no_mangle)]`.
- Updated current MSRV-facing docs and the Rust toolchain baseline investigation to reflect the new baseline.

Sources used:

- `Cargo.toml`
- `examples/desktop_app/Cargo.toml`
- `examples/web_app/backend/Cargo.toml`
- `src/ffi.rs`
- `README.md`
- `examples/desktop_app/README.md`
- Rust Edition Guide, unsafe attributes, `https://doc.rust-lang.org/edition-guide/rust-2024/unsafe-attributes.html`

Sources used:

- `src/plc_manager.rs`
- `src/subscription.rs`
- `src/tag_subscription.rs`
- `src/tag_group.rs`
- `src/lib.rs`
- `src/main.rs`
- `examples/desktop_app/src/main.rs`
- `tests/health_check_tests.rs`
- `tests/subscription_tests.rs`
- `tests/array_read_write_tests.rs`
- `tests/batch_operations_tests.rs`
- `tests/udt_data_tests.rs`
- `src/ffi.rs`

## [2026-04-19] lint | align rust baseline wiki wording

- Updated `wiki/index.md` to describe the Rust toolchain investigation as the finalized Rust 2024 / Rust 1.95 baseline, not a pre-migration assessment.
- Recorded that the current wiki summary should match the already-applied manifest and FFI export changes.

Sources used:

- `wiki/index.md`
- `wiki/investigations/rust-toolchain-baseline-2026-04-19.md`
- `Cargo.toml`

## [2026-04-19] reframe | move active draft line to 0.8.0

- Reframed the active unreleased line from `0.7.1` to `0.8.0` to match the larger migration and planned feature scope.
- Renamed the active release-notes draft and release-validation synthesis pages from `0.7.1` to `0.8.0`.
- Updated README, docs index, official-sources notes, validation notes, and wiki references to point at the new `0.8.0` draft line.

Sources used:

- `README.md`
- `CHANGELOG.md`
- `docs/README.md`
- `docs/OFFICIAL_SOURCES.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `docs/validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `wiki/index.md`
- `wiki/README.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `wiki/investigations/rockwell-official-docs-2026-04-16.md`
- `wiki/investigations/rust-toolchain-baseline-2026-04-19.md`

## [2026-04-19] ingest | add software architecture map

- Added a maintainer-facing architecture document in `docs/SOFTWARE_ARCHITECTURE.md` describing the layer model, ownership boundaries, design patterns, and current refactor seams.
- Added a wiki synthesis page so the architecture map is discoverable from the wiki entry point.
- Updated `wiki/index.md` and `docs/README.md` to register the new architecture references.

Sources used:

- `src/lib.rs`
- `src/ffi.rs`
- `src/subscription.rs`
- `src/tag_subscription.rs`
- `src/plc_manager.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs`
- `docs/README.md`

## [2026-04-19] ingest | capture platform expansion backlog

- Added a backlog document for Python, data-platform, and service-layer expansion informed by public Rockwell, Ignition-community, and Design Group patterns.
- Added a detailed Codex prompt file for planning and implementing the Python/data-platform path against the repo's current `0.8.0` line and current toolchain baselines.
- Added a wiki investigation page summarizing the main external ecosystem takeaways and how they inform this repo's strategic direction.

Sources used:

- `https://github.com/RockwellAutomation`
- `https://github.com/RockwellAutomation/ra-logix-cicd`
- `https://github.com/IgnitionModuleDevelopmentCommunity/ignition-extensions`
- `https://github.com/IgnitionModuleDevelopmentCommunity/IgnitionNode-RED`
- `https://github.com/design-group`
- `https://github.com/design-group/ignition-docker`
- `https://github.com/design-group/ignition-tag-cicd-module`
- `docs/README.md`

## [2026-04-19] ingest | define python wrapper strategy

- Added a Python wrapper strategy document recommending the existing Rust FFI boundary as the primary foundation instead of starting with a separate PyO3-first architecture.
- Recorded that historical docs still reference earlier `pywrapper/`-style work that is not present in the current live repo tree.
- Updated the docs index and wiki index so the Python wrapper recommendation is discoverable.

Sources used:

- `Cargo.toml`
- `src/ffi.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs`
- `docs/ALL_WRAPPERS_UPDATE_COMPLETE.md`
- `docs/WRAPPER_UPDATE_SUMMARY.md`

## [2026-04-19] ingest | define python mvp surface

- Added a concrete Python MVP API and FFI mapping document.
- Recorded the recommended minimal native surface for Python and the main FFI gaps that affect wrapper ergonomics.
- Updated docs and wiki indexes so the Python MVP design is easy to find from the current repo documentation layer.

Sources used:

- `src/ffi.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.NativeMethods.cs`
- `docs/PYTHON_WRAPPER_STRATEGY.md`

## [2026-04-19] ingest | curate industrial research papers

- Added a curated reading list of ten industrial, IIoT, digital-twin, and ML/security papers relevant to the repo's wrapper and platform roadmap.
- Recorded that some initially proposed `arXiv` links did not match the named industrial papers and replaced them with verified references.
- Added a wiki synthesis page describing which papers are most actionable for this repo and which are broader context only.

Sources used:

- `https://www.mdpi.com/2624-831X/3/4/27`
- `https://www.mdpi.com/2079-9292/8/6/600`
- `https://www.mdpi.com/1424-8220/24/7/2072`
- `https://www.mdpi.com/1999-5903/11/3/66`
- `https://www.sciencedirect.com/science/article/pii/S2542660521000846`
- `https://colab.ws/articles/10.1109/access.2020.2998358`
- `https://www.mdpi.com/2071-1050/12/19/8211`
- `https://www.mdpi.com/2078-2489/16/9/737`
- `https://link.springer.com/article/10.1186/s42400-021-00095-5`

## [2026-04-19] ingest | extend industrial research reading list

- Extended the research reading list with five more targeted papers on Asset Administration Shell modeling, edge/CPPS deployment, low-latency data-collection architecture, unified-namespace transport tradeoffs, and semantic OPC UA pub/sub patterns.
- Updated the research synthesis page to highlight which additions are most actionable for schema export, collector-service architecture, and future MQTT or streaming adapters.

Sources used:

- `https://www.mdpi.com/1424-8220/21/6/2004`
- `https://www.sciencedirect.com/science/article/pii/S1877050919309317`
- `https://www.sciencedirect.com/science/article/pii/S1383762122001564`
- `https://www.sciencedirect.com/science/article/pii/S2542660525002483`
- `https://pmc.ncbi.nlm.nih.gov/articles/PMC9606965/`

## [2026-04-19] ingest | map research papers to repo features

- Added a concrete paper-to-feature map in `docs/RESEARCH_FEATURE_MAP.md`.
- Added a matching wiki synthesis page so agents can find the paper-driven priorities quickly.
- Updated the platform backlog to carry the paper-driven items as explicit to-dos.

Sources used:

- `docs/research/CURATED_INDUSTRIAL_RESEARCH_READING_LIST.md`
- `docs/PLATFORM_EXPANSION_BACKLOG.md`
- `docs/PYTHON_WRAPPER_STRATEGY.md`
- `docs/PYTHON_MVP_API_AND_FFI_MAPPING.md`

## [2026-04-19] ingest | advance python wrapper mvp skeleton

- Added the initial `python/` package skeleton with bindings, client API, examples, and lightweight tests.
- Updated the Python strategy and MVP wiki pages to reflect that the package now loads the native library and validates with `unittest`.
- Added local Python development and test instructions in `python/README.md`.

Sources used:

- `python/pyproject.toml`
- `python/rust_ethernet_ip/bindings.py`
- `python/rust_ethernet_ip/client.py`
- `python/tests/test_client_value_mapping.py`
- `python/tests/test_import.py`
- `python/README.md`

## [2026-04-19] ingest | add python simulator integration tests

- Added optional Python integration tests for simulator-backed connect/read/write/batch/health flows.
- Matched the existing C# test pattern by keying the integration path off `SIM_PLC_ADDRESS` and skipping cleanly when it is not configured.
- Updated Python strategy docs and wiki pages to record that the test skeleton exists and what still remains is actual configured execution.

Sources used:

- `python/tests/test_integration.py`
- `python/README.md`
- `csharp/RustEtherNetIp.Tests/SimulatorIntegrationTests.cs`

## [2026-04-19] ingest | add auto-launch python simulator path

- Added `examples/python_test_simulator.rs` so the in-repo deterministic simulator can be launched directly for Python validation.
- Added a Python-side simulator harness that can auto-start the simulator when `RUST_ETHERNET_IP_START_SIM=1` is set.
- Initial end-to-end validation surfaced a batch STRING parsing gap and a Python float-inference mismatch.

Sources used:

- `examples/python_test_simulator.rs`
- `python/tests/sim_harness.py`
- `python/tests/test_integration.py`
- `python/README.md`

## [2026-04-19] ingest | fix python simulator batch validation issues

- Fixed batch-result parsing for Allen-Bradley `STRING` values (`0x00CE`) in the Rust core.
- Changed Python float inference to default to PLC `REAL` instead of `LREAL`, with explicit override still available.
- Updated Python loader precedence to prefer freshly built debug libraries during local development.
- Revalidated the full Python suite end-to-end with `RUST_ETHERNET_IP_START_SIM=1`.

Sources used:

- `src/lib.rs`
- `python/rust_ethernet_ip/client.py`
- `python/rust_ethernet_ip/bindings.py`
- `python/tests/test_client_value_mapping.py`
- `python/tests/test_integration.py`

## [2026-04-19] ingest | define metadata schema export design

- Added a Rust-first metadata and schema export design document based on the repo's current discovery and UDT APIs.
- Added a wiki synthesis page linking that design back to the research-driven roadmap.
- Updated the backlog to mark the design task complete and make implementation of `export_schema()` the next concrete follow-up.

Sources used:

- `docs/tag_introspection.md`
- `docs/RESEARCH_FEATURE_MAP.md`
- `src/lib.rs`
- `src/ffi.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.Support.cs`

## [2026-04-19] ingest | implement rust schema export

- Added explicit Rust-side schema export structs in `src/schema.rs`.
- Implemented `EipClient::export_schema()` and `EipClient::export_schema_json()` on top of the current discovery and UDT APIs.
- Added unit coverage for schema type classification and timestamp formatting, and updated the backlog to move schema export from design into implemented status.

Sources used:

- `src/schema.rs`
- `src/lib.rs`
- `src/tag_manager.rs`
- `src/udt.rs`
- `docs/METADATA_SCHEMA_EXPORT_DESIGN.md`

## [2026-04-19] ingest | expand schema export tests

- Added focused schema-export unit coverage for tag mapping, UDT mapping, and top-level JSON serialization shape.
- Updated backlog/docs to distinguish completed assembly/serialization tests from still-pending real-PLC validation.

Sources used:

- `src/schema.rs`
- `docs/METADATA_SCHEMA_EXPORT_DESIGN.md`
- `docs/PLATFORM_EXPANSION_BACKLOG.md`

## [2026-04-19] ingest | add python analytics and api examples

- Added optional Python examples for pandas/dataframe export and a simple FastAPI service.
- Added isolated Python extras for analytics and API examples so the core package remains dependency-light.
- Updated backlog and strategy docs to mark the batch-first analytics example step complete.

Sources used:

- `python/pyproject.toml`
- `python/README.md`
- `python/examples/pandas_dataframe_example.py`
- `python/examples/fastapi_service_example.py`

## [2026-04-19] ingest | define collector service mvp

- Added a collector-service MVP design document that recommends a Python example service using batch polling and CSV/SQLite sinks.
- Added a wiki synthesis page linking the collector direction back to the paper-driven roadmap and current repo service patterns.
- Updated backlog/docs to mark collector design complete and implementation as the next concrete follow-up.

Sources used:

- `docs/RESEARCH_FEATURE_MAP.md`
- `python/examples/log_tags_to_csv.py`
- `python/examples/log_tags_to_sqlite.py`
- `examples/web_app/backend/src/main.rs`

## [2026-04-19] ingest | implement collector service example

- Added a batch-first Python collector example driven by a JSON config file.
- Added a starter collector config and updated the Python README with collector usage.
- Marked the collector-service MVP implementation task complete while leaving real-PLC validation as follow-up work.

Sources used:

- `python/examples/collector_service.py`
- `python/examples/collector_config.example.json`
- `python/README.md`

## [2026-04-19] ingest | define rest and mqtt adapter boundaries

- Added a design note that keeps REST and MQTT above the Rust core and thin wrappers instead of turning them into parallel protocol implementations.
- Marked the adapter-boundary design step complete and corrected the backlog to reflect that the repo already has a small REST example and a configurable collector example.
- Added a wiki synthesis page linking the new boundary guidance to the collector, schema-export, and paper-driven roadmap work.

Sources used:

- `docs/REST_MQTT_ADAPTER_BOUNDARIES.md`
- `docs/PLATFORM_EXPANSION_BACKLOG.md`
- `python/examples/fastapi_service_example.py`
- `python/examples/collector_service.py`
- `examples/web_app/backend/src/main.rs`

## [2026-04-19] ingest | add mqtt publisher example

- Added a Python MQTT example that publishes normalized batch snapshots above the wrapper layer using an optional dependency.
- Added a starter MQTT config and updated the Python README to document install and usage for the example.
- Marked the MQTT example step complete while leaving broker-backed validation and Docker packaging as follow-up work.

Sources used:

- `python/examples/mqtt_publisher_example.py`
- `python/examples/mqtt_publisher_config.example.json`
- `python/README.md`
- `docs/REST_MQTT_ADAPTER_BOUNDARIES.md`

## [2026-04-19] ingest | define monitoring and diagnostics plan

- Added a Rust-first monitoring and diagnostics improvement plan focused on stable health and error telemetry before anomaly-oriented work.
- Added a wiki synthesis page linking the plan back to the current monitoring module, paper-driven roadmap, and real-PLC validation records.
- Applied a small code cleanup in `src/monitoring.rs` by removing an unnecessary async metrics-update path while holding the write lock.

Sources used:

- `src/monitoring.rs`
- `docs/MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md`
- `docs/RESEARCH_FEATURE_MAP.md`
- `docs/validation/REAL_PLC_TESTING.md`
- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`

## [2026-04-19] ingest | implement rust diagnostics snapshot

- Added a first Rust-side diagnostics snapshot, explicit passive-versus-verified health mode, and first-pass operational error categories in the monitoring module.
- Re-exported the new diagnostics types from the public Rust API and added focused unit tests for error classification and health-state transitions.
- Updated the backlog and monitoring plan to treat wrapper/FFI exposure as the next follow-up step.

Sources used:

- `src/monitoring.rs`
- `src/lib.rs`
- `docs/MONITORING_DIAGNOSTICS_IMPROVEMENT_PLAN.md`
- `docs/PLATFORM_EXPANSION_BACKLOG.md`

## [2026-04-19] ingest | expose diagnostics through ffi and wrappers

- Added a JSON-based FFI diagnostics export built on the Rust-side diagnostics snapshot and updated the basic/detailed health FFI calls to use real client health checks.
- Added thin C# DTOs and wrapper methods plus thin Python dataclasses and client accessors for diagnostics snapshots.
- Updated the backlog and monitoring plan to mark wrapper exposure complete and leave real-PLC diagnostics validation as the next follow-up step.

Sources used:

- `src/lib.rs`
- `src/ffi.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.Diagnostics.cs`
- `csharp/RustEtherNetIp/DiagnosticsSnapshot.cs`
- `python/rust_ethernet_ip/client.py`
- `python/rust_ethernet_ip/types.py`

## [2026-04-19] reframe | add session checkpoint for tomorrow

- Added an explicit session checkpoint to the backlog so the next session can resume from the completed diagnostics-wrapper exposure work.
- Recorded the preferred next step as the real-PLC validation block, with Docker example stacks as the fallback if hardware is still unavailable.

Sources used:

- `docs/PLATFORM_EXPANSION_BACKLOG.md`

## [2026-04-19] ingest | add docker example stacks

- Added a first local Docker packaging layer for the Python API, collector, and optional MQTT example services.
- Updated the Python examples to support environment-driven PLC and broker overrides so they can run cleanly in containers.
- Marked the Docker-stack backlog item complete and added a small C# diagnostics contract test alongside the stack work.

Sources used:

- `docker/python-stack/Dockerfile`
- `docker/python-stack/docker-compose.yml`
- `docs/DOCKER_EXAMPLE_STACKS.md`
- `python/examples/fastapi_service_example.py`
- `python/examples/collector_service.py`
- `python/examples/mqtt_publisher_example.py`
- `csharp/RustEtherNetIp.Tests/DiagnosticsSnapshotContractTests.cs`

## [2026-04-20] query | validate pending 0.8.0 ControlLogix follow-up

- Updated the real-PLC checklist and release-validation synthesis with the 2026-04-20 routed ControlLogix follow-up results.
- Confirmed routed Rust and C# health/diagnostics paths are healthy on `1756-L81ES` via `1756-EN3TR` slot `0`.
- Recorded that routed Rust `export_schema()` now succeeds after fixing the malformed request shape and adding paged Symbol Object discovery.
- Recorded that routed schema export still produces warnings and `udts=0` because UDT definition resolution for discovered structured tags is not yet succeeding on this target.
- Recorded that the Python wrapper needed a rebuilt cdylib to load diagnostics, then was extended with route-path support so routed reads, diagnostics, and collector validation now pass on valid `gTest*` tags.

Sources used:

- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `examples/readonly_plc_probe.rs`
- `tests/health_check_tests.rs`
- `src/lib.rs`
- `src/ffi.rs`
- `python/rust_ethernet_ip/client.py`
- `python/examples/collector_service.py`
- `csharp/RustEtherNetIp/EthernetNetIpClient.Diagnostics.cs`

## [2026-04-20] ingest | clear routed schema export blocker on ControlLogix

- Updated the routed ControlLogix validation record after fixing Template Object attribute parsing, Template Read request framing, and paged Template Object reads in the Rust core.
- Recorded that live routed schema export on `1756-L81ES` via `1756-EN3TR` slot `0` now returns `43` tags and `9` UDT definitions, with the remaining warning reduced to target-address omission in export metadata.
- Recorded the durable protocol finding that Template Read on this target requires `offset:u32 + byte_count:u16` request data; size-only requests fail with `0x13 Not enough data`.
- Updated the `0.8.0` draft release synthesis so schema export is no longer listed as the primary remaining real-PLC blocker.

Sources used:

- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `src/lib.rs`
- `src/udt.rs`

## [2026-04-20] ingest | validate mqtt publisher on routed controllogix

- Validated the Python MQTT publisher end to end against `1756-L81ES` via `1756-EN3TR` slot `0` using a temporary virtualenv with `paho-mqtt 2.1.0` and a local Mosquitto broker.
- Confirmed the published topic `factory/lab/plc/controllogix-l81es/snapshot` and observed a live payload containing `timestamp_utc`, `plc_name`, routed `gTest*` values, and empty `errors`.
- Updated the 2026-04-20 validation checklist, `0.8.0` draft release notes, and release synthesis so MQTT is no longer listed as an unvalidated blocker.

Sources used:

- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `python/examples/mqtt_publisher_example.py`
- `python/examples/mqtt_publisher_config.example.json`
- `docker/python-stack/docker-compose.yml`
- `docker/python-stack/mosquitto.conf`

## [2026-04-20] ingest | rerun live controllogix regression and make docker optional

- Re-ran the routed `1756-L81ES` matrix against the live PLC for Rust, C#, and Python with Docker treated as optional rather than release-blocking.
- Recorded fresh Rust passes for `schema::`, `discovery_tests`, ignored health checks, and `route_path_operations_tests` on `192.168.0.101:44818` slot `0`.
- Recorded fresh C# wrapper smoke, benchmark, and full validation-app results; the full matrix non-passes remained limited to expected STRING/UDT-array-member write constraints plus three tags whose live values no longer match the historical fixture baseline.
- Recorded fresh Python routed health, diagnostics, single-read, batch-read, and collector results, updated the local collector example config to validated `gTest*` tags, and captured the remaining Python `STRING` decode gap.
- Updated the release notes and validation synthesis so Docker example-stack smoke testing is explicitly optional for `0.8.0`.

Sources used:

- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `wiki/log.md`
- `python/examples/collector_config.example.json`
- `tests/health_check_tests.rs`
- `tests/route_path_operations_tests.rs`
- `examples/CSharpWrapperSmoke/Program.cs`
- `examples/CSharpWrapperBenchmark/Program.cs`
- `examples/CSharpWrapperTest/Program.cs`
- `python/rust_ethernet_ip/client.py`

## [2026-04-20] ingest | clear python controllogix string decode gap

- Updated the Python wrapper decoder so live ControlLogix `STRING` tags returned through the raw `{symbol_id,data}` path are normalized back to plain text when the payload matches the known Logix `STRING` layout.
- Added targeted Python tests for the raw-header Logix `STRING` shape and confirmed generic UDT payloads are still preserved.
- Re-ran the Python unit suite, Python bytecode compile check, live routed `gTest_STRING` single/batch reads, and the one-shot collector against `192.168.0.101:44818` slot `0`.
- Updated the validation checklist, `0.8.0` draft release notes, and release synthesis to remove the stale Python `STRING` decode blocker.

Sources used:

- `python/rust_ethernet_ip/client.py`
- `python/tests/test_client_value_mapping.py`
- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`

## [2026-04-20] ingest | confirm csharp full-matrix mismatches were concurrency artifacts

- Re-ran `examples/CSharpWrapperTest` by itself against `1756-L81ES` via `1756-EN3TR` slot `0` and confirmed the full validation app returns `333/392` with only the `59` documented firmware-limited write failures.
- Recorded that the earlier `gTestArray_DINT[5..7]` mismatches were caused by running the smoke, benchmark, and full validation app in parallel against the same live write targets.
- Updated the validation checklist, release notes draft, and release synthesis so the C# full-matrix result no longer claims live fixture drift on those tags.

Sources used:

- `examples/CSharpWrapperTest/Program.cs`
- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`

## [2026-04-20] ingest | capture final serial controllogix release-gate pass

- Ran a final serial release-gate pass against `1756-L81ES` via `1756-EN3TR` slot `0` to avoid shared-tag interference between live write-heavy validations.
- Reconfirmed Rust `schema::`, `discovery_tests`, ignored health checks, and route-path tests on the live target.
- Reconfirmed C# smoke, benchmark, and full validation app results in serial order; the full matrix remained at `333/392` with only the `59` documented firmware-limited write failures.
- Reconfirmed Python unit tests, bytecode compile checks, routed live reads including decoded `gTest_STRING="HELLO"`, and the one-shot collector writing `4` rows to SQLite.
- Updated the release notes, validation checklist, and release synthesis to treat this serial pass as the consolidated `0.8.0` live release gate for the exercised surfaces.

Sources used:

- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `docs/release/0.8.0_RELEASE_NOTES_DRAFT.md`
- `wiki/releases/0.8.0-validation-synthesis.md`
- `wiki/log.md`
- `examples/CSharpWrapperSmoke/Program.cs`
- `examples/CSharpWrapperBenchmark/Program.cs`
- `examples/CSharpWrapperTest/Program.cs`
- `python/rust_ethernet_ip/client.py`
- `python/examples/collector_service.py`
- `tests/health_check_tests.rs`
- `tests/route_path_operations_tests.rs`

## [2026-04-20] lint | assess documentation state

- Added a documentation-state investigation page capturing which current docs look healthy and which older docs are likely to confuse readers without historical framing.
- Updated `wiki/index.md` to register the new investigation page.
- Recorded that the main risk is not the active README/manual/wrapper/wiki surfaces, but older secondary docs that still describe pre-release `0.7.0` or removed `pywrapper/` / `gowrapper/` trees.

Sources used:

- `README.md`
- `docs/programmer_manual.md`
- `csharp/RustEtherNetIp/README.md`
- `wiki/index.md`
- `docs/validation/REAL_PLC_TESTING.md`
- `docs/compat/0.7.0_plc_simulator_compatibility_matrix.md`
- `docs/0.7.0_HARDENING_GATE.md`
- `docs/audit/0.7.0_docs_api_audit.md`
- `docs/ALL_WRAPPERS_UPDATE_COMPLETE.md`
- `docs/WRAPPER_UPDATE_SUMMARY.md`
- `docs/WRAPPER_LIMITATIONS_UPDATE_SUMMARY.md`
- `docs/DLL_DEPLOYMENT.md`
- `docs/LIBRARY_COMPARISON_AND_IMPROVEMENTS.md`
- `docs/UDT_DISCOVERY_v0.5.4.md`
- `docs/VERSION_0.6.0_CHANGELOG.md`

## [2026-04-20] reframe | add historical-reference banners to legacy docs

- Added a consistent historical-reference banner to older docs that still contain removed wrapper-tree references, earlier roadmap assumptions, or pre-release `0.7.0` wording.
- Focused the pass on high-confusion files in `docs/` rather than rewriting their detailed historical content.
- Left the active README/manual/wrapper docs untouched because they are part of the current surface, not the historical-reference cleanup set.

Sources used:

- `docs/ALL_WRAPPERS_UPDATE_COMPLETE.md`
- `docs/WRAPPER_UPDATE_SUMMARY.md`
- `docs/WRAPPER_LIMITATIONS_UPDATE_SUMMARY.md`
- `docs/DLL_DEPLOYMENT.md`
- `docs/LIBRARY_COMPARISON_AND_IMPROVEMENTS.md`
- `docs/UDT_DISCOVERY_v0.5.4.md`
- `docs/VERSION_0.6.0_CHANGELOG.md`
- `docs/validation/REAL_PLC_TESTING.md`
- `docs/compat/0.7.0_plc_simulator_compatibility_matrix.md`
- `docs/0.7.0_HARDENING_GATE.md`
- `docs/audit/0.7.0_docs_api_audit.md`

## [2026-04-21] query | fix python live write status mismatch on routed ControlLogix

- Validated the routed `1756-L81ES` target at `192.168.0.101:44818` / slot `0` across Rust, C#, and Python live paths.
- Confirmed a Python wrapper mismatch where some successful live writes were reported as `0x1E` failures when routed through the native multi-write helper.
- Updated the Python wrapper to execute `write_tag()` through `eip_execute_batch` and `write_tags()` sequentially per tag so per-tag live results remain accurate on the validated ControlLogix path.
- Documented the current Python `write_tags(...)` behavior in `python/README.md`.

Sources used:

- `python/rust_ethernet_ip/client.py`
- `python/rust_ethernet_ip/bindings.py`
- `python/README.md`
- `src/ffi.rs`
- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
