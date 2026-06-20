# Roadmap — future work

> Status as of 2026-06-19: **1.1.0 is shipped** to crates.io (×5), NuGet (multi-RID
> win/linux/osx-arm64), and PyPI (win/macOS/manylinux + sdist), CI green on all
> three OSes. The items below are the planned follow-up work, grouped by target
> release. Each is scoped enough to become a `docs/agents/` CODEX brief when it
> is picked up; nothing here is in progress yet.

## 1.2.0 — next minor (additive, non-breaking)

### 1. Full documentation refresh  *(priority for this release)*
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
  duplicate `RUST_*TEST_RESULTS.md`, the Python-expansion planning docs).
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

## 2.0.0 — next major (SemVer-breaking)

### 6. Remove dead public surface
- Delete the dead `TagCache` struct (`src/tag_manager.rs`) — entirely
  `#[allow(dead_code)]`, only deferred because it's publicly re-exported at
  `src/lib.rs` (removal is SemVer-major).
- Delete the unused `PlcConnection::update_health` (publicly re-exported), or
  wire it into the connection-pool health-reset paths that currently duplicate
  its logic inline.

### 7. C# true non-blocking async (optional, larger)
The 1.1.0 C# async API is `Task.Run` wrappers over blocking FFI (one pool thread
per in-flight call). True non-blocking would need async FFI entry points that
don't occupy a thread — a meaningful ABI addition (bump `ABI_VERSION` /
capability). Evaluate demand before committing.

## Validation / ops (any release)

### 8. Multi-chassis / multi-hop Ethernet routing — hardware validation
`RouteHop::Ethernet` ASCII extended-link-address encoding (from CODEX-F) has
only been validated on direct-connect / single-chassis hardware. It needs a real
**2-chassis ControlLogix bench** (e.g. local rack → 1756-EN2T → remote chassis)
to exercise a true multi-hop route end-to-end. On success, promote
`wiki/protocol/route-path-behavior.md` from `likely` to `confirmed`. Not
blocking — the wire format is unchanged from validated single-hop paths.

---

*To pick up an item: author a `docs/agents/tasks/CODEX-XY-*.md` brief, add a row
to `docs/agents/board.md`, and log it — per `docs/agents/README.md`.*
