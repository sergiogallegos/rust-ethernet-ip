# CIP Path Validation

## Summary

`confirmed` as of 2026-05-24: CIP request encoding validates the request path before emitting bytes. Empty paths, odd byte lengths, and paths longer than 510 bytes now fail locally instead of relying on a truncating path-word cast.

## Current Understanding

- CIP request path size is encoded in 16-bit words and stored in one byte.
- The largest encodable request path is therefore 255 words, or 510 bytes.
- Request paths must be word-aligned.
- No current `CipRequest` service in this repository has a documented legitimate empty-path exception, so empty paths are rejected.
- The current implementation keeps the generic protocol `Encode` trait infallible for unrelated encoders and uses a checked inherent `CipRequest::encode(&mut BytesMut) -> Result<()>`.

## Evidence

- [`src/protocol/cip.rs`](../../src/protocol/cip.rs) validates `CipRequest` paths before writing the service, path word count, path bytes, or payload.
- [`src/protocol/tests.rs`](../../src/protocol/tests.rs) covers valid even path lengths through 510 bytes and rejects empty, odd, and 512-byte paths.
- [`docs/agents/tasks/CODEX-N-cip-path-encoding-validation.md`](../../docs/agents/tasks/CODEX-N-cip-path-encoding-validation.md) records the implementation decision and verification.

## Open Questions

- `unclear`: whether a future service builder should explicitly allow an empty path. If such a service is added, it should use an allow-list with a source citation instead of weakening validation globally.

## Related Pages

- [route-path-behavior.md](route-path-behavior.md)
