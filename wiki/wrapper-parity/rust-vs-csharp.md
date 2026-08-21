# Rust Vs Wrapper Parity

## Summary

The strongest current parity evidence is the `1.2.0` CompactLogix
`5069-L330ERM` fw38 release gate: Rust, C#, Python, and C/C++ completed the same
2,338 reads and 2,319 writes/verifications with zero anomalies. API shape and
error/lifecycle behavior still differ by language even when wire behavior is
shared.

## Current Understanding

- `confirmed`: Real-hardware wrapper validation was completed on the same CompactLogix and ControlLogix targets used for the Rust validation pass.
- `confirmed`: C/C++ joined the real-hardware parity gate in `1.2.0` on the
  5069-L330ERM fw38 target.
- `confirmed`: Primitive reads and writes, batch operations, route-path connection, subscriptions, tag groups, and health-check flows were exercised successfully in the wrapper on both validated PLC targets.
- `confirmed`: Wrapper failures in the comprehensive matrix matched the Rust/native limitation profile rather than exposing a separate wrapper-only regression class.
- `confirmed`: `ConfigureBatchOperations(...)` and `GetBatchConfig()` are intentionally unsupported in this release line and throw `NotSupportedException`.
- `needs-care`: Parity should be interpreted as "validated for the exercised feature set", not "all wrapper behavior is identical to Rust internals."
- `confirmed`: The checked-in C ABI is the complete native wrapper contract;
  the header-only C++ RAII class is an intentionally smaller example covering
  direct connection, DINT/REAL/STRING, and batch calls.
- `confirmed`: C++ compilation, header parity, and the simulator smoke are
  blocking on Linux, Windows, and macOS in the `1.2.1` preparation line.
- `needs-care`: Generic C/C++ consumers still lack installable
  CMake/`pkg-config` metadata, standalone SDK archives, and generated reference
  documentation.

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

- `confirmed`: C# and Python preserve native last-error detail through
  wrapper-specific exception types; C/C++ callers use return codes plus the
  last-error API.
- `superseded`: Earlier direct STRING and scalar UDT-array-member `0x2107`
  limitations were request-encoding/path defects, not permanent wrapper gaps.

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
- [docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md](../../docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md)
- [docs/audit/1.2.1_wrapper_and_platform_gap_analysis.md](../../docs/audit/1.2.1_wrapper_and_platform_gap_analysis.md)
- [docs/CPP_INTEGRATION.md](../../docs/CPP_INTEGRATION.md)

## Open Questions

- Whether wrapper parity should get split into separate pages for API parity, performance parity, and failure-surface parity as more features are validated.
- Whether richer native PLC error detail should be preserved through the wrapper for direct `STRING` write failures instead of the current summarized `0x1E` surface.
- Whether the C++ RAII example should become a supported SDK surface or remain
  an example while the C ABI stays the sole native compatibility contract.

## Related Pages

- [../releases/0.7.0-validation-synthesis.md](../releases/0.7.0-validation-synthesis.md)
- [../limitations/string-and-udt-write-behavior.md](../limitations/string-and-udt-write-behavior.md)
