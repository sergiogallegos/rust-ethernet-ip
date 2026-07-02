# Repository analysis — 2026-07-01

- **Author:** claude [Fable 5]
- **Baseline:** `main` @ `47b99a4`, clean tree, v1.1.0 published
- **Method:** Four parallel deep-review passes (core protocol/client, FFI + language bindings, peripheral modules, project health) over the full source tree, followed by independent spot-verification of every critical/high finding cited below. Independent verification run: `cargo fmt -- --check`, `cargo clippy --all-features -- -D warnings`, `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — **all pass**.
- **Caveat:** Wire-format findings rest on the Logix Read/Write Tag service formats (1756-PM020) and cross-checks against the crate's own correct code paths — none were validated against live hardware in this analysis. Findings marked *(hardware-confirm)* deserve a packet capture before a fix lands.

---

## Executive summary

The project is in strong shape operationally and structurally at the edges, with a concentrated debt problem in the middle:

1. **Release engineering, CI, packaging, and version parity are excellent** — verified, broad (3 OS × stable/beta, MSRV gate, ffi feature, 3×3 Python matrix, C# on all OSes), and fully automated. Better than CLAUDE.md describes.
2. **The extracted codec layer (`crates/protocol`, `crates/tag-path`, `crates/types`, `crates/udt`) and the actor layer (`client/actor.rs`) are the best code in the repo** — bounds-checked decoding, no reachable panics on malformed peer input, typed retry policy, documented clone semantics.
3. **`src/client.rs` and `src/client/string.rs` are an archaeology site.** Multiple generations of exploratory protocol code coexist with the clean codec. The bugs cluster with near-perfect precision: everything routed through `CipRequest`/`CipResponse` is fine; nearly every hand-rolled byte-emission path has a framing, offset, or status-byte error — several public APIs (`write_string`, `write_ab_string_udt`, connected messaging, `get_tag_attributes`, `read_udt_member_by_offset`) can never have worked against real hardware.
4. **The test simulator is written to match the client rather than the spec** (explicitly commented at `tests/plc_sim.rs:733`), so the sim-test suite mirrors bugs instead of catching them. Two pinned byte tests pin incorrect encodings.
5. **The C# wrapper — the stated product focus — carries the single worst defect:** `WriteUdtMember` deadlocks on every real invocation, invisible to CI because the C# tests exercise Moq mocks, never the real P/Invoke path.
6. **An older stratum of publicly exported but never-wired modules** (`ProductionMonitor`, `ProductionConfig`, `PlcManager`, `SubscriptionManager`, `TagCache`, the `TagManager` UDT pipeline) ships plausible-looking but fabricated or dead behavior. Most of this is already acknowledged in ROADMAP items 5–9.
7. **CLAUDE.md's architecture and build documentation is two refactors stale**, including an FFI build command that produces a cdylib with no exports.

---

## Top priorities

| # | Finding | Where | Severity |
|---|---------|-------|----------|
| 1 | C# `WriteUdtMember` self-deadlocks on every call (nested non-reentrant `SemaphoreSlim`) | `EthernetNetIpClient.cs:1525` | **Critical** |
| 2 | UDT write type encoding collapses marker+handle into one u16 (`0x02A0 + symbol_id`); pinned test pins this shape. **Conflict note:** this encoding was deliberately introduced by CODEX-O and the 2026-05-26/2026-06-19 full-coverage hardware runs verified UDT RMW writes byte-identical on two real controllers — so either the spec reading here is wrong, the controllers tolerate the collapsed form, or the coverage matrix doesn't exercise the failing shape. Must be resolved by packet capture, not code change. | `crates/types/src/lib.rs:103`, `crates/protocol/src/tests.rs:124` | **Investigate** *(hardware-confirm)* |
| 3 | UDT reads leave the 2-byte structure handle inside `UdtData.data`, never populate `symbol_id` → member offsets shift by 2; combined with `to_hash_map`/`from_hash_map` silent-skip/zero-fill, read-modify-write can zero PLC data | `crates/protocol/src/values.rs:152`, `crates/udt/src/lib.rs:500` | **High** *(hardware-confirm)* |
| 4 | `get_tag_attributes` malformed request (missing path-size byte) + misaligned response parse — poisons `get_udt_definition`, the `write_tag` symbol_id fallback, and `Client::write_udt_member` | `src/client.rs:2329, 2410` | **High** |
| 5 | `write_tag("Array[i].Member", …)` silently drops the member suffix and writes to the whole element | `src/client.rs:2948` | **High** |
| 6 | `read_array_range` chunk parser assumes an element-count field real PLCs don't send; simulator was bent to match it | `src/client.rs:1268`, `tests/plc_sim.rs:733` | **High** |
| 7 | `read_tag`/`write_tag` on `"Tag.15"` bit syntax silently operate on the whole word | `crates/tag-path/src/lib.rs:240` + absent handling in read/write paths | **High** |
| 8 | No stream resync after timeout; next request reads the stale response (misattributed data on an industrial write path) | `src/client.rs:3610` | **High** |
| 9 | `discover_udt_members` fabricates UDT definitions (invented member names/offsets, `Value: DINT` fallback) behind an off-by-one path size its own tests pin | `src/tag_manager.rs:356, 373` | **High** |
| 10 | C# ANSI string marshalling corrupts non-ASCII on Windows (Rust side requires UTF-8) | `EthernetNetIpClient.cs` throughout | **High (Windows)** |
| 11 | cargo-semver-checks CI gate baselines against 0.7.0 — permits any breaking change on the 1.x line | `.github/workflows/ci.yml` | **High (process)** |
| 12 | CLAUDE.md build docs: `cargo build --release` without `--features ffi` produces a cdylib with no `eip_*` exports | `CLAUDE.md` build section | **High (docs)** |

---

## 1. Protocol correctness (core crate)

Verified-by-reading findings, ordered roughly by user impact. Items 2–8 above expand as follows; the rest are new here.

### Public APIs that cannot ever have worked

- **`write_string`** (`src/client/string.rs:1014`): request builder omits the path-size word count, and the success check reads `cip_response[0]` — the service-reply byte (`0xCD` on success) — as the status, so even an accepted write reports `WriteError { status: 0xCD }`. (The commonly used path, `write_string_tag` → `write_tag(PlcValue::String)`, is separate and unaffected.)
- **`write_ab_string_udt`** (`string.rs:121`): checks `response[2]` of the **raw CPF envelope** (always 0 — part of the interface handle) instead of extracting the CIP reply, so it returns `Ok(())` unconditionally regardless of what the PLC said. Silent false success.
- **Connected messaging / Forward Open** (`string.rs:190, 346`): the response is parsed at the wrong layer (raw CPF bytes, expects `response[0] == 0xD4`) so every Forward Open fails after 6 attempts × 100 ms; even with extraction fixed, status and connection IDs are read at wrong offsets. The entire connected-session subsystem is dead on arrival.
- **`read_udt_member_by_offset` / `write_udt_member_by_offset`** (`src/client.rs:2045, 2104`): index into the full CIP reply envelope rather than the UDT payload — offset 0 lands on the service byte; the write path round-trips envelope bytes back to the PLC as tag data.
- **`read_udt_chunked` strategy ladder** (`src/client.rs:1719–2021`): `read_udt_chunk_advanced` emits protocol-invalid requests and returns CIP error/status bytes as "data" without checking status; two of the loops (`read_udt_with_chunk_size`, `read_udt_progressive`) never terminate against a peer that keeps answering full-size chunks (network-paced unbounded growth); strategy D converts total failure into `Ok(Udt { data: vec![] })`.
- **`discover_program_tags`** (`src/client.rs:2678`): ignores its `program_name` parameter entirely and queries the Template object class — returns wrong data by construction.
- **`Tag.DATA[i]` paths** (`crates/tag-path/src/lib.rs:307`): the `StringData` arm emits a malformed 8-bit element segment (`0x28, 0x04, u32`) — the four index bytes trail as garbage segments.

### Wire-format and addressing bugs in working paths

- **Batch BOOL array element reads** (`src/client/batch_exec.rs:687`): the request addresses element `i` directly but the reply handler extracts bit `i % 32` — nothing divides by 32. The non-batch path (`read_bool_array_element_workaround`, `client.rs:1096`) does this correctly; the batch path was never given the same workaround.
- **UDT template parsing** (`crates/udt/src/lib.rs:196, 605`): empty member-name strings (consecutive NULs, legal in real templates) shift every subsequent name onto the wrong member; BOOL members ignore the `info` field's bit index, so multiple BOOLs packed in one host byte all read as the same value.
- **`register_session`** (`src/client.rs:378`): assumes the 28-byte reply arrives in a single `read()` — fragmented replies fail registration spuriously. It also takes the stream lock separately for write and read (unlike `send_rr_data_item`, which correctly holds one guard across the transaction), so a concurrent request from another clone can interleave into the registration exchange.
- **Length truncation**: `build_unconnected_send` (`client.rs:3524`) casts `embedded_message.len() as u16` unchecked; several manual builders cast path sizes with `as u8`. `CipRequest::validate` does this correctly — another argument for routing everything through it.
- **`RoutePath` extended segments** (`src/route.rs:171`): >254-char link addresses silently saturate a u8; ports >15 are silently masked instead of using the extended-port form.
- **`parse_extended_error`** (`src/client.rs:3180`): treats general status `0xFF` as an "extended error indicator" (not a CIP concept) and tries both endiannesses of the code — messages only, but misleads debugging.

### Dead configuration and dead state

- `BatchConfig.max_packet_size` / `packet_timeout_ms` are public, documented, and never enforced; the negotiated `max_packet_size` on the client is stored and never consulted by request sizing. Packet-size negotiation is dead state.
- `optimize_packet_packing: true` (default) reorders all reads before all writes — a batch `[Write X, Read X]` reads the stale value. Documented, but a surprising default for a PLC library.
- `read_udt_chunked` dispatches on `msg.contains("Partial transfer")` — stringly-typed matching on the library's own error wording.
- Dead code kept via `_`-prefix (`_build_ab_string_write_request`, `_get_connected_session`) and two `unreachable!` in `batch_exec.rs` — against the repo's own discipline (delete, or `#[expect]` with reason).
- Every array-element read/write does a preliminary probe read of the base tag to sniff BOOL-ness, doubling round trips; the result is never cached (perf, not correctness).

## 2. Concurrency, lifecycle, and session state

- **Stream desync after timeout** (`src/client.rs:3610`): a timed-out request leaves its response unread on the socket; the next request consumes it — response N attributed to request N+1. Nothing invalidates the stream, `is_retriable()` encourages retrying on the same client, and the encap `sender_context` is constant and never verified on receive, so the mismatch is undetectable. `send_connected_cip_request` (`string.rs:603`) additionally has **no timeout at all**. The single highest-leverage transport fix: a wrapper that owns timeout → invalidate/reconnect semantics plus sender-context correlation.
- **`session_handle` divergence across clones** (`src/client/diagnostics.rs:32` vs the documented field contract at `client.rs:234`): keep-alive failure triggers `register_session()` on one clone, giving it a new handle while all other clones (subscription pollers, FFI registry, tag-group tasks) keep sending the old one on the shared stream. Violates the crate's own "never mutate post-insert" invariant. Moving `session_handle` behind `Arc<AtomicU32>` fixes this and deletes the FFI `store_client` pattern (§3).
- **Subscription lifecycle** (`src/client/subscriptions.rs:38`, `src/subscription.rs:41`): `stop()` only silences notifications — the spawned poll task keeps issuing TCP reads forever (the tag-group path checks `is_active()`; the single-tag path doesn't). The task `break`s permanently on the first transient read error with no error event to the consumer. Because the client's `Vec<TagSubscription>` retains a receiver clone, the channel can never disconnect: an abandoned subscription fills its 100-slot buffer and the poll task blocks in `send().await` **forever** — one leaked, deadlocked task per abandoned subscription. Same pattern in `TagGroupSubscription::publish_event` (64-slot buffer). The subscription Vec itself grows unboundedly (no removal API).
- **Fleet event forwarding** (`src/fleet.rs:102`): `while let Ok(event) = recv().await` exits on `RecvError::Lagged` — a recoverable condition — so one event burst permanently kills fleet-level forwarding for that PLC. Replacing a PLC id leaves the old forwarding task alive, emitting under the same id.
- **`Client::events()`** (`src/client/actor.rs:153`): sends a spurious `Connected` to all existing subscribers as a side effect of subscribing; `Disconnected` only fires at actor shutdown — there is no reconnect/health loop behind the event vocabulary.
- **`monitoring::start_monitoring`** (`src/monitoring.rs:506`): each call leaks an unstoppable 30 s-interval task pinning the metrics `Arc` forever.

## 3. FFI layer and language bindings

The handle-based design itself is sound and well-tested (opaque `i32` ids, mutex-guarded registry, clone-per-call with documented shared-vs-copied fields, ABI version/capability handshake, paired alloc/free). The defects are at the edges:

### Rust FFI (`src/ffi.rs`)

- **Raw `*mut EipClient` on the public ABI** (`ffi.rs:2176, 2305, 2405`): three exported symbols take raw client pointers no external caller can legitimately produce (handles are ids). They exist only to serve the `_by_id` wrappers and should be private — as exported symbols they invite arbitrary-deref UB.
- **`store_client` resurrection race** (`ffi.rs:115, 1447, 1486`): clone → await → unconditional re-insert. If `eip_disconnect` lands between clone and store, the removed client is re-inserted under an id nothing will ever remove — a live TCP connection leaked for process lifetime. Practically reachable because the C# keep-alive calls `eip_check_health_detailed` concurrently with `Dispose()`.
- **No `catch_unwind` at the boundary**: a panic escaping `extern "C"` aborts the host .NET/Python process. The stated mitigation ("no panics in the crate") is convention, not enforcement — a `catch_unwind` shim in the dispatch macro is cheap insurance.
- **`#![allow(clippy::missing_safety_doc)]` + pervasive missing `// SAFETY:` comments** — directly contradicts `docs/agents/notes/ffi-safety.md` ("not appropriate here") and CLAUDE.md's non-negotiable SAFETY rule. No unsoundness found in the blocks themselves (null checks are consistent), but the file violates its own review contract wholesale.
- **Last-error registry** (`ffi.rs:23`): entries never removed (unbounded across connect cycles), never cleared on success, and many failure paths never set an error — so `eip_get_last_error` frequently returns a stale message from an earlier unrelated failure, which the C# wrapper then attaches to the current exception.
- **`eip_write_udt` degenerate fallback** (`ffi.rs:1354`): when HashMap→UdtData conversion fails, it proceeds with `UdtData { symbol_id: 0, data: vec![] }` — a real write of zero data bytes to the PLC instead of returning −1.
- **`eip_read_string` ASCII-scan heuristic** (`ffi.rs:963`): for non-STRING UDTs, scans for any printable run and returns it as a *successful* string read. The Python binding's equivalent decode is documented and careful; the Rust FFI path predates that rigor.
- Smaller: `eip_discover_tags` returns success while doing nothing; `eip_discover_tags_detailed` leaves `tag_count` set on malloc failure (UB for a caller that doesn't zero-init the out-struct); id wraparound lacks an occupancy check; `eip_disconnect` never calls `unregister_session` (TCP FIN only).
- **Implicit JSON ABI**: `PlcValue`'s serde shape is the de-facto wire format for tag/batch results across both wrappers, but `eip_abi_version` covers only the C surface — renaming a variant silently breaks both bindings with no version bump.

### C# wrapper (`csharp/RustEtherNetIp/`)

- **`WriteUdtMember` guaranteed deadlock** (`EthernetNetIpClient.cs:1525`) — confirmed by direct read: the method body runs inside `ExecuteWithLock` and calls `ReadUdt`/`WriteUdt`, which each take the same non-reentrant `SemaphoreSlim(1,1)`. Every real invocation hangs the calling thread permanently. Survives CI because the test suite exercises it only through Moq mocks. The neighboring `ReadUdtWithChunkedFallback` even documents the correct pattern ("called from within ExecuteWithLock, so we don't need another lock").
- **ANSI marshalling** (`StringToHGlobalAnsi`/`PtrToStringAnsi` throughout): Rust requires UTF-8 in and produces UTF-8 out. On Windows, "Ansi" is the active codepage — non-ASCII strings are rejected or silently mis-encoded. The codebase already uses `PtrToStringUTF8` in one place (`Infrastructure.cs:80`); all marshalling should move to the UTF-8 APIs.
- **Keep-alive bypasses the operation lock** (`Connection.cs:126`): runs concurrently with user operations on the same handle (triggering the Rust-side register-interleave and `store_client` races) and on failure swaps `_clientId` under a different lock than the one operations hold — spurious failures mid-flight.
- **NuGet packaging in the csproj** ships the build-OS native library via `Content` items rather than `runtimes/{rid}/native/` — a locally produced package misses cross-platform native resolution. (The *release pipeline* builds the multi-RID package correctly; this affects local/CI-produced packages.)
- Connect failures carry no diagnostics (last-error is keyed by client id, which doesn't exist for failed connects; the −2 runtime-init code is defined but never surfaced). Async layer parks a pool thread per queued op in a synchronous `Wait()`; one sync-over-async on the subscribe path. `Dispose` is mostly correct (proper pattern, finalizer, double-dispose guard) with minor races noted.
- **No test exercises the real P/Invoke path** — the root cause of items 1–2 surviving. A small simulator-backed C# integration test project would have caught both.

### Python binding (`python/`)

The cleanest surface reviewed: ctypes signatures cross-check function-by-function against `ffi.rs`, UTF-8 everywhere, out-pointers freed in `finally`, careful documented STRING-vs-UDT decode, value-range validation before hitting the FFI. Residuals: no `__del__`/finalizer backstop (a GC'd client leaks the native connection), `disconnect()` wedges the object as "connected" if the native call fails, and the library-search loop aborts on the first ABI-mismatched candidate instead of trying the bundled one.

## 4. Dead and aspirational modules (grep-verified)

An older "enterprise checklist" stratum is publicly exported from `lib.rs` but wired to nothing:

| Export | State |
|---|---|
| `ProductionMonitor` (`monitoring.rs`) | Never instantiated by `EipClient` or FFI. `get_memory_usage`/`get_cpu_usage` return hardcoded `10.0`/`5.0`. Meanwhile `eip_get_diagnostics_json` hand-builds a snapshot with **every operation counter hardcoded to zero** (`client/diagnostics.rs:86`) — consumers can mistake "0 failed reads" for health. ROADMAP item 9 acknowledges this. |
| `ProductionConfig` + all of `config.rs` | ~380 lines of knobs (`SecurityConfig` with encryption/rate-limiting, `LoggingConfig`, `MemoryLimits`) consumed by nothing. Also: `Duration` fields serialize as `{secs, nanos}` tables, so a natural hand-written TOML config fails to parse; no test covers `from_file`. |
| `SubscriptionManager` (+ alias `RealTimeSubscriptionManager`) | Zero callers; `EipClient` re-implements its fan-out inline. |
| `TagCache` | `#[allow(dead_code)]` *and* publicly exported — suppressed and shipped. |
| `PlcManager` | Used only by tests; predates `Fleet`. Its health lifecycle is unreachable (`update_health` is the only setter of `is_active = false` and has zero callers), and `get_connection` returns `&mut self`-borrowing references so the "pool" can only ever use one connection at a time. |
| `TagManager` UDT pipeline | The dangerous one: `discover_udt_members` (public, on `EipClient`) builds a request with an off-by-one path size (`2 + div_ceil` where the emitted path is `1 + div_ceil` words — pinned wrong by its own unit tests at `tag_manager.rs:1046`), then "parses" the response by scanning for byte pairs that look like type codes, inventing `Member_1…` names and sequential offsets, falling back to a fabricated `Value: DINT` member. Real UDT parsing lives in `udt.rs`/`get_udt_definition`. This path returns invented data as authoritative. |
| `TagManager` discovery helpers | `validate_tag_name` rejects `_`-prefixed, `Program:`-scoped, and bracketed names (all legal), silently dropping them from discovery; `is_structure()` disagrees with its own type parser (checks `0x00A0..=0x00AF`, real handles surface as `0x02A0`), so UDT drill-down essentially never fires; array metadata reports 0 elements for every array; the cache never self-evicts. |

**Recommendation:** a deliberate 1.2 deprecation pass — delete or `#[deprecated]`-quarantine `TagCache`, `SubscriptionManager`, `PlcManager`, `ProductionConfig`, `ProductionMonitor`, and the `TagManager` UDT parser; fold monitoring into the one diagnostics path that is real. This aligns with (and extends) ROADMAP items 5–9 and the 2.0 dead-surface removals.

## 5. Tests, simulator, and benchmarks

- **The in-process simulator (`tests/plc_sim.rs`) is genuinely good** — real encapsulation framing, Unconnected Send unwrapping, all 13 types, DWORD-packed BOOL arrays, Multiple Service Packet with correct offset tables, error injection. The 13 sim tests are real protocol coverage.
- **But it is client-derived, not spec-derived**, in at least one load-bearing place: it injects a 2-byte element count into multi-element read replies (comment at `tests/plc_sim.rs:733`) specifically so the client's offset-8 chunk parse works — real Read Tag replies put data at offset 6. This converts the sim suite from oracle into mirror: `read_array_range` passes green against the sim and shifts every chunk by 2 bytes against real hardware.
- **Two pinned byte tests pin incorrect encodings** (UDT write type at `crates/protocol/src/tests.rs:124`; the tag_manager path size at `tag_manager.rs:1046`). The pinned-test mechanism is right; the pins need re-derivation from 1756-PM020 rather than from the implementation.
- Sim fidelity gaps worth closing: session handles never validated, route-path bytes ignored (so route encoding isn't actually verified), multi-dimensional indices collapse to the last index, no fragmented-read service.
- **Two hand-maintained simulators drift**: `src/bin/plc_sim.rs` is a stale ~40% copy (4 types, no batch/MSP, unknown reads *succeed* with `Dint(0)`, unknown writes silently insert) yet advertises itself for C# tests. Collapse to one source.
- Test-convention split: `SKIP_PLC_TESTS` env-skip (~5 files) vs `#[ignore]` (~7 files); CLAUDE.md says "most" use the env var — wrong. Some `#[ignore]` tests return early on connection failure, i.e. can pass without asserting (ROADMAP item 11 acknowledges).
- `tests/unit/element_addressing_tests.rs` is never compiled (nothing includes `tests/unit/`) and has diverged from its top-level twin. `test_isolated/` at the repo root is unreferenced scratch. `tests/release_readiness_tests.sh` is the only shell suite not run in CI.
- `benches/performance_benchmark.rs` measures the real codec hot path. `benches/udt_discovery_benchmark.rs` is largely fake — every body re-implements the logic inline (hand-written `contains` checks, string formatting into HashMaps) and calls no crate code, so regressions in the real functions are invisible.

## 6. Project health, CI, and documentation

**Verified healthy:** version parity across `VERSION`/Cargo manifests/pyproject/csproj/`version.rs` (release-readiness script passes locally); MSRV 1.96 real and CI-enforced; `build.rs` tarball-safe; `audit.toml` ignores scoped and documented; release automation genuinely complete (multi-RID NuGet, ordered 5-crate crates.io publish, manylinux + auditwheel + blocking install-smoke gate).

**Issues:**

- **semver gate is toothless** (`ci.yml`): `cargo semver-checks --baseline-version 0.7.0` — a 0.x→1.x baseline permits any breaking change. Re-point to 1.1.0.
- **CLAUDE.md drift (high, because it is agent-load-bearing):** build section omits `--features ffi` (the exact mistake that caused past `EntryPointNotFound` CI failures); architecture section describes the pre-CODEX-U tree (`protocol/{encap,cip,values,tests}.rs` under `src/`, "client.rs ~6.7k lines", 2-member workspace) — reality is 1-line shims over four published `crates/*`, a 4.2k-line `client.rs` plus 7 submodules, and a 6-member workspace; the module/type tables omit `fleet.rs`, `schema.rs`, and the entire actor-based `Client`/`RetryClient` API (the 1.0 headline), while still recommending the `Arc<Mutex<EipClient>>` pattern the actor was built to replace; the CI paragraph undersells actual coverage; the `SKIP_PLC_TESTS` claim is wrong. `docs/agents/notes/cip-framing.md` also still points at `src/protocol/`. ROADMAP item 1 (doc refresh) already flags this as top priority — this analysis corroborates it.
- **README** claims the NuGet package bundles `osx-x64` — false; release.yml intentionally omits it (3 RIDs ship). Quick Start teaches the legacy `RoutePath::new().add_slot()` builder slated for deprecation.
- **`Cargo.toml`:** `vergen` is listed under `[dependencies]` (runtime) as well as `[build-dependencies]` — every downstream consumer compiles it for nothing. `libc` is unconditional but only used behind the `ffi` feature (make it `optional = true`, `ffi = ["dep:libc"]`). `authors` carries a placeholder email (`sergio@example.com`) in published crates.io metadata.
- **release.yml:** `cargo publish || echo "...continuing"` swallows every failure mode, not just already-published — a genuinely failed sibling publish leaves the tag half-published with a green job. Discriminate the "already exists" error.
- cargo-audit has no scheduled run (new advisories unnoticed between commits); no cargo-deny (ROADMAP item 12); Python matrix stops at 3.12 with open-ended `requires-python`.
- `version.rs` hardcodes `MAJOR/MINOR/PATCH` (parity-checked, but hand-synced) and its doc comment still says "v0.1.0" format.
- Outstanding from the log: registry tokens pasted in maintainer chat on 2026-06-19; rotation advised and deliberately deferred — still open.

---

## Recommended sequence

1. **Hotfix tier (small, high value):** C# `WriteUdtMember` deadlock; C# UTF-8 marshalling; semver-checks baseline; `vergen`/`libc` dependency hygiene; CLAUDE.md build-command fix; README osx-x64 claim; delete `tests/unit/` dead file and `test_isolated/`.
2. **Transport hardening brief:** timeout → stream invalidation/reconnect + sender-context correlation in `send_rr_data_item`; `session_handle` behind `Arc<AtomicU32>` (also deletes the FFI `store_client` race); single-lock `register_session`; timeout on connected reads.
3. **Hardware-capture campaign** before touching UDT paths: capture real Read/Write Tag exchanges for a struct tag, `read_array_range`, and Get Attribute List — then fix F-UDT encoding (type marker + handle), the response-handle stripping, `get_tag_attributes`, and `read_array_range`, re-pinning the byte tests from the captures. Make the simulator spec-derived at the same time so it becomes an oracle.
4. **String/UDT graveyard cleanup:** delete the never-worked paths (`write_string`, `write_ab_string_udt`, connected messaging, chunking strategies B–D, `*_by_offset`) or rewrite them through `CipRequest`/`CipResponse`; one documented workaround per firmware quirk, per `ab-firmware-quirks.md`.
5. **Dead-stratum deprecation pass** (§4) aligned with ROADMAP 5–9 and the 2.0 removals.
6. **Subscription/fleet lifecycle brief:** `stop()` actually stops, bounded channels with drop-oldest or `try_send`, `Lagged` handled as recoverable, subscription eviction.
7. **FFI polish:** privatize raw-pointer exports, `catch_unwind` shim, SAFETY-comment pass to bring `ffi.rs` in line with `ffi-safety.md`, last-error lifecycle (clear on success, remove on disconnect, set on all failure paths).
8. **C# native integration tests** against the simulator in CI — the single change most likely to prevent the next `WriteUdtMember`-class escape.

---

*Findings in this document were produced by four parallel review passes and individually cite file:line at `main` @ `47b99a4`. Every critical/high item was re-verified by direct source read during synthesis. Wire-format items marked (hardware-confirm) should be validated with a packet capture before code changes land, per the maintainer's hardware-validation role.*
