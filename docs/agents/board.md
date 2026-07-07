# Agent Task Board

> Snapshot of every cross-agent task. Update the row whenever a task's status changes. Authoring rules: see [`README.md`](README.md).

## Open

> The ten CODEX-AJ…CODEX-AS briefs originate from the 2026-07-01 repository analysis ([`repo-analysis-2026-07-01.md`](repo-analysis-2026-07-01.md)) — read its executive summary before picking any of them up. Suggested sequencing: AJ and AK are independent and immediate; AL before AS (AS rebases on AL's `ffi.rs` changes); AM and AN are independent sim-verifiable wire fixes (AN starts with a reconciliation step against the full-coverage evidence); AO phase 2 is **blocked on maintainer packet captures** (phase 1 can land any time); AP after AM (item 5 depends on AM's `.DATA[i]` fix) and it removes the connected-messaging code AL deliberately scoped out; AQ and AR are independent of the rest but AQ's diagnostics counters coordinate with AL if concurrent.
>
> CODEX-AT and CODEX-AU (2026-07-02, maintainer-requested) sit outside the analysis set. AT is grounded in new hardware evidence ([`../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md`](../validation/2026-07-02_string_write_probe_5069-L330ERM_fw38.md)) that **disproves the STRING-write firmware quirk** — it answers CODEX-AP's item 5 decision point and shares files with AM (`.DATA[i]` consumer note), AN (`tests/plc_sim.rs`), and AP (`client/string.rs`, 0x2107 text); whichever lands second rebases. AU is packaging-only (C header + C++ example + Qt guide) and should sequence after AS if AS is in flight (AS privatizes three raw exports the header must exclude).
>
> **1.2.0 release plan (2026-07-02 maintainer direction).** The open CODEX-AJ…AU set accumulates on `main` with no interim releases. When the set is merged: full local matrix, then a maintainer hardware full-coverage pass (all three bindings; expected counts: CODEX-AV completed the blocked-label re-validation at `463854a` — the gate run uses the AV-corrected manifest of 2304 total / 2268 writeable / 17 expected-blocked / 19 read-only; zero unexpected anomalies required), then version bump and publish as **1.2.0** — minor, not major: the set is behavioral fixes, deprecations, and additive surface (header, C++ example) with no Rust-API signature breaks; the deferred deletions stay queued for 2.0 per `docs/ROADMAP.md`. **ABI-version caveat (CODEX-AS, merged `00db89e`):** the C FFI ABI is now v2 — a linker-visible removal of three unusable `*mut EipClient` exports. This does not break the crates.io SemVer contract (not Rust API) or the shipped C#/Python packages (wrappers import only `_by_id` and move in lockstep with the native lib), and the `eip_abi_version()` bump fails-fast any hypothetical third-party direct C consumer. Maintainer sign-off requested at tag time that this is acceptable within a **minor** release. Packet captures for CODEX-AO phase 2 should be bundled into that same hardware session. Recommended implementation order: **AK → AJ → AT → AM → AN → AL → AS → AP → AQ → AR → AU**, with AO phase 1 free to interleave anywhere. Decisions relayed for open questions: AK item 4 resolves as "omit `authors` entirely from all five manifests" (matches the sibling-crate convention); AU waits for AS.

| Id | Title | Owner | Status | Created |
|---|---|---|---|---|
| CODEX-AO | UDT wire-format investigation — capture-gated audit of struct read/write encoding and udt-crate strictness | codex | open | 2026-07-01 |
| CODEX-AQ | Dead-stratum deprecation — TagManager UDT pipeline, ProductionMonitor/Config, PlcManager, SubscriptionManager, TagCache; diagnostics honesty | codex | open | 2026-07-01 |
| CODEX-AR | Subscription, fleet, and event lifecycle — stop() stops, no blocked poll tasks, lag-tolerant forwarding | codex | open | 2026-07-01 |
| CODEX-AU | C++ consumer support — C header with parity gate, RAII example, Qt integration guide | codex | open | 2026-07-02 |

> CODEX-AI merged (2026-06-19): the manylinux Linux x86_64 wheel now ships. Root cause was a platlib-layout bug (native lib routed to `.data/purelib/`, rejected by auditwheel); fixed with an `install_lib = install_platlib` override in `python/setup.py`. `release.yml` builds the cdylib + wheel inside a `manylinux_2_28` container, auditwheel-repairs it, and a blocking smoke job installs+imports it in a clean container before publish. `rust_ethernet_ip-1.1.0-py3-none-manylinux_2_28_x86_64.whl` was added to the existing PyPI 1.1.0 release (no version bump) — `pip install` now works on Linux x86_64.

> 2026-06-19 **1.1.0 SHIPPED** — merged to `main`, tagged `v1.1.0`, and published to crates.io (×5), NuGet (multi-RID), and PyPI (Windows/macOS wheels + sdist; Linux wheel deferred → CODEX-AI). Originated as branch `review-fixes-v1.1`. Post-1.0.0 file-by-file review drove a three-tier fix bundle: Tier 1 correctness (client-side bit RMW — hardware-validated; C# UDT-write serialization + `eip_write_udt` `{symbol_id,data}` shape; C# no-double-write; C# finalizer/Dispose; `add_port` Rust+C#; Python UDT→STRING guard; subscription deadband; tag-list offset; no-git build; Python wheel native-lib bundling), Tier 2 debt (docs sweep + API_STABILITY/MIGRATION; deleted `examples/rust_examples/`; `client.rs` dead-code purge; `enip-test` moved out of the published crate; Docker/bench fixes), Tier 3 features (`thiserror` 2.0 + dep trim; macro-generated scalar FFI wrappers; `eip_get_last_error` + `CAP_LAST_ERROR` → C# `PlcException`/Python error detail; C# Task.Run async API; crates.io/PyPI/multi-RID-NuGet release automation). Version bumped to 1.1.0 across all 22 readiness-checked locations + CHANGELOG. Recommended as a **minor** (additive public API). Deferred (with reason): FFI registry `Arc<Mutex>`, `TagCache`/`update_health` (SemVer-major), data-type-table dedup. Hardware-validated on CompactLogix 5069-L330ERM @ 192.168.0.101: all three bindings byte-identical 2299/2206/2206/60/0. New release secrets needed for full automation: `CARGO_REGISTRY_TOKEN`, `PYPI_API_TOKEN` (both jobs no-op without them).
>
> 2026-05-24 release status: **v1.0.0 shipped**. `main` is at `f02eef5`; annotated tag `v1.0.0` pushed to origin. Five crates published to crates.io (`rust-ethernet-ip-types`, `rust-ethernet-ip-tag-path`, `rust-ethernet-ip-protocol`, `rust-ethernet-ip-udt`, `rust-ethernet-ip`), all at `1.0.0`. NuGet `RustEtherNetIp 1.0.0` ships via the GitHub release workflow triggered by the tag (assuming `NUGET_API_KEY` is configured). Single residual: multi-hop ethernet hardware validation remains a documented confidence upgrade — see post-1.0.0 polish list.
>
> 2026-05-25 agent-infra quartet merged: CODEX-Z (validate-agent-files + pre-commit hook), CODEX-AA (release-readiness checker), CODEX-AB (structured Claude-review template), CODEX-AC (committer wrapper). Inspired by [`steipete/agent-scripts`](https://github.com/steipete/agent-scripts). All four landed in one bundle; CI gates now enforce frontmatter shape, version-string parity, and the agent-commit wrapper contract on every PR + push.
>
> 2026-05-25 cross-binding harness CODEX-AD/AE merged at `59a2176`. Shared `examples/full_coverage_tags.json` manifest is now the single source of truth for the 2299-tag inventory; all three runners (Rust/C#/Python) consume it. 6-variant writeability enum replaces the over-broad `FirmwareBlocked` bucket. Per-run JSON results emitted to `examples/full_coverage_results/`. Preflight phase distinguishes PLC project errors (exit 2) from library regressions (exit 1). Hardware-validated by Codex against the maintainer's ControlLogix: all three bindings reported byte-identical parity (2299/2206/2206/14/60/0 anomalies).
>
> **Patch-release policy (2026-05-25 maintainer direction):** post-1.0.0 changes accumulate on `main` without triggering a new crates.io / NuGet release. Agent-infra (CODEX-Z/AA/AB/AC), tooling, docs, and test-only work do NOT cut a patch. The next patch release (1.0.1) is queued for when a real library change lands — current candidates are CODEX-G (`plc_manager.rs` unwrap cleanup), CODEX-H (dead-code purge), and CODEX-O (`PlcValue::Udt::get_data_type` placeholder honesty). At that point the maintainer decides which accumulated changes to roll into 1.0.1, runs the staged publish sequence (gating item below), and tags. Until then: push to `main` freely, no release pressure.
>
> Scope note: the v0.8.0 bundle is effectively a 1.0.0-shape release (FFI contract pin + behavioral refactor + new public API + structural split + release-window break sweep). Renaming the version to `1.0.0` is defensible and would signal the stability story to NuGet/PyPI/crates.io consumers; left to maintainer decision.
>
> Six earlier briefs merged (CODEX-A → CODEX-F). All belong to the v0.8.0 draft, which sits on `main` unreleased — no `v0.8.0` tag, no NuGet/crates.io publish.

## Next agenda

> **Canonical post-1.1.0 backlog: [`docs/ROADMAP.md`](../ROADMAP.md).** That doc
> is the current source of truth for future work — grouped by target release
> (1.2.0 minor, 2.0.0 major, and validation/ops). It supersedes the
> 1.0.0-era candidate entries below for planning purposes. Highlights: **full
> documentation refresh** (1.2.0 priority), Linux aarch64 + macOS-Intel wheels,
> data-type-table dedup, FFI registry `Arc<Mutex>`, the 2.0 dead-public-surface
> removals (`TagCache` / `update_health`), and **multi-chassis Ethernet routing
> hardware validation**.

Resume order recommended by Claude. Each candidate brief is unwritten; the entry below summarises what the brief would cover so the next session can author and execute it without re-deriving context from chat history.

### v1.0.0 release sequence — completed 2026-05-24

1. ✅ `git push origin main` → `main` at `f02eef5`
2. ✅ `cargo publish -p rust-ethernet-ip-types` v1.0.0
3. ✅ `cargo publish -p rust-ethernet-ip-tag-path` v1.0.0
4. ✅ `cargo publish -p rust-ethernet-ip-protocol` v1.0.0
5. ✅ `cargo publish -p rust-ethernet-ip-udt` v1.0.0
6. ✅ Main crate dry-run passed after sibling-crate index propagation
7. ✅ `cargo publish -p rust-ethernet-ip` v1.0.0
8. ✅ Annotated tag `v1.0.0` created on `f02eef5` and pushed to origin
9. NuGet `RustEtherNetIp 1.0.0` ships via the tag-triggered GitHub release workflow (assumes `NUGET_API_KEY` configured); workflow run was not monitored locally.

### Post-1.0.0 confidence upgrades

1. **Multi-hop ethernet hardware validation** — the 2026-05-24 release run validated direct-connect (1756-L75 in slot 0 via 1756-EN2T) end-to-end across Rust/C#/Python with zero anomalies. Multi-chassis ethernet routing (`RouteHop::Ethernet` ASCII extended-link-address encoding from CODEX-F at `9a3d192`) still needs a real 2-chassis bench. `wiki/protocol/route-path-behavior.md` keeps ethernet hops at `likely` until that lands. Not blocking for `1.0.x` because the wire format is unchanged from prior validated paths; promotes to `confirmed` on first multi-chassis run.

### Post-1.0.0 polish (no version assigned; brief on resume)

These are non-breaking improvements deferred until after v1.0.0 ships. They are not yet briefed; the entries below summarise what each brief would cover. When the maintainer is ready to resume, claude authors the brief, codex implements, claude reviews.

1. **CODEX-H residual — dead-code purge (remaining items).** The first pass merged at `2690669` (2026-05-26) removed `PlcManager::health_check_interval` and the dead BOOL-array `len >= 8` decode branch. These items remain:
   - `TagCache` struct in `src/tag_manager.rs:73-113` — entirely `#[allow(dead_code)]`; deferred because it's publicly re-exported at `src/lib.rs:150` (`pub use tag_manager::{TagCache, ...}`). Removal is SemVer-major and belongs in the 1.0.0 release-window bundle (CODEX-K), not a patch.
   - Nine `#[allow(dead_code)]` annotations in `src/client.rs` (lines 1617, 2112, 2163, 3326, 3837, 6486, 6597, 6607, 6628). Per-method audit needed; most are unused FFI helpers or half-finished features. Patch-eligible if all are internal.
   - Leftover `#[allow(dead_code)] fn serialize_value` at `src/client.rs:3326` — pre-existing dead method.
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

7. **CODEX-P — Request-correlator actor + cloneable `Client` handle.** Internal worker task owns the TCP stream; the public `Client` becomes a cheap-clone handle that sends `(request_bytes, oneshot::Sender<response_bytes>)` over an mpsc to the actor. Solves the cancellation-safety issue (a dropped future no longer leaves half-read response bytes on the wire) and removes the documented "wrap me in `Arc<Mutex<EipClient>>`" pattern. **Behaviorally breaking** — request ordering, cancellation, clone-share semantics are observable contract. Requires C# and Python wrapper smoke tests as part of acceptance. Runs after CODEX-J (mechanical split) so the actor lives in its own submodule.
8. **CODEX-R — `Client::events()` connection state stream.** Public method returning a `Stream<ConnectionEvent>` (Connected, Reconnecting, Disconnected, SessionRecycled). Today consumers learn about connection loss only by getting `ConnectionLost` back from a *next* operation; HMIs need a push notification. Sourced from the actor — depends on CODEX-P.

### Post-books-review roadmap (Phase 3 — bundle into the 1.0.0 release window)

9. **CODEX-Q — Service Layer for restricted writes.** Add `Client::write_udt_member`, `Client::write_string_tag`, `Client::write_udt_array_member` methods that internally implement the read-modify-write dance for the documented firmware limitations (see `lib.rs:46-62` and the 20-line doctest at `client.rs:131-150`). Removes the workaround ritual from consumer code. Stay concrete to the Logix STRING / UDT-array-member-write quirks; do not generalize into a broader pattern framework.
10. **CODEX-S — `RetryPolicy` primitive.** Builder + decorator combinator: `client.with_retry(policy).read_tag(...).await`. Backoff (constant / exponential / decorrelated jitter), max attempts, per-error-class predicate (already have `EtherNetIpError::is_retriable` at `src/error.rs:104`). Each consumer currently writes its own retry loop; centralizing prevents policy drift across the C# and Python wrappers.

### Post-books-review roadmap (Phase 4 — scale and extensibility, post-1.0.0 scope)

11. **CODEX-T — `Fleet<PlcId, Client>` multi-PLC pool.** Today `PlcManager` (242 LOC) hints at this. Make it an explicit per-PLC actor pool with fleet-level health check and a fleet-level event stream. Industrial deployments routinely talk to N PLCs at once; per-PLC backpressure and shared metrics collection belong in the library, not in every consumer.
12. **CODEX-U — Promote `protocol`, `tag_path`, `udt` to sibling workspace crates.** Once their APIs stabilize after 1.0. Cargo features in the main crate let consumers pay only for what they use (an HMI that doesn't need UDT discovery shouldn't link `udt.rs`). Long-term modularity payoff; no short-term value.

## Done

| Id | Title | Owner | Merge commit |
|---|---|---|---|
| CODEX-AP | Retire the string/UDT strategy graveyard — never-worked public paths return honest errors | codex | `1821e59` |
| CODEX-AS | FFI polish — private raw-pointer exports, unwind guard, SAFETY discipline, last-error lifecycle; Python residuals | codex | `00db89e` |
| CODEX-AL | Transport & session hardening — timeout desync, sender-context correlation, shared session handle | codex | `253706e` |
| CODEX-AN | read_array_range + get_tag_attributes wire fixes; make the simulator an oracle, not a mirror | codex | `8645f67` |
| CODEX-AV | Re-validate the firmware_blocked_* write labels — the 0x2107 lore is falling to correct paths | codex | `463854a` |
| CODEX-AM | Tag addressing correctness — member-suffix drop, bit syntax, batch BOOL arrays, .DATA[i] segment, program discovery | codex | `4ec2cec` |
| CODEX-AT | STRING wire format — direct structure writes work; fix encoding, decode reads, retire the firmware-quirk misdiagnosis | codex | `026c4e2` |
| CODEX-AK | Hygiene hotfixes — semver baseline, dependency scoping, release publish error handling, dead test files, doc corrections | codex | `2cf2d96` |
| CODEX-AJ | C# wrapper critical fixes — WriteUdtMember deadlock, UTF-8 marshalling, keep-alive serialization + native P/Invoke integration tests | codex | `2cf2d96` |
| CODEX-AI | Publish a manylinux Linux x86_64 Python wheel to PyPI | codex | `98fc460` |
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
| CODEX-M | FFI registry clone-semantics audit + Phase B (Arc/Atomic enforcement) | codex | `71c0d7e` |
| CODEX-J | Mechanical client.rs submodule split | codex | `71c0d7e` |
| CODEX-P | Request-correlator actor + cloneable Client handle | codex | `71c0d7e` |
| CODEX-Q | Service layer for restricted writes | codex | `71c0d7e` |
| CODEX-R | Client connection event stream | codex | `71c0d7e` |
| CODEX-S | RetryPolicy primitive | codex | `71c0d7e` |
| CODEX-K | Release-window SemVer bundle | codex | `71c0d7e` |
| CODEX-T | Fleet multi-PLC actor pool | codex | `71c0d7e` |
| CODEX-U | Promote protocol, tag_path, and udt to sibling crates (publish deferred) | codex | `71c0d7e` |
| CODEX-Z | Validate agent task file frontmatter + board/log consistency on pre-commit | codex | `3770e3a` |
| CODEX-AA | Release-readiness checker — version-string parity + cargo package chain | codex | `3770e3a` |
| CODEX-AB | Structured Claude-review template — six-question contract + fixed output shape | codex | `3770e3a` |
| CODEX-AC | Committer wrapper script — enforce specific-file staging + non-empty message | codex | `3770e3a` |
| CODEX-AD | Fix Rust full-coverage classification + close the settle verification loop | codex | `59a2176` |
| CODEX-AE | Cross-binding hardware harness — shared tag manifest, JSON output, granular firmware classification, preflight inventory check | codex | `59a2176` |
| CODEX-AF | Full-coverage exerciser — cwd-independent manifest resolution across all three bindings | codex | `6ec3f8d` |
| CODEX-AG | cross_language_compatibility_tests — honor SKIP_PLC_TESTS + TEST_PLC_ADDRESS, migrate to gTest* tag set | codex | `fe5059c` |
| CODEX-G | `plc_manager.rs` unwrap cleanup — return `EtherNetIpError::Connection` on pool lookups | codex | `2690669` |
| CODEX-H | Dead-code purge (partial) — `PlcManager::health_check_interval` + dead BOOL `len >= 8` branch (TagCache + `client.rs` allow-list deferred) | codex | `2690669` |
| CODEX-I | Real codec benchmarks — replace placeholder no-ops with PlcValue/EncapsulationHeader/CipRequest encode/decode | codex | `2690669` |
| CODEX-O | `PlcValue::Udt::get_data_type()` placeholder honesty — added `known_data_type() -> Option<u16>` + symbol-derived UDT type-prefixed encode | codex | `2690669` |
| CODEX-AH | Bump MSRV to Rust 1.96 + adopt `std::assert_matches` in tests | codex | `e8e336b` |

## Project context

- **Current released version:** `v1.1.0` (tagged 2026-06-19; published to crates.io ×5, NuGet multi-RID, PyPI incl. the CODEX-AI manylinux wheel).
- **Previous released version:** `v1.0.0` (tagged 2026-05-24, `f02eef5`).
- **Active development line:** the CODEX-AJ…AU remediation set on `main`, targeting **1.2.0** after a full hardware validation pass (see the 1.2.0 release plan note in `## Open`).
- **Current development focus:** the .NET stack — C# wrappers and examples (per `CLAUDE.md` Project Overview).
- **Hardware validation gate:** integration tests against real CompactLogix / ControlLogix PLCs are the maintainer's responsibility; CI runs `SKIP_PLC_TESTS=1` plus simulator-backed `plc_sim_tests`.

## Conventions

- **Status values:** `open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`.
- **`merged` rows** move to the `## Done` section with their merge commit reference.
- **Owner ≠ author of brief.** Owner is who is currently *doing* the work. Briefs are always authored by claude.
- **One row per task file.** If a task spawns subtasks, give them their own ids.
