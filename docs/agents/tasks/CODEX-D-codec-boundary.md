---
id: CODEX-D
title: Extract Encoder/Decoder boundary for the wire protocol
owner: codex
status: open
created: 2026-05-05
last-update: 2026-05-05 claude
depends-on: CODEX-C
---

## Brief

### Goal

Extract a clean codec boundary between byte-level wire encoding/decoding and the `EipClient` business logic, so the protocol layer becomes testable without a `TcpStream` or simulator and gains fuzz-target-shaped pure-functional entry points. After this brief, every wire message has a `Frame → BytesMut` (encode) and `&[u8] → Result<Frame, _>` (decode) path that does not touch async or I/O. `EipClient` methods reduce to: build request frame → send via `EtherNetIpStream` → parse response frame.

The reference shape is `tokio_util::codec::{Encoder, Decoder}`. The library does not need to depend on `tokio-util`; defining local `Encoder` / `Decoder` traits with the same idea is sufficient and keeps the dependency surface unchanged.

This brief depends on CODEX-C. Do not start work until CODEX-C is merged. The acceptance criterion `src/types.rs` and `src/client.rs` must already exist as separate files.

### Context to read first

- `docs/agents/README.md` — protocol, voice, lifecycle.
- `docs/agents/tasks/CODEX-C-lib-decomposition.md` — the prior decomposition; this brief reshapes the moved code.
- `src/types.rs` (post-CODEX-C) — contains `PlcValue`, `UdtData`, `ConnectedSession`, `ConnectionParameters`. The encoding/decoding logic for `PlcValue` (the 13 AB types) currently lives as inline byte arithmetic on the `EipClient` methods that read/write them.
- `src/client.rs` (post-CODEX-C) — contains every `EipClient` method that does wire I/O. The byte-level work to extract is interleaved with the async I/O calls.
- Reference repos for shape:
  - `tokio_util::codec::{Encoder, Decoder}` source — the canonical pattern.
  - `redis-rs` `src/parser.rs` — pure-functional `&[u8] → Result<Value, ParseError>` decoder used by the async client.
  - `hyper` `src/proto/h1/decode.rs` — boundary between framing and business logic.
- Existing `bytes = "1.0"` dependency in `Cargo.toml` — already pulled, ready to use.

### Behavior

Three contained changes that should land in this order. Whether they ship in one PR or three is the implementer's choice; phased landing is recommended for review tractability.

**1. Define the codec module.**

Create `src/protocol/mod.rs` with the trait definitions:

```rust
use bytes::{Buf, BytesMut};
use crate::error::Result;

pub trait Encode {
    fn encode(&self, buf: &mut BytesMut);
}

pub trait Decode: Sized {
    fn decode(buf: &mut impl Buf) -> Result<Self>;
}
```

Keep these traits `pub(crate)` for now — they're not part of the public API surface, and exposing them would force SemVer constraints on internal refactors. Only promote to `pub` if a future brief identifies a downstream user.

Also add `src/protocol/encap.rs` (EtherNet/IP encapsulation header), `src/protocol/cip.rs` (CIP request/response), and `src/protocol/values.rs` (the 13 `PlcValue` variants). These are placeholder files at this step; they get populated in changes 2 and 3.

In `src/lib.rs`, add `pub(crate) mod protocol;` alongside the existing `pub mod` declarations. The trait definitions stay crate-private.

**2. Move `PlcValue` encoding/decoding into the codec module.**

`PlcValue` currently has its 13-arm encoding logic spread across `EipClient::write_tag`-shaped methods in `src/client.rs`, with each method doing inline byte arithmetic for the type tag, length prefix, and payload bytes. Decoding is similarly inline in the read paths.

Create `impl Encode for PlcValue` and `impl Decode for PlcValue` in `src/protocol/values.rs`. The implementations should be byte-for-byte identical to the existing inline encoding/decoding — same type tags, same endianness, same length-prefix sizes. Move the inline arithmetic out of `client.rs` into these impls; replace the call sites with `value.encode(&mut buf)` and `PlcValue::decode(&mut buf)?`.

Critical: the wire output must not change. Round-trip tests in step 4 are the proof.

`UdtData` has its own encoding shape (it carries an opaque `Vec<u8>` plus a `symbol_id`). Add `impl Encode for UdtData` / `impl Decode for UdtData` in the same `values.rs` for consistency, but note that UDT decoding requires a `UdtDefinition` for full interpretation — the codec layer only handles the byte-level container; semantic interpretation stays where it is.

**3. Move EtherNet/IP encapsulation and CIP framing into the codec module.**

The encapsulation header (24 bytes: command, length, session handle, status, sender context, options) is currently constructed inline in every request method on `EipClient`. Extract:

- An `EncapsulationHeader` struct with named fields in `src/protocol/encap.rs`, with `Encode` and `Decode` impls.
- A `CipRequest` / `CipResponse` shape in `src/protocol/cip.rs` covering the common request and reply structures.

`EipClient` request paths reduce to:

```rust
let header = EncapsulationHeader { command: 0x6F, ... };
let request = CipRequest { service: 0x4C, path: ..., data: ... };
let mut buf = BytesMut::new();
header.encode(&mut buf);
request.encode(&mut buf);
self.send(&buf).await?;
```

Where `self.send` is a thin wrapper over the existing stream-write path. Decoding the response goes through the symmetric path.

The exact decomposition of which fields belong on `EncapsulationHeader` vs `CipRequest` should mirror what the spec puts at each layer (encapsulation header is the EtherNet/IP wrapper; CIP request body sits inside it). Use the wire format reference at `docs/EtherNetIP_Connection_Paths_and_Routing.md` for guidance.

**4. Add round-trip codec tests.**

Create `src/protocol/tests.rs` (or `tests/codec_roundtrip.rs` if the implementer prefers an integration test). The tests should encode known `PlcValue` / `EncapsulationHeader` / `CipRequest` instances to bytes, then decode the bytes back, then assert equality with the original. Cover:

- Every `PlcValue` variant including `Bool`, all integer widths (`Sint`, `Int`, `Dint`, `Lint`, `Usint`, `Uint`, `Udint`, `Ulint`), `Real`, `Lreal`, `String`, and `Udt`.
- A representative `EncapsulationHeader` for each command code the library uses (`RegisterSession`, `UnRegisterSession`, `SendRRData`, `SendUnitData`).
- A read-tag CIP request, a write-tag CIP request, a batch-read multi-service request, and at least one error response with a non-zero CIP status code.

Also add a pinned-bytes test set: hand-crafted `&[u8]` sequences captured from real PLC traffic, fed to the decoder. These prove that the codec hasn't drifted from the wire reality. If captured traffic is not available, generate the bytes once via the encoder, paste them as hex-literal arrays into the test, and use the encoder output as the future-proof fixture.

### Test requirements

- `cargo fmt -- --check` — must pass.
- `cargo clippy --all-features -- -D warnings` — must pass.
- `cargo clippy --no-default-features --lib -- -D warnings` — must pass.
- `SKIP_PLC_TESTS=1 cargo test --workspace --locked` — must pass with **at least 30 new test cases** covering the codec round-trips. Document the new test count in `## Codex log`.
- `cargo test --test plc_sim_tests` — must pass. The simulator sees the same wire bytes; if any sim test fails, the codec is not byte-identical to the prior implementation.
- `cargo bench` — run the existing benchmarks (`benches/performance_benchmark.rs`) before the change and after. The post-change median latency for read and write paths must not regress by more than 5%. Document both numbers in `## Codex log`. If the regression is >5%, **stop and ask** in `## Codex log` before proceeding — the codec may be allocating more aggressively than the inline path.
- `cd csharp/RustEtherNetIp && dotnet build && cd ../RustEtherNetIp.Tests && dotnet test` — must pass. The C# wrapper exercises the FFI surface; if any wire byte changed, the wrapper tests will surface it.
- `cargo build --release --features ffi` — produces a cdylib with exactly `56` `_eip_` exports.

### Acceptance criteria

- [ ] `src/protocol/mod.rs` exists and declares `pub(crate) trait Encode` and `pub(crate) trait Decode` with the specified signatures.
- [ ] `src/protocol/encap.rs`, `src/protocol/cip.rs`, and `src/protocol/values.rs` exist and contain the migrated encoding/decoding logic.
- [ ] No inline byte arithmetic for `PlcValue` encoding remains on any `EipClient` method. Verify with `grep -n 'to_le_bytes\|from_le_bytes' src/client.rs` — every match should be in a context that delegates to a codec function rather than reimplementing it. Some matches will remain (e.g. session handle math); the criterion is that the *13 AB type encoding/decoding* is no longer inline.
- [ ] No inline construction of the EtherNet/IP encapsulation header in `EipClient` methods. The header is built by `EncapsulationHeader::encode`.
- [ ] At least 30 new round-trip codec tests pass.
- [ ] Benchmarks show no >5% regression on read and write paths.
- [ ] FFI symbol parity preserved: 56 `_eip_` exports.
- [ ] `cargo doc --no-deps --all-features` produces no new broken-link warnings.
- [ ] CHANGELOG entry under "Internal" or "Cleanup" describing the codec extraction; no SemVer-relevant claims.

### Out of scope

- Any change to public API, including the `pub use` re-exports established in CODEX-C.
- Promoting the `Encode` / `Decode` traits to `pub` (kept `pub(crate)` for now).
- Adding a generic `EipClient<S>` parameter over `EtherNetIpStream` to remove `Box<dyn EtherNetIpStream>` — that is a separate architectural concern.
- Adding a fuzz harness target to `Cargo.toml`. The codec is now fuzz-shaped, but actually wiring up `cargo fuzz` is its own brief.
- Switching the read path to use `tokio_util::codec::FramedRead`. The current sync-read-into-buffer pattern stays; only the byte-level parsing moves into the codec layer.
- Adding any new dependency. `bytes` is already pulled and is sufficient.
- Splitting `client.rs` into per-feature submodules. CODEX-C deliberately left `client.rs` monolithic; further sub-splitting is a follow-up.

### Risks and gotchas

- **Wire byte parity is the only correctness guarantee.** Every encode/decode change must be byte-for-byte identical to the prior inline implementation. The round-trip tests catch encoder regressions; the simulator tests catch end-to-end regressions; the C# wrapper tests catch any drift the previous two miss. If all three pass and the maintainer's hardware test passes, the codec is correct.
- **Endianness.** EtherNet/IP is little-endian on the wire. Every multi-byte field uses `to_le_bytes` / `from_le_bytes`. Do not introduce `to_be_bytes` anywhere; if the existing inline code does, it's a bug — flag it in `## Codex log` and **stop**, do not silently fix.
- **CIP path encoding has its own subtleties.** Symbolic segments, padding bytes, byte-counts vs word-counts. The existing `RoutePath::to_cip_bytes` is the reference for backplane routing; tag-name path encoding is in the read/write request builders. Move both into the codec module without merging them — they're related but not the same shape.
- **Allocations.** `BytesMut` reuse is the perf story. If every encoding call does `BytesMut::new()`, the allocator becomes a bottleneck. Prefer threading `&mut BytesMut` through the encode chain so the caller controls the buffer. The current inline code does the same with `Vec<u8>`; preserve that pattern.
- **`PlcValue::Udt` is the trickiest variant.** Its byte content is opaque (`Vec<u8>`) plus a `symbol_id`. Encoding writes the bytes verbatim; decoding cannot reconstruct the original `UdtData` without the structure handle from the prior read. The codec layer should encode/decode the *byte container* — semantic interpretation (member offsets, type recovery) stays where it is.
- **Bench-driven scope discipline.** If the >5% regression check fails, the temptation is to optimize the codec inline. Resist. The brief is contract-bound: byte-identical encoding, no new dependencies. Optimizations belong in a follow-up.
- **Review burden.** This brief touches every wire-facing method on `EipClient`. The diff will be large. Land it after CODEX-C so the diff can be reviewed against a `client.rs` of known shape rather than against the post-mvoe `lib.rs`.

## Codex log

*(empty — codex appends entries on starting work)*

## Claude review

*(empty — claude appends after submission)*

## Verdict

*(empty — claude writes on merge or rejection)*
