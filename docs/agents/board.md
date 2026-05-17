# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Open

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|

> No tasks in flight. Six briefs merged (CODEX-A → CODEX-F). All six belong to the **v0.8.0 draft**, which sits on `main` unreleased — no `v0.8.0` tag, no NuGet/crates.io publish. Per maintainer direction (2026-05-17), v0.8.0 is held back until real-hardware validation passes; there is no v0.9.0 plan yet. The listed post-0.8.0 polish entries (formerly CODEX-G…K) are roadmap candidates, not authored briefs — turn them into briefs only after 0.8.0 ships.

## Next agenda

Resume order recommended by Claude. Each candidate brief is unwritten; the entry below summarises what the brief would cover so the next session can author and execute it without re-deriving context from chat history.

### Gating items for v0.8.0 release (maintainer-owned)

1. **Hardware validation** — direct-connect + multi-hop ethernet topology against real Allen-Bradley hardware. Specifically validate the new `RouteHop::Ethernet` ASCII extended-link-address encoding from CODEX-F (`9a3d192`). The validated targets in `CLAUDE.md` are CompactLogix `5069-L320ERMS3` and ControlLogix `1756-L81ES`; neither exercises the ethernet routing path. Until a real multi-hop topology accepts the new bytes, `wiki/protocol/route-path-behavior.md` marks the encoding as `likely` rather than `confirmed`. Maintainer's job; no agent can do this.
2. **Cut v0.8.0** — once hardware validation passes. Needs: tag `v0.8.0`, run the existing NuGet + crates.io pack/publish flow. `Cargo.toml` is already at `0.8.0` (set at `972b10b`); `src/lib.rs` head doc lines 5 and 48 already reference `0.8.0`; `CHANGELOG.md` already has the `[Unreleased]` section targeting `0.8.0`. The release-prep edit is just promoting `[Unreleased]` → `[0.8.0] - YYYY-MM-DD` and tagging.

### Post-0.8.0 polish (no version assigned; brief on resume)

These are non-breaking improvements deferred until after v0.8.0 ships. They are not yet briefed; the entries below summarise what each brief would cover. When the maintainer is ready to resume, claude authors the brief, codex implements, claude reviews.

1. **CODEX-G — `plc_manager.rs` unwrap cleanup.** Five-to-six `.unwrap()` calls on live paths: `src/plc_manager.rs:25` (parse in `Default`), and lines 135, 139, 163, 173, 183 (HashMap `get_mut`/`last_mut` on connection pool lookups). Soundness-adjacent — a panic from a connection-pool miss is a real failure mode, especially when reached from the FFI side. Convert to `Result` propagation; pair with a `From<std::net::AddrParseError>` impl if needed.
2. **CODEX-H — dead-code purge.** Six concrete items:
   - `TagCache` struct in `src/tag_manager.rs:73-113` — entirely `#[allow(dead_code)]`; never wired into `TagManager`. Either build the feature or delete the type.
   - `PlcManager::health_check_interval` field at `src/plc_manager.rs:95` — initialized to default, never read.
   - Nine `#[allow(dead_code)]` annotations in `src/client.rs` (lines 1617, 2112, 2163, 3326, 3837, 6486, 6597, 6607, 6628). Per-method audit needed; most are unused FFI helpers or half-finished features.
   - `BOOL_ARRAY_DWORD` dead `else if` branch at `src/protocol/values.rs:158-176` — preserved from pre-CODEX-D inline code, but `len >= 4` always matches before `len >= 8`. Tidy.
   - Leftover `#[allow(dead_code)] fn serialize_value` at `src/client.rs:3326` — pre-existing dead method.
   - Add a `debug_assert!(self.path.len() % 2 == 0)` to `CipRequest::encode` in `src/protocol/cip.rs` so caller bugs surface in dev builds.

   Note: removing `TagCache` from the public re-export at `src/lib.rs:150` (`pub use tag_manager::{TagCache, ...}`) is technically a SemVer-major change — verify it's actually re-exported and decide whether to defer that one item to the 1.0.0 brief.
3. **CODEX-I — real codec benchmarks.** Replace the placeholder `benches/performance_benchmark.rs` (three mock functions that don't exercise the codec at all — `black_box(PlcValue::Dint(42))`, `Vec<PlcValue>` push, no-op) with benchmarks that actually call `PlcValue::encode`, `PlcValue::decode`, `EncapsulationHeader::encode`, and a realistic batch-request build via `BytesMut`. Closes the brief-error gap from CODEX-D where the `>5%` regression gate was a sub-nanosecond noise check.
4. **CODEX-J — sub-split `client.rs`.** Still 6762 lines after CODEX-D. Codec extraction made these boundaries natural — see the audit table in this turn's chat record for line ranges. Suggested submodules:
   - `client::tag_io` (read_tag, write_tag, read_bit, write_bit, read_array_range — ~530 lines)
   - `client::udt` (read_udt_chunked, read_udt_member, write_udt_member, get_udt_definition, get_tag_attributes — ~1321 lines)
   - `client::string` (the STRING-specific write logic — ~958 lines)
   - `client::batch_exec` (execute_batch, read_tags_batch, write_tags_batch — ~274 lines; data types stay in `batch.rs`)
   - `client::diagnostics` (check_health, get_diagnostics_snapshot — ~289 lines)
   - `client::discovery` (discover_tags, discover_udt_members, discover_program_tags — ~215 lines)
   - `client::schema_export` (export_schema, export_schema_json — ~132 lines)
   - `client::subscriptions` (subscribe_to_tag, subscribe_to_tags, tag-group polling — ~222 lines)

   Also split `src/types.rs`: `ConnectedSession` and `ConnectionParameters` are internal wire-state and should move under `client::session` or `protocol::session`; `PlcValue` and `UdtData` stay as the user-facing `types` module. Pure mechanical move, mirrors CODEX-C's shape.

### 1.0.0 release-window brief (bundled SemVer-major)

5. **CODEX-K — release-window bundle.** Single brief covering every deferred SemVer-major item so the breakage happens once, cleanly, paired with a 1.0.0 tag:
    - **`RoutePath` private storage.** Remove `pub slots`, `pub ports`, `pub addresses` from `src/route.rs:17-20`. `hops: Vec<RouteHop>` becomes the only field, made private. Remove the legacy-grouped-fields fallback in `to_cip_bytes`. Builder methods are the only construction path. Deprecate then remove `add_slot`/`add_port`/`add_address` in favour of `add_backplane`/`add_ethernet`/`add_ethernet_with_port`.
    - **`#[non_exhaustive]` on public enums.** Apply to `EtherNetIpError` (`src/error.rs:11`), `BatchError` (`src/batch.rs:51`), `RouteHop` (`src/route.rs:3`), `TagPath` (`src/tag_path.rs:20`), `HealthStatus`, `HealthCheckMode`, `ErrorCategory` (`src/monitoring.rs:94, 102, 108`), `TagGroupEventKind`, `TagGroupFailureCategory` (`src/tag_group.rs:33, 41`).
    - **`try_init_tracing` typed signature.** Drop `Box<dyn Error>` from `src/lib.rs:207`; return `Result<(), EtherNetIpError>` with a new `Tracing(String)` variant or via the existing `Other(String)`. Same fix for `ProductionConfig::from_file` and `to_file` in `src/config.rs:268, 275`.
    - **Stringly-typed config fields → enums.** `LoggingConfig::level`, `LoggingConfig::format`, `LogRotationConfig::schedule` in `src/config.rs:143, 145, 165` become enums with Serde representation that preserves the existing string values (so JSON / TOML configs continue to round-trip). Removes the `valid_levels` runtime check at `src/config.rs:319`.
    - **Error type consolidation.** `EtherNetIpError` has overlapping variants — `StringWriteError`, `StringReadError`, `InvalidStringResponse` (lines 71-81) duplicate `WriteError`, `ReadError`, `InvalidResponse` shapes. Collapse into `CipError { code, message }` plus `Protocol(String)` where possible.
    - **Demote internal types from `pub` to `pub(crate)`.** `ConnectedSession`, `ConnectionParameters` in `src/types.rs` are wire-state types no user should construct; they shouldn't be at the crate root.
    - **`EipClient: Clone` semantics.** Either add a doc comment now (cheaper, non-breaking) or hide `Clone` at the major boundary in favour of an explicit `EipClient::handle()` method that returns a cheap clone. Decide during brief authoring.
    - **FFI ordered-hop shape.** `eip_connect_with_route` currently takes flat `slots[]` + `ports[]` + `addresses[]` arrays from the C# wrapper. After private-storage `RoutePath`, the FFI needs a parallel ordered-hop API. Coordinate with the wrapper change; bump the FFI return code namespace if needed.
    - **C# / Python wrapper sync.** Mirror the new `RouteHop` shape in `csharp/RustEtherNetIp/` and `python/` so downstream users get the same API.

## Done

| Id | Title | Owner | Merge commit |
|---|---|---|---|
| CODEX-A | FFI safety, runtime hardening, and lint baseline | codex | `3d98abf` |
| CODEX-B | Contained API cleanup — thiserror, dead deps, dead state, must_use | codex | `9aca8d2` |
| CODEX-E | Small polish — runtime-init log dedupe, regex caching, re-export merge, dev-dep audit | codex | `fc63735` |
| CODEX-C | Decompose lib.rs into route, batch, types, and client modules | codex | `476f21c` |
| CODEX-D | Extract Encoder/Decoder boundary for the wire protocol | codex | `c58a905` |
| CODEX-F | RoutePath ordered hops + ASCII ethernet link-address encoding | codex | `9a3d192` |

## Project context

- **Last released version:** `v0.7.0` (tagged 2026-04-07; see CHANGELOG).
- **Current draft:** `v0.8.0` — `Cargo.toml` bumped to `0.8.0` at `972b10b`, all CODEX-A through CODEX-F merged into the `[Unreleased]` section. No `v0.8.0` git tag exists; no NuGet/crates.io publish has run. Held pending real-hardware validation per maintainer direction (2026-05-17).
- **Current development focus:** the .NET stack — C# wrappers and examples (per `CLAUDE.md` Project Overview).
- **Hardware validation gate:** integration tests against real CompactLogix / ControlLogix PLCs are the maintainer's responsibility; CI runs `SKIP_PLC_TESTS=1` plus simulator-backed `plc_sim_tests`.

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
