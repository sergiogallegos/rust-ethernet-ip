# Rust Vs C# Wrapper Parity

## Summary

The current C# wrapper is close to parity with the exercised Rust core feature set for `0.7.0`, with the clearest exceptions being intentionally unsupported batch-configuration APIs and wrapper-specific surface differences in error reporting and lifecycle semantics.

## Current Understanding

- `confirmed`: Real-hardware wrapper validation was completed on the same CompactLogix and ControlLogix targets used for the Rust validation pass.
- `confirmed`: Primitive reads and writes, batch operations, route-path connection, subscriptions, tag groups, and health-check flows were exercised successfully in the wrapper on both validated PLC targets.
- `confirmed`: Wrapper failures in the comprehensive matrix matched the Rust/native limitation profile rather than exposing a separate wrapper-only regression class.
- `confirmed`: `ConfigureBatchOperations(...)` and `GetBatchConfig()` are intentionally unsupported in this release line and throw `NotSupportedException`.
- `needs-care`: Parity should be interpreted as "validated for the exercised feature set", not "all wrapper behavior is identical to Rust internals."

## Areas With Strong Parity Evidence

- Primitive tag reads and writes for common scalar types
- Program-scoped tags
- Route-path connection flows
- `ReadTagsBatch(...)`
- `WriteTagsBatch(...)`
- `ExecuteBatch(...)`
- Invalid subscription fail-fast handling
- Tag-group registration, one-shot reads, and polling subscription flows

## Known Surface Differences

### Batch Configuration

- `confirmed`: The C# interface exposes batch-configuration methods, but they are contractually unsupported in this release line.
- `confirmed`: This is documented and covered by tests, rather than being a silent capability gap.

### Error Mapping

- `confirmed`: Some PLC write-limit failures surface as wrapper-specific user-facing messages rather than raw native status detail.
- `confirmed`: Direct `STRING` write failures may surface as batch-level `0x1E` embedded service errors in the wrapper.
- `confirmed`: Direct UDT-array-member writes are surfaced as the documented `0x2107` limitation in the wrapper.

### Lifecycle And Cleanup

- `confirmed`: Wrapper-specific subscription and tag-group lifecycle bugs were found and fixed during `0.7.0` validation.
- `confirmed`: Invalid tag subscriptions now fail fast instead of succeeding initially and failing later in the polling loop.
- `confirmed`: Tag-group disposal cleanup was hardened so client disposal does not double-stop already disposed groups.

## Performance Interpretation

- `confirmed`: Native batch-read support is now active in the wrapper and materially improved throughput relative to the older sequential wrapper path.
- `confirmed`: On the exercised real-PLC scenarios, wrapper batch throughput stayed reasonably close to the Rust native baseline.
- `needs-care`: The wrapper still adds an API surface and error-mapping layer, so "close to Rust native" is the right claim, not "identical."

## Evidence

- [csharp/RustEtherNetIp/README.md](../../csharp/RustEtherNetIp/README.md)
- [csharp/RustEtherNetIp/IEtherNetIpClient.cs](../../csharp/RustEtherNetIp/IEtherNetIpClient.cs)
- [csharp/RustEtherNetIp.Tests/BatchConfigContractTests.cs](../../csharp/RustEtherNetIp.Tests/BatchConfigContractTests.cs)
- [csharp/RustEtherNetIp.Tests/TagGroupApiTests.cs](../../csharp/RustEtherNetIp.Tests/TagGroupApiTests.cs)
- [csharp/RustEtherNetIp.Tests/TagGroupEventDiagnosticsTests.cs](../../csharp/RustEtherNetIp.Tests/TagGroupEventDiagnosticsTests.cs)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)
- [docs/audit/0.7.0_docs_api_audit.md](../../docs/audit/0.7.0_docs_api_audit.md)

## Open Questions

- Whether wrapper parity should get split into separate pages for API parity, performance parity, and failure-surface parity as more features are validated.
- Whether richer native PLC error detail should be preserved through the wrapper for direct `STRING` write failures instead of the current summarized `0x1E` surface.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
