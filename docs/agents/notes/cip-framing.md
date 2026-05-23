# CIP and Encapsulation Framing

Use this page when reviewing or modifying anything that emits or parses bytes on the wire — encapsulation headers, CIP requests, or `PlcValue` codecs. The wire codec layer is the contract with Allen-Bradley controllers; it is not a refactoring playground.

Verified against the `src/protocol/` layout as of CODEX-C/D refactor.

## Wire codec lives in `src/protocol/`, not `client.rs`

- `protocol/mod.rs` defines the `Encode` / `Decode` traits and the public re-exports.
- `protocol/encap.rs` owns EtherNet/IP encapsulation framing (`EncapsulationHeader`, `SendRRData`, `RegisterSession`).
- `protocol/cip.rs` owns CIP framing (request/response paths, service codes).
- `protocol/values.rs` owns `PlcValue` encode/decode for all 13 AB types.
- `protocol/tests.rs` owns pinned-byte tests against the AB spec.

`src/client.rs` orchestrates sessions, retries, batching, and tag-level operations. It calls *into* `protocol/`. It must not emit raw bytes inline. If a code change makes it tempting to write a `BytesMut::extend_from_slice` in `client.rs`, the codec for that thing belongs in `protocol/`.

## Why the split exists

Prior to CODEX-C/D, `client.rs` was ~7k+ lines with encap and CIP framing inline. Verifying that a 12-byte encapsulation header matched the AB spec required spinning up a full client. The split lets `protocol/tests.rs` pin specific byte sequences against the spec with no session state, no async, no network.

Do not collapse the split back into `client.rs` even when an inline call site would be shorter. The cost being paid is testability and review surface, not line count.

## Pinned-byte tests are the spec contract

- Tests in `protocol/tests.rs` encode an `EncapsulationHeader` or CIP request and assert exact byte sequences (e.g. `[0x65, 0x00, 0x04, 0x00, …]`).
- These byte sequences come from the AB CIP and EtherNet/IP specifications. They are not arbitrary fixtures.
- If a pinned-byte test fails after an implementation change, the implementation drifted from the spec — the test is correct. Do not "update the expected bytes to match the new output" without first proving the new output is what the spec actually requires.

## Adding a new CIP service or `PlcValue` type

1. Add the encode/decode logic in `protocol/cip.rs` or `protocol/values.rs`.
2. Add a pinned-byte test in `protocol/tests.rs` against the spec.
3. Surface the operation through `EipClient` in `client.rs` — that file orchestrates, it does not encode.
4. Wire the C FFI export in `ffi.rs` if the surface should be reachable from C#.

Skipping step 2 is the most common shortcut and the most expensive one. Without a pinned test, a future refactor to `protocol/` can silently break the wire contract while the unit tests around `EipClient` keep passing.

## Encapsulation status vs CIP general status

- Encapsulation status (in the 24-byte EtherNet/IP header) and CIP general status (inside the CIP response body) are different layers. Both must be checked.
- `send_rr_data_item` checks encapsulation status. CIP-level errors come back as a non-zero general status inside the CIP response and are unwrapped by the caller. Don't conflate the two when surfacing errors.
