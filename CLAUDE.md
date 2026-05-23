# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Rust EtherNet/IP is a high-performance EtherNet/IP communication library for Allen-Bradley CompactLogix and ControlLogix PLCs. Written in pure Rust with a C FFI layer (`cdylib`) and a C# wrapper for .NET integration. The current development focus is on the .NET stack (C# wrappers and examples).

## Build & Test Commands

```bash
cargo build                          # Debug build
cargo build --release                # Release build (needed for C# FFI .dll/.so/.dylib)
cargo fmt -- --check                 # Check formatting (CI enforced)
cargo clippy -- -D warnings          # Lint with warnings as errors (CI enforced)
cargo test                           # Run all tests (integration tests need a PLC)
SKIP_PLC_TESTS=1 cargo test          # Run tests without a physical PLC
cargo test --test plc_sim_tests      # Run simulator-backed tests only (no PLC needed)
cargo test --lib                     # Run unit tests only
cargo test --test integration_test   # Run a specific test file
cargo test test_name                 # Run a single test by name
cargo bench                          # Run Criterion benchmarks
cargo run --bin plc_sim              # Start standalone PLC simulator
```

### Test Environment Variables

| Variable | Default | Purpose |
|---|---|---|
| `SKIP_PLC_TESTS` | unset | Set to any value to skip tests requiring a physical PLC |
| `TEST_PLC_ADDRESS` | `192.168.0.1:44818` | PLC IP address and port |
| `TEST_PLC_SLOT` | `0` | CPU slot (0 for CompactLogix) |

Most integration tests call `should_skip_plc_tests()` and return early when `SKIP_PLC_TESTS` is set. The `plc_sim_tests.rs` always run using an in-process `SimulatedPlc`.

### C# Wrapper

```bash
cd csharp/RustEtherNetIp && dotnet build
cd csharp/RustEtherNetIp.Tests && dotnet test
```

## Architecture

### Core Design

The library is built around `EipClient` (in `src/client.rs`, ~6.7k lines), which implements the EtherNet/IP encapsulation protocol and CIP (Common Industrial Protocol) over async TCP via Tokio. It is the single entry point for all PLC communication. `src/lib.rs` (~220 lines) is a thin crate-root that re-exports the public API; the post-CODEX-C/D layout splits the implementation into `route.rs`, `batch.rs`, `types.rs`, `client.rs`, and `protocol/{mod,encap,cip,values,tests}.rs`.

```
Rust/C# Application
        |
   EipClient (src/client.rs) -- async TCP via Box<dyn EtherNetIpStream>
        |
   protocol/ (src/protocol/) -- Encode/Decode boundary for encap, CIP, PlcValue
        |
   FFI layer (src/ffi.rs) -- #[no_mangle] extern "C", global Tokio runtime
        |
   C# P/Invoke wrapper (csharp/RustEtherNetIp/)
```

### Key Source Modules

| Module | Responsibility |
|---|---|
| `lib.rs` | Thin crate root — public re-exports, `try_init_tracing`, version string. ~220 lines. |
| `client.rs` | `EipClient`: session management, tag reads/writes, batch execution, UDT/STRING paths, diagnostics, subscriptions. ~6.7k lines. |
| `route.rs` | `RoutePath` and `RouteHop` (`Backplane` / `Ethernet`) — ordered CIP route hops with ASCII ethernet link-address encoding. |
| `batch.rs` | `BatchOperation`, `BatchError`, `BatchConfig` — batch read/write/execute data model. |
| `types.rs` | `PlcValue` (13 AB types), `UdtData`, `ConnectedSession`, `ConnectionParameters` — shared data model. |
| `protocol/` | Wire codec boundary — `Encode`/`Decode` traits, encapsulation framing (`encap.rs`), CIP framing (`cip.rs`), `PlcValue` codecs (`values.rs`), pinned-byte tests (`tests.rs`). |
| `error.rs` | `EtherNetIpError` enum with `is_retriable()` for retry vs reconnect decisions. |
| `tag_path.rs` | `TagPath` parser for complex addressing: arrays, bits, program-scoped, UDT members, nested paths. |
| `udt.rs` | `UdtDefinition`, `UdtManager`, `UserDefinedType` for UDT discovery and serialization. |
| `ffi.rs` | C FFI exports using `lazy_static` global `RUNTIME` and `FFI_CLIENTS: Mutex<HashMap<i32, EipClient>>`. Gated behind the `ffi` Cargo feature. |
| `subscription.rs` | `TagSubscription`, `SubscriptionManager` with mpsc channels. |
| `monitoring.rs` | `ProductionMonitor`, health checks, metrics collection. |
| `config.rs` | `ProductionConfig` with connection/performance/monitoring sub-configs. |

### Key Types

- **`EipClient`**: Primary client. Not thread-safe for concurrent use — wrap in `Arc<Mutex<>>` for shared access. Supports `connect()`, `with_route_path()`, and `connect_with_stream()` (stream injection for testing/metrics).
- **`PlcValue`**: Tagged enum covering all 13 AB types: `Bool`, `Sint`, `Int`, `Dint`, `Lint`, `Usint`, `Uint`, `Udint`, `Ulint`, `Real`, `Lreal`, `String`, `Udt(UdtData)`.
- **`UdtData`**: Opaque `{ symbol_id: i32, data: Vec<u8> }`. Must be parsed with a `UdtDefinition` obtained from the PLC. Always read before write to capture the `symbol_id`.
- **`EtherNetIpError`**: All operations return `Result<T, EtherNetIpError>`. Use `is_retriable()` to distinguish transient errors (timeout, connection lost) from permanent ones (protocol, CIP).
- **`RoutePath`**: Slot/port routing for ControlLogix backplane. When set, CIP messages are wrapped in Unconnected Send (service 0x52).

### Tag Path Addressing

The library handles full Allen-Bradley tag path syntax internally:
- Controller tags: `"MyTag"`
- Program-scoped: `"Program:MainProgram.MyTag"`
- Array elements: `"MyArray[5]"`, `"MyArray[1,2,3]"`
- BOOL arrays: `"gBoolArray[5]"` (automatic DWORD bit extraction)
- Bit access: `"StatusWord.15"` or `client.read_bit("StatusWord", 15)`
- UDT members: `"MotorData.Speed"`
- Nested: `"Cell_NestData[90].PartData.Member"`

### PLC Firmware Limitations

These are Allen-Bradley restrictions, not library bugs:
1. **Cannot write STRING tags directly** — CIP Error 0x2107. Workaround: write the entire containing UDT.
2. **Cannot write individual UDT array element members** — CIP Error 0x2107. Workaround: read entire UDT element, modify in memory, write back the whole element.

## Workspace Layout

The root `Cargo.toml` defines a workspace containing `.` (main crate) and `examples/desktop_app`. The crate produces both `rlib` and `cdylib` outputs. Rust MSRV is 1.95.

## CI

GitHub Actions runs on ubuntu/windows/macos with stable+beta Rust: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --verbose`. C# tests and coverage (tarpaulin) run on ubuntu/stable only.

## Rust code discipline

Rules that apply to all changes to the Rust crate. Briefs and implementations must hold to these unless the brief explicitly waives one.

### Error handling and unsafe

- Avoid `panic!`, `unreachable!`, and `.unwrap()`. Encode constraints in the type system instead — `Result<T, EtherNetIpError>` exists for this reason. Verbose code that surfaces a fallible path beats terse code that hides one.
- Every `unsafe` block needs a `// SAFETY:` comment naming the invariant being upheld. Non-negotiable for `src/ffi.rs` — the FFI surface is what C# consumers run against, and an unsound block crashes the host process.
- Prefer `#[expect(<lint>, reason = "…")]` over `#[allow(<lint>)]` for clippy suppressions. `expect` fails the build when the lint stops triggering, so suppressions don't outlive their cause. If clippy flags dead code, delete the code instead of suppressing.

### Dependency and lockfile hygiene

- Never bulk-run `cargo update`. Use `cargo update --precise <crate>@<version>` to bump a specific dependency, keeping PRs reviewable and avoiding unrelated drift.
- Don't assume clippy warnings on `main` are pre-existing. CI gates `cargo clippy -- -D warnings`, so a warning surfacing in a branch was almost certainly introduced by that branch.

### Tests

- Default: add new tests to the existing file for the module being changed. A new file is justified when the area genuinely has no test home, not when it's mildly tidier.
- Before writing a new test, read two or three nearby tests in the same file and copy their setup and assertion style. Test conventions in this repo encode real PLC behavior — drifting from them often means missing a guard.

### Code review self-check

- If neighboring code does something differently than the change about to land, find out *why* before deviating. Patterns in this repo are often load-bearing (CIP framing, async ordering, FFI lifetimes), not stylistic.
- Don't take a bug report's suggested fix at face value. The user-facing symptom may be two layers below where it presents — verify the right layer before patching.
- Before writing code that makes a non-obvious choice, ask "why this and not the alternative?" If there's no answer, research until there is.

---

# Agent collaboration

This repo uses a two-agent collaboration model. The substantive cross-agent state lives in [`docs/agents/`](docs/agents/) — the protocol is documented in [`docs/agents/README.md`](docs/agents/README.md). What follows is the short version any agent needs at the start of a turn.

## Division of labor

- **Codex** — implementation, debugging, refactoring.
- **Claude** — design, brief authoring, code review, merge bookkeeping.
- **Maintainer** — routes messages between the two agents, makes strategic decisions, runs hardware-backed validation against real CompactLogix / ControlLogix PLCs.

## How to resume any session

1. `git pull` first — the durable state is on origin, not local.
2. Read **`docs/agents/board.md`** — the entry point. The status table lists open tasks (`open`, `in-progress`, `submitted`, `under-review`, `merged`, `rejected`). Anything not `merged` is in-flight.
3. For any non-merged row, open `docs/agents/tasks/CODEX-{ID}-{slug}.md` and read the frontmatter (`status:` is authoritative), the Brief, the Codex log, and any prior Claude review.
4. Skim the last ~20 lines of `docs/agents/log.md` for chronological context.

Don't re-derive state by reading every file. The agent docs are the durable handoff — `board.md` tells you what's open in 60 seconds. Trust it.

## Review and merge lifecycle

When Codex submits (status `submitted` in the task frontmatter):

1. Run the full test matrix independently — don't trust Codex's verification claim:
   ```
   cargo fmt -- --check
   cargo clippy -- -D warnings
   SKIP_PLC_TESTS=1 cargo test --workspace --locked
   cargo test --test plc_sim_tests
   ```
   Plus task-specific extras (C# wrapper `dotnet build` + `dotnet test` for FFI-touching tasks; benches for perf-touching tasks; manual hardware smoke for protocol-touching tasks — that one is the maintainer's job).
2. Read the changed files — at minimum the impl module, the test file, and any wiki entry the brief asked for.
3. Write the `## Claude review` section with strong points (✅), findings (🟡 polish, 🟠 real concern), and acceptance-criteria tally.
4. Set frontmatter `status: merged`, write the `## Verdict` section, update `board.md` (move row to Done, record merge commit), append to `log.md`.
5. Commit. Push only on explicit maintainer request — see "Commit and push expectations" below.

## When to reject vs fix-during-merge

**Reject** when the implementation is fundamentally wrong — the submitted code can't fulfill the brief's contract even when its tests pass. Example: a driver task that opens a TCP socket but doesn't actually speak the wire protocol.

**Fix during merge** when the bug is mechanical and mirrors an existing pattern (≤5 lines, no architectural change). Document Claude-applied fixes transparently in the verdict.

**Always flag honestly:**
- Brief errors — when a Claude-authored brief was wrong (e.g. pinned a crate version that doesn't exist, named an API that doesn't match upstream). Own them in the verdict.
- Verification mismatches — Codex's environment vs the local merge environment (Rust toolchain drift, missing system deps, no PLC available).
- Don't undersell load-bearing items as "polish".

## Brief authoring conventions

When opening a new task, use the next id (`CODEX-A`, `CODEX-B`, …) and frontmatter:

```yaml
---
id: CODEX-XY
title: <short title>
owner: codex
status: open
created: YYYY-MM-DD
last-update: YYYY-MM-DD claude [Opus 4.7]
---
```

The `last-update` field carries the underlying model in square brackets (e.g. `claude [Opus 4.7]`, `codex [gpt-5.5]`) so the maintainer can audit model-vs-quality over time.

Then sections: `## Brief` with goal + context to read first + files to create + behavior + test requirements + acceptance criteria + out of scope + risks/gotchas. Plus empty `## Codex log`, `## Claude review`, `## Verdict`. Entry headers inside `## Codex log` and `## Claude review` also carry the model tag — `### YYYY-MM-DD HH:MM  <author> [<model>]` — same format as `log.md` lines.

When the task is opened, also: add a row to `board.md`, append a one-line entry to `log.md`, commit.

## Voice

Use neutral framing in everything written into this directory and into project docs (`CLAUDE.md`, `README.md`, `docs/`, task files, commit messages, PR descriptions):

- **No first-person.** Write "Codex implemented X" / "Claude-authored brief" / "the original brief" / "brief error owned by Claude". Not "I added X" / "my brief".
- **No maintainer profiling.** Write "the maintainer requested" / "per maintainer direction". Not "the user wants X" / direct quotes of maintainer chat.
- **No agent attribution in commit messages or PR descriptions.** Public artifacts read as the project's own voice; agent identity belongs in `docs/agents/`, not in git history surfaced to crates.io / NuGet consumers.
- **Agent + model tags belong only in `docs/agents/`.** Log entries, task section headers, verdicts, and frontmatter `last-update` carry the agent's underlying model — `claude [Opus 4.7]`, `codex [gpt-5.5]`, etc. This gives the maintainer a model-vs-quality audit trail without leaking it into public git history.
- **End-user references are fine when domain-relevant.** "the user's PLC tag", "the integrator's HMI", "calling code" are correct when they refer to actual end-users of the library.
- **Paraphrase, don't quote.** If a maintainer message defines a project convention, restate it neutrally as the convention. Don't embed the original message verbatim.

This repo is published as a public Rust crate and NuGet package. Personal phrasing leaks behavioral signals (work patterns, incidents, preferences) that belong in private agent memory, not in project history.

## Ambiguity threshold — when to stop vs proceed

- **Stop and ask** when the ambiguity affects acceptance criteria, contradicts a brief assumption, or requires source-of-truth context the brief doesn't provide. Examples: brief pins a crate version that doesn't exist on crates.io; brief contradicts an architectural decision in CLAUDE.md; acceptance test description is internally inconsistent; brief specifies an API that conflicts with upstream's actual surface.
- **Document and proceed** when the ambiguity is a normal implementation choice with no contract impact. Examples: variable naming, internal helper structure, log message wording, choice between two equivalent stdlib calls. Add a one-line entry to `## Codex log` recording the assumption ("Assumed X because Y; revisit if review disagrees.") so review can cheaply override.

The cost of stalling on small details is higher than the cost of a v1.1 polish item.

## Commit and push expectations

Both agents may stage and commit edits to task files, `board.md`, and `log.md` as part of normal task work. The lifecycle three-place update (frontmatter + board + log) commits together.

**Pushing to the remote is not automatic:**

- Push only when the maintainer explicitly asks ("commit and push", "ship it"), or when an unambiguous task convention requires it (e.g. backfilling a merge ref in a follow-up commit).
- A successful local commit is not a successful push. Always confirm the push step ran before claiming a task moved to `merged` or `submitted`.
- If push is blocked (network, auth, safe-directory), surface the blocker — don't retry silently or work around it.

This prevents the case where one agent's session pushes to the remote while the other agent's session has unpushed local commits, leaving the two views diverged.
