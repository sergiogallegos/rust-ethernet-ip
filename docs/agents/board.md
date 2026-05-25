# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Open

| Id | Title | Owner | Status | Last update | File |
|---|---|---|---|---|---|
| CODEX-M | FFI registry clone-semantics audit and fix (Phase B in progress) | codex | in-progress | 2026-05-24 claude [Opus 4.7] | [`tasks/CODEX-M-ffi-registry-clone-audit.md`](tasks/CODEX-M-ffi-registry-clone-audit.md) |

> Per maintainer directive (2026-05-18), all post-books briefs are **in scope for v0.8.0** alongside the pre-existing v0.8.0 draft work and the existing CODEX-G through CODEX-K agenda. The full plan and per-book lesson extraction live at [`wiki/investigations/architecture-review-2026-05-18.md`](../../wiki/investigations/architecture-review-2026-05-18.md) and [`wiki/investigations/books-lessons-2026-05-18.md`](../../wiki/investigations/books-lessons-2026-05-18.md). 2026-05-24 status: CODEX-L (ABI baseline), CODEX-N (CIP path validation), CODEX-V (semver-checks CI), CODEX-W (Python typed writes), CODEX-X (BOOL DWORD-offset), and CODEX-Y (nested BOOL UDT workaround) all merged. CODEX-M Phase A audit complete; Phase B in progress under Claude-amended direction (Option C with structural Arc/Atomic enforcement, not documentation-only). Remaining open: CODEX-M (Phase B), then J (mechanical split), then P (actor), then R/Q/S (events, service layer, retry), then CODEX-K (release-window) last. CODEX-T (fleet) and CODEX-U (sibling crates) remain v0.9.0 deferrals.
>
> Scope note: the v0.8.0 bundle is effectively a 1.0.0-shape release (FFI contract pin + behavioral refactor + new public API + structural split + release-window break sweep). Renaming the version to `1.0.0` is defensible and would signal the stability story to NuGet/PyPI/crates.io consumers; left to maintainer decision.
>
> Six earlier briefs merged (CODEX-A → CODEX-F). All belong to the v0.8.0 draft, which sits on `main` unreleased — no `v0.8.0` tag, no NuGet/crates.io publish.

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

### Post-books-review roadmap (Phase 2 — behavioral refactors, brief on activation)

These items came from the 2026-05-18 architecture review at [`wiki/investigations/architecture-review-2026-05-18.md`](../../wiki/investigations/architecture-review-2026-05-18.md). They change observable behavior (request ordering, cancellation, clone semantics, event surface) and must be treated as semver-meaningful, *not* internal refactors. Each requires its own wrapper-level compatibility test pass.

6. **CODEX-O — `PlcValue::Udt::get_data_type()` placeholder honesty.** Currently returns `0x00A0` placeholder (`src/types.rs:463,478`). Either: (a) return `Option<u16>` / `Result` instead of synthesizing a fake type code, or (b) capture the real type code in `UdtData` at parse time and propagate it. Verify via test that the placeholder never escapes through the FFI as a misleading real CIP type. Small, contained brief; can run any time after CODEX-L.
7. **CODEX-P — Request-correlator actor + cloneable `Client` handle.** Internal worker task owns the TCP stream; the public `Client` becomes a cheap-clone handle that sends `(request_bytes, oneshot::Sender<response_bytes>)` over an mpsc to the actor. Solves the cancellation-safety issue (a dropped future no longer leaves half-read response bytes on the wire) and removes the documented "wrap me in `Arc<Mutex<EipClient>>`" pattern. **Behaviorally breaking** — request ordering, cancellation, clone-share semantics are observable contract. Requires C# and Python wrapper smoke tests as part of acceptance. Runs after CODEX-J (mechanical split) so the actor lives in its own submodule.
8. **CODEX-R — `Client::events()` connection state stream.** Public method returning a `Stream<ConnectionEvent>` (Connected, Reconnecting, Disconnected, SessionRecycled). Today consumers learn about connection loss only by getting `ConnectionLost` back from a *next* operation; HMIs need a push notification. Sourced from the actor — depends on CODEX-P.

### Post-books-review roadmap (Phase 3 — bundle into the 1.0.0 release window)

9. **CODEX-Q — Service Layer for restricted writes.** Add `Client::write_udt_member`, `Client::write_string_tag`, `Client::write_udt_array_member` methods that internally implement the read-modify-write dance for the documented firmware limitations (see `lib.rs:46-62` and the 20-line doctest at `client.rs:131-150`). Removes the workaround ritual from consumer code. Stay concrete to the Logix STRING / UDT-array-member-write quirks; do not generalize into a broader pattern framework.
10. **CODEX-S — `RetryPolicy` primitive.** Builder + decorator combinator: `client.with_retry(policy).read_tag(...).await`. Backoff (constant / exponential / decorrelated jitter), max attempts, per-error-class predicate (already have `EtherNetIpError::is_retriable` at `src/error.rs:104`). Each consumer currently writes its own retry loop; centralizing prevents policy drift across the C# and Python wrappers.

### Post-books-review roadmap (Phase 4 — scale and extensibility, post-1.0)

11. **CODEX-T — `Fleet<PlcId, Client>` multi-PLC pool.** Today `PlcManager` (242 LOC) hints at this. Make it an explicit per-PLC actor pool with fleet-level health check and a fleet-level event stream. Industrial deployments routinely talk to N PLCs at once; per-PLC backpressure and shared metrics collection belong in the library, not in every consumer.
12. **CODEX-U — Promote `protocol`, `tag_path`, `udt` to sibling workspace crates.** Once their APIs stabilize after 1.0. Cargo features in the main crate let consumers pay only for what they use (an HMI that doesn't need UDT discovery shouldn't link `udt.rs`). Long-term modularity payoff; no short-term value.

## Done

| Id | Title | Owner | Merge commit |
|---|---|---|---|
| CODEX-A | FFI safety, runtime hardening, and lint baseline | codex | `3d98abf` |
| CODEX-B | Contained API cleanup — thiserror, dead deps, dead state, must_use | codex | `9aca8d2` |
| CODEX-E | Small polish — runtime-init log dedupe, regex caching, re-export merge, dev-dep audit | codex | `fc63735` |
| CODEX-C | Decompose lib.rs into route, batch, types, and client modules | codex | `476f21c` |
| CODEX-D | Extract Encoder/Decoder boundary for the wire protocol | codex | `c58a905` |
| CODEX-F | RoutePath ordered hops + ASCII ethernet link-address encoding | codex | `9a3d192` |
| CODEX-W | Python wrapper — route single-tag writes through typed FFI exports | codex | `4bab25a` |
| CODEX-L | FFI ABI version + capability handshake | codex | `5037133` |
| CODEX-N | CIP path encoding hard validation | codex | `5037133` |
| CODEX-V | Add cargo-semver-checks to CI as the SemVer gate | codex | `5037133` |
| CODEX-X | BOOL array element RMW addresses the wrong DWORD for indices ≥ 32 | codex | `5037133` |
| CODEX-Y | BOOL workaround not applied to nested BOOL arrays inside UDT array elements | codex | `5037133` |

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
