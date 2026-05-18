# Engineering lessons from the 2026-05-18 reference reading

**Date:** 2026-05-18
**Sources (kept locally in `wiki/books/`, gitignored):**
- *Clean Architecture: A Craftsman's Guide to Software Structure and Design* — Robert C. Martin
- *Designing Data-Intensive Applications* — Martin Kleppmann
- *Understanding Distributed Systems* (2nd ed) — Roberto Vitillo
- *Patterns of Enterprise Application Architecture* — Martin Fowler
- *Rust for Network Programming and Automation* — Brian Anderson

This document captures the highest-value engineering lessons from each book *as they apply to this library*. It is intentionally not a generic summary. Each lesson lists:

- **What** the concept is, in one or two sentences.
- **Why** it matters in real-world systems software.
- **How** it binds to this codebase (with `file:line` citations where applicable).
- **Tracked by** which CODEX brief (or "no brief yet" with a recommendation).

The architectural decisions and roadmap derived from these lessons live in [`architecture-review-2026-05-18.md`](architecture-review-2026-05-18.md). This document is the *why* behind those decisions.

---

## Clean Architecture (Robert C. Martin)

### 1. The Dependency Rule

**What:** Source code dependencies point inward, toward higher-level policy. Nothing in an inner circle can know anything about an outer circle.

**Why it matters:** Inversion lets the inner core be tested, reasoned about, and evolved without dragging the outer infrastructure along. Changes ripple outward, not inward. Without it, a transport-layer change forces every business rule to recompile and re-test.

**How it binds here:** `monitoring.rs`, `subscription.rs`, `tag_group.rs`, and `schema.rs` all depend directly on the concrete `EipClient` type (`src/client.rs:225`). They live "outside" the core but reach inward to a concrete struct, not a trait. That's the dependency rule running backwards — the *outer* modules should define the capability traits they need (`TagReader`, `TagWriter`, `BatchExecutor`) and the *inner* `EipClient` should implement them.

**Tracked by:** CODEX-J (mechanical split). The split should also extract these capability traits to the inner core.

### 2. Single Responsibility (class-level, not module-level)

**What:** A class should have one reason to change. The "reason" is a stakeholder group, not a code concern.

**Why it matters:** When a single class serves multiple stakeholders, every stakeholder's change collides with every other stakeholder's change. Merge conflicts, reasoning cost, and onboarding cliffs compound.

**How it binds here:** `EipClient` (`src/client.rs`, 6,762 lines) owns transport, session, CIP framing, tag I/O, batch coordination, UDT discovery, subscriptions, schema export, monitoring integration, and FFI runtime hookup. That is at least seven distinct change reasons. Every PR touches the same file.

**Tracked by:** CODEX-J.

### 3. Component principles — REP, CCP, CRP

**What:**
- **REP** (Release Equivalence Principle): The granule of reuse is the granule of release.
- **CCP** (Common Closure Principle): Classes that change together belong together.
- **CRP** (Common Reuse Principle): Don't force consumers to depend on what they don't use.

**Why it matters:** Decides what is a separate library, what is a separate module, and what is just a separate file. Wrong granularity is invisible until you ship — then it's the wrong unit of dependency hell.

**How it binds here:** `tag_path.rs` (811 LOC) parses a domain-specific syntax with zero coupling to the network. `udt.rs` (950 LOC) models a structural type system. `protocol/` (cip + encap + values, ~825 LOC) is the wire codec. These are the natural separate-crate boundaries. A consumer building an offline analysis tool over `udt` should not have to pull in `tokio`.

**Tracked by:** CODEX-U (promote `protocol`, `tag_path`, `udt` to sibling workspace crates — post-1.0).

### 4. Stable Abstractions Principle

**What:** Stable components (depended on heavily, hard to change) should be abstract. Volatile components (change often) should be concrete.

**Why it matters:** A stable *concrete* component rigidifies the entire dependency graph rooted at it. Every change is a breaking change for everyone downstream.

**How it binds here:** `EipClient` is maximally stable (depended on by `monitoring`, `subscription`, `tag_group`, `schema`, the FFI layer, the C# and Python wrappers, every downstream consumer) and maximally concrete (no abstraction; concrete struct with concrete methods). That is the worst quadrant.

**Tracked by:** CODEX-P (request-correlator actor + cloneable `Client` handle). The handle is the abstraction; `EipClient` retreats behind it.

### 5. Boundaries and use cases via dependency injection

**What:** Cross-boundary calls follow the dependency rule via dependency injection — the outer ring is given to the inner core by a composition root.

**Why it matters:** Lets you swap the outer ring (transport, persistence, presentation) for testing or for adaptation without touching the inner core.

**How it binds here:** `connect_with_stream` (`src/client.rs`, accessed via `EtherNetIpStream` at `src/lib.rs:69-102`) is exactly this pattern at the stream boundary. Confirms the public API for swapping the transport already exists; the gap is at the request/response/correlation layer, *not* at the stream layer.

**Tracked by:** CODEX-P (the actor is the request-correlation abstraction over the existing stream injection point).

---

## Designing Data-Intensive Applications (Martin Kleppmann)

### 1. Reliability / Scalability / Maintainability as the load-bearing triad

**What:** Three axes by which to evaluate any data system. Trade-offs among them are explicit; trade-offs *within* them have to be measured.

**Why it matters:** Without naming the axes, optimization drifts to whichever one is currently painful. Naming them gives the discussion vocabulary.

**How it binds here:** Today's posture, honestly:
- **Reliability:** mixed. Good retries-via-`is_retriable` (`src/error.rs:104`), weak connection-state model (no event stream; consumers learn about loss only via `ConnectionLost` errors), no automatic reconnect.
- **Scalability:** unmeasured. No connection pool, single-PLC focus, no per-PLC backpressure, no benchmarks at fleet scale.
- **Maintainability:** poor. The god-object dominates.

**Tracked by:** Reliability → CODEX-R (events stream), CODEX-S (retry policy). Scalability → CODEX-T (fleet). Maintainability → CODEX-J (split).

### 2. Encoding and schema evolution

**What:** Library APIs are schemas. Every change is a schema change. The discipline of forward and backward compatibility applies.

**Why it matters:** Without `#[non_exhaustive]` and similar opt-outs, adding a single enum variant breaks every downstream exhaustive `match`. That's a major version bump for what feels like a minor feature.

**How it binds here:** `PlcValue` has 13 variants, no `#[non_exhaustive]` (`src/types.rs:219, 325`). `EtherNetIpError` has 22 variants, no `#[non_exhaustive]` (`src/error.rs:11`). `RouteHop` (`src/route.rs`), `TagPath`, `HealthStatus`, `HealthCheckMode`, `ErrorCategory`, `TagGroupEventKind`, `TagGroupFailureCategory` — all exhaustively matchable. Adding a 14th AB type is currently a breaking change.

**Tracked by:** CODEX-K (release-window bundle includes the `#[non_exhaustive]` sweep).

### 3. End-to-end argument applied to errors

**What:** Errors at intermediate layers should not pretend to be errors at outer layers. Each layer's failure modes are distinct and the public type should reflect that.

**Why it matters:** A flat error enum that mixes "TCP closed" with "tag not found" with "string too long" forces every consumer to handle every variant defensively, even when 90% of them cannot occur in a given code path.

**How it binds here:** `EtherNetIpError` (`src/error.rs:11-102`) mixes transport (`Io`), protocol (`Protocol`), CIP-level (`CipError`, `StringWriteError`, `StringReadError`, `InvalidStringResponse`, `StringTooLong`), and semantic (`TagNotFound`, `Permission`, `WriteError`, `ReadError`) all at the same level. The four near-duplicate STRING variants are the clearest noise.

**Counter-position** (from the codex second-pass review, accepted): don't explode this into a deep hierarchy. The right shape is *flatter* with structured CIP details (a `CipError { code, additional, message }` carrying the real ODVA status code), not a multi-layer error taxonomy.

**Tracked by:** CODEX-K (collapses the duplicates; introduces structured CIP detail).

### 4. Idempotency for retry safety

**What:** Retry is safe only for idempotent operations. A read is idempotent; a write generally is not.

**Why it matters:** Naive retry on a write that succeeded but whose ack was lost applies the write twice. For a PLC tag whose value is incremented, that's a real correctness bug.

**How it binds here:** `is_retriable()` (`src/error.rs:104`) returns `bool` without distinguishing read-retry from write-retry. Today every consumer's retry loop has the same correctness hole.

**Tracked by:** CODEX-S (`RetryPolicy` primitive). Default behavior: retry reads, do not retry writes unless caller explicitly opts in (and document the at-most-once-vs-at-least-once trade-off).

### 5. Batch vs stream processing

**What:** Two distinct latency/throughput trade-offs. Batch is throughput-optimized with explicit completion. Stream is latency-optimized with continuous output.

**Why it matters:** Conflating them makes APIs hard to reason about and benchmark against. Consumers don't know which they're getting.

**How it binds here:** `BatchOperation` (`src/batch.rs`) is the batch model — explicit, transactional in intent, returns per-op results. `tag_group` polling (`src/tag_group.rs`, 275 LOC) is *closer to* the stream model — periodic poll, event emission, change detection — but isn't called that. Naming and documenting these models distinctly clarifies the consumer mental model.

**Tracked by:** No dedicated brief; CODEX-J split is the natural place to rename `tag_group` to `tag_stream` or similar if the bigger restructure agrees.

---

## Understanding Distributed Systems, 2nd edition (Roberto Vitillo)

### 1. The client/PLC pair IS a distributed system

**What:** N=2 nodes still suffer every distributed pathology: partition, partial failure, omission, byzantine behavior.

**Why it matters:** A library author who thinks of the PLC as "just a function on the other end of a wire" will miss every interesting failure mode.

**How it binds here:** Real failure modes observed in this domain include: PLC reboots mid-session (session ID becomes invalid); PLC processes a write but the ack is lost (consumer thinks the write failed); the network drops out for 5 seconds and recovers (TCP appears alive but no progress is made); the PLC firmware silently returns wrong data for certain UDT shapes (Byzantine). The library must have a vocabulary for each of these.

**Tracked by:** CODEX-R (state events), CODEX-S (retry with bounded backoff), CODEX-M (FFI registry consistency under reconnect).

### 2. Failure detection is not fault tolerance

**What:** Knowing a node is down is necessary but not sufficient; tolerating the failure is a separate engineering problem.

**Why it matters:** Detection alone gets you a stack trace. Tolerance gets you a service that stays up.

**How it binds here:** Today the library detects via error return (`EtherNetIpError::ConnectionLost`). It does not automatically reconnect. It does not surface state transitions to consumers. Every consumer ends up rolling their own reconnect loop.

**Tracked by:** CODEX-R (events stream), CODEX-S (retry policy includes reconnect policy).

### 3. The perfect failure detector doesn't exist

**What:** Any timeout choice is arbitrary; tune for the dominant failure mode of your environment.

**Why it matters:** A hardcoded timeout is an unconfigurable opinion. Different deployment environments (LAN vs WAN vs marginal industrial wireless) need different opinions.

**How it binds here:** Many `EipClient` methods wrap operations in `tokio::time::timeout(...)` with hardcoded durations. The `RetryPolicy` primitive should let consumers override the per-operation deadline.

**Tracked by:** CODEX-S.

### 4. Observability as a first-class API surface

**What:** You cannot debug a black box. Observability — structured logs, metrics, traces — must be part of the public surface, not a side channel bolted on later.

**Why it matters:** Production incidents are the test the library has to pass. If consumers cannot answer "why is this slow / failing / silent", they cannot ship.

**How it binds here:** `tracing` is already integrated (good). `monitoring.rs` (637 LOC) exposes metrics. *Missing:* a `Stream<ConnectionEvent>` for state transitions; a Prometheus exporter behind a feature flag; `#[tracing::instrument]` annotations on every public async fn so consumers see structured spans automatically.

**Tracked by:** CODEX-R (events stream); `#[tracing::instrument]` pass bundled into CODEX-K. Prometheus exporter deferred — not blocking for v0.8.0 but should be a v0.9 brief.

### 5. Idempotency keys for at-least-once semantics

**What:** Network ack does not guarantee server-side application. Some operations need explicit dedup tokens for correct retry.

**Why it matters:** Without idempotency keys, the consumer has no way to retry safely. They must choose between exactly-zero (fail on first failure) and at-least-N (retry blindly, may double-apply).

**How it binds here:** PLC tag writes have no native idempotency key in the CIP protocol. The library cannot synthesize one. The honest move is to document this clearly — `RetryPolicy` defaults to no-retry-on-write — and let consumers who know their write is idempotent (e.g., writing a configuration tag to a known value) opt in.

**Tracked by:** CODEX-S brief explicitly states this default.

---

## Patterns of Enterprise Application Architecture (Martin Fowler)

### 1. Repository

**What:** A collection-like interface for accessing domain objects, hiding the persistence mechanism.

**Why it matters:** Lets the domain layer think in terms of "the tag I want" rather than "the bytes I send and parse." Easier to test, easier to mock.

**How it binds here:** `EipClient` already *is* a Tag Repository (`read_tag`, `write_tag`, `read_tags_batch`, `write_tags_batch`). It just isn't called that explicitly, and the CIP-level concerns leak into its name and surface.

**Tracked by:** CODEX-J. The split's `client/tags.rs` submodule should expose a `TagRepository` trait that `EipClient` implements.

### 2. Unit of Work

**What:** Groups a set of operations into a transactional unit with explicit commit boundary.

**Why it matters:** Consumer code becomes declarative — "these things together" — and the implementation can optimize (batching, ordering, atomic commit) under the hood.

**How it binds here:** `BatchOperation` (`src/batch.rs`) is implicitly a UoW but doesn't expose the pattern explicitly. A `client.unit_of_work().read(x).write(y, v).commit().await` API would let the library decide whether to send as one CIP multi-service request or as N separate requests based on size.

**Tracked by:** Optional sub-item of CODEX-Q. Defer if it inflates scope.

### 3. Gateway

**What:** Wraps an external system behind a domain-shaped interface.

**Why it matters:** The external system's quirks (protocol versions, byte layouts, sequence numbers) are isolated to one place.

**How it binds here:** `src/protocol/` (cip + encap + values) is exactly this. Already extracted via CODEX-D. No further action.

**Tracked by:** Already merged (CODEX-D).

### 4. Service Layer

**What:** A higher-level operation composed of repository calls, hiding the orchestration from the consumer.

**Why it matters:** Workflow logic and workaround logic live in one place. Consumers don't reimplement them.

**How it binds here:** The STRING-write and UDT-array-member-write firmware workarounds are textbook service-layer use cases. Today the workaround is a 20-line ritual in the `lib.rs` doctest at `src/client.rs:131-150`. A `Client::write_string_tag`, `Client::write_udt_member`, `Client::write_udt_array_member` set of methods hides it.

**Tracked by:** CODEX-Q.

### 5. Data Mapper vs Active Record

**What:** Where does conversion between in-memory and persistence representation live? Data Mapper: in a separate mapping layer. Active Record: on the object itself.

**Why it matters:** Mixing them creates leaky abstractions and circular dependencies.

**How it binds here:** `PlcValue` and the codec in `src/protocol/values.rs` play the Data Mapper role. `PlcValue` itself is a plain enum without protocol awareness. This is the right shape already; no change needed.

**Tracked by:** No action required.

---

## Rust for Network Programming and Automation (Brian Anderson)

This is the most operational of the five and the least canonical text. The lessons it contributes:

### 1. Tokio's split-read-write pattern

**What:** A `TcpStream` (or any duplex stream) can be split into independent reader and writer halves, owned by separate tasks.

**Why it matters:** Decouples reading from writing so an in-flight write doesn't block reads, and lets a dedicated reader task handle response correlation while writers stay simple.

**How it binds here:** The actor refactor (CODEX-P) exploits this directly. One task owns the writer and accepts `(request_bytes, oneshot::Sender<response>)` from an mpsc; a sibling task owns the reader and dispatches responses back through the oneshots by correlation ID.

**Tracked by:** CODEX-P.

### 2. Cancellation as the silent reliability bug

**What:** Every `.await` in async Rust is a cancellation point. A dropped future leaves the protocol state in whatever position the await happened to suspend in.

**Why it matters:** If a caller's `select!` or `tokio::time::timeout` cancels an in-flight CIP request after the bytes are on the wire but before the response is consumed, the *next* operation reads the previous response. Silent state corruption.

**How it binds here:** Today every public async method on `EipClient` is a cancellation hazard. There is no request-ID correlation; responses are read in arrival order and matched to whoever holds the mutex next.

**Tracked by:** CODEX-P. The actor isolates cancellation to the consumer's send call; the wire state stays consistent.

### 3. `AsyncRead + AsyncWrite` as the transport abstraction

**What:** Don't invent a new transport trait when stdlib + tokio's already gives you one.

**Why it matters:** Adding a competing abstraction confuses consumers and duplicates the seam.

**How it binds here:** `EtherNetIpStream` at `src/lib.rs:69-102` already does this. The original architecture review proposed introducing a separate `Transport` trait; the codex second pass correctly pushed back. **No new abstraction.**

**Tracked by:** Decision recorded; nothing to do.

### 4. Backpressure via bounded channels

**What:** An unbounded mpsc is a memory leak waiting for a slow consumer.

**Why it matters:** Industrial deployments routinely have HMI consumers that fall behind under load spikes. An unbounded subscription channel becomes a slow OOM.

**How it binds here:** `src/subscription.rs` uses `mpsc` without documenting bounds. Should be bounded with a documented drop or block policy.

**Tracked by:** Bundle into CODEX-P (the actor naturally introduces a bounded request channel; subscriptions become bounded by inheriting the actor's policy).

### 5. Structured concurrency: tasks should have known parents and cancellation paths

**What:** Spawned tasks that outlive their logical parent are a leak. Every `tokio::spawn` needs a story for graceful shutdown.

**Why it matters:** A consumer who drops their `Client` expects the background work to stop. If background tasks keep running after the handle is dropped, the consumer leaks a thread / a socket / a callback indefinitely.

**How it binds here:** `tag_group` polling spawns background tasks (`src/tag_group.rs:275 LOC`). Subscriptions in `src/subscription.rs` spawn tasks. The future actor (CODEX-P) will spawn its own. Audit needed: does each task observe a shutdown signal (channel close, `CancellationToken`, `Drop` on the parent handle) and exit cleanly?

**Tracked by:** No dedicated brief; bundle the audit into CODEX-P (which is the natural restructuring point).

---

## Cross-book themes

A few ideas show up in three or more of the books and deserve special attention here:

| Theme | Books | This library |
|---|---|---|
| Dependency inversion at the right boundary | Clean Arch, PEAA, Rust networking | CODEX-J split + CODEX-P actor |
| Schemas evolve; design for it | DDIA, Clean Arch | CODEX-K `#[non_exhaustive]` sweep + error consolidation |
| Observability is API, not afterthought | DDIA, Understanding DS, Rust networking | CODEX-R events stream + bundle `#[tracing::instrument]` pass into CODEX-K |
| Cancellation and retry must be explicit | Understanding DS, Rust networking, DDIA | CODEX-P actor + CODEX-S retry policy |
| Hide protocol quirks behind a domain interface | PEAA, Clean Arch | CODEX-Q service layer for firmware workarounds |

## Lessons deliberately NOT applied

Each of these came up in one or more of the books but was judged not worth pursuing in this library:

- **Full CQRS / event sourcing** (DDIA, PEAA). The library serves a single PLC at a time over a serial-by-nature TCP session. Splitting reads from writes adds complexity without a benefit.
- **Saga / distributed transaction** (DDIA, Understanding DS). The PLC is a single resource; cross-resource consistency is not a concern at this layer.
- **Capability-based security** (Clean Arch). Rust's ownership model already provides 80% of what capability passing buys you.
- **Generic message bus** (PEAA "Message Channel" pattern). The library is EtherNet/IP-specific. Generalizing the transport layer to support arbitrary protocols inflates the surface for no gain.
- **DSL for tag paths** (PEAA "Domain Specific Language" pattern). The current `tag_path` parser is fine; a DSL adds learning curve for marginal expressiveness gain.
