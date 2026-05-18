# Architecture review — post-books synthesis

**Date:** 2026-05-18
**Authors:** claude [Opus 4.7] — original review · codex [gpt-5.5] — independent second pass
**Trigger:** Maintainer-driven study of five reference texts staged locally in `wiki/books/` (gitignored): *Clean Architecture* (Martin), *Designing Data-Intensive Applications* (Kleppmann), *Understanding Distributed Systems* 2e (Vitillo), *Patterns of Enterprise Application Architecture* (Fowler), *Rust for Network Programming and Automation* (Anderson).

This page is the durable artifact of that review. It supersedes any in-chat synthesis and is the source of truth that the new task briefs (CODEX-L through CODEX-U) reference.

## Scope

The library has grown from a 2k-line prototype into a 17k-line cross-language SDK (Rust crate, NuGet, PyPI) with two wrappers, a real-PLC test matrix, a simulator suite, and an internal two-agent collaboration workflow. The architecture review is the first formal pass on whether the *current shape* will survive a 1.0 cut and the next 2–3 years of evolution.

## Findings — reconciled

The two reviewers disagreed on three points. The reconciled position records what survived the second pass.

| # | Claim | Position | Notes |
|---|---|---|---|
| 1 | `EipClient` is too large (6,762 LOC, ≥ 7 responsibilities) | **Confirmed** | Tracked by the existing CODEX-J agenda entry; must stay facade-preserving. |
| 2 | Public error type is over-flat and leaks CIP-level concerns | **Confirmed** | Tracked by CODEX-K (release-window bundle); collapse the four near-duplicate STRING variants, layer the rest. |
| 3 | "Wrap me in `Arc<Mutex<EipClient>>`" is the wrong default | **Confirmed** | Drives CODEX-P (request-correlator actor). Treat as behaviorally breaking, *not* internal-refactor. |
| 4 | Introduce a public `Transport` trait | **Withdrawn** | `EtherNetIpStream` already exists at `src/lib.rs:69-102`. The genuine gap is at the request/response/correlation layer, not at the stream layer. |
| 5 | Service Layer for firmware-limit workarounds | **Confirmed**, scope tightened | Drives CODEX-Q. Stay concrete to STRING / UDT-array-member-write; do not generalize. |
| 6 | `#[non_exhaustive]` on public enums | **Confirmed** | Already part of CODEX-K's release-window bundle. |
| 7 | FFI ABI versioning | **Confirmed**, priority elevated | Drives CODEX-L. Moved to Phase 0 (was Phase 2 in original review). |
| 8 | FFI registry clone semantics are dangerous | **New finding** (codex pass) | Drives CODEX-M. Concrete correctness bug, not just ABI risk. |
| 9 | CIP path encoding silently truncates | **New finding** (codex pass) | Drives CODEX-N. Supersedes the `debug_assert!` mention in the existing CODEX-H agenda entry — use hard validation, not a dev-build assertion. |
| 10 | `PlcValue::Udt::get_data_type()` returns placeholder `0x00A0` | **New finding** (codex pass) | Drives CODEX-O. Verify placeholder never escapes through FFI as a misleading real CIP type. |

## Phase plan

The original four-phase ordering was wrong about Phase 0. Codex's second pass corrected it: **ABI versioning and compatibility tests come before any internal refactor**, because the FFI symbol table is the contract the C# and Python wrappers (and any future binding) depend on. The reconciled plan:

### Phase 0 — Contract first (non-breaking; can ship in patch releases)

These do not change the public Rust API and do not break the FFI symbol table. They establish the contract that protects every subsequent phase.

| Id | Title | Status | File |
|---|---|---|---|
| CODEX-L | FFI ABI version + capability handshake | open | [`tasks/CODEX-L-ffi-abi-version-handshake.md`](../../docs/agents/tasks/CODEX-L-ffi-abi-version-handshake.md) |
| CODEX-M | FFI registry clone-semantics audit and fix | open | [`tasks/CODEX-M-ffi-registry-clone-audit.md`](../../docs/agents/tasks/CODEX-M-ffi-registry-clone-audit.md) |
| CODEX-N | CIP path encoding hard validation | open | [`tasks/CODEX-N-cip-path-encoding-validation.md`](../../docs/agents/tasks/CODEX-N-cip-path-encoding-validation.md) |
| CODEX-O | `PlcValue::Udt::get_data_type` placeholder honesty | roadmap | (brief on activation) |

### Phase 1 — Mechanical internal split (facade-preserving)

| Id | Title | Status | Source |
|---|---|---|---|
| CODEX-J | Sub-split `client.rs` into 6–8 submodules | roadmap | Existing agenda entry; activates after Phase 0 lands. |

The split is mechanical — no semantic change, no behavior change, no public API change. The `EipClient` facade preserves every existing method signature. This is the lowest-risk piece of work in the entire plan.

### Phase 2 — Behavioral refactors (semver-meaningful)

These change observable behavior (request ordering, cancellation, clone semantics, event surface) even when method signatures are preserved. Each requires its own wrapper-level compatibility test pass.

| Id | Title | Status | File |
|---|---|---|---|
| CODEX-P | Request-correlator actor + cloneable `Client` handle | roadmap | (brief on activation) |
| CODEX-R | `Client::events()` connection state stream | roadmap | Depends on CODEX-P. |

### Phase 3 — 1.0 API cleanup (single bundled SemVer-major)

| Id | Title | Status | Source |
|---|---|---|---|
| CODEX-K | Release-window bundle (`#[non_exhaustive]`, error consolidation, `RoutePath` private storage, typed `try_init_tracing`, internal-type demotion, FFI ordered-hop shape, wrapper sync) | roadmap | Existing agenda entry; already comprehensive. |
| CODEX-Q | Service Layer for restricted writes (STRING / UDT-array-member) | roadmap | (brief on activation) |
| CODEX-S | `RetryPolicy` primitive + decorator combinator | roadmap | (brief on activation) |

All Phase 3 items are bundled into the 1.0.0 release window so the breakage happens once, cleanly.

### Phase 4 — Scale and extensibility

| Id | Title | Status | Source |
|---|---|---|---|
| CODEX-T | `Fleet<PlcId, Client>` for multi-PLC deployments | roadmap | (brief on activation) |
| CODEX-U | Promote `protocol`, `tag_path`, `udt` to sibling workspace crates | roadmap | (brief on activation) |

## Activation gate

Per maintainer standing direction (2026-05-17, recorded in `board.md`), all post-0.8.0 work is held pending real-hardware validation of CODEX-F (ethernet routing) and the v0.8.0 release tag. The three Phase 0 briefs (CODEX-L, M, N) are authored at `status: open` so they are ready to run, but should not be activated until that gate passes. CODEX-L specifically should run *first* of the three — establishing the ABI baseline before the FFI clone audit (CODEX-M) potentially restructures FFI internals.

## What was deliberately not included

- A general `Transport` trait at the public surface. `EtherNetIpStream` already exists; the real seam is at request/response framing, which the actor brief (CODEX-P) addresses internally.
- A deep error hierarchy (`Transport(...)`, `Protocol(...)`, `Cip(...)`, `Semantic(...)`, `Encoding(...)`). The codex pass argued, correctly, that wrappers benefit from a *flatter* error API with structured CIP details — not a layered taxonomy for its own sake. CODEX-K's existing consolidation plan is the right shape.
- A full Clean Architecture rewrite. This is a protocol client, not an enterprise application. The valuable boundaries are codec, transport/session, tag services, UDT/string policy, and wrappers — and most of those are already extracted.
- A "support any industrial protocol" generalization. The library is EtherNet/IP; stay narrow.
- A DSL for tag paths. The current `tag_path` parser is fine.

## Reference

The books contributed the following concepts that *actually* shaped this plan (not generic summaries):

- **Clean Architecture** — Dependency Rule applied to `monitoring`/`subscription`/`tag_group`/`schema` taking traits not concrete `EipClient` (consequence: CODEX-J split should define those capability traits in the inner core).
- **DDIA** — Schema evolution applied to `PlcValue` (consequence: `#[non_exhaustive]` in CODEX-K). Reliability triad framing for prioritization.
- **Understanding Distributed Systems** — Retry/idempotency framing (CODEX-S). State observability (CODEX-R). The client/PLC pair *is* a distributed system; failure modes are real.
- **PEAA** — Service Layer pattern (CODEX-Q). Repository/Gateway already present in the codec extraction; no new work.
- **Rust for Network Programming and Automation** — Cancellation as a silent reliability bug (motivates CODEX-P).
