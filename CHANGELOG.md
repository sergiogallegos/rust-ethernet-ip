# Changelog

All notable changes to the rust-ethernet-ip project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.2.1] - 2026-08-22

### Added

- Added a clone-shared controller-schema generation and comprehensive Rust
  `refresh_schema()` operation. Refreshes and route changes invalidate packed
  BOOL classification, tag metadata, TagManager UDT definitions, and UDT
  templates/attributes; generation guards prevent older in-flight operations
  from repopulating refreshed cache state.
- Added bounded schema-drift recovery for array-element reads. Contradictory
  packed-BOOL/ordinary-array responses and symbolic-path failures evict the
  affected classification and retry the read once; writes are never replayed
  after they have been sent. Batch recovery preserves per-tag result order.
- Added `eip_refresh_schema` and matching C#, Python, and C++ convenience APIs.
  Schema diagnostics now report generation, refreshes, array classification
  hits/misses/evictions, datatype contradictions, and read-recovery outcomes.
  The coordinated additive C ABI is v3 with `CAP_SCHEMA_REFRESH`.
- Added complete public Rust API documentation across the main crate and all
  four published support crates. The workspace now denies undocumented public
  items, and rustdoc is verified with warnings treated as errors.
- Added a real-hardware compatibility program with an exact
  processor/firmware/topology/binding matrix, a contributor result template,
  safe controller setup, 1–24 hour endurance profiles, and repeatable
  performance characterization guidance.
- Added restore-safe Rust, C#, Python, and C/C++ companion hardware runners for
  native batch reads, applicable batch/grouped writes, whole-UDT reads, and
  the discovery surfaces each binding exposes.
- Added opt-in, full-manifest sequential latency benchmarks to the Rust, C#,
  Python, and C/C++ hardware runners, including average, min, p50, p95, p99,
  max, throughput, sample counts, failure counts, and JSON output. A 1756-L75
  firmware 33 baseline records 27,648 successful reads and 27,420 successful
  writes across the four bindings with no failures.
- Added batch-size performance modes at 1, 5, 10, 20, 50, and 100 tags with
  30-second/1,000-tag-operation sampling floors, raw latency distributions,
  Tukey-IQR filtered averages, outlier counts, logical batches/second,
  tags/second, terminal-value verification, and explicit Python grouped-write
  labeling. The L75 firmware 33 baseline reached about 2,830 native DINT
  writes/second at size 100 across Rust, C#, and C/C++.
- Added a dependency-free light-theme project website under `website/`, ready
  for Cloudflare Pages or GitHub Pages and linking Rust, C#, Python, and C/C++
  documentation, packages, source, releases, and hardware evidence.
- Added production launch metadata for `rustethernetip.com`, including canonical
  and social URLs, a sitemap, crawler rules, browser security headers, a custom
  404 page, a security contact, and a Cloudflare Pages launch checklist.
- Added website privacy, MIT license, industrial safety, security, and
  contribution links; the shipped site uses no analytics, cookies, forms, or
  third-party font requests.
- Added prominent GitHub Sponsors links to the website navigation, final
  call-to-action, and community footer.
- Switched website navigation to a responsive CSS text wordmark while retaining
  the full mascot logo in the footer.
- Added a responsive application-to-controller architecture flow showing where
  the language bindings, Rust core, EtherNet/IP/CIP, and Logix tag operations
  fit, including the correct backend boundary for browser-based applications.
- Refined the website launch copy around the common “EtherNet/IP library” and
  “PLC driver” search terms, replaced decorative live-state claims with a real
  hardware validation snapshot, linked every headline result to its dated
  evidence, and strengthened mobile keyboard and viewport behavior.
- Launched the project at `rustethernetip.com`, added the live site to the main
  README, aligned legal-page canonical URLs with Cloudflare's clean paths, and
  made direct Logix PLC read/write access the homepage headline.
- Matched the website footer to the mascot logo's white canvas so the original
  full-brand artwork blends cleanly without a visible rectangular background.
- Added a linked `Stable release 1.2.0` marker near the website hero actions so
  visitors can identify the latest published version without confusing it with
  the unpublished `1.2.1` development line.
- Added draft `1.2.1` release notes and a tracked Markdown release-state audit.
- Added a README Tier 1 release-target matrix and a wrapper/platform gap
  analysis, separating automated OS/toolchain gates from exact PLC hardware
  and firmware evidence.
- Extended the blocking C++ header/export/CMake/simulator job to macOS, matching
  the three operating systems already used by the Rust and managed wrappers.
- Rebuilt the C#, Python, and C/C++ onboarding guides around their primary
  industrial use cases; corrected superseded 1.2.0 STRING-write guidance and
  added buildable examples for scalar I/O, batches, program paths, controller
  discovery, ControlLogix routing, diagnostics, and polling.
- Added cross-language decision guidance and runnable UDT examples covering
  single versus batch access, whole-structure reads versus member writes,
  controller/program scope, built-in/custom STRING byte capacity, and CIP
  fragmentation boundaries.
- Expanded the website with a Rust-core capability catalog, a stable-size C++
  quick-start tab, keyboard and remembered-language behavior, Rust-core/wrapper
  architecture rationale, and a plain-language cross-platform explanation.
- Added a responsive, site-native GitHub Sponsors card without weakening the
  no-third-party-frame Content Security Policy, and documented the local-only
  quick-start language preference in the privacy policy.

### Changed

- Python `write_tags()` now sends contiguous native-safe atomic writes through
  the real Multiple Service Packet batch endpoint. STRING/custom STRING, UDT,
  member/bit, packed-BOOL element, and duplicate-name writes retain ordered
  typed fallbacks with per-tag results.
- Updated first-party GitHub Actions to Node.js 24-native majors in CI and
  release workflows, removing reliance on GitHub's Node.js 20 compatibility
  shim.
- Lowered the workspace MSRV from Rust 1.96 to Rust 1.88, the oldest compiler
  supported by the locked dependency set and the complete Rust test suite.
- Updated active guides, examples, historical banners, and wiki synthesis so
  `1.2.0` remains clearly identified as the latest published baseline while
  the repository prepares `1.2.1`.
- Added an explicit development-line mode to release readiness so `main` can
  carry 1.2.1 machine metadata without advertising unpublished registry
  packages; final release checks remain strict.

### Fixed

- Fixed `get_tag_attributes`/`get_udt_definition` failing outright (CIP
  "Path segment error") against every tag shape on a real ControlLogix
  through a routed 1756-EN2T bridge, even though plain reads and bulk tag
  discovery succeeded on the same connection. The per-tag Get Attribute
  List request now falls back to the already-working bulk discovery sweep
  when the direct request fails, restoring `get_udt_definition` and
  `write_tag`'s zero-`symbol_id` UDT-write fallback on affected
  controllers.
- Fixed macOS CMake examples loading a stale Cargo `deps` dylib install name
  instead of the native library copied beside each executable.
- Fixed C# `WriteTagsBatch` classifying a program-scoped scalar path such as
  `Program:MainProgram.Counter` as a UDT member instead of sending it through
  the native batch path.
- Fixed intermittent Linux .NET testhost crashes by staging the Rust shared
  library before testhost starts and never overwriting, manually reloading, or
  unloading an image that the CLR may already be using for P/Invoke.
- Fixed development-line package validation so the unpublished `1.2.1` root
  crate is checked through the dependency-order-aware readiness gate instead
  of requiring its same-version sibling crates to already exist on crates.io.
- Removed stale C# fallback errors and IntelliSense guidance that described
  validated handle-aware STRING and UDT-array-member writes as universally
  unsupported firmware paths.
- Program-scoped tag discovery aborted on CIP 0x06 "partial transfer" instead of
  paging, so a program whose symbol table spans more than one response yielded no
  tags at all.
- Program-scoped tag discovery reported every tag as controller-scoped: the
  Symbol Object reply carries no scope field, so the requested scope is now
  threaded from the caller into every discovered tag. Both accepted spellings of
  a program name (`Dashboard` and `Program:Dashboard`) normalize to the same
  `TagScope::Program("Dashboard")`.
- Upgraded the desktop example to `egui`/`eframe` 0.33.3 and `webbrowser`
  1.2.2, removing the `RUSTSEC-2026-0257` browser argument-injection
  vulnerability from the workspace lockfile while preserving Rust 1.88 support.
- Cached positive and negative packed-BOOL array classification per connection,
  shared the cache across FFI client clones, and invalidate it on route changes
  or explicit cache clearing.
  On the 1756-L75 firmware 33 size-100 DINT workload, build-identical Rust, C#,
  and C/C++ reads improved from 286–303 to about 3,305 tags/second while native
  write throughput remained stable.
- Fixed the Python source-checkout native loader preferring a debug artifact
  over an available release artifact, with a regression test for search order.

## [1.2.0] - 2026-07-08

Minor release: behavioral fixes, deprecations, and additive surface (C/C++ header
and example, custom-string support, CIP fragmentation) with no Rust-API signature
breaks. Hardware-validated on CompactLogix 5069-L330ERM fw38 across Rust/C#/Python/C++
(full-coverage 2304 reads / 2285 writes / 2285 verify / 0 anomalies). The C FFI ABI
is v2 (removal of three unusable `*mut EipClient` exports; `eip_abi_version()` bumped).

### Added
- CODEX-AZ added CIP Read Tag Fragmented (`0x52`) and Write Tag
  Fragmented (`0x53`) support for large string/structure payloads. Custom
  strings that exceed the single-packet ceiling can now be read and written via
  the same string APIs, with simulator coverage for a 600-byte custom string.
- Python `Client.read_string()` now exposes the native string-read path, so
  Python callers can read built-in and custom Logix string types as text.
- CODEX-AU added first-class C/C++ consumer support: checked-in
  `include/rust_ethernet_ip.h`, a simulator-backed CMake C++ smoke example,
  CI header/export parity checks, and a Qt threading guide.
- CODEX-AR added `EipClient::unsubscribe` for live tag subscriptions and
  `TagSubscriptionEvent` / `wait_for_event` / `into_event_stream` so single-tag
  poll errors can be observed without changing the existing value stream API.
- Added a Rust blocked-write-label hardware probe for CODEX-AV. It selects one
  representative per current `firmware_blocked_*` class by default, can sweep
  every blocked manifest tag with `--all-blocked`, verifies read-back and
  sibling integrity, restores original values, and writes JSON evidence for the
  1.2.0 manifest relabel decision.

### Deprecated
- CODEX-AQ deprecated dead 1.x compatibility surfaces that remain exported but
  are not part of the maintained runtime path: `ProductionMonitor`,
  `ProductionConfig`, `SubscriptionManager` / `RealTimeSubscriptionManager`,
  `TagCache`, and `PlcManager`. Use `EipClient` diagnostics and subscriptions,
  direct client configuration, `TagManager`, and `Fleet` instead. The deprecated
  types are retained for SemVer compatibility and queued for removal in 2.0.
- CODEX-AP deprecated legacy STRING and UDT offset APIs that never had a valid
  wire contract: Rust `write_string`, `write_ab_string_components`,
  `write_ab_string_udt`, `write_string_connected`,
  `write_string_unconnected`, `read_udt_member_by_offset`, and
  `write_udt_member_by_offset`, plus the C# offset-member wrappers and matching
  FFI exports. They now return explicit unsupported errors and are queued for
  removal in 2.0. Use `write_tag(..., PlcValue::String(...))`,
  `write_string_tag`, direct member tags, `read_udt_chunked` plus
  `UdtData::parse`, or the service-layer UDT member helpers instead.

### Fixed
- CODEX-AW made batch operation grouping enforce both
  `max_operations_per_packet` and `max_packet_size`, splitting oversized
  Multiple Service Packet batches instead of sending a packet the controller
  rejects with EIP status `0x65`.
- CODEX-AX wires STRINGs into the Rust, C#, and Python full-coverage runners:
  all manifest string targets now write, verify, and settle through string-aware
  reads; the manifest now reflects CODEX-AY's handle-aware string fix at 2304
  total / 2285 writeable / 0 expected-blocked / 19 read-only targets.
- The Python full-coverage runner now uses ASCII status markers so redirected
  stdout on Windows no longer depends on a UTF-8 console code page.
- CODEX-AO Phase 1 made UDT member-map conversion fail closed: truncated UDT
  bytes now error instead of skipping members, missing member values now error
  instead of zero-filling them, template parsing preserves empty member-name
  slots, and packed BOOL members can use Logix template bit metadata.
- CODEX-AR fixed live subscription and fleet lifecycle edges: `stop()` and
  `unsubscribe()` now halt single-tag polling, stopped subscriptions are pruned,
  value/tag-group notifications use drop-oldest backpressure instead of blocking
  poll tasks, single-tag polling emits error events and survives retriable CIP
  connection-failure statuses, `Fleet` forwarding survives broadcast lag and
  aborts old forwarders on replacement, and `Client::events()` is now
  observation-only instead of injecting synthetic `Connected` events.
- CODEX-AQ removed the fabricated TagManager UDT definition request/parser.
  `discover_udt_members` now uses the real template-definition path, and tag
  discovery accepts legal Logix names such as `_Tag`, `Program:Main.Tag`, and
  `Arr[3]`, recognizes `0x02A0`-class structure type words, reports malformed
  tag-list pages instead of byte-pattern resyncing, and no longer invents zero
  array dimensions.
- CODEX-AQ made diagnostics operation/error counters real per-client atomics on
  the CIP send path. CPU/memory/system metrics remain explicitly marked as
  placeholders (`MonitoringMetrics::system_metrics_are_placeholders()`), and
  deprecated `ProductionMonitor::start_monitoring` no longer spawns an
  unstoppable placeholder task.
- Enabled Tokio's `signal` feature so the checked-in `python_test_simulator`
  example builds from a clean target directory when C# simulator-backed tests
  need to spawn it.
- CODEX-AP retired the invalid UDT "advanced chunked" strategy ladder behind
  `read_udt_chunked`; the compatibility method now delegates to the maintained
  `read_tag` UDT path instead of fabricating empty UDT payloads. CIP
  additional-status parsing now keys off the additional-status word count and
  reports `0x2107` as the little-endian Read/Write Tag data-type mismatch.
- CODEX-AS hardened the FFI boundary: the three raw `*mut EipClient` entry
  points (`eip_get_udt_definition`, `eip_get_tag_attributes`,
  `eip_discover_tags_detailed`) are no longer exported into the C ABI symbol
  table in favor of the handle-based `_by_id` exports, so `eip_abi_version()` is
  now 2. The functions remain in the crate's Rust API as non-exported
  `pub unsafe extern "C" fn`s, so there is no Rust-source breaking change and
  the crate stays on the 1.2.0 minor line; the C ABI is versioned independently
  via `ABI_VERSION`. Panics under runtime-dispatched FFI calls become `-1` plus
  last-error instead of unwinding into host processes, last-error entries clear
  on success and disconnect, and Python clients now finalize/disconnect native
  handles without wedging local state on native disconnect failure.
- CODEX-AL hardened transport/session state: SendRRData now uses checked
  per-request sender contexts, incomplete transactions poison the shared stream
  until reconnect, session handles are shared across cloned clients, and FFI
  health/diagnostics calls no longer reinsert cloned clients after disconnect.
- CODEX-AN corrected `read_array_range` chunk parsing to consume Logix Read
  Tag replies as `[type][data...]` with no synthetic element-count word,
  rebuilt `get_tag_attributes` on the CIP request/response codec, and made the
  simulator serve spec-shaped Read Tag and Get Attribute List replies.
- CODEX-AV relabeled the full-coverage manifest from the 2026-07-03 hardware
  matrix: 60 scalar UDT-array-element-member targets became writeable and the
  missing program-scope `Member5_String` array-member entry was added. CODEX-AY
  and CODEX-AX subsequently moved the STRING-member labels to writeable, so the
  current runner expectation is 2304 total / 2285 writeable / 0
  expected-blocked / 19 read-only targets.
- Service-layer UDT member writes now try direct scalar member writes first and
  fall back to whole-UDT read-modify-write only on the `0x2107` data-type
  mismatch shape; STRING members keep the read-modify-write path
  unconditionally.
- Tag addressing now preserves member suffixes on writes such as
  `Array[i].Member`, routes `Tag.n` bit syntax through client-side bit
  read-modify-write, addresses batch BOOL array elements by containing DWORD,
  emits well-formed `.DATA[i]` element segments, and includes the requested
  program scope in program tag discovery requests.
- Standard Logix `STRING` writes now use the hardware-validated structure
  encoding (`0x02A0` + `0x0FCE` handle, 88-byte payload), batch STRING writes
  use the same path, and standard STRING read replies decode from the
  structure-shaped hardware payload back to `PlcValue::String`.
- C# `WriteUdtMember` no longer self-deadlocks by recursively acquiring the
  wrapper operation lock during its read-modify-write path.
- C# native marshalling now uses UTF-8 for tag names, JSON payloads, STRING
  values, native result strings, and runtime metadata instead of ANSI code-page
  conversion.
- C# keep-alive health checks now serialize with user operations on the native
  handle and skip a tick if the operation lock is busy.
- C# `Connect`/`ConnectWithRoute` now populate `LastConnectError` with the
  native failure code, including the runtime-initialization failure case.
- The standalone simulator now supports the C# native integration-test path,
  including raw UDT reads and Multiple Service Packet batch requests.
- CI now checks public API compatibility against the published `1.1.0`
  baseline instead of the obsolete `0.7.0` baseline.
- The crates.io release workflow now tolerates only already-published crate
  versions; other `cargo publish` failures fail the job.
- Local `dotnet pack` output for the C# wrapper now places the native library
  under `runtimes/<rid>/native/` instead of package-root content.
- NuGet runtime documentation now matches the shipped RID set:
  `win-x64`, `linux-x64`, and `osx-arm64`.

### Changed
- UDT array element member write documentation now reflects the CODEX-AM
  hardware finding: at least one DINT member write succeeds with a correct path
  on the 5069-L330ERM fw38, so the remaining `firmware_blocked_*` labels are
  under CODEX-AV revalidation rather than treated as a blanket firmware rule.
- C# CI now runs a simulator-backed P/Invoke integration test project on the
  stable toolchain legs, covering connect/disconnect, scalar and UTF-8 STRING
  round trips, native batch read/write, `WriteUdtMember` watchdog completion,
  and keep-alive contention.
- Cargo metadata no longer publishes the placeholder author email, matching the
  sibling crates' manifest convention.
- The FFI-only `libc` dependency is optional behind the `ffi` feature, and
  `vergen` is no longer a runtime dependency.
- The `futures` dependency no longer pulls its default features (landed earlier
  as PR #24; recorded here for release visibility).
- `cargo audit` runs weekly in CI, and the release-readiness shell test now runs
  in CI.

### Removed
- Deleted stale, uncompiled scratch test files under `tests/unit/` and
  `test_isolated/`.

## [1.1.0] - 2026-06-19

Post-1.0.0 review pass: correctness fixes across all three language bindings,
tech-debt cleanup, and additive feature work. No breaking changes to the public
Rust API or the C ABI — the ABI version remains `1` and a new capability bit
`CAP_LAST_ERROR` is advertised. MSRV remains 1.96.

### Fixed
- Bit access (`MyDINT.15`, `read_bit`/`write_bit`) now resolves the bit entirely
  client-side (mask on read, read-modify-write on write); the previous wire
  encoding emitted a malformed logical-member segment. Hardware-validated.
- C#: typed UDT writes (`WriteUdt`/`WriteUdtData`) serialize `data` as a byte
  array matching the native `Vec<u8>`, so they no longer fail silently.
- C#: a failed scalar write no longer re-issues the write to the PLC to harvest
  an error message, which previously double-applied side-effecting writes.
- C#: added a finalizer and a thread-safe `Dispose`, so a forgotten `Dispose()`
  no longer leaks the native session and teardown no longer races keep-alive.
- `RoutePath::add_port` / `RoutePath.AddPort` before an address no longer
  silently drops the port (Rust and C#).
- Python: a generic UDT is no longer mis-decoded as a STRING (the headerless
  decode now requires zero padding past the length and valid UTF-8).
- Subscription `change_threshold` is documented as the absolute deadband it
  implements (not a percentage).
- `TagManager` tag-list parsing reads the item count at the correct offset when
  additional-status words are present.
- `build.rs` / `version.rs` tolerate building without a git checkout (a
  crates.io tarball) instead of failing.

### Added
- FFI: `eip_get_last_error(client_id, buffer, max_len)` (capability
  `CAP_LAST_ERROR`) exposes the underlying failure reason. C# surfaces it via a
  new `PlcException` (`NativeError` property); Python appends it to
  `PlcOperationError` (e.g. "CIP Error 0x04: Path segment error").
- C#: asynchronous API — `ReadDintAsync` / `WriteBoolAsync` / `ReadStringAsync`
  / batch / `CheckHealthAsync` etc. (Task.Run wrappers) so callers can `await`
  operations and keep UI threads responsive.
- Python: the wheel now bundles the native library and is platform-tagged, so a
  plain `pip install` yields a working package.
- Docs: `docs/API_STABILITY.md` (SemVer/MSRV/ABI policy) and
  `docs/MIGRATION_0.7_to_1.0.md`.

### Changed
- Dependencies: `thiserror` 2.0; removed unused `log` and `env_logger`; `tokio`
  trimmed from the `full` feature to the set actually used (`rt-multi-thread`,
  `macros`, `net`, `time`, `sync`, `io-util`).
- Internal: the 22 scalar FFI wrappers are macro-generated (ABI-identical); the
  standalone `enip-test` harness and vestigial dead code were removed from the
  published crate; the unrunnable `examples/rust_examples/` tree was deleted.
- Release packaging now builds multi-RID NuGet and per-platform PyPI wheels and
  publishes the five crates to crates.io in dependency order.

## [1.0.0] - 2026-05-24

> Release-window cut. Bundles every deferred SemVer-major change (CODEX-K) with the actor refactor (CODEX-P), event stream (CODEX-R), service layer (CODEX-Q), retry primitive (CODEX-S), FFI clone-semantics fix (CODEX-M Phase B), `client.rs` mechanical split (CODEX-J), fleet pool (CODEX-T), and sibling-crate workspace structure (CODEX-U). The four sibling crates (`rust-ethernet-ip-{types,protocol,tag-path,udt}`) are publishable as independent crates.io artifacts; release-day publish order is `types` + `tag-path`, then `protocol` + `udt`, then `rust-ethernet-ip`.

### ✨ Added
- **Ordered route hops with explicit `RouteHop` variants**: `RoutePath` now stores ordered private `RouteHop` values (`Backplane { slot }` / `Ethernet { port, address }`) with builder methods `add_backplane`, `add_ethernet`, and `add_ethernet_with_port`. The new `RouteHop::Ethernet` variant emits the spec-correct ASCII extended link-address encoding (`[0x10 | port, ascii_len + 1, ascii…, 0x00, optional_pad]`) instead of raw IPv4 octets. Legacy Rust public grouped fields were removed in the release-window cleanup; legacy grouped FFI calls remain as compatibility shims.
- **Windows-first NuGet release workflow**: Added a GitHub Actions release workflow that builds the Rust native library on `windows-latest`, packs the C# wrapper, uploads the `.nupkg` artifact, and publishes to NuGet on version tags when `NUGET_API_KEY` is configured.
- **NuGet packing scripts**: Added `scripts/pack-nuget.ps1` for Windows `win-x64` native runtime packaging and `scripts/pack-nuget.sh` for local macOS package staging.
- **Maintainer wiki**: Added `AGENTS.md` workflow instructions plus a `wiki/` synthesis layer for controller behavior, route-path behavior, wrapper parity, limitations, release validation, and investigation notes.
- **MacBook manufacturing dashboard demo**: Expanded the web dashboard demo with a richer backend/frontend implementation, frontend lockfile, persistent backend data ignore rules, and maintainer strategy notes.
- **Python wrapper MVP**: Added an in-repo Python wrapper, unit tests, simulator-backed integration support, diagnostics accessors, and data/service examples.
- **Schema export**: Added Rust-side schema export structs plus `export_schema()` / `export_schema_json()` for stable tag and UDT metadata export.
- **Diagnostics snapshot surfaces**: Added Rust diagnostics snapshot and error categorization, FFI JSON export, and thin C# / Python wrapper accessors.
- **FFI ABI handshake**: Added ABI version, library version, and capability bitmap exports plus C# and Python wrapper load-time compatibility checks.
- **CI SemVer gate**: Added a `cargo-semver-checks` GitHub Actions job against the crates.io `0.7.0` baseline, required on `main` and informational on pull requests.
- **Actor-backed client handle**: Added a cloneable `Client` handle that serializes read/write/batch requests through a worker task owning the underlying `EipClient`.
- **Connection events**: Added `ConnectionEvent` and `Client::events()` for actor-backed connection lifecycle notifications.
- **Restricted-write service helpers**: Added actor-client helpers for Logix STRING and UDT-member write flows that internally perform the documented read-modify-write workaround.
- **RetryPolicy primitive**: Added constant/exponential retry policy support through `Client::with_retry(...)`, using the existing retriable-error classification and opt-in write retries.
- **Fleet multi-PLC pool**: Added `Fleet<PlcId>` and `FleetEvent` as an actor-client based pool for per-PLC handles, fleet health checks, and fleet-level connection events.
- **Collector, MQTT, and Docker examples**: Added Python collector, MQTT publisher, FastAPI service, and Docker-based example stacks for local service packaging.

### 🐛 Fixed
- **FFI runtime hardening**: Gated the Rust FFI module behind the existing `ffi` Cargo feature, required FFI-producing build paths to enable it, and converted Tokio runtime initialization failure into a documented native return code instead of a process abort.
- **NuGet publish package lookup**: Hardened the Windows release workflow so NuGet publishing resolves the generated package path explicitly instead of relying on wildcard expansion in `dotnet nuget push`.
- **C# package metadata/runtime packaging**: Adjusted the wrapper project to package as a library, align the NuGet license metadata with the repository MIT license, include the package README, and mark native runtime libraries as package content.
- **C# validation examples on macOS**: Fixed wrapper smoke/benchmark/matrix example projects so they copy `librust_ethernet_ip.dylib` on macOS and `rust_ethernet_ip.dll` on Windows using `MSBuild::IsOSPlatform(...)`.
- **Python routed ControlLogix support**: Fixed the Python wrapper ControlLogix route-path path so routed connections and routed live validation work on `1756-L81ES` via `1756-EN3TR` slot `0`.
- **Python live write result handling**: Fixed the Python wrapper so routed live `write_tag()` and the exercised `write_tags()` paths no longer misreport successful ControlLogix writes as failures for the validated `DINT` and `REAL` cases.
- **Python typed single-tag writes**: Fixed the Python wrapper so `write_tag()` uses typed single-tag FFI exports instead of the batch path for scalar writes, addressing CIP `0x1E` failures on plain `BOOL[]` element writes found during ControlLogix hardware validation.
- **FFI clone-state consistency**: Made route-path and max-packet-size client state shared across cloned FFI registry lookups, and implemented `eip_set_max_packet_size` against the shared state instead of returning a no-op success.
- **BOOL array DWORD addressing**: Fixed BOOL array element read/write so indices `>= 32` address the correct packed DWORD instead of aliasing every bit operation to DWORD `[0]`.
- **Nested BOOL array members**: Applied the BOOL array workaround to nested BOOL array members inside UDT array elements, fixing DWORD-as-`UDINT` reads and CIP `0x05` failures for paths such as `gTestUDT_Array[3].Array_BOOL[5]`.
- **CIP path validation**: Hardened `CipRequest` encoding so empty, odd-length, or over-510-byte paths fail before encoding instead of silently truncating or overflowing the path word count.
- **Rust 1.95 Clippy cleanup**: Removed new Clippy warnings in FFI write helpers and packed BOOL-array decoding without changing runtime behavior.
- **crates.io README logo rendering**: Switched README logo image URLs from relative paths to absolute GitHub raw URLs so the crate page can render the logo outside the GitHub repository context.
- **Rust 2024 migration without wrapper breakage**: Moved the repo to Rust `2024` / `1.95`, updated FFI exports to Rust 2024 `#[unsafe(no_mangle)]`, refreshed dependency baselines, and verified that Rust tests plus C# wrapper build/tests still pass against the updated native library.

### 📚 Documentation
- **Release-prep version reminder**: Updated the literal `src/lib.rs` head-doc release-line references to `1.0.0`; future release prep should keep those literal strings in sync with `Cargo.toml`.
- **Main README NuGet guidance**: Documented the published `RustEtherNetIp` NuGet package, CLI install command, `.NET 10` target, and current Windows `win-x64` native runtime focus.
- **Release process docs**: Updated version-management guidance to use the Windows-first NuGet pack/publish flow.
- **Official Rockwell source check**: Rechecked official Rockwell EtherNet/IP/data-access publications on 2026-04-16. The repository already tracks the current `1756-PM020I-EN-P` September 2025 data-access manual; `ENET-UM006C-EN-P` September 2025 was added to the traceability matrix as a relevant network-device reference for EtherNet/IP connection/message behavior.
- **Wiki documentation**: Added maintainer-oriented pages for LLM-maintained repository knowledge, including how synthesis differs from user-facing docs.
- **Toolchain baseline docs**: Updated build/readme/wiki references to the current Rust `1.95` / Rust 2024 baseline and current `.NET 10` wrapper outputs.

### 🧹 Cleanup
- **Wire codec boundary**: Added crate-private protocol codec modules for EtherNet/IP encapsulation, CIP framing, and PLC value encode/decode paths with round-trip coverage.
- **Library module decomposition**: Split the crate-root implementation into focused `route`, `batch`, `types`, and `client` modules while preserving crate-root public re-exports.
- **Small Rust polish**: Deduplicated FFI runtime-initialization failure logging, cached tag-name validation regex compilation, merged subscription re-exports, and removed the unused `cargo-tarpaulin` dev-dependency.
- **Contained Rust API cleanup**: Derived `thiserror::Error` for `BatchError`, removed unused `EipClient` fields and the unused direct `async-trait` dependency, and added selected `#[must_use]` annotations for builder/getter APIs.
- **Consolidated subscription re-exports**: Removed the one-line `tag_subscription` module shim; crate-root `RealTimeSubscription*` aliases now re-export directly from `subscription`.
- **Repository ignore rules**: Added Obsidian workspace ignore coverage.
- **Rust formatting pass**: Applied `cargo fmt` across the Rust tree to clear the pre-release formatter drift before tagging `1.0.0`.
- **Client module split**: Moved batch execution, diagnostics, schema export, STRING handling, and subscription/tag-group APIs out of the monolithic `client.rs` while preserving the `EipClient` facade.
- **Release-window API cleanup**: Added `#[non_exhaustive]` to public enums, changed tracing/config file helpers to return the crate error type, replaced stringly logging config fields with enums, and demoted connected-session wire state from crate-root public exports.
- **RoutePath and wrapper route API cleanup**: Migrated Rust `RoutePath` to private ordered-hop storage, added ordered-hop FFI route functions, and updated C# / Python wrappers to preserve route-hop ordering explicitly.
- **Sibling workspace crates**: Promoted shared PLC value types, EtherNet/IP protocol codecs, Logix tag-path parsing, and UDT helpers into `rust-ethernet-ip-types`, `rust-ethernet-ip-protocol`, `rust-ethernet-ip-tag-path`, and `rust-ethernet-ip-udt` while preserving main-crate re-exports/wrappers.

### ✅ Verification
- `cargo clippy --lib -p rust-ethernet-ip --` passes on Rust `1.96.0`.
- `cargo clippy --all-targets -- -D warnings` passes on Rust `1.96.0`.
- `cargo test --workspace --all-targets` passes when run with local TCP listener permissions for simulator-backed tests.
- `cargo build --release` passes on Rust `1.96.0`.
- `PYTHONPATH=python python3 -m unittest discover -s python/tests` passes.
- `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -v minimal` passes: 42/42 tests.
- Real ControlLogix validation on 2026-04-16 passed for the exercised Rust and C# wrapper feature sets on `1756-L81ES` via `1756-EN3TR` slot `0`; the remaining 59/392 matrix failures match documented PLC firmware limitations.

## [0.7.0] - 2026-04-07

### ✅ 0.7.0 Release Checklist
- [x] Preserve full commit-by-commit history on `main` for traceability.
- [x] Keep `0.6.3` as last stable published version while `0.7.0` remains unreleased.
- [x] Add unreleased documentation notes in README and CHANGELOG.
- [x] Run full cross-language regression matrix (Rust + C#) after each high-impact change.
- [x] Add/expand simulator-based failure-mode tests (timeouts, reconnect, partial batch failures).
- [x] Add explicit FFI contract tests for mixed batch operations, including malformed payload handling.
- [x] Complete FFI batch config APIs (`eip_configure_batch_operations`, `eip_get_batch_config`) or clearly gate them as unsupported across wrappers.
- [x] Add performance baseline report (single read/write, batch read/write, mixed execute) and compare against `0.6.3`.
- [x] Add compatibility test pass for route-path scenarios and UDT-heavy workloads.
- [x] Perform docs/API audit to ensure examples and behavior match implemented semantics.
- [x] Complete real-PLC validation on CompactLogix and ControlLogix hardware.
- [x] Freeze release candidate, finalize release metadata, and publish `0.7.0`.

### 🐛 Fixed — Core Library
- **Connection pool reuse**: `PlcManager::get_connection` now reuses the least-recently-used active client when the pool is full instead of recreating a new TCP/session connection each time.
- **Unconnected-send interoperability**: Added `0xD2` reply unwrapping and direct-CIP fallback retry (when no route path is configured) to improve discovery/read interoperability across mixed PLC/simulator behaviors.
- **UDT fail-fast contract**: Unimplemented UDT paths (`UserDefinedType::from_cip_data`, `UdtManager::serialize_udt_instance`) now return explicit protocol errors instead of silent empty/placeholder success values.
- **CompactLogix BOOL batch decoding**: Native Multiple Service batch reads now decode packed `0x00D3` BOOL-array responses correctly on CompactLogix-class controllers, eliminating the previous wrapper fallback for mixed BOOL batches.
- **Batch STRING write diagnostics**: Batch-level `0x1E` failures on direct STRING writes now produce a CompactLogix/ControlLogix-specific firmware-limitation explanation instead of only raw protocol text.

### ✨ Added — Tag Group Polling (Rust/C# Parity)
- **Rust tag-group API**: Added `upsert_tag_group`, `remove_tag_group`, `list_tag_groups`, `read_tag_group_once`, and `subscribe_tag_group`.
- **Rust event classification**: Added `TagGroupEventKind` (`Data`, `PartialError`, `ReadFailure`) plus `TagGroupFailureDiagnostic` (`category`, `retriable`, `status_code`) for structured failure handling.
- **C# wrapper parity APIs**: Added `UpsertTagGroup`, `RemoveTagGroup`, `ListTagGroups`, `ReadTagGroupOnce`, and `SubscribeToTagGroup`.
- **C# event diagnostics parity**: Added `TagGroup.PollingEvent`, `TagGroupEventKind`, and `TagGroupFailureDiagnostic` with categorized `ReadFailure` payloads.
- **C# interface parity**: `IEtherNetIpClient` now includes tag-group APIs and matches nullable `GetUdtMember` return semantics.

### 🐛 Fixed — FFI
- **Native batch execution**: Implemented `eip_write_tags_batch` and `eip_execute_batch` with typed JSON payload parsing and structured per-operation results.
- **Batch config API status code**: `eip_configure_batch_operations` now returns failure (`-1`) while unimplemented.
- **Response buffer safety cleanup**: Centralized FFI output buffer write handling for batch responses to reduce duplicated unsafe boundary code.

### 🐛 Fixed — C# Wrapper
- **Native batch wiring**: `WriteTagsBatch` and `ExecuteBatch` now use native FFI batch paths for non-UDT-member operations with per-item result mapping.
- **Native batch reads**: `ReadTagsBatch` now uses the native batch-read FFI path instead of wrapper-only sequential type probing for validated CompactLogix workloads.
- **Batch config API behavior**: `ConfigureBatchOperations` and `GetBatchConfig` now throw `NotSupportedException` instead of silently behaving as if configuration succeeded.
- **UDT batch payload compatibility**: UDT raw bytes now serialize as numeric JSON arrays for Rust `Vec<u8>` compatibility (with regression test).
- **UDT conversion contract hardening**: `UdtData.ToDictionary(UdtTemplate)` now performs template-driven parsing and throws explicit `InvalidOperationException` on parse errors, avoiding silent empty dictionary results.
- **Known firmware-limit diagnostics**: Invalid subscription handling now fails fast, direct UDT-array-member writes surface the `0x2107` firmware limitation more clearly, and direct STRING write failures preserve the observed native `0x1E` form on CompactLogix.

### 🧹 Cleanup
- **Desktop app warnings**: Removed unused fields/locals in `examples/desktop_app` that were generating compile warnings.
- **Repository hygiene**: Added recursive `**/target/` ignore rule and untracked committed `examples/web_app/backend/target` build artifacts.

### ✅ Test Hardening
- **Simulator failure-mode coverage expanded**: Added deterministic simulator tests for timeout behavior, transport disconnect with reconnect workflow, and partial batch failure isolation for mixed batch paths.
- **FFI batch contract coverage expanded**: Added explicit tests for mixed execute/write batch contracts, malformed JSON payload rejection, count mismatch rejection, and per-item parse error isolation for batch payload validation.
- **Wrapper batch-config contract tests**: Added C# unit tests to enforce `NotSupportedException` behavior for `ConfigureBatchOperations` and `GetBatchConfig`.
- **Performance baseline report generated**: Added simulator-based baseline harness and produced `HEAD` vs `v0.6.3` comparison report for single read/write, batch read/write, and mixed execute scenarios with raw JSON artifacts.
- **Route-path + UDT compatibility pass**: Added deterministic simulator route-path compatibility tests and generated a consolidated Rust/C# compatibility matrix report for route-path and UDT-heavy workloads.
- **Docs/API audit completed**: Updated stale release-state/version messaging, aligned wrapper/API docs with current batch-config unsupported semantics, and published an audit artifact capturing findings and remediations.
- **Cross-language regression matrix (2026-03-20) passed**: `cargo test --workspace --all-targets` and `dotnet test csharp/RustEtherNetIp.Tests/RustEtherNetIp.Tests.csproj -v minimal` (26/26 passed).
- **Rust tag-group unit coverage**: Added tests for event classification (`Data`/`PartialError`), `ReadFailure` diagnostic payload preservation, and subscription lifecycle behavior.
- **C# tag-group API coverage**: Added tests for registration/list/remove semantics, idempotent subscribe behavior, and one-shot snapshot behavior when disconnected.
- **C# diagnostic classification coverage**: Added tests verifying exception-to-category mapping (`Timeout`, `Network`, `Data`).
- **Simulator integration coverage (C#)**: Added tag-group tests validating `PartialError` (mixed valid+invalid tags) and `ReadFailure` diagnostics after disconnect.
- **UDT contract tests**: Added Rust/C# contract tests to enforce explicit not-implemented behavior for currently unsupported UDT conversion/parsing paths.
- **Real PLC validation completed**: Added hardware-backed Rust and C# validation records for CompactLogix `5069-L320ERMS3` firmware `35` and ControlLogix `1756-L81ES` firmware `37` via `1756-EN3TR` slot `0`, including read/write coverage and benchmark baselines.

### 📚 Documentation
- **README updates**: Added Rust/C# tag-group event handling section with `Data`, `PartialError`, and `ReadFailure` consumption patterns.
- **Programmer manual updates**: Added a dedicated Rust/C# event-handling subsection and updated API catalog notes for diagnostics/events.
- **C# wrapper README updates**: Documented `TagGroup.PollingEvent`, event kinds, and structured failure diagnostics.
- **Compatibility docs**: Added 0.7.0 PLC/simulator compatibility matrix.
- **Release evidence docs**: Added real-PLC validation records and a reusable `docs/validation/REAL_PLC_TESTING.md` guide for future hardware sign-off runs.

### 🧪 Examples / Demos
- **WPF and WinForms demo hardening**: Updated tag-group demo flows to consume `PollingEvent`, surface partial/read-failure states in UI status/logs, and keep cleanup/unsubscription deterministic.

### 🐛 Fixed — C# Wrapper
- **Empty STRING tag handling**: Fixed `ReadString` to return empty string for zeroed/cleared STRING tags (LEN=0) instead of falling through to error handling. Both the direct read path and the UDT member fallback path now correctly handle empty strings.

## [0.6.3] - 2026-03-01

### 🐛 Fixed — Critical
- **Missing CIP type handlers**: Added LINT, USINT, UINT, UDINT, ULINT, and LREAL to `parse_cip_response` — these types were silently falling through to unknown-type error
- **CIP response bounds check**: Fixed `parse_cip_response` guard from `< 2` to `< 4`, preventing index-out-of-bounds panic
- **UDT STRING parsing**: Fixed `parse_member_value` to use 4-byte DINT length (was incorrectly using 2-byte), matching Allen-Bradley STRING format; updated `get_data_type_size` from 84 to 88

### 🐛 Fixed — High
- **Packet negotiation**: Rewrote `negotiate_packet_size` with correct CIP Get Attribute List format (proper path-size byte, 8-bit logical segments, correct attribute ID)
- **Keep-alive packet**: Replaced malformed 4-byte SendRRData stub with proper 24-byte EtherNet/IP NOP command
- **Unregister session**: Fixed packet length field (was 4, should be 0) and removed extraneous protocol-version payload
- **PlcValue::String type code**: Fixed `get_data_type()` returning 0x02A0 (structure handle) instead of correct 0x00CE (STRING)
- **TagPath "LEN" greedy match**: Fixed parser to check "LEN" is a complete segment (not prefix of e.g. "LENGTH")
- **FFI client ID overflow**: Changed `next_id += 1` to `wrapping_add(1)` with reset to 1 when negative, preventing i32 overflow

### 🐛 Fixed — Medium
- **Subscription threshold for all types**: Extended change detection to LREAL (deadband), all integer types, BOOL, and STRING (equality); previously only REAL was checked
- **UDT member bounds checks**: Added empty-data guards for BOOL, SINT, and USINT in `parse_member_value`

### 🐛 Fixed — C# Wrapper
- **WriteUdtMember phantom keys**: Removed injection of `_last_modified` and `_modified_member` keys that corrupted UDT data
- **IEtherNetIpClient.ReadTagsBatch**: Fixed return type from `Dictionary<string, TagReadResult>` to `Dictionary<string, TagReadResultBatch>` to match implementation
- **WriteTag missing types**: Added Sint, Lint, Usint, Uint, Udint, Ulint, and Lreal cases — these types previously threw at runtime
- **Keep-alive reconnect**: Fixed reconnect to preserve route path via `_currentRoutePath` field instead of falling back to direct connection
- **Debug output cleanup**: Removed 20+ `Console.WriteLine` debug statements from library code

### ✨ Added
- **PLC Simulator for testing without hardware**
  - New `plc_sim` binary and in-process test simulator
  - Expanded simulator-backed Rust and C# test coverage
- **Broader automated test coverage**
  - FFI safety checks, concurrency tests, bounds parsing, network failure tests
- **Tag introspection**: `get_tag_attributes` for discovering tag type, size, and scope
- **Subscriptions API**: Real-time tag monitoring with `subscribe_tag` / `unsubscribe_tag`
- **Bit-level API**: Read/write individual bits within DINT tags
- **Structured error types**: Rich `EtherNetIpError` enum replacing string errors

### 📚 Documentation
- **Tag introspection guide**: `docs/tag_introspection.md`

## [0.6.2] - 2026-01-24

### ✨ Added
- **Stream Injection API**: New `connect_with_stream()` method for custom TCP transport
  - Enables wrapping streams for metrics/observability (bytes in/out)
  - Supports custom socket options (keepalive, timeouts, bind local address)
  - Allows reusing pre-established tunnels/connections
  - Supports in-memory streams for deterministic testing
  - New `EtherNetIpStream` trait for stream type requirements
- **Test Configuration**: Environment variable support for PLC testing
  - `TEST_PLC_ADDRESS` - Set PLC IP address for tests (default: `192.168.0.1:44818`)
  - `TEST_PLC_SLOT` - Set CPU slot number (default: `0`)
  - `SKIP_PLC_TESTS` - Skip all PLC-dependent tests when set
  - Comprehensive test helper functions in `tests/test_helpers.rs`
  - Documentation in `tests/README.md` and `tests/TEST_CONFIG.md`

### 🐛 Fixed
- **Nested UDT Member Access**: Fixed reading nested UDT members from array elements
  - Correctly handles complex paths like `Cell_NestData[90].PartData.Member`
  - Fixed array element detection to use `TagPath::parse()` for paths with member access
  - Changed from `rfind(']')` to `find('[')` + `find(']')` to use first bracket pair
  - Now correctly builds full CIP paths instead of incorrectly using array workaround
  - Fixes issue where PLC returned entire UDT instead of specific member value

### 📚 Documentation
- **Stream Injection**: Added comprehensive documentation and example for `connect_with_stream()`
- **Test Configuration**: Added detailed guides for configuring tests with environment variables
- **Updated Examples**: Added `stream_injection_example.rs` demonstrating custom stream usage
- **CHANGELOG**: Updated with v0.6.2 changes

## [0.6.1] - 2026-01-17

### 🧹 Removed
- **Go Wrapper**: Removed `gowrapper/` directory to focus on Rust library and C# integration
- **Python Wrapper**: Removed `pywrapper/` directory to focus on Rust library and C# integration
- **Go Examples**: Removed `GoWrapperTest` and `gonextjs` examples
- **Python Examples**: Removed `PythonWrapperTest` and `PLC_Monitor_Dashboard` examples
- **TypeScript/Vue Examples**: Removed `TypeScriptExample`, `VueExample` to streamline examples

### ✨ Changed
- **Repository Focus**: Streamlined to focus on Rust library, Rust native examples, C# wrapper, and C# examples (WinForms, WPF, ASP.NET)
- **Documentation**: Updated all documentation to reflect current focus on Microsoft stack
- **Cargo.toml**: Removed Python dependencies and workspace members

### 📚 Documentation
- **README.md**: Updated to remove Go/Python references and focus on Microsoft stack
- **Version References**: Updated all version references from 0.6.0 to 0.6.1

## [0.6.0] - 2025-01-XX

### ✨ Added - C# Wrapper Enhancements
- **Batch Operations**: `ReadTagsBatch()` and `WriteTagsBatch()` for high-performance multi-tag operations
- **TagGroup**: Periodic polling with event-driven updates (`TagGroup` class with `DataChanged` event)
- **Performance Statistics**: `ClientStatistics` class tracking read/write counts, errors, and average response times
- **Data Quality & Timestamp**: `TagReadResult` with `Quality`, `TimeStamp`, and detailed error information
- **Value Scaling**: `ValueScaling` utility class with `ScaleLinear()` and `ScaleSquareRoot()` methods
- **Enhanced Error Handling**: Detailed error messages with quality indicators and timestamps

### 🔧 Fixed - Connection & RoutePath
- **WinForms Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **WPF Application**: Fixed connection to use `ConnectWithRoute()` when RoutePath is enabled
- **ASP.NET Application**: Updated `PlcService.Connect()` to accept and use RoutePath parameters
- **Connection Verification**: Added automatic connection tests after successful connection
- **Error Handling**: Improved error messages and exception handling across all example applications

### 🐛 Fixed - Type System
- **TagReadResult Duplicate**: Renamed internal `TagReadResult` to `TagReadResultBatch` to resolve naming conflicts
- **Nullability Warnings**: Fixed nullable reference type warnings in `PlcValue`, `TagSubscription`, `UdtData`, and `EthernetNetIpClient`
- **DLL Deployment**: Fixed DLL path in `RustEtherNetIp.csproj` to ensure `rust_ethernet_ip.dll` is correctly copied

### 📚 Documentation
- **Known Limitations**: Added comprehensive documentation for STRING and UDT array write limitations
- **AB_String_UDT_Write_Limitations.md**: Detailed technical document explaining PLC firmware restrictions
- **Updated Examples**: All example applications (WinForms, WPF, ASP.NET) updated with new features and proper error handling

### 🎯 Current Development Focus
- **.NET Stack**: Actively polishing C# wrapper and example applications to production quality

## [0.5.3] - 2025-01-15

### Fixed
- **Tag Discovery**: Fixed `discover_tags()` function to properly discover and parse tag lists from PLCs
- **Program Tag Reading**: Fixed reading of program tags like `Program:ProgramName.TagName` that were failing with "Path segment error"
- **CIP Request Format**: Updated tag list requests to use correct `GET_INSTANCE_ATTRIBUTE_LIST` service
- **Response Parsing**: Fixed tag list response parsing to handle proper attribute list format
- **Tag Path Building**: Improved tag path building to correctly handle program prefixes

### Technical Details
- Updated CIP request building to match working Node.js implementation
- Fixed response parsing format from `[name_len][name][type]` to `[InstanceID(4)][NameLength(2)][Name][Type(2)]`
- Added proper program tag path splitting and segment building
- Enhanced error handling and debugging output for tag operations

## [0.5.2] - 2025-01-15

### 🔧 Code Quality & Documentation Improvements
- **Enhanced FFI safety documentation**: Added comprehensive `# Safety` sections to all unsafe functions
- **Clippy optimizations**: Fixed needless range loops, vec initialization patterns, and pointer arithmetic
- **PyO3 integration**: Resolved non-local impl definition warnings with proper allow attributes
- **Memory safety**: Enhanced pointer validation and buffer overflow protection
- **Build system**: Added criterion dependency for benchmarks and improved build scripts
- **Code formatting**: Consistent formatting across all files with proper doc comment structure
- **Test infrastructure**: All 47 tests pass with enhanced coverage and reliability

### 🛠️ Development Experience
- **Benchmark compatibility**: Fixed criterion version compatibility issues
- **Error handling**: Improved error handling in FFI layer and connection management
- **Documentation**: Enhanced API documentation with better examples and safety guidelines
- **Wrapper updates**: Synchronized all wrapper versions (Python, C#, JavaScript/TypeScript, Go)

## [0.5.1] - 2025-01-15

### ⚡ Performance Improvements
- **Memory allocation optimizations**: 20-30% reduction in allocation overhead for network operations
- **Vec::with_capacity() implementation**: Pre-allocated buffers for CIP requests and packet building
- **Code quality enhancements**: Fixed clippy lints with more idiomatic Rust patterns
- **Network efficiency**: Optimized packet building with reduced memory fragmentation
- **Throughput improvements**: 20% increase in single tag operations (2,500+ → 3,000+ ops/sec)
- **Memory usage reduction**: 20% reduction in memory footprint per operation

## [0.5.0] - 2025-01-15

### 🎯 Production-Ready Release
- **Professional HMI/SCADA Demo** with real-time production monitoring
- **Production Monitoring System** with comprehensive metrics and health checks
- **Configuration Management** for production deployment
- **Production API Endpoints** for system management and monitoring
- **Performance Benchmarking Framework** for optimization and testing
- **Enhanced Real-time Monitoring** with stable continuous updates

### ✨ Added - Professional HMI/SCADA Demo
- **Real-time Production Dashboard** with live monitoring capabilities
- **OEE Analysis** (Overall Equipment Effectiveness) with availability, performance, and quality metrics
- **Process Parameter Monitoring** with color-coded alerts for temperature, pressure, vibration, and cycle time
- **Machine Status Tracking** with shift information and operator identification
- **Maintenance Management** with scheduled maintenance tracking
- **Responsive Design** that works seamlessly on desktop, tablet, and mobile devices
- **Professional UI/UX** with modern industrial aesthetics

### ✨ Added - Production Monitoring System
- **Comprehensive Metrics Collection** for connections, operations, performance, and errors
- **Health Status Monitoring** with configurable thresholds and alerting
- **Real-time Performance Tracking** with latency and throughput metrics
- **Error Categorization** with detailed error analysis and reporting
- **System Uptime Tracking** with automatic health status calculation
- **Memory and CPU Usage Monitoring** for resource management

### ✨ Added - Configuration Management
- **Production-Ready Config System** with validation and environment-specific settings
- **PLC-Specific Configuration** for different Allen-Bradley models
- **Security and Performance Tuning** options for production deployment
- **Configuration Validation** with comprehensive error checking
- **Development vs Production** configuration presets

### ✨ Added - Production API Endpoints
- **Health Check Endpoint** (`/api/health`) for system status monitoring
- **Metrics Endpoint** (`/api/metrics`) for performance and operational data
- **Configuration Management** (`/api/config`) for runtime configuration updates
- **System Status** (`/api/status`) for comprehensive system information
- **RESTful API Design** following industry best practices

### ✨ Added - Performance Benchmarking Framework
- **Criterion-Based Benchmarking** for Rust operations
- **Comparative Analysis** capabilities for performance optimization
- **Stress Testing Framework** for long-term stability validation
- **Automated Performance Regression Testing**

### 🐛 Fixed - Real-time Monitoring Stability
- **Fixed Monitoring Flashing Issue** - Resolved the problem where monitoring status was flashing and buttons became unresponsive
- **Stable Continuous Updates** - Monitoring now works continuously without stopping after the first read
- **Proper State Management** - Fixed React closure issues that were causing monitoring to stop unexpectedly
- **Improved Error Handling** - Better error recovery and user feedback for monitoring operations

### 🚀 Performance Improvements
- **Optimized Batch Operations** with improved packet packing
- **Better Connection Pooling** for concurrent operations
- **Reduced Memory Footprint** with more efficient data structures
- **Faster Tag Path Parsing** with optimized algorithms
- **Enhanced Network Resilience** with improved connection handling

### 📚 Documentation Updates
- **Production Deployment Guide** with step-by-step instructions
- **Configuration Reference** with all available options and examples
- **Troubleshooting Guide** for common issues and solutions
- **Performance Tuning Guide** for optimal system configuration
- **Updated All Examples** with the latest features and best practices

## [0.4.0] - 2025-01-15

### 🎯 Major Production Release
- **Real-time tag subscriptions** with millisecond-level updates
- **High-performance batch operations** for enterprise applications
- **Critical stability fixes** resolving all hanging and timeout issues
- **Enhanced Allen-Bradley STRING support** with complete CIP protocol compliance
- **Industrial-grade reliability** with comprehensive error handling and recovery
- **Python wrapper** with full API coverage and type-safe bindings

### ✨ Added - Real-Time Subscriptions
- **Real-time tag monitoring** with configurable update intervals (1ms - 10s)
- **Event-driven notifications** for tag value changes
- **Subscription management** with automatic reconnection and error recovery
- **Multi-tag subscriptions** supporting hundreds of concurrent tag monitors
- **Callback-based architecture** for responsive industrial applications
- **Memory-efficient subscription engine** with minimal CPU overhead

### ✨ Added - High-Performance Batch Operations
- **Batch read operations** - read up to 100+ tags in a single request
- **Batch write operations** - write multiple tags atomically
- **Configurable batch sizes** with automatic optimization for PLC capabilities
- **Parallel processing** with concurrent batch execution
- **Transaction support** with rollback capabilities for critical operations
- **Performance monitoring** with detailed timing metrics (2,000+ ops/sec throughput)
- **Intelligent packet packing** to maximize network efficiency

### 🔧 Fixed - Critical Stability Issues
- **RESOLVED: Complete hanging in send_cip_request method**
  - Fixed EtherNet/IP command codes (0x6F,0x00 for SendRRData)
  - Added proper session handle management
  - Implemented 10-second timeout protection with tokio::time::timeout
  - Enhanced debug logging for troubleshooting
- **RESOLVED: String read parsing failures**
  - Fixed CPF (Common Packet Format) extraction algorithm
  - Added proper handling for Unconnected Data Item type (0x00B2)
  - Implemented correct CIP data extraction before response parsing
- **RESOLVED: Connection timeout and recovery issues**
  - Enhanced session management with automatic keep-alive
  - Improved error detection and graceful recovery
  - Added connection health monitoring and diagnostics

### 🔧 Enhanced - Allen-Bradley STRING Support
- **Complete STRING format compliance** with Allen-Bradley specifications
- **Proper CIP type 0x02A0 handling** matching PLC read/write expectations
- **Optimized string serialization** with length + data format (no padding)
- **Support for all string operations** including empty strings and special characters
- **String length validation** with proper 82-character limit enforcement
- **Enhanced debug output** for STRING operation troubleshooting

### 🔧 Enhanced - Error Handling & Diagnostics
- **Comprehensive CIP error mapping** with detailed extended status codes
- **Enhanced debug logging** throughout the protocol stack
- **Connection health monitoring** with automatic diagnostics
- **Graceful error recovery** for network interruptions and PLC restarts
- **Detailed error messages** with actionable troubleshooting information
- **Protocol-level validation** to prevent malformed requests

### 🚀 Performance Improvements
- **50% faster tag operations** due to protocol optimizations
- **2x improved throughput** for batch operations (2,000+ ops/sec)
- **Reduced memory footprint** with optimized buffer management
- **Lower latency** with streamlined packet processing (sub-millisecond improvements)
- **Enhanced connection pooling** for multi-client scenarios
- **Optimized network utilization** with intelligent request batching

### 📚 Enhanced - Documentation & Examples
- **Updated README** with v0.4.0 capabilities and performance metrics
- **Comprehensive subscription examples** showing real-time monitoring patterns
- **Batch operation tutorials** with enterprise application patterns
- **Troubleshooting guides** for common industrial networking scenarios
- **Performance tuning documentation** for high-throughput applications
- **Updated API documentation** with all new subscription and batch methods

### 🧪 Enhanced - Testing & Validation
- **Production validation** with extensive PLC testing on CompactLogix and ControlLogix
- **Stress testing** with thousands of concurrent operations
- **Network resilience testing** with connection interruption scenarios
- **Memory leak detection** and long-running stability validation
- **Performance benchmarking** with detailed metrics collection
- **Integration testing** with real industrial environments

### 🔗 Enhanced - Integration Capabilities
- **Improved C# wrapper** with subscription and batch operation support
- **Enhanced FFI exports** for better C/C++ integration
- **Thread-safe operations** with proper synchronization
- **Async/await support** throughout the API surface
- **Cross-platform validation** on Windows, Linux, and macOS
- **Docker compatibility** for containerized industrial applications
- **Python wrapper** with PyO3 integration:
  - Full API coverage with type-safe bindings
  - Synchronous and asynchronous APIs
  - Comprehensive error handling with Python exceptions
  - Easy installation via pip or maturin
  - Cross-platform support (Windows, Linux, macOS)
  - Example scripts and documentation

### 📊 Updated Performance Metrics
- **Single Tag Read**: 2,500+ ops/sec, <1ms latency (67% improvement)
- **Single Tag Write**: 1,200+ ops/sec, <2ms latency (50% improvement)
- **Batch Operations**: 2,000+ ops/sec, 5-20ms latency (NEW)
- **Real-time Subscriptions**: 1000+ tags/sec, 1-10ms update intervals (NEW)
- **Memory Usage**: ~1KB per operation, ~4KB per connection (50% reduction)
- **Connection Setup**: 50-200ms typical (60% improvement)

### 🏭 Production Readiness
- **Enterprise deployment ready** with comprehensive testing and validation
- **24/7 operation capable** with automatic error recovery
- **Scalable architecture** supporting hundreds of concurrent connections
- **Industrial network compatibility** with common plant floor configurations
- **Comprehensive logging** for production monitoring and diagnostics
- **Support for critical applications** with millisecond-level responsiveness

## [0.3.0] - 2025-06-01

### 🎯 Major Focus Shift
- **Specialized for Allen-Bradley CompactLogix and ControlLogix PLCs**
- **Optimized for PC applications** (Windows, Linux, macOS)
- **Enhanced for industrial automation** and SCADA systems
- **Production-ready Phase 1 completion** with comprehensive feature set

### ✨ Added - Enhanced Tag Addressing
- **Advanced tag path parsing** with comprehensive support for:
  - Program-scoped tags: `Program:MainProgram.Tag1`
  - Array element access: `MyArray[5]`, `MyArray[1,2,3]`
  - Bit-level operations: `MyDINT.15` (access individual bits)
  - UDT member access: `MyUDT.Member1.SubMember`
  - String operations: `MyString.LEN`, `MyString.DATA[5]`
  - Complex nested paths: `Program:Production.Lines[2].Stations[5].Motor.Status.15`

### ✨ Added - Complete Data Type Support
- **All Allen-Bradley native data types** with proper CIP encoding:
  - **SINT**: 8-bit signed integer (-128 to 127) - CIP type 0x00C2
  - **INT**: 16-bit signed integer (-32,768 to 32,767) - CIP type 0x00C3
  - **LINT**: 64-bit signed integer - CIP type 0x00C5
  - **USINT**: 8-bit unsigned integer (0 to 255) - CIP type 0x00C6
  - **UINT**: 16-bit unsigned integer (0 to 65,535) - CIP type 0x00C7
  - **UDINT**: 32-bit unsigned integer (0 to 4,294,967,295) - CIP type 0x00C8
  - **ULINT**: 64-bit unsigned integer - CIP type 0x00C9
  - **LREAL**: 64-bit IEEE 754 double precision float - CIP type 0x00CB
  - Enhanced **BOOL** (CIP type 0x00C1), **DINT** (CIP type 0x00C4), **REAL** (CIP type 0x00CA)
  - Enhanced **STRING** (CIP type 0x00DA) and **UDT** (CIP type 0x00A0) support

### ✨ Added - C# Wrapper Integration
- **Complete C# wrapper** with full .NET integration
- **22 FFI functions** covering all data types and operations:
  - Connection management: `eip_connect`, `eip_disconnect`
  - Boolean operations: `eip_read_bool`, `eip_write_bool`
  - Signed integers: `eip_read_sint`, `eip_read_int`, `eip_read_dint`, `eip_read_lint`
  - Unsigned integers: `eip_read_usint`, `eip_read_uint`, `eip_read_udint`, `eip_read_ulint`
  - Floating point: `eip_read_real`, `eip_read_lreal`
  - String and UDT operations: `eip_read_string`, `eip_read_udt`
  - Tag management: `eip_discover_tags`, `eip_get_tag_metadata`
  - Configuration: `eip_set_max_packet_size`, `eip_check_health`
- **Type-safe C# API** with comprehensive error handling
- **Cross-platform support** (Windows .dll, Linux .so, macOS .dylib)
- **NuGet package ready** for easy distribution

### ✨ Added - Build System and Automation
- **Automated build scripts** for all platforms:
  - `build.bat` for Windows with error checking and progress reporting
  - `build.sh` for Linux/macOS with cross-platform library handling
- **4-step build process**: Rust compilation → Library copy → C# build → Testing
- **Comprehensive BUILD.md guide** with:
  - Prerequisites and setup instructions
  - Cross-platform build procedures
  - Troubleshooting section
  - CI/CD pipeline examples
  - Distribution packaging instructions

### ✨ Added - Comprehensive Examples
- **Advanced Tag Addressing Example** (`examples/advanced_tag_addressing.rs`):
  - Demonstrates all tag addressing capabilities with real-world scenarios
  - Production line monitoring, motor control, recipe management
  - Complex nested UDT access and array operations
- **Data Types Showcase Example** (`examples/data_types_showcase.rs`):
  - Shows all supported data types with encoding details
  - Precision comparisons and boundary value testing
  - Performance demonstrations and validation

### 🔧 Enhanced - Core Infrastructure
- **TagPath module** (`src/tag_path.rs`):
  - Complete tag path parsing with error handling
  - CIP path generation for network transmission
  - Support for all addressing patterns
- **Enhanced error handling** with detailed CIP error mapping (40+ error codes)
- **Improved session management** with proper registration/unregistration
- **Memory safety** with proper resource cleanup and FFI safety documentation

### 🔧 Enhanced - Protocol Implementation
- **Proper CIP type codes** for all data types with correct 16-bit identifiers
- **Little-endian byte encoding** for network transmission consistency
- **Robust response parsing** for all data types with comprehensive validation
- **Enhanced EtherNet/IP encapsulation** with proper packet structure
- **Improved timeout handling** and network resilience

### 📚 Enhanced - Documentation
- **Comprehensive README** updates:
  - Focus on CompactLogix/ControlLogix PLCs
  - Production-ready status with Phase 1 completion
  - C# wrapper integration information
  - Updated performance characteristics and roadmap
- **Detailed API documentation** with examples for each function
- **C# wrapper documentation** (`csharp/RustEtherNetIp/README.md`):
  - Complete usage guide with all data types
  - Advanced tag addressing examples
  - Performance characteristics and thread safety guidance
  - Real-time monitoring examples
- **Build documentation** with comprehensive instructions
- **Updated lib.rs header** with current capabilities and architecture diagrams

### 🧪 Enhanced - Testing
- **30+ comprehensive unit tests** covering:
  - All data types with encoding/decoding validation
  - Tag path parsing for complex addressing scenarios
  - Boundary value testing for all numeric types
  - CIP type code verification
  - Little-endian encoding consistency
- **C# wrapper tests** with integration validation
- **Documentation tests** for all public APIs (marked as `no_run` for PLC examples)
- **Build verification** with automated testing in build scripts

### 🚀 Performance Improvements
- **Optimized tag path parsing** with efficient CIP path generation (10,000+ ops/sec)
- **Zero-copy operations** where possible for memory efficiency
- **Enhanced memory management** for large data operations (~8KB per connection)
- **Improved error handling** with minimal overhead
- **Network optimization** with configurable packet sizes

### 🔧 Code Quality Improvements
- **Fixed all linter warnings** and compilation issues
- **Resolved rust-analyzer warnings** about unsafe FFI operations
- **Added proper safety documentation** for all FFI functions
- **Fixed redundant closures** and error handling patterns
- **Added `#[allow(dead_code)]` attributes** for future API methods
- **Consistent error handling** using `EtherNetIpError` throughout

### 📋 Roadmap Updates
- **Phase 1**: Enhanced tag addressing ✅ **COMPLETED**
- **Phase 1**: Complete data type support ✅ **COMPLETED**
- **Phase 1**: C# wrapper integration ✅ **COMPLETED**
- **Phase 1**: Build automation ✅ **COMPLETED**
- **Phase 1**: Comprehensive testing ✅ **COMPLETED**
- **Phase 2**: Batch operations (planned Q3 2025)
- **Phase 2**: Real-time subscriptions (planned Q3-Q4 2025)
- **Phase 3**: Production v1.0 release (planned Q4 2025)

### 🏗️ Build and Distribution
- **Cross-platform library generation**:
  - Windows: `rust_ethernet_ip.dll` (783KB optimized)
  - Linux: `librust_ethernet_ip.so`
  - macOS: `librust_ethernet_ip.dylib`
- **C# NuGet package structure** ready for distribution
- **Automated build verification** with success/failure reporting
- **CI/CD ready** with GitHub Actions examples

### 📊 Performance Metrics
- **Single Tag Read**: 1,500+ ops/sec, 1-3ms latency
- **Single Tag Write**: 800+ ops/sec, 2-5ms latency
- **Tag Path Parsing**: 10,000+ ops/sec, <0.1ms latency
- **Memory Usage**: ~2KB per operation, ~8KB per connection
- **Connection Setup**: 100-500ms typical

### 🔗 Integration Capabilities
- **Native Rust API** with full async support
- **C FFI exports** for C/C++ integration
- **C# wrapper** with comprehensive .NET integration
- **Cross-language compatibility** with proper marshaling
- **Thread safety guidance** and best practices

## [0.2.0] - Previous Release

### Added
- Basic EtherNet/IP communication
- BOOL, DINT, REAL data types
- C FFI exports
- Session management

## [0.1.0] - Initial Release

### Added
- Initial project structure
- Basic PLC connection
- Simple tag operations

### Fixed
- Fixed Python wrapper's write_tag method to correctly return a boolean indicating success or failure.
