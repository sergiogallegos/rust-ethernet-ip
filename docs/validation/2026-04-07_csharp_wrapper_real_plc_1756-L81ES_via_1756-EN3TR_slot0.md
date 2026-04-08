Date: 2026-04-07
Tester: Codex + Sergio Gallegos
PLC model: 1756-L81ES
Firmware revision: 37
Network topology: Routed Ethernet connection to `192.168.0.101:44818` via `1756-EN3TR`, backplane slot 0 using `ConnectWithRoute`

Scope:
- C# wrapper validation against the same real PLC and `gTest*` tag set used for the Rust validation pass.
- Focused on wrapper behavior, not just native Rust core behavior.

Commands executed:
- `dotnet run --project examples/CSharpWrapperSmoke/CSharpWrapperSmoke.csproj`
- `dotnet run --project examples/CSharpWrapperBenchmark/CSharpWrapperBenchmark.csproj -- --iterations 100`
- `dotnet run --project examples/CSharpWrapperTest/CSharpWrapperTest.csproj`

Environment used:
- `TEST_PLC_ADDRESS=192.168.0.101:44818`
- `TEST_PLC_SLOT=0`

Result:
- PASS: Route-path connection succeeded to `192.168.0.101:44818` with slot 0.
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
- PASS: Comprehensive wrapper read/write validation matched the Rust native pass at `333/392` passed and `59/392` failed.

Observed documented PLC limitations:
- Direct writes to standalone STRING tags fail on this controller path.
- Direct writes to STRING members inside UDTs fail on this controller path.
- Direct writes to UDT array element members fail on this controller path.
- These failure categories matched the same hardware limitations observed from the Rust native validation.

Observed wrapper-specific findings:
- No new ControlLogix-only wrapper regression was found.
- The wrapper fixes previously made during CompactLogix validation carried over correctly here:
  - invalid subscriptions fail fast
  - tag-group disposal remains safe
  - native batch read remains active
  - native write-limit errors remain classified into documented PLC limitations
- On this ControlLogix target, direct STRING write failures surfaced as batch-level `0x1E` embedded service errors in the wrapper, while direct UDT-array-member writes surfaced as the known `0x2107` limitation.

Comprehensive tag-matrix result:
- Total tests: 392
- Passed: 333
- Failed: 59
- Skipped: 0
- Success rate: 84.9%
- Failure grouping:
  - 4 direct STRING write paths
  - 55 direct UDT array element member write paths

Hardware benchmark:
- Iterations per scenario: 100
- Tags used:
  - Single read: `gTestArray_DINT[0]`
  - Single write: `gTestArray_DINT[5]`
  - Batch read: `gTestArray_DINT[0-4]`, `gTestArray_REAL[0-1]`, `gTestArray_BOOL[0]`, `gTestArray_INT[0]`, `gTestUDT.Member1_DINT`
  - Batch write: `gTestArray_DINT[5-7]`
  - Mixed execute: read `gTestArray_DINT[0]`, write `gTestArray_DINT[5]`, read `gTestArray_REAL[0]`, read `gTestUDT.Member1_DINT`
- Results:
  - `single_read`: 236.4368 ms total, 2.3644 ms avg call, 422.95 ops/sec
  - `single_write`: 288.3109 ms total, 2.8831 ms avg call, 346.85 ops/sec
  - `batch_read`: 248.0661 ms total, 2.4807 ms avg call, 4031.18 logical ops/sec
  - `batch_write`: 327.7272 ms total, 3.2773 ms avg call, 915.40 logical ops/sec
  - `mixed_execute`: 255.1135 ms total, 2.5511 ms avg call, 1567.93 logical ops/sec

Benchmark interpretation:
- The wrapper remained stable on this routed ControlLogix path for single reads, writes, and mixed batches.
- Native batch-read throughput remained materially better than the old sequential type-probing wrapper path.
- Wrapper throughput on this ControlLogix target stayed close to the Rust native baseline for the exercised batch workloads.

Status assessment:
- The C# wrapper is functionally ready for the exercised real-PLC ControlLogix feature set when using supported operations.
- The same remaining gaps are controller limitations, not wrapper-only regressions.
- Real-hardware evidence now covers the wrapper on both CompactLogix and ControlLogix families for the current release gate.

Follow-up issue candidates:
- Preserve richer native PLC status detail where available for direct STRING write failures beyond the current `0x1E` surfaced message.
