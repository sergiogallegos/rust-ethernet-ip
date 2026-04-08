Date: 2026-04-07
Tester: Codex + Sergio Gallegos
PLC model: 5069-L320ERMS3
Firmware revision: 35
Network topology: Direct Ethernet connection to 192.168.0.1:44818, slot 0 via `ConnectWithRoute`

Scope:
- C# wrapper validation against the same real PLC and `gTest*` tag set used for the Rust validation pass.
- Focused on wrapper behavior, not just native Rust core behavior.

Commands executed:
- `dotnet run --project examples/CSharpWrapperTest/CSharpWrapperTest.csproj`
- `dotnet run --project examples/CSharpWrapperSmoke/CSharpWrapperSmoke.csproj`
- `dotnet run --project examples/CSharpWrapperBenchmark/CSharpWrapperBenchmark.csproj -- --iterations 100`

Result:
- PASS: Route-path connection succeeded to `192.168.0.1:44818` with slot 0.
- PASS: Comprehensive wrapper read/write validation matched the Rust native pass at `333/392` passed and `59/392` failed.
- PASS: Focused wrapper smoke validation passed for:
  - `ConnectWithRoute`
  - `CheckHealth`
  - Primitive read/write paths (`DINT`, `REAL`, `BOOL`, `INT`)
  - Program-scoped tag read/write
  - `ReadTagsBatch`
  - `WriteTagsBatch`
  - `ExecuteBatch`
  - `SubscribeToTag`
  - fail-fast invalid subscription handling
  - `UpsertTagGroup`
  - `ReadTagGroupOnce`
  - `SubscribeToTagGroup`

Observed documented PLC limitations:
- Direct writes to standalone STRING tags fail on this PLC.
- Direct writes to STRING members inside UDTs fail on this PLC.
- Direct writes to UDT array element members fail on this PLC.
- These failure categories matched the same hardware limitations observed from the Rust native validation.

Observed wrapper-specific findings:
- Subscription contract gap: before this validation, `SubscribeToTag("NonExistentTag")` returned a subscription immediately and only failed later inside the polling task. The wrapper was updated during this validation to perform an initial read and fail fast.
- Tag-group disposal cleanup bug: disposing a `TagGroup` manually and later disposing the client could throw because the client attempted an extra `Stop()` on the already-disposed group. The wrapper was updated so client disposal only disposes the group.
- Error classification improved during this pass:
  - direct STRING write failures now surface the native batch-write error (`Multiple Service Response error: 0x1E`)
  - direct UDT-array-member write failures are now surfaced in the wrapper as `PLC does not support writing to UDT array element members directly (Error 0x2107)`
- Batch read path improved during this pass:
  - the Rust FFI now returns structured JSON for native batch reads
  - the C# wrapper now uses native batch read first, and the CompactLogix BOOL-array decoding gap in the Rust/native multiple-service path was fixed during this pass
  - the earlier BOOL fallback path is no longer required for the exercised mixed batch on this PLC
  - this preserved wrapper compatibility while materially improving batch-read throughput on the real PLC

Comprehensive tag-matrix result:
- Total tests: 392
- Passed: 333
- Failed: 59
- Skipped: 0
- Success rate: 84.9%
- The 59 failures aligned with known PLC restrictions, not with new unexpected wrapper regressions.

Hardware benchmark:
- Iterations per scenario: 100
- Tags used:
  - Single read: `gTestArray_DINT[0]`
  - Single write: `gTestArray_DINT[5]`
  - Batch read: `gTestArray_DINT[0-4]`, `gTestArray_REAL[0-1]`, `gTestArray_BOOL[0]`, `gTestArray_INT[0]`, `gTestUDT.Member1_DINT`
  - Batch write: `gTestArray_DINT[5-7]`
  - Mixed execute: read `gTestArray_DINT[0]`, write `gTestArray_DINT[5]`, read `gTestArray_REAL[0]`, read `gTestUDT.Member1_DINT`
- Results:
  - `single_read`: 207.8403 ms total, 2.0784 ms avg call, 481.14 ops/sec
  - `single_write`: 386.4760 ms total, 3.8648 ms avg call, 258.75 ops/sec
  - `batch_read`: 240.2616 ms total, 2.4026 ms avg call, 4162.13 logical ops/sec
  - `batch_write`: 539.7276 ms total, 5.3973 ms avg call, 555.84 logical ops/sec
  - `mixed_execute`: 454.7525 ms total, 4.5475 ms avg call, 879.60 logical ops/sec

Benchmark interpretation:
- Single reads and writes through the wrapper were acceptable on this PLC and did not show an obvious wrapper stability problem.
- Mixed execute and batch write paths showed good throughput because they use native typed batch FFI.
- Batch read throughput improved by roughly `21.7x` versus the earlier wrapper result on the same PLC (`191.85 -> 4162.13 logical ops/sec`) after enabling native batch read and fixing CompactLogix BOOL-array decoding in the Rust/native multiple-service path.
- Batch read is now much closer to Rust native throughput on this CompactLogix target for the exercised mixed batch.

Status assessment:
- The C# wrapper is functionally ready for the exercised real-PLC feature set when using supported operations.
- The wrapper now has improved subscription semantics and safer tag-group cleanup after fixes made during this validation.
- The wrapper now has materially better batch-read performance and clearer error reporting for the known PLC write-limit cases.
- The previous CompactLogix BOOL batch-read gap is now closed for the exercised mixed batch.
- Remaining readiness gaps on this PLC are primarily the documented firmware write limitations for direct STRING writes and direct writes to UDT array element members.

Follow-up issue candidates:
- Preserve richer native PLC status detail where available for direct STRING write failures beyond the current `0x1E` surfaced message.
