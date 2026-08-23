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
- `csharp/RustEtherNetIp/UDT_README.md`
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

## [2026-05-26] ingest | post-1.0.0 polish and .NET CI failure

- Added a .NET testhost shutdown investigation page for the Ubuntu stable CI failure in GitHub Actions run `26424069105`.
- Updated the index to register the CI investigation.
- Recorded the current mitigation: remove the preview .NET 10 channel from CI and update the C# test runner stack before deeper simulator/native-unload investigation.

Sources used:

- `.github/workflows/ci.yml`
- `csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj`
- `csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs`
- GitHub Actions run `26424069105`
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

## [2026-04-21] ingest | add python ControlLogix real-plc validation record

- Added a dedicated Python wrapper real-PLC validation record for the routed `1756-L81ES` ControlLogix target used in the current validation line.
- Recorded the commands executed, the live success surface, the write-status bug found and fixed during validation, and the remaining open follow-up on one routed BOOL write path.
- Updated the active 2026-04-20 release-gate checklist so it points at the dedicated Python validation outcome.

Sources used:

- `docs/validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-20_real_plc_validation_checklist.md`
- `python/rust_ethernet_ip/client.py`
- `python/README.md`

## [2026-04-21] ingest | update Rust and C# ControlLogix rerun records

- Updated the stable routed `1756-L81ES` Rust and C# validation records with a `2026-04-21` rerun section instead of creating duplicate per-target files.
- Recorded that the Rust rerun reproduced the same `44/44` readonly probe, green live test suites, and `333 passed / 59 failed / 0 skipped` full-matrix result with successful restore.
- Recorded that the C# rerun reproduced the same smoke + `333/392` full-matrix profile and captured the isolated `25`-iteration benchmark numbers from the rerun.
- Noted that a concurrent C# benchmark/full-harness launch caused a local `.pdb` file-lock conflict and that rerunning the benchmark serially resolved it.
- Updated the active 2026-04-20 validation checklist with the rerun outcomes for both Rust and C#.

Sources used:

- `docs/validation/2026-04-16_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`
- `docs/validation/2026-04-20_real_plc_validation_checklist.md`

## [2026-04-21] reframe | strengthen integration, deployment, and support docs

- Added a new active integration/deployment guide that explains Rust, C#, and Python integration tracks, runtime artifact expectations, ControlLogix routing usage, and rollout checks.
- Updated the root README, docs index, C# wrapper README, and Python wrapper README to point users at the active deployment story instead of scattered legacy DLL notes.
- Clarified that the project is open to priority issue handling, feature sponsorship, integration support, and hardware-backed validation collaboration.

Sources used:

- `README.md`
- `docs/README.md`
- `docs/INTEGRATION_AND_DEPLOYMENT.md`
- `docs/DLL_DEPLOYMENT.md`
- `csharp/RustEtherNetIp/README.md`
- `python/README.md`

## [2026-05-14] query | ordered route-hop review for 0.8.0 draft

- Updated `wiki/protocol/route-path-behavior.md`.
- Confirmed a user-reported design gap: grouped route fields could not faithfully represent mixed CIP route order such as backplane -> Ethernet -> backplane.
- Captured that the Rust `0.8.0` draft now has ordered `RouteHop` storage while preserving legacy grouped fields.
- Captured that Ethernet route hops now use extended ASCII/NUL link-address encoding, and that complex multi-hop hardware validation remains pending.
- Captured the legacy grouped-field fallback for empty ordered-hop lists to avoid silently breaking direct public-field construction.

Sources used:

- `src/route.rs`
- `src/client.rs`
- `src/schema.rs`
- `tests/udt_discovery_tests.rs`
- `wiki/protocol/route-path-behavior.md`

## [2026-05-18] query | assess Rust C# Python test coverage strength

- Added `wiki/investigations/test-coverage-strength-2026-05-18.md`.
- Updated `wiki/index.md` to register the new investigation page.
- Recorded that Rust automated coverage is strong, while C# and Python wrapper coverage need clearer simulator/native execution and broader parity coverage.
- Recorded local command results for `cargo test`, `dotnet test`, Python pure tests, and the failing Python auto-start simulator path.

Sources used:

- `Cargo.toml`
- `src/`
- `tests/`
- `csharp/RustEtherNetIp.Tests/`
- `python/tests/`
- `python/rust_ethernet_ip/`
- `docs/validation/2026-04-21_python_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md`

## [2026-05-18] reframe | strengthen Python simulator-backed tests

- Updated the Python simulator harness to prefer the prebuilt `python_test_simulator` binary instead of invoking `cargo run` for each integration test.
- Added Python simulator-backed checks for partial batch read failure preservation and native diagnostics snapshot retrieval.
- Tightened the Rust deterministic simulator so unknown reads return a CIP path error instead of silently returning `DINT(0)`.
- Updated the Python README to document the `--features ffi` build requirement for wrapper integration testing.
- Updated the test coverage strength investigation to mark the Python simulator auto-start blocker as resolved locally.

Sources used:

- `python/tests/sim_harness.py`
- `python/tests/test_integration.py`
- `python/rust_ethernet_ip/bindings.py`
- `python/README.md`
- `tests/plc_sim.rs`
- `wiki/investigations/test-coverage-strength-2026-05-18.md`

## [2026-05-18] reframe | strengthen C# simulator-backed tests

- Added a C# simulator test harness that stages an FFI-enabled native library, verifies required exports, and auto-starts the deterministic Rust simulator when `SIM_PLC_ADDRESS` is not provided.
- Disabled C# test parallelization to avoid native-library staging and load races.
- Expanded C# simulator-backed coverage for scalar read/write, array ranges, batch read/write, mixed execute batch, route connect/diagnostics, and tag-group error events.
- Updated the native-load smoke test to verify key FFI exports instead of only checking that a DLL can be loaded.
- Updated the test coverage strength investigation to record the new `44/44` C# test result and the remaining Moq-based integration-test naming concern.

Sources used:

- `csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs`
- `csharp/RustEtherNetIp.Tests/SimulatorIntegrationTests.cs`
- `csharp/RustEtherNetIp.Tests/MinimalLoadTest.cs`
- `csharp/RustEtherNetIp.Tests/AssemblyInfo.cs`
- `wiki/investigations/test-coverage-strength-2026-05-18.md`

## [2026-05-18] reframe | add C# wrapper contract unit tests

- Added C# wrapper-only tests for `PlcValue` JSON parsing, simple fallback values, raw UDT data, nested UDT dictionaries, invalid payloads, and type-safe access defaults.
- Added route-path contract tests for grouped field preservation, invalid address input, and FFI preparation of pinned address strings.
- Added client contract tests for not-connected behavior, disposed-client behavior, route argument validation, batch input validation, and safe pre-connection statistics.
- Added batch operation/result DTO contract tests.
- Re-ran the C# test project and recorded the suite at `75/75` passing.

Sources used:

- `csharp/RustEtherNetIp/PlcValue.cs`
- `csharp/RustEtherNetIp/RoutePath.cs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs`
- `csharp/RustEtherNetIp.Tests/PlcValueContractTests.cs`
- `csharp/RustEtherNetIp.Tests/RoutePathContractTests.cs`
- `csharp/RustEtherNetIp.Tests/EtherNetIpClientContractTests.cs`
- `csharp/RustEtherNetIp.Tests/BatchOperationContractTests.cs`
- `wiki/investigations/test-coverage-strength-2026-05-18.md`

## [2026-05-18] reframe | add Python wrapper contract unit tests

- Added fake-native Python wrapper tests for `Client` lifecycle, route-path argument forwarding, read-tag decoding, batch partial errors, write failure reporting, sequential write behavior, diagnostics retrieval, and unconnected-operation errors.
- Added helper-contract tests for write-request normalization and write-result parsing.
- Added binding-loader tests to ensure missing native symbols are reported through `NativeLibraryLoadError` with candidate-path detail.
- Re-ran Python tests without simulator (`21 passed, 5 skipped`) and with simulator auto-start (`26 passed`).

Sources used:

- `python/rust_ethernet_ip/client.py`
- `python/rust_ethernet_ip/bindings.py`
- `python/rust_ethernet_ip/types.py`
- `python/rust_ethernet_ip/exceptions.py`
- `python/tests/test_client_contract.py`
- `python/tests/test_bindings.py`

- `wiki/investigations/test-coverage-strength-2026-05-18.md`

## [2026-05-18] query | book-derived architecture documentation check

- Confirmed the post-books architecture synthesis is documented in `wiki/investigations/architecture-review-2026-05-18.md`.
- Added the missing wiki index entry for that synthesis page.
- Linked the post-books synthesis from the authoritative software architecture document.

Sources used:

- `wiki/investigations/architecture-review-2026-05-18.md`
- `wiki/index.md`
- `docs/SOFTWARE_ARCHITECTURE.md`

## [2026-05-24] ingest | record FFI ABI contract

- Added `wiki/protocol/abi-contract.md`.
- Updated `wiki/index.md`.
- Captured ABI version `1`, capability bits, pointer-lifetime rules for `eip_library_version()`, and the wrapper load-time handshake policy from CODEX-L.

Sources used:

- `docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md`
- `src/version.rs`
- `src/ffi.rs`

## [2026-05-24] ingest | record CIP path validation and FFI clone audit

- Added `wiki/protocol/cip-path-validation.md`.
- Added `wiki/wrapper-parity/ffi-registry-clone-audit.md`.
- Updated `wiki/index.md`.
- Captured CODEX-N's checked CIP path encoding rules and CODEX-M Phase A's clone-semantics audit recommendation.

Sources used:

- `docs/agents/tasks/CODEX-N-cip-path-encoding-validation.md`
- `docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md`
- `src/protocol/cip.rs`
- `src/protocol/tests.rs`
- `src/client.rs`
- `src/ffi.rs`

## [2026-05-24] ingest | update FFI clone audit after Phase B

- Updated `wiki/wrapper-parity/ffi-registry-clone-audit.md`.
- Updated `wiki/investigations/architecture-review-2026-05-18.md`.
- Recorded that CODEX-M Phase B structurally shares route-path and max-packet-size state across cloned FFI registry lookups.
- Recorded maintainer direction that CODEX-T and CODEX-U remain in the same v0.8.0 scope.

Sources used:

- `docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md`
- `docs/agents/board.md`
- `src/client.rs`
- `src/ffi.rs`
- `tests/ffi_state_consistency.rs`

## [2026-05-24] ingest | actor client, service helpers, and retry policy

- Added `wiki/investigations/client-actor-service-retry-2026-05-24.md`.
- Updated `wiki/index.md`.
- Captured CODEX-P/R/Q/S synthesis: actor-backed Rust client handle, lifecycle event broadcast, concrete restricted-write service helpers, and opt-in retry policy semantics.

Sources used:

- `src/client/actor.rs`
- `src/client/service_layer.rs`
- `src/client.rs`
- `src/lib.rs`
- `tests/client_actor_tests.rs`
- `docs/agents/tasks/CODEX-P-client-actor.md`
- `docs/agents/tasks/CODEX-R-client-events.md`
- `docs/agents/tasks/CODEX-Q-service-layer.md`
- `docs/agents/tasks/CODEX-S-retry-policy.md`

## [2026-05-24] ingest | fleet API and release-window status

- Added `wiki/investigations/fleet-api-2026-05-24.md`.
- Updated `wiki/index.md`.
- Recorded that CODEX-T is additive over the actor client and that CODEX-U remains open pending a dedicated sibling-crate boundary brief.

Sources used:

- `src/fleet.rs`
- `src/lib.rs`
- `tests/fleet_tests.rs`
- `docs/agents/tasks/CODEX-T-fleet-client-pool.md`
- `docs/agents/tasks/CODEX-U-sibling-crates.md`
- `docs/agents/tasks/CODEX-K-release-window-bundle.md`

## [2026-05-24] ingest | release-window route API and tag-path crate

- Updated `wiki/protocol/route-path-behavior.md`.
- Updated `wiki/investigations/fleet-api-2026-05-24.md`.
- Recorded that CODEX-K moved Rust `RoutePath` to private ordered-hop storage and added ordered-hop FFI/wrapper calls.
- Recorded that CODEX-U extracted shared value types, protocol codecs, tag-path parsing, and UDT helpers into sibling workspace crates.

Sources used:

- `src/route.rs`
- `src/ffi.rs`
- `csharp/RustEtherNetIp/RoutePath.cs`
- `python/rust_ethernet_ip/types.py`
- `crates/types/src/lib.rs`
- `crates/protocol/src/lib.rs`
- `crates/tag-path/src/lib.rs`
- `crates/udt/src/lib.rs`
- `docs/agents/tasks/CODEX-K-release-window-bundle.md`
- `docs/agents/tasks/CODEX-U-sibling-crates.md`

## [2026-05-24] reframe | current docs to 1.0.0 release candidate

- Updated `wiki/protocol/abi-contract.md`.
- Reframed ABI v1 as the `1.0.0` release-candidate contract instead of the superseded `0.8.0` draft line.
- Kept historical 0.7.0 and 0.8.0 release-validation pages unchanged as historical evidence.

Sources used:

- `Cargo.toml`
- `VERSION`
- `docs/agents/board.md`
- `CHANGELOG.md`
- `src/version.rs`

## [2026-06-20] query | review post-1.1.0 roadmap gaps

- Added `wiki/investigations/roadmap-gap-review-2026-06-20.md`.
- Updated `wiki/index.md` to register the review.
- Updated `docs/ROADMAP.md` with additional next-version candidates covering Rust API positioning, Python parity, placeholder/passive-surface decisions, diagnostics placeholders, C# wrapper maintainability, test-suite quality, supply-chain policy, simulator expansion, and post-publish package smoke checks.

Sources used:

- `docs/ROADMAP.md`
- `src/lib.rs`
- `src/client.rs`
- `src/client/actor.rs`
- `src/client/service_layer.rs`
- `src/fleet.rs`
- `src/ffi.rs`
- `src/monitoring.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs`
- `csharp/RustEtherNetIp/README.md`
- `python/rust_ethernet_ip/client.py`
- `python/rust_ethernet_ip/bindings.py`
- `wiki/investigations/client-actor-service-retry-2026-05-24.md`
- `wiki/investigations/fleet-api-2026-05-24.md`
- `wiki/investigations/python-wrapper-strategy-2026-04-19.md`
- `wiki/investigations/test-coverage-strength-2026-05-18.md`

## [2026-07-06] reframe | FFI ABI v2 after CODEX-AS

- Updated `wiki/protocol/abi-contract.md`.
- Recorded that current mainline uses ABI version 2 after removing the raw-pointer FFI exports while keeping handle-based `_by_id` exports.
- Added `CAP_LAST_ERROR` to the capability table and clarified that symbol removal triggers an ABI bump.

Sources used:

- `src/version.rs`
- `src/ffi.rs`
- `docs/agents/tasks/CODEX-AS-ffi-polish-python-residuals.md`
- `docs/API_STABILITY.md`

## [2026-07-07] reframe | STRING and UDT write behavior after CODEX-AP

- Updated `wiki/limitations/string-and-udt-write-behavior.md`.
- Reframed standalone standard `STRING` writes and scalar UDT-array-element-member writes as writeable on current evidence when encoded correctly.
- Recorded that UDT `STRING` members remain current-encoding `0x2107` cases and that CODEX-AP retired legacy STRING and offset-based UDT member APIs as unsupported compatibility stubs.

Sources used:

- `docs/agents/tasks/CODEX-AP-string-udt-graveyard.md`
- `docs/agents/notes/ab-firmware-quirks.md`
- `docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md`
- `docs/validation/2026-07-03_blocked_write_label_probe_plan.md`
- `src/client.rs`
- `src/client/string.rs`

## [2026-07-07] reframe | dead-stratum deprecation after CODEX-AQ

- Updated `wiki/investigations/monitoring-diagnostics-plan-2026-04-19.md`.
- Updated `wiki/investigations/fleet-api-2026-05-24.md`.
- Recorded that diagnostics operation/error counters are now real per-client atomics, while system CPU/memory metrics remain explicitly placeholder.
- Recorded that `PlcManager` is now deprecated 1.x compatibility surface and `Fleet` is the maintained Rust multi-PLC replacement.

Sources used:

- `docs/agents/tasks/CODEX-AQ-dead-stratum-deprecation.md`
- `src/client.rs`
- `src/client/diagnostics.rs`
- `src/monitoring.rs`
- `src/plc_manager.rs`
- `src/fleet.rs`

## [2026-07-07] ingest | subscription and fleet lifecycle after CODEX-AR

- Added `wiki/investigations/subscription-lifecycle-2026-07-07.md`.
- Updated `wiki/investigations/fleet-api-2026-05-24.md`.
- Updated `wiki/investigations/client-actor-service-retry-2026-05-24.md`.
- Updated `wiki/index.md`.
- Recorded that live single-tag subscriptions now stop cooperatively, expose
  value/error events, and use drop-oldest backpressure; fleet forwarding now
  survives lag and aborts replaced forwarders; `Client::events()` is
  observation-only.

Sources used:

- `docs/agents/tasks/CODEX-AR-subscription-fleet-lifecycle.md`
- `src/subscription.rs`
- `src/client/subscriptions.rs`
- `src/tag_group.rs`
- `src/fleet.rs`
- `src/client/actor.rs`
- `tests/subscription_tests.rs`
- `tests/fleet_tests.rs`
- `tests/client_actor_tests.rs`

## [2026-07-08] ingest | hardware-free CODEX-AO phase 1 and CODEX-AU

- Updated `wiki/limitations/string-and-udt-write-behavior.md`.
- Added `wiki/wrapper-parity/cpp-consumer-support.md`.
- Updated `wiki/index.md`.
- Recorded that CODEX-AO Phase 1 closes the UDT read-modify-write zero-fill
  hazard without resolving the packet-capture-gated UDT wire-format question.
- Recorded that CODEX-AU adds first-class C/C++ consumption through the
  existing C ABI: checked-in header, export parity gate, CMake smoke example,
  and Qt worker-thread guidance.

Sources used:

- `docs/agents/tasks/CODEX-AO-udt-wire-format-investigation.md`
- `docs/agents/tasks/CODEX-AU-cpp-consumer-support.md`
- `crates/udt/src/lib.rs`
- `include/rust_ethernet_ip.h`
- `scripts/check-ffi-header-parity.py`
- `examples/cpp/`
- `.github/workflows/ci.yml`
- `docs/CPP_INTEGRATION.md`

## [2026-07-08] ingest | AW AX AZ packet-size and string coverage fixes

- Updated `wiki/limitations/string-and-udt-write-behavior.md`.
- Updated `wiki/investigations/test-coverage-strength-2026-05-18.md`.
- Recorded that CODEX-AW enforces batch packet byte budgets using the same
  service-request bytes that the MSP sender uses.
- Recorded that CODEX-AX moves the full-coverage manifest to 2304 total / 2285
  writeable / 0 expected-blocked / 19 read-only after handle-aware STRING
  writes, and that Rust/C#/Python runners now write and verify STRINGs.
- Recorded that CODEX-AZ adds simulator-covered CIP fragmented read/write for
  large string/structure payloads, with real `Str500+` hardware confirmation
  still pending.
- Recorded that CODEX-AO Phase 2 remains blocked because no packet captures
  satisfying the checklist are present.

Sources used:

- `docs/agents/tasks/CODEX-AW-batch-read-packet-size.md`
- `docs/agents/tasks/CODEX-AX-full-coverage-string-writes.md`
- `docs/agents/tasks/CODEX-AZ-cip-fragmentation.md`
- `docs/agents/tasks/CODEX-AO-udt-wire-format-investigation.md`
- `docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`
- `src/client.rs`
- `src/client/batch_exec.rs`
- `tests/plc_sim.rs`
- `tests/plc_sim_tests.rs`
- `examples/full_coverage_tags.json`
- `examples/test_plc_full_coverage.rs`
- `examples/CSharpFullCoverage/Program.cs`
- `python/examples/test_plc_full_coverage.py`
- `python/rust_ethernet_ip/client.py`
- `docs/STRING_HANDLING.md`
- `docs/agents/notes/ab-firmware-quirks.md`

## [2026-07-14] reframe | verify and centralize the Rust 1.88 workspace MSRV

- Updated `wiki/investigations/rust-toolchain-baseline-2026-04-19.md` and
  `wiki/index.md`.
- Reframed the toolchain baseline from a current-stable policy to the exact
  oldest compiler supporting the locked dependencies and complete Rust test
  suite.
- Recorded the adjacent boundary: Rust `1.87.0` fails because `time 0.3.47` and
  `time-core 0.1.8` require Rust `1.88.0`; Rust `1.88.0` compiles the workspace
  tests with all features.
- Recorded the passing full Rust `1.88.0` suite and stable formatting/Clippy
  gates.

Sources used:

- `Cargo.toml`
- `Cargo.lock`
- `.github/workflows/ci.yml`
- `README.md`
- `BUILD.md`
- `docs/API_STABILITY.md`
- `examples/desktop_app/README.md`

## [2026-07-14] lint | reconcile toolchain baseline after PR 27

- Updated `wiki/investigations/rust-toolchain-baseline-2026-04-19.md`, which
  was omitted from the merged PR even though the preceding log entry and
  `wiki/index.md` described it as updated.
- Aligned the page with the centralized Rust `1.88` workspace MSRV, the
  `1.87.0` dependency boundary, and the manifest-driven CI test gate.
- Preserved the Rust `1.95` / `1.96` current-stable policy as historical
  context and clarified that downstream consumers resolve their own lockfiles.
- No additional user-facing documentation change was needed; the merged PR
  already aligned the active MSRV documentation.

Sources used:

- `Cargo.toml`
- `Cargo.lock`
- `.github/workflows/ci.yml`
- `README.md`
- `BUILD.md`
- `docs/API_STABILITY.md`
- `examples/desktop_app/Cargo.toml`
- `examples/desktop_app/README.md`
- `docs/agents/tasks/CODEX-AH-rust-1.96-msrv-and-assert-matches.md`
- GitHub PR `#27`

## [2026-08-13] query | review program-tag discovery PRs 28 and 29

- Added `wiki/protocol/program-tag-discovery.md` and updated `wiki/index.md`.
- Confirmed that both PRs address real defects: program Symbol Object paging on
  CIP `0x06` and request-derived `TagScope` propagation.
- Recorded a merge-blocking normalization defect in PR #29 for callers using
  the already-supported `Program:Name` input form.
- Recorded the textual conflict and recommended merge order: #28 first, then a
  rebased #29 that keeps scope-aware page parsing and drops the deleted wrapper.
- Verified formatting, focused discovery tests, targeted program-tag tests, and
  all-feature Clippy independently on both PR heads. The full workspace run
  exceeded the review timeout without a PR-specific failure before timeout.

Sources used:

- `src/client.rs`
- `crates/udt/src/lib.rs`
- `src/schema.rs`
- `tests/udt_discovery_tests.rs`
- `docs/agents/tasks/CODEX-AM-tag-addressing-correctness.md`
- `docs/validation/2026-07-02_tag_addressing_smoke_5069-L330ERM_fw38.md`
- GitHub PR `#28`
- GitHub PR `#29`

## [2026-08-13] query | verify PR 28 merge and PR 29 review request

- Updated `wiki/protocol/program-tag-discovery.md`.
- Confirmed PR #28 was squash-merged as `afac5ee` and is the tip of
  `origin/main`.
- Confirmed the maintainer comment on PR #29 requests a post-#28 rebase,
  normalized `TagScope::Program` values for both accepted input forms, and
  regression coverage.
- Confirmed GitHub currently reports PR #29 as conflicting; the comment is a
  regular issue comment rather than a formal request-changes review.

Sources used:

- `origin/main`
- GitHub PR `#28`
- GitHub PR `#29`

## [2026-08-13] ingest | remediate desktop webbrowser audit failure

- Updated `wiki/investigations/rust-toolchain-baseline-2026-04-19.md`.
- Upgraded the desktop example from `egui`/`eframe 0.27.2` to `0.33.3`, which
  permits the patched `webbrowser 1.2.2` dependency without raising the
  workspace's Rust `1.88` MSRV.
- Adapted the `eframe` application factory to its result-returning API.
- Confirmed `cargo audit` exits successfully with no vulnerabilities; the
  existing `RUSTSEC-2026-0221` `event-listener` item remains an allowed warning
  in the Linux accessibility dependency chain.
- Confirmed stable formatting, full workspace/all-target tests, and
  all-target/all-feature Clippy with warnings denied.
- Installed Rust `1.88.0` and confirmed the locked all-features workspace and
  documentation tests pass on the declared MSRV.

Sources used:

- `Cargo.lock`
- `examples/desktop_app/Cargo.toml`
- `examples/desktop_app/src/main.rs`
- `.github/workflows/ci.yml`
- RustSec `RUSTSEC-2026-0257`
- RustSec `RUSTSEC-2026-0221`

## [2026-08-14] query | verify PR 29 rebase, normalization, tests, and CI

- Updated `wiki/protocol/program-tag-discovery.md` and `wiki/index.md`.
- Confirmed PR #29 head `fcedeb4` is based directly on current `main` at
  `1d675bd`, is cleanly mergeable, and preserves PR #28's paging structure.
- Confirmed both accepted program-name forms normalize to the bare program name
  for `TagScope::Program` and build byte-identical requests.
- Independently passed formatting, all-target Clippy with warnings denied, all
  16 focused discovery tests, the release FFI build, and all 86 C# unit tests.
- Confirmed the regression tests discriminate the normalization fix: replacing
  the helper with an identity function failed exactly the two new tests and left
  the other 14 focused discovery tests green.
- Confirmed GitHub's CI run completed successfully across all 29 jobs, including
  the stable/beta Rust platform matrix and stable C# unit/native integration
  coverage.

Sources used:

- `src/client.rs` at PR #29 head `fcedeb4`
- `CHANGELOG.md` at PR #29 head `fcedeb4`
- `.github/workflows/ci.yml`
- GitHub PR `#29` and Actions run `31825631742`

## [2026-08-14] ingest | record PR 29 merge and green main workflow

- Updated `wiki/protocol/program-tag-discovery.md` and `wiki/index.md`.
- Fast-forwarded local `main` from `1d675bd` to the PR #29 squash merge
  `481f20d` while preserving the pending wiki synthesis edits.
- Confirmed the post-merge `main` CI/CD workflow completed successfully.

Sources used:

- `src/client.rs` at `481f20d`
- `CHANGELOG.md` at `481f20d`
- GitHub PR `#29`
- GitHub Actions post-merge run for `481f20d`

## [2026-08-21] reframe | prepare 1.2.1 documentation, hardware program, and website

- Added `wiki/releases/1.2.0-validation-synthesis.md` and
  `wiki/controllers/hardware-validation-program.md`.
- Updated `wiki/index.md`, controller/firmware behavior, route behavior,
  wrapper parity, documentation state, and stale source links.
- Established `1.2.0` as the latest published evidence baseline and `1.2.1` as
  the patch in preparation.
- Documented the exact meaning of hardware-matrix `Done` cells and separated
  functional, endurance, and performance evidence.

Sources used:

- `docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md`
- `docs/validation/2026-05-25_real_plc_two-controller_cross-binding_full-coverage.md`
- `docs/HARDWARE_COMPATIBILITY.md`
- `docs/audit/1.2.0_markdown_release_audit.md`
- `docs/release/1.2.1_RELEASE_NOTES_DRAFT.md`

## [2026-08-21] reframe | rebuild wrapper onboarding around 1.2.0 behavior

- Updated `wiki/wrapper-parity/rust-vs-csharp.md`.
- Corrected C# Markdown, IntelliSense, and fallback error text that still
  described handle-aware UDT STRING-member writes as firmware-blocked.
- Added a buildable six-step C# learning path, expanded Python core examples
  alongside its analytics/service focus, and added build-checked C++ routing,
  diagnostics, and controller-discovery programs.
- Made the wrapper discovery boundary explicit: known program paths work in all
  bindings, while program-scoped enumeration is currently Rust-only.

Sources used:

- `csharp/RustEtherNetIp/README.md`
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs`
- `csharp/RustEtherNetIp/Examples/`
- `python/README.md`
- `python/examples/`
- `docs/CPP_INTEGRATION.md`
- `examples/cpp/`
- `docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md`
- `wiki/limitations/string-and-udt-write-behavior.md`
- `CHANGELOG.md`

## [2026-08-21] ingest | compare validation tiers and native-wrapper adoption gaps

- Updated `wiki/wrapper-parity/rust-vs-csharp.md` and `wiki/index.md`.
- Confirmed that software OS/toolchain release gates and exact PLC hardware
  evidence must remain separate matrices.
- Recorded that the C ABI is the complete native contract while the C++ RAII
  class is a smaller example; added macOS to blocking C++ CI; and prioritized
  native SDK packaging, generated reference docs, RAII coverage, and
  compiler/sanitizer breadth.

Sources used:

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `include/rust_ethernet_ip.h`
- `examples/cpp/eip_client.hpp`

- `examples/cpp/CMakeLists.txt`
- `docs/audit/1.2.1_wrapper_and_platform_gap_analysis.md`
- Public native-library platform, packaging, C API, and contributor-testing
  conventions reviewed 2026-08-21

## [2026-08-21] ingest | resolve Ubuntu .NET testhost crash from captured dump

- Updated `wiki/investigations/dotnet-testhost-shutdown-2026-05-26.md` and
  `wiki/index.md`.
- Confirmed the faulting path was the test harness manually inspecting a shared
  library after the CLR had already loaded it, not batch FFI or Tokio shutdown.
- Replaced runtime copy/load/free behavior with deterministic pre-testhost
  MSBuild staging, added a non-mutation regression, and removed CI retries.

Sources used:

- GitHub Actions run `32528795950` crash dump and test sequence
- `csharp/RustEtherNetIp.Tests/SimulatorTestHarness.cs`
- `csharp/RustEtherNetIp.Tests/MinimalLoadTest.cs`
- `csharp/RustEtherNetIp.IntegrationTests/PlcSimulatorFixture.cs`
- `.github/workflows/ci.yml`

## [2026-08-21] ingest | align package CI with unpublished 1.2.1 development line

- Confirmed raw `cargo package` for the root crate cannot resolve unpublished
  same-version workspace siblings from crates.io, even with `--no-verify`.
- Removed the contradictory root-only package invocation from the wrapper
  package job and made that job depend on the existing dependency-order-aware
  Rust release-readiness gate.
- Preserved strict release-day behavior: `check-release-readiness --strict`
  still requires sibling crates to be published in dependency order.

Sources used:

- `.github/workflows/ci.yml`
- `scripts/check-release-readiness`
- `tests/release_readiness_tests.sh`
- `docs/release/1.2.1_RELEASE_NOTES_DRAFT.md`

## [2026-08-21] reframe | clarify wrapper operation choices and STRING byte limits

- Updated `wiki/wrapper-parity/rust-vs-csharp.md`.
- Added language-specific decision tables and runnable examples for single
  tags, batches, whole-UDT reads, member writes, and controller/program paths.
- Clarified that built-in Logix `STRING` has an 82-byte `DATA` capacity and
  that the measured 494-byte single-request ceiling includes CIP/path overhead.
- Added a website Rust-core capability catalog, C++ quick start, architecture
  rationale, and explicit Windows/Linux/macOS explanation.

Sources used:

- `README.md`
- `docs/STRING_HANDLING.md`
- `csharp/RustEtherNetIp/README.md`
- `csharp/RustEtherNetIp/UDT_README.md`
- `csharp/RustEtherNetIp/Examples/GettingStarted/`
- `python/README.md`
- `python/examples/`
- `docs/CPP_INTEGRATION.md`
- `examples/cpp/`
- `src/client.rs`
- `docs/validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md`
- `wiki/limitations/string-and-udt-write-behavior.md`

## [2026-08-21] reframe | keep website sponsorship native and privacy-preserving

- Chose a responsive site-native sponsorship card linking to GitHub Sponsors
  instead of a third-party iframe.
- Preserved the restrictive frame policy and disclosed the local-only
  quick-start language preference accurately.

Sources used:

- `website/index.html`
- `website/launch.css`
- `website/_headers`
- `website/privacy.html`

## [2026-08-21] query | separate full-coverage, batch, UDT, and discovery evidence

- Updated `wiki/releases/1.2.0-validation-synthesis.md`.
- Confirmed that the 2,338-tag four-binding release gate used per-tag reads and
  writes, including STRING members and fragmented whole-UDT reads.
- Clarified that batch operations and discovery were validated separately and
  were not invoked by the four full-coverage runners.

Sources used:

- `docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md`
- `docs/validation/2026-07-08_cross-binding_full-coverage_5069-L330ERM_fw38.md`
- `examples/test_plc_full_coverage.rs`
- `examples/CSharpFullCoverage/Program.cs`
- `python/examples/test_plc_full_coverage.py`
- `examples/cpp/full_coverage.cpp`

## [2026-08-21] ingest | add cross-binding hardware feature companion gate

- Updated `wiki/controllers/hardware-validation-program.md`.
- Added restore-safe Rust, C#, Python, and C/C++ runners for batch operations,
  whole-UDT reads, and the discovery surfaces each binding exposes.
- Added explicit `--allow-writes` protection, a serial macOS/Linux
  orchestration script, offline CI dry runs, and a maintainer runbook.
- Fixed and unit-tested the C# program-scope versus UDT-member path classifier
  found while preparing program-scoped batch-write coverage.
- Kept Python and program-discovery `N/A` cells explicit rather than treating
  unavailable wrapper APIs as hardware failures.

Sources used:

- `docs/validation/CROSS_BINDING_FEATURE_GATE.md`
- `examples/hardware_feature_gate.rs`
- `examples/CSharpHardwareFeatureGate/Program.cs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.cs`
- `csharp/RustEtherNetIp.Tests/WriteTagsBatchTests.cs`
- `python/examples/hardware_feature_gate.py`
- `examples/cpp/hardware_feature_gate.cpp`
- `scripts/run-cross-binding-feature-gate.sh`

## [2026-08-21] ingest | record 1756-L75 fw33 four-binding performance baseline

- Updated `wiki/controllers/hardware-validation-program.md`.
- Added a repeatable opt-in benchmark mode to all four full-coverage runners.
- Recorded the 1756-L75 fw33 / 1756-EN2T topology and MacBook Pro M2 host.
- Confirmed 27,648/27,648 sequential reads and 27,420/27,420 sequential writes
  across Rust, Python, C#, and C/C++ with zero failures.
- Kept this single-tag mixed-manifest baseline distinct from future batch,
  endurance, reconnect, host-resource, and controller-load measurements.

Sources used:

- `docs/validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md`
- `docs/HARDWARE_COMPATIBILITY.md`
- `examples/full_coverage_tags.json`
- `examples/test_plc_full_coverage.rs`
- `python/examples/test_plc_full_coverage.py`
- `examples/CSharpFullCoverage/Program.cs`
- `examples/cpp/full_coverage.cpp`

## [2026-08-21] ingest | add 1756-L75 fw33 batch-size performance baseline

- Updated `wiki/controllers/hardware-validation-program.md`.
- Added sizes 1, 5, 10, 20, 50, and 100 to all four performance runners with
  30-second and 1,000-tag-operation sampling floors.
- Preserved complete latency distributions while adding Tukey-IQR filtered
  averages and outlier counts rather than deleting tail evidence.
- Confirmed zero call/per-tag failures across the four bindings.
- Recorded approximately 2,830 native DINT writes/second at size 100 through
  Rust, C#, and C/C++; kept Python grouped sequential writes distinct.
- Identified uncached array-type detection as the main batch-read optimization
  candidate for a controlled future before/after benchmark.

Sources used:

- `docs/validation/2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md`
- `docs/validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md`
- `docs/HARDWARE_COMPATIBILITY.md`
- `src/client/batch_exec.rs`
- `src/client.rs`
- `examples/test_plc_full_coverage.rs`
- `python/examples/test_plc_full_coverage.py`
- `examples/CSharpFullCoverage/Program.cs`
- `examples/cpp/full_coverage.cpp`

## [2026-08-21] ingest | validate cached array classification on 1756-L75 fw33

- Updated `wiki/controllers/hardware-validation-program.md` and
  `wiki/wrapper-parity/ffi-registry-clone-audit.md`.
- Preserved the original six-size batch run as a before baseline and added a
  controlled optimized rerun on the same controller, route, host, and workload.
- Confirmed zero failures and 100/100 terminal-value verification in all four
  bindings. Size-100 native reads converged near 3,305 tags/second.
- For build-identical Rust, C#, and C/C++, size-100 read throughput improved
  10.9–11.6x while native writes remained near 2,830 tags/second.
- Corrected Python source-checkout artifact priority; its optimized result is
  valid, but its historical debug-build baseline is excluded from exact
  build-identical attribution.

Sources used:

- `docs/validation/2026-08-21_1756-L75_fw33_cross-binding-batch-performance.md`
- `docs/validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md`
- `docs/HARDWARE_COMPATIBILITY.md`
- `src/client.rs`
- `python/rust_ethernet_ip/bindings.py`
- `python/tests/test_bindings.py`

## [2026-08-21] query | prioritize post-cache performance work

- Updated `wiki/controllers/hardware-validation-program.md`.
- Ranked a controlled packet-policy sweep and a 24-hour read soak ahead of
  speculative allocation-level optimizations.
- Identified Python grouped writes as the largest measured wrapper-specific
  opportunity: size-100 grouped writes were about 272 tags/second versus about
  2,830 tags/second for native Rust/C#/C++ writes.
- Kept a native Python write change conditional on parity coverage for atomic,
  STRING, packed-BOOL, partial-failure, and result-order behavior.

Sources used:

- `docs/validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md`
- `docs/HARDWARE_COMPATIBILITY.md`
- `src/client/batch_exec.rs`
- `src/batch.rs`
- `src/ffi.rs`
- `python/rust_ethernet_ip/client.py`

## [2026-08-21] query | define array-type cache lifecycle after PLC downloads

- Added `wiki/investigations/array-type-cache-lifecycle.md` and updated the
  wiki index.
- Confirmed that new native clients, route changes, and Rust `clear_caches()`
  invalidate the cache, but a PLC download is not directly detected.
- Identified transitions to or from packed BOOL arrays as the safety-relevant
  stale-classification case.
- Recommended explicit cross-language cache clearing plus response-validated,
  read-only self-healing before treating downloads as transparent.

Sources used:

- `src/client.rs`
- `src/client/batch_exec.rs`
- `src/client/actor.rs`
- `src/ffi.rs`
- `csharp/RustEtherNetIp/EthernetNetIpClient.Connection.cs`
- `python/rust_ethernet_ip/client.py`

## [2026-08-22] reframe | include online tag replacement in cache lifecycle

- Updated `wiki/investigations/array-type-cache-lifecycle.md`.
- Added the practitioner-confirmed online workflow where a temporary tag
  replaces an old tag under the same final symbolic name without reconnecting.
- Expanded invalidation requirements from PLC downloads to online Symbol Not
  Found/delete/recreate transitions and offline UDT layout changes.
- Recommended one cross-language schema refresh operation covering array,
  metadata, STRING-handle, and UDT-definition caches.

Sources used:

- Maintainer-provided 1756-L75/Studio 5000 workflow description, 2026-08-22
- `src/client.rs`
- `src/client/batch_exec.rs`
- Rockwell Studio 5000 Tag Editor documentation
- Rockwell Studio 5000 download options documentation

## [2026-08-22] query | define cache-safety implementation checklist

- Updated `wiki/investigations/array-type-cache-lifecycle.md`.
- Found that Rust `clear_caches()` does not clear `TagManager`'s separate UDT
  definition map, so it is not yet a complete schema refresh.
- Added a shared schema generation requirement to prevent in-flight FFI/client
  clones from repopulating stale entries after invalidation.
- Prioritized comprehensive invalidation, wrapper parity, safe read
  self-healing, tests, hardware validation, and diagnostics before further
  packet-level performance tuning.

Sources used:

- `src/client.rs`
- `src/tag_manager.rs`
- `src/client/batch_exec.rs`
- `src/ffi.rs`

## [2026-08-22] ingest | open schema-safety and performance task sequence

- Updated `wiki/investigations/array-type-cache-lifecycle.md` with links to the
  durable agent backlog.
- Opened CODEX-BA through CODEX-BD as the ordered, release-blocking 1.2.1
  cache/schema safety sequence.
- Retained packet-policy tuning, Python native writes, endurance, and tag-shape
  characterization as CODEX-BE through CODEX-BH non-blocking follow-ups.

Sources used:

- `docs/agents/tasks/CODEX-BA-schema-cache-generation.md`

- `docs/agents/tasks/CODEX-BB-schema-drift-self-healing.md`
- `docs/agents/tasks/CODEX-BC-cross-binding-schema-refresh-diagnostics.md`
- `docs/agents/tasks/CODEX-BD-schema-change-validation-gate.md`
- `docs/agents/tasks/CODEX-BE-batch-packet-policy-sweep.md`
- `docs/agents/tasks/CODEX-BF-python-native-batch-writes.md`
- `docs/agents/tasks/CODEX-BG-cross-binding-endurance-soak.md`
- `docs/agents/tasks/CODEX-BH-tag-shape-performance-matrix.md`
- `docs/agents/board.md`

## [2026-08-22] query | classify the library's EtherNet/IP role

- Added `wiki/protocol/device-role-classification.md` and updated the wiki
  index.
- Confirmed that the active implementation is a CIP explicit-messaging
  client/originator using TCP, `SendRRData`, and primarily Unconnected Send.
- Distinguished an explicit-message originator from an I/O Scanner/connection
  originator and confirmed that the project implements neither I/O Scanner nor
  I/O Adapter behavior.

Sources used:

- `src/client.rs`
- `src/client/string.rs`
- `docs/agents/notes/unconnected-send.md`
- ODVA EtherNet/IP Technology Overview
- ODVA Common Industrial Protocol and the Family of CIP Networks

## [2026-08-22] reframe | align public driver-role wording

- Updated the main README and website metadata, hero, architecture flow, scope
  note, and footer wording.
- Public copy now identifies the project as an EtherNet/IP/CIP
  explicit-messaging client driver and says that it is not a cyclic implicit-I/O
  Scanner or Adapter.
- Documented that a registered TCP encapsulation session is not, by itself, a
  connected CIP Class 3 connection.
- Updated the website maintenance note and protocol-role wiki evidence.

Sources used:

- `README.md`
- `website/index.html`
- `website/privacy.html`
- `website/license.html`
- `website/README.md`
- `wiki/protocol/device-role-classification.md`

## [2026-08-22] ingest | add schema generation and comprehensive refresh

- Updated `wiki/investigations/array-type-cache-lifecycle.md` after CODEX-BA.
- Recorded the clone-shared monotonic schema generation, comprehensive Rust
  refresh operation, route-change invalidation, and stale-insertion guards.
- Kept response-driven recovery and cross-binding exposure as explicit CODEX-BB
  and CODEX-BC follow-up work.

Sources used:

- `src/client.rs`
- `docs/agents/tasks/CODEX-BA-schema-cache-generation.md`
- `CHANGELOG.md`
- `docs/release/1.2.1_RELEASE_NOTES_DRAFT.md`

## [2026-08-22] ingest | add bounded schema-drift recovery

- Updated `wiki/investigations/array-type-cache-lifecycle.md` after CODEX-BB.
- Confirmed path-local eviction and exactly one logical read retry for stale
  packed-BOOL classification; sent writes remain fail-closed and are not
  replayed.
- Added dynamic simulator evidence for controller/program paths, both sides of
  the 32-bit BOOL boundary, batch correlation, and delete/recreate behavior.

Sources used:

- `src/client.rs`
- `src/client/batch_exec.rs`
- `tests/plc_sim.rs`
- `tests/schema_drift_recovery_tests.rs`

## [2026-08-22] ingest | expose schema maintenance across bindings

- Updated `wiki/protocol/abi-contract.md` for coordinated additive ABI v3 and
  the schema-refresh capability bit.
- Exposed comprehensive schema refresh through C, C#, Python, and C++, and
  added backward-compatible schema/cache/recovery diagnostic fields.
- Documented the safe maintenance sequence: pause writes, edit/download,
  refresh, optionally rediscover and verify, then resume writes.

Sources used:

- `src/version.rs`
- `src/ffi.rs`
- `include/rust_ethernet_ip.h`
- `csharp/RustEtherNetIp/`
- `python/rust_ethernet_ip/`
- `examples/cpp/eip_client.hpp`

## [2026-08-22] ingest | prepare schema-change hardware gate

- Added the reusable controller schema-change procedure and the dated
  1756-L75 firmware-33 record.
- Recorded an offline cross-binding PASS from one release artifact for dynamic
  schema mutation, explicit refresh, diagnostics generation, and header parity.
- Kept the live Studio 5000 portion explicitly pending; it is not hardware
  compatibility evidence until the edit/download, session, write-count, UDT
  rediscovery, full-coverage, and restoration rows are completed.

Sources used:

- `scripts/schema-change-gate`
- `tests/schema_drift_recovery_tests.rs`
- `docs/validation/SCHEMA_CHANGE_GATE.md`
- `docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md`

## [2026-08-22] ingest | add live schema-change gate companion

- Independently reran the offline gate (`scripts/schema-change-gate`) plus the
  full workspace matrix — `cargo fmt`, all-feature clippy with warnings
  denied, `SKIP_PLC_TESTS=1 cargo test --workspace --locked`, and
  `plc_sim_tests` — all green on the CODEX-BA/BB/BC/BD tree.
- Added `examples/schema_change_gate_live.rs`, a Rust companion that
  automates the non-editing steps of the live procedure against a real
  controller: baseline capture, warm reads at indices 5/40 in controller and
  program scope, an optional restore-safe pre-edit write smoke check, a
  stdin pause for the maintainer's Studio 5000 action, post-edit reads with
  automatic-recovery counters, explicit `refresh_schema()`, rediscovery,
  post-refresh reads, and an optional restore-safe post-refresh write/verify.
  It never sends a schema edit and prints a result block for the dated
  record. The UDT layout/download section and the C#/Python/C++ companions
  remain manual.
- `SCHEMA_CHANGE_GATE.md` documents the new tool alongside the manual
  per-binding steps it does not yet cover.

Sources used:

- `examples/schema_change_gate_live.rs`
- `docs/validation/SCHEMA_CHANGE_GATE.md`
- `docs/agents/tasks/CODEX-BD-schema-change-validation-gate.md`

## [2026-08-22] ingest | Capture exact ControlLogix and EN2T revisions

- Updated `wiki/controllers/hardware-validation-program.md`.
- Recorded the maintainer-confirmed `1756-L75/B` firmware `33.011` and
  `1756-EN2T/D` firmware `10.007`, replacing the earlier major-only evidence.
- Sources: `docs/validation/2026-08-22_1756-L75_fw33_schema-change-gate.md`,
  `docs/validation/2026-08-21_1756-L75_fw33_cross-binding-performance.md`, and
  maintainer-supplied Studio 5000/module revision details.

## [2026-08-22] ingest | Record post-schema cross-binding performance rerun

- Updated `wiki/controllers/hardware-validation-program.md`.
- Recorded the four-binding 2,304-path sequential rerun: zero failures, with
  average reads 49–58% lower and writes 7–19% lower than the prior-day
  baseline, scoped to the exact host/controller/route.
- Sources:
  `docs/validation/2026-08-22_1756-L75_fw33_post-schema-cross-binding-performance.md`
  and the four local JSON benchmark artifacts generated by the runners.

## [2026-08-22] ingest | Validate Python native batch writes on 1756-L75

- Updated `wiki/controllers/hardware-validation-program.md` after the Python
  grouped-write API moved its safe atomic subset to native MSP batching.
- Recorded the controlled 1756-L75/B firmware-33.011 result: size-100 DINT
  writes reached 2,773 tags/s, 10.21x the retained sequential baseline, with
  zero failures and 100/100 terminal verification.
- Kept STRING/UDT, member/bit, packed-BOOL-array, and duplicate-name operations
  explicitly outside the native-throughput claim because they use typed
  sequential fallbacks.

Sources used:

- `python/rust_ethernet_ip/client.py`
- `python/tests/test_client_contract.py`
- `python/tests/test_integration.py`
- `examples/full_coverage_results/python_batch_benchmark_20260823T020359Z.json`
- `docs/validation/2026-08-21_1756-L75_fw33_batch-array-cache-before-after.md`
- `docs/validation/2026-08-22_1756-L75_fw33_python-native-batch-writes.md`

## [2026-08-22] ingest | Repeat post-BF/BI four-binding full coverage

- Updated `wiki/controllers/hardware-validation-program.md` with the
  maintainer-requested post-merge regression result.
- Rust, C#, Python, and C/C++ each passed 2,304 reads, 2,285 writes and
  read-back verifies, 2,285 settle operations, and 18 settle samples on the
  1756-L75/B firmware-33.011 target with zero anomalies.
- Recorded that the terminal PLC state was restored to the established
  settled-value family.

Sources used:

- `examples/full_coverage_tags.json`
- `examples/full_coverage_results/2026-08-22_post-bi-bf-rerun/*.json`
- `docs/validation/2026-08-22_1756-L75_fw33_post-BF-BI-cross-binding-full-coverage.md`
