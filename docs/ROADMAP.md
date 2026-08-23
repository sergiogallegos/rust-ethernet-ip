# Roadmap — future work

> Status as of 2026-08-22: **1.2.1 is shipped** to crates.io (×5), NuGet, and
> PyPI. It carries the schema-cache safety sequence (CODEX-BA–BD), Python
> native batch writes (CODEX-BF), the `get_tag_attributes`/`get_udt_definition`
> real-hardware fix (CODEX-BJ), and a CI action-version bump (CODEX-BI), all
> hardware-validated live on a ControlLogix 1756-L75. Accepted, non-blocking
> follow-ups (CODEX-BE packet-policy characterization, CODEX-BG endurance
> soak, CODEX-BH tag-shape performance matrix) remain queued without a
> target version. Historical `1.2.0`/`1.2.1` planning below is retained for
> traceability.

## 1.2.1 — shipped patch (historical plan)

- Finished the full Markdown/link audit and API documentation gate.
- Shipped the static project website and deployment instructions.
- Published the exact hardware matrix, contributor result template, endurance
  profiles, and performance characterization protocol.
- Re-ran the cross-binding hardware gate for program discovery paging/scope
  propagation, plus the full schema-cache safety sequence, on a live
  ControlLogix 1756-L75.
- Final release notes:
  [release/1.2.1_RELEASE_NOTES.md](release/1.2.1_RELEASE_NOTES.md).

## 1.2.0 — shipped minor (historical plan)

Recommended sequencing for this release line:

1. Documentation refresh plus Rust API positioning.
2. Placeholder / passive-surface decisions, so public APIs stop implying
   unsupported capability.
3. Python parity expansion, but only on top of native surfaces that are honest
   and stable.
4. Platform/package coverage and internal refactors as capacity allows.

### 1. Full documentation refresh *(completed during 1.2.1 preparation)*
The 1.1.0 version bump only swept the 22 release-readiness-checked files plus the
root and C# READMEs. A complete pass is owed:

- **Version sweep** — update every doc that still presents `1.0.0` as the current
  line (e.g. `docs/programmer_manual.md`, `docs/SOFTWARE_ARCHITECTURE.md`, the
  `0.7.0` historical notes in the root `README.md`, `BUILD.md`,
  `docs/INTEGRATION_AND_DEPLOYMENT.md`).
- **Feature coverage** — document the 1.1.0 additions everywhere relevant: the C#
  async API (and its Task.Run / not-true-async caveat), `PlcException` /
  `eip_get_last_error` / `CAP_LAST_ERROR`, the multi-RID NuGet and
  manylinux/win/macOS PyPI wheels, and the `thiserror 2.0` / trimmed-`tokio`
  dependency posture.
- **Per-language getting-started accuracy** — verify the Rust/C#/Python
  quick-starts compile/run against the current API; confirm `docs.rs` renders a
  real landing page for the main crate *and* the four sibling crates (thin
  crate-level `//!` docs likely missing on the siblings).
- **Historical-doc triage** — finish the archive/banner pass started in the 1.0.0
  cleanup (the empty `docs/api`/`docs/protocol`/`docs/examples` dirs, the
  duplicate `RUST_*TEST_RESULTS.md`, the Python-expansion planning docs, and
  stale wiki pages that still describe pre-1.1.0 architecture as current).
- Add `README.md` historical-note + compat-matrix links refresh, and keep
  `docs/API_STABILITY.md` / `docs/MIGRATION_*` current.

### 2. Linux `aarch64` (arm64) wheel + NuGet `linux-arm64`
Extend the manylinux build (CODEX-AI) to `manylinux_2_28_aarch64` so
`pip install` works on ARM Linux, and add a `linux-arm64` native to the NuGet
RID set. Reuse the existing manylinux-container + auditwheel + smoke-gate
pattern. *(musl/Alpine wheels are a separate, optional follow-up.)*

### 3. macOS Intel (`osx-x64`) coverage
Dropped in 1.1.0 because the GitHub `macos-13` free runner queues indefinitely.
Revisit: either a reliable Intel runner, or cross-compile `osx-x64` from the
`macos-14` (Apple Silicon) runner, to restore Intel-Mac NuGet/PyPI artifacts.

### 4. Data-type-table dedup
Centralize the CIP type → size / type → name maps that are currently duplicated
across `src/client.rs`, `src/schema.rs`, `crates/udt`, and
`crates/protocol/src/values.rs` (5 copies, already drifting) into
`rust_ethernet_ip_protocol`. Internal, non-breaking.

### 5. FFI registry `Arc<Mutex<EipClient>>` (internal hardening)
Replace the `get_client` clone-and-maybe-`store_client` pattern in `src/ffi.rs`
with an `Arc<tokio::sync::Mutex<EipClient>>` registry, removing the
copied-field-mutation footgun the reviews flagged as "correct today, brittle."
Behavior-preserving; verify against the Rust/C#/Python FFI suites.

### 6. Rust high-level API stabilization pass
`Client`, `RetryClient`, `Fleet`, connection events, and the service-layer helper
APIs are now public alongside the older `EipClient` facade, but the docs and
examples still mostly teach `EipClient` plus manual `Arc<Mutex<_>>` sharing.
Decide and document the recommended Rust entry point for new applications:

- Promote actor-backed `Client` / `Fleet` examples where appropriate, or clearly
  mark them as advanced APIs.
- Clarify how `Client`, `Fleet`, `PlcManager`, `RetryPolicy`, and `EipClient`
  relate so users do not pick the wrong abstraction.
- Document current event semantics (`Connected`, `Disconnected`,
  `WorkerStopped`) and what is not promised yet, such as reconnect/session
  recycled events.
- Decide whether wrappers should ever adopt actor semantics, or whether the
  FFI registry remains the wrapper boundary.

### 7. Python wrapper parity expansion
The Python package is installable and covers connection, routing, scalar
read/write, batch read/write, health, diagnostics, and last-error mapping. It
does not yet expose the richer native surface that Rust/C# expose or imply:

- Tag discovery / detailed discovery, tag attributes, and UDT definition APIs.
- Array-range reads and UDT/member helper APIs.
- Tag group / subscription ergonomics, if those remain a supported wrapper
  concept.
- Package-level examples for data collection and schema inspection that match
  the current FFI instead of historical Python planning docs.
- Parity tests that prove Python/C#/Rust agree on error mapping and JSON value
  decoding for scalar, array, STRING, and UDT payloads.

Keep this additive. Do not widen the Python API just to mirror every internal
Rust type; prioritize surfaces that are already stable through FFI. Do not build
Python parity on top of placeholder native exports such as `eip_get_tag_metadata`
until those exports are either implemented or explicitly retired.

### 8. Placeholder native/API surface decisions
C# exposes `ConfigureBatchOperations()` and `GetBatchConfig()`, but they
intentionally throw `NotSupportedException`; native `eip_configure_batch_operations`
and `eip_get_batch_config` are also placeholders. Native `eip_discover_tags` /
`eip_get_tag_metadata` are also legacy placeholder exports, while newer detailed
discovery / attribute APIs exist separately. Pick explicit directions:

- Prefer deprecating batch-configuration methods now, then remove them in 2.0,
  unless a concrete product need appears for configurable native batch packing.
- Either implement `eip_get_tag_metadata` / `eip_discover_tags` through the
  maintained detailed discovery path, or mark the legacy exports unsupported and
  move wrappers to the maintained exports.
- Add Rust/C#/Python contract tests for whichever direction is chosen.

### 9. Diagnostics and passive-config honesty pass
CODEX-AQ made operation/error diagnostics counters real per-client atomics on
the CIP send path and marked the old passive configuration/monitoring shells as
deprecated compatibility surfaces. Diagnostics snapshots and
`MonitoringMetrics::system_metrics_are_placeholders()` still mark CPU and memory
values as placeholders because they are not OS-derived telemetry.

Before promoting diagnostics as a full operational feature, either replace the
remaining system values with real cross-platform metrics or split unsupported
system metrics out so consumers cannot mistake placeholders for telemetry.

### 10. C# wrapper maintainability split
`EtherNetIpClient.cs` remains the largest wrapper file and mixes scalar access,
UDT helpers, batch operations, discovery, configuration, statistics, and native
interop structures. Split it into behavior-focused partials without changing the
public API, similar to the existing async/native-method partials. This should be
paired with contract tests so the split is mechanical and low risk.

### 11. Test-suite quality pass
Several `#[ignore]` hardware-oriented tests still return early on connection
failure or swallow operational failures, which means they can pass without
proving behavior. `tests/TEST_COVERAGE_SUMMARY.md` is also stale relative to the
current simulator, wrapper, and release-gate coverage.

- Port tests to the deterministic simulator where protocol behavior is not
  hardware-specific.
- For real-hardware tests, make missing PLC configuration skip explicitly and
  make configured runs assert failures unconditionally.
- Update the coverage summary so it reflects Rust, C#, Python, simulator,
  package, and real-hardware coverage by current release line.

### 12. Supply-chain policy gate
CI runs `cargo-audit`, but it does not yet enforce license, duplicate-version,
source, or banned-dependency policy with `cargo-deny`. Add a checked-in
`deny.toml` and CI job with a pragmatic initial policy:

- fail on known-bad advisories and disallowed licenses;
- warn or fail on duplicate dependencies once the baseline is understood;
- restrict registry/git sources to intentional sources;
- document how to update the policy during dependency upgrades.

## 2.0.0 — next major (SemVer-breaking)

### 13. Remove dead public surface
- Delete CODEX-AQ deprecated dead compatibility structs retained through the 1.x
  line: `ProductionMonitor`, `ProductionConfig`, `SubscriptionManager` /
  `RealTimeSubscriptionManager`, `TagCache`, and `PlcManager`.
- Remove CODEX-AP deprecated STRING/UDT compatibility stubs that now return
  explicit unsupported errors: Rust `write_string`,
  `write_ab_string_components`, `write_ab_string_udt`,
  `write_string_connected`, `write_string_unconnected`,
  `read_udt_member_by_offset`, and `write_udt_member_by_offset`; C FFI
  `eip_read_udt_member_by_offset` and `eip_write_udt_member_by_offset`; and
  the C# `ReadUdtMemberByOffset` / `WriteUdtMemberByOffset` wrappers.
- Delete the unused `PlcConnection::update_health` (publicly re-exported), or
  wire it into the connection-pool health-reset paths that currently duplicate
  its logic inline.
- Remove any deprecated placeholder exports or wrapper methods from item 8 if
  they were not implemented in the 1.x line.

### 14. C# true non-blocking async (optional, larger)
The 1.1.0 C# async API is `Task.Run` wrappers over blocking FFI (one pool thread
per in-flight call). True non-blocking would need async FFI entry points that
don't occupy a thread — a meaningful ABI addition (bump `ABI_VERSION` /
capability). Evaluate demand before committing. If this stays deferred, consider
adding an async interface for the existing `Task.Run` methods so DI/test users
can depend on the async surface without binding to the concrete class.

## Validation / ops (any release)

### 15. Multi-chassis / multi-hop Ethernet routing — hardware validation
`RouteHop::Ethernet` ASCII extended-link-address encoding (from CODEX-F) has
only been validated on direct-connect / single-chassis hardware. It needs a real
**2-chassis ControlLogix bench** (e.g. local rack → 1756-EN2T → remote chassis)
to exercise a true multi-hop route end-to-end. On success, promote
`wiki/protocol/route-path-behavior.md` from `likely` to `confirmed`. Not
blocking — the wire format is unchanged from validated single-hop paths.

### 16. Restricted-write and metadata simulator expansion
The simulator and wrapper tests cover the broad happy paths, but several
high-value behaviors still depend on real PLC validation or thin contract tests:

- UDT definition discovery, tag attributes, detailed tag discovery, and schema
  export.
- STRING writes and UDT member / UDT array member restricted-write helpers,
  including expected firmware failures.
- Wrapper value decoding for binary UDT payloads that must not be mistaken for
  Logix STRING data.

Add deterministic simulator coverage where possible, then reserve real hardware
only for firmware-specific behavior.

### 17. Cross-registry package install smoke matrix
The 1.1.0 release gates now prove built artifacts before upload. Add a separate
post-publish or manual gate that installs from the public registries and runs a
minimal import/load smoke for:

- crates.io main crate plus sibling crates;
- NuGet package on `win-x64`, `linux-x64`, `osx-arm64`, and any newly added RIDs;
- PyPI wheels/sdist on Windows, macOS, Linux x86_64, and future Linux arm64.

This is not a substitute for release CI; it catches registry metadata, packaging
selection, and native-library loading issues after publication.

---

*To pick up an item: author a `docs/agents/tasks/CODEX-XY-*.md` brief, add a row
to `docs/agents/board.md`, and log it — per `docs/agents/README.md`.*
