# Rust EtherNet/IP C# Wrapper

`RustEtherNetIp` is the C# wrapper and NuGet package for the `rust-ethernet-ip` native core.

It is intended for `.NET` applications that need direct communication with Allen-Bradley CompactLogix and ControlLogix controllers without OPC or RSLinx.

Validated scope today:

- individual reads and writes
- route-path access for ControlLogix
- batch read, batch write, and mixed execute
- subscriptions and tag-group polling
- UDT access
- health checks and diagnostics

## Package Status

- current published package: `1.1.0`
- previous published package: `1.0.0`
- current published package target: `.NET 10`
- current packaged native runtimes: `win-x64`, `linux-x64`, `osx-arm64`

If you are evaluating deployment, read:

- [root integration and deployment guide](../../docs/INTEGRATION_AND_DEPLOYMENT.md)
- [programmer manual](../../docs/programmer_manual.md)

## Installation

### NuGet

```bash
dotnet add package RustEtherNetIp --version 1.1.0
```

Or:

```xml
<PackageReference Include="RustEtherNetIp" Version="1.1.0" />
```

### Source-based builds

If you are building from this repository instead of consuming the published package:

1. build the native Rust library with `cargo build --release --features ffi`
2. build your `.NET` project
3. ensure the native library is copied beside your app output

Expected native library names:

- Windows: `rust_ethernet_ip.dll`
- macOS: `librust_ethernet_ip.dylib`
- Linux: `librust_ethernet_ip.so`

## Why Use It

- full Allen-Bradley primitive type coverage plus `STRING` and `UDT`
- program-scoped tags, arrays, bit access, and UDT member paths
- route-path support for routed ControlLogix targets
- native batch read/write/mixed execution paths
- tag-group polling and subscriptions
- health and diagnostics surfaces
- real-PLC validation evidence for both CompactLogix and ControlLogix targets

## Supported PLC Focus

- CompactLogix
- ControlLogix

Current real-hardware validation references:

- [CompactLogix Rust/C# validation](../../docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md)
- [ControlLogix C# validation](../../docs/validation/2026-04-16_csharp_wrapper_real_plc_1756-L81ES_via_1756-EN3TR_slot0.md)

## Quick Start

### Basic Usage

```csharp
using RustEtherNetIp;

// Connect to PLC
using var client = new EtherNetIpClient();
if (client.Connect("192.168.0.1:44818"))
{
    // Individual operations
    bool startButton = client.ReadBool("StartButton");
    int counter = client.ReadDint("ProductionCount");
    float temperature = client.ReadReal("BoilerTemp");
    
    client.WriteBool("EnableFlag", true);
    client.WriteDint("SetPoint", 1500);
    client.WriteReal("TargetTemp", 75.5f);
}
```

## 🚀 Batch Operations

### Batch Read Operations

Read multiple tags in a single optimized operation:

```csharp
string[] tags = {
    "ProductionCount",
    "Temperature_1", 
    "Temperature_2",
    "Pressure_1",
    "FlowRate"
};

var results = client.ReadTagsBatch(tags);

foreach (var result in results)
{
    if (result.Value.Success)
        Console.WriteLine($"{result.Key}: {result.Value.Value}");
    else
        Console.WriteLine($"{result.Key}: Error - {result.Value.ErrorMessage}");
}
```

### Batch Write Operations

Write multiple tags efficiently:

```csharp
var tagValues = new Dictionary<string, object>
{
    { "SetPoint_1", 1500 },
    { "SetPoint_2", 1750 },
    { "TargetTemp", 75.5f },
    { "EnableFlag", true },
    { "RecipeNumber", 42 }
};

var results = client.WriteTagsBatch(tagValues);

foreach (var result in results)
{
    if (result.Value.Success)
        Console.WriteLine($"{result.Key}: Write successful");
    else
        Console.WriteLine($"{result.Key}: Error - {result.Value.ErrorMessage}");
}
```

### Mixed Batch Operations

Execute reads and writes together for coordinated control:

```csharp
var operations = new[]
{
    BatchOperation.Read("CurrentTemp"),
    BatchOperation.Read("CurrentPressure"),
    BatchOperation.Write("TempSetpoint", 78.5f),
    BatchOperation.Write("PressureSetpoint", 15.2f),
    BatchOperation.Write("AutoModeEnabled", true)
};

var results = client.ExecuteBatch(operations);

foreach (var result in results)
{
    string operation = result.IsWrite ? "Write" : "Read";
    if (result.Success)
    {
        string valueInfo = result.IsWrite ? "" : $" = {result.Value}";
        Console.WriteLine($"✅ {operation} {result.TagName}{valueInfo} ({result.ExecutionTimeMs:F1}ms)");
    }
    else
    {
        Console.WriteLine($"❌ {operation} {result.TagName}: {result.ErrorMessage}");
    }
}
```

## Performance Configuration

Batch configuration APIs are currently **unsupported** in this release line.
The following methods intentionally throw `NotSupportedException`:

- `ConfigureBatchOperations(BatchConfig config)`
- `GetBatchConfig()`

Default native batch behavior is still available through:

- `ReadTagsBatch(...)`
- `WriteTagsBatch(...)`
- `ExecuteBatch(...)`

Ordering note:
- `ReadTagsBatch(...)` and `WriteTagsBatch(...)` preserve per-tag association in their dictionary results.
- `ExecuteBatch(...)` returns per-operation results, but mixed operations may be regrouped natively for packet optimization, so callers should match on `TagName` and operation type rather than assuming strict mixed-input ordering.

## Performance Comparison

Illustrative only: actual timings depend on hardware, network, route path, and workload shape.

| Operation Type | Individual | Batch | Improvement |
|----------------|------------|-------|-------------|
| 5 tag reads | 15ms | 3ms | **5x faster** |
| 10 tag writes | 25ms | 5ms | **5x faster** |
| 20 mixed ops | 50ms | 8ms | **6.25x faster** |
| Network packets | 20 packets | 1 packet | **20x reduction** |

## Tag Group Polling Events

Use TagGroup polling when you need periodic multi-tag updates with explicit quality classification.

```csharp
client.UpsertTagGroup("cell_1", new[] { "DINT_TAG", "PressureTag" }, updateRateMs: 250);
var group = client.SubscribeToTagGroup("cell_1");

group.PollingEvent += (_, evt) =>
{
    switch (evt.Kind)
    {
        case TagGroupEventKind.Data:
            // All reads succeeded in this cycle
            break;
        case TagGroupEventKind.PartialError:
            // Some tags failed; check evt.Errors
            Console.WriteLine($"PartialError: {evt.Errors.Count} tag(s)");
            break;
        case TagGroupEventKind.ReadFailure:
            // Full scan failed; check evt.ErrorMessage + evt.Failure
            Console.WriteLine($"ReadFailure: {evt.ErrorMessage}");
            break;
    }
};
```

Event model:
- `Data`: all configured tags read successfully.
- `PartialError`: mixed cycle; some reads succeeded and some failed.
- `ReadFailure`: cycle-level failure (for example disconnect/transport/session issue).

Compatibility note:
- `DataChanged` remains available and is still useful for direct UI value binding.
- `PollingEvent` adds explicit diagnostic semantics for robust industrial workflows.

## Advanced Tag Addressing

The wrapper supports all advanced Allen-Bradley tag addressing features:

```csharp
// Program-scoped tags
var motorStatus = client.ReadBool("Program:MainProgram.Motor.Status");

// Array element access
var arrayElement = client.ReadDint("MyArray[5]");
var multiDimArray = client.ReadDint("Matrix[2,3,1]");

// Bit-level operations
var statusBit = client.ReadBool("StatusWord.15");

// UDT member access
var udtMember = client.ReadReal("MyUDT.Temperature.Value");

// String operations
var stringLength = client.ReadDint("MyString.LEN");
var stringData = client.ReadString("MyString.DATA");
```

## Error Handling

The wrapper provides comprehensive error handling:

```csharp
try
{
    var value = client.ReadDint("NonExistentTag");
}
catch (Exception ex)
{
    Console.WriteLine($"Error: {ex.Message}");
    
    // Specific error types available:
    // - TagNotFoundException
    // - DataTypeMismatchException  
    // - NetworkException
    // - CipProtocolException
}
```

## Known Limitations

The following operations are **not supported** due to PLC firmware restrictions. These limitations are inherent to the Allen-Bradley PLC firmware and cannot be bypassed at the library level.

### STRING Writing

Top-level standard Logix `STRING` tags can be written directly with the structure encoding used by the Rust core.

Direct writes to `STRING` members inside UDTs remain a restricted hardware path until separately validated.

**What Works:**
- ✅ Reading STRING tags: `gTest_STRING` (read successfully)
- ✅ Writing top-level STRING tags: `gTest_STRING` (write successfully)
- ✅ Reading STRING members in UDTs: `gTestUDT.Member5_String` (read successfully)

**What Doesn't Work:**
- ❌ Writing STRING members in UDTs directly: `gTestUDT.Member5_String` (write fails)

**Workaround for STRING Members in UDTs:**
```csharp
// Read entire UDT
var udt = client.ReadUdt("gTestUDT");

// Modify STRING member in memory (if UDT structure is known)
// ... modify UDT structure ...

// Write entire UDT back
client.WriteUdt("gTestUDT", udt);
```

### UDT Array Element Member Writing

**Cannot write directly to members of UDT array elements** (e.g., `gTestUDT_Array[0].Member1_DINT`).

**Root Cause:** PLC firmware limitation (CIP Error 0x2107). The PLC does not support direct write operations to individual members within UDT array elements.

**What Works:**
- ✅ Reading UDT array element members: `gTestUDT_Array[0].Member1_DINT` (read successfully)
- ✅ Writing entire UDT array elements: `gTestUDT_Array[0]` (write full UDT structure)
- ✅ Writing UDT members (non-array): `gTestUDT.Member1_DINT` (write individual members)
- ✅ Writing simple array elements: `gArray[5]` (write elements of simple arrays)

**What Doesn't Work:**
- ❌ Writing UDT array element members: `gTestUDT_Array[0].Member1_DINT` (write fails)
- ❌ Writing program-scoped UDT array element members: `Program:TestProgram.gTestUDT_Array[0].Member1_DINT` (write fails)

**Workaround:**
```csharp
// Read entire UDT array element
var element = client.ReadUdt("gTestUDT_Array[0]");

// Modify member in memory (if UDT structure is known)
// ... modify UDT structure ...

// Write entire UDT array element back
client.WriteUdt("gTestUDT_Array[0]", element);
```

### Summary

**Important Notes:**
- These limitations are **PLC firmware restrictions**, not library bugs
- The library correctly implements the EtherNet/IP and CIP protocols
- All read operations work correctly for all tag types
- Workarounds are available for UDT array element members and STRING members in UDTs
- Standalone standard STRING tag writes are supported through the direct `WriteString` API
- Real-hardware validation on `5069-L320ERMS3` firmware `35` is recorded in:
  - `docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md`

## Use Cases

### Data Acquisition

Perfect for reading multiple sensor values:

```csharp
string[] sensors = {
    "Temperature_Zone1", "Temperature_Zone2", "Temperature_Zone3",
    "Pressure_Tank1", "Pressure_Tank2", 
    "FlowRate_Line1", "FlowRate_Line2"
};

var sensorData = client.ReadTagsBatch(sensors);
```

### Recipe Management

Efficiently update multiple setpoints:

```csharp
var recipe = new Dictionary<string, object>
{
    { "Temp_Setpoint_1", 180.5f },
    { "Temp_Setpoint_2", 165.0f },
    { "Pressure_Setpoint", 25.7f },
    { "Speed_Setpoint", 1200 },
    { "Recipe_Active", true }
};

client.WriteTagsBatch(recipe);
```

### Coordinated Control

Atomic read-then-write operations:

```csharp
var operations = new[]
{
    // Read current states
    BatchOperation.Read("Current_Position"),
    BatchOperation.Read("Current_Speed"),
    BatchOperation.Read("System_Ready"),
    
    // Update control outputs based on logic
    BatchOperation.Write("Target_Position", newPosition),
    BatchOperation.Write("Speed_Command", calculatedSpeed),
    BatchOperation.Write("Start_Command", true)
};

var results = client.ExecuteBatch(operations);
```

## System Requirements

- **.NET 6.0 or later**
- **Windows 10/11, Linux, or macOS**
- **Network access to Allen-Bradley PLC**
- **rust_ethernet_ip.dll** (included)

## Architecture

```
┌─────────────────────────────────────────┐
│           C# Application                │
│  ┌─────────────────────────────────────┐│
│  │     Your Business Logic             ││
│  └─────────────────────────────────────┘│
└─────────────┬───────────────────────────┘
              │
┌─────────────┴───────────────────────────┐
│        C# Wrapper (This Library)       │
│  • Type-safe API                       │
│  • Batch Operations                    │
│  • Error Handling                      │
│  • Memory Management                   │
└─────────────┬───────────────────────────┘
              │ P/Invoke
┌─────────────┴───────────────────────────┐
│         Rust Core Library              │
│  • EtherNet/IP Protocol                │
│  • CIP Implementation                  │
│  • Network Communication               │
│  • Performance Optimization            │
└─────────────┬───────────────────────────┘
              │ TCP/IP
┌─────────────┴───────────────────────────┐
│        Allen-Bradley PLC               │
│  • CompactLogix / ControlLogix         │
│  • EtherNet/IP Port 44818              │
└─────────────────────────────────────────┘
```

## Thread Safety

The `EtherNetIpClient` is **NOT** thread-safe. For multi-threaded applications:

- Use one client per thread, OR
- Implement external synchronization, OR  
- Use a connection pool pattern

## Troubleshooting

### Common Issues

1. **Connection Failed**
   - Verify PLC IP address and port (44818)
   - Check network connectivity
   - Ensure PLC EtherNet/IP is enabled

2. **Tag Not Found**
   - Verify tag name spelling and case
   - Check tag scope (global vs program-scoped)
   - Ensure tag exists in PLC program

3. **Data Type Mismatch**
   - Use correct read method for tag data type
   - Check PLC tag definition

4. **Performance Issues**
   - Use batch operations for multiple tags
   - Batch configuration APIs are currently unsupported in this release line
   - Monitor network packet size limits

## API Reference

### Core Classes

- **`EtherNetIpClient`**: Main client class
- **`TagGroup`**: Periodic multi-tag polling helper
- **`TagGroupPollingEventArgs`**: Classified tag-group cycle result payload
- **`TagGroupFailureDiagnostic`**: Structured failure diagnostics for read-failure cycles
- **`BatchOperation`**: Represents a batch operation
- **`BatchConfig`**: Batch configuration model (currently not applied via API in this release line)
- **`TagReadResult`**: Result of a tag read operation
- **`TagWriteResult`**: Result of a tag write operation
- **`BatchOperationResult`**: Result of a batch operation

### Extension Methods

- **`EtherNetIpExtensions.ConnectToPlc()`**: One-line connection
- **`EtherNetIpExtensions.TryConnectToPlc()`**: Connection with retry logic

## Contributing

This wrapper is part of the larger rust-ethernet-ip project. Contributions are welcome!

## License

This project is licensed under the MIT License. See the LICENSE file for details.

## Support

For issues and questions:

- check the troubleshooting and limitation sections above
- review the example projects in `examples/`
- use [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues) for reproducible bugs
- use [GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions) for integration questions

The project is also open to:

- priority issue handling
- priority feature sponsorship
- integration support for real deployments
- OEM or system-integrator deployment feedback
- companies willing to provide specific hardware for validation

## Version History

### v1.1.0 (Current Stable)
- ✅ New async API — `ReadDintAsync` / `WriteBoolAsync` / `ReadStringAsync` / batch / `CheckHealthAsync`. **Note:** these are `Task.Run` wrappers over the blocking native calls — they let you `await` and keep UI threads responsive, but they do **not** make the underlying socket I/O non-blocking (one thread-pool thread is occupied per in-flight call). True non-blocking FFI is future work.
- ✅ Richer errors — operations now throw `PlcException` carrying the native CIP failure reason via `eip_get_last_error` (capability `CAP_LAST_ERROR`), e.g. "CIP Error 0x04: Path segment error"
- ✅ Fixed: typed UDT writes (`WriteUdt`/`WriteUdtData`) serialize correctly; a failed scalar write is no longer re-issued to the PLC; added a finalizer + thread-safe `Dispose`; `RoutePath.AddPort` before an address no longer drops the port
- ✅ Multi-RID NuGet package: native runtimes for `win-x64`, `linux-x64`, `osx-arm64`
- ✅ Real CompactLogix 5069-L330ERM fw38 validation: 2299/2299 reads, 2206/2206 writes, 2206/2206 verify, 0 anomalies
- No breaking changes to the public API or the C ABI (ABI version still `1`)

### v1.0.0
- ✅ SemVer-major release-window bundle (`#[non_exhaustive]` on public enums, `RoutePath` private storage, typed `try_init_tracing`, etc.)
- ✅ Native FFI ABI version + capability handshake — wrapper rejects mismatched native libraries at load time via `BadImageFormatException`
- ✅ BOOL-array DWORD-offset fix: `gTestArray_BOOL[i]` for `i >= 32` now addresses the correct DWORD instead of aliasing to DWORD[0]
- ✅ Nested BOOL-in-UDT-array-element path: `gTestUDT_Array[i].Array_BOOL[j]` now returns `Bool` (was returning the whole DWORD as `Udint` or failing with CIP `0x05`)
- ✅ Ordered route-hop FFI exports (`eip_connect_with_route_hops`, `eip_set_route_path_hops`); legacy grouped exports kept as compat shims
- ✅ Real ControlLogix 1756-L75 fw33 validation: 2299/2299 reads, 2206/2206 writes, 2206/2206 verify, 0 anomalies

### v0.7.0
- ✅ Rust/C# parity improvements for batch, subscriptions, and tag-group polling
- ✅ Real CompactLogix and ControlLogix validation evidence on file
- ✅ Improved native batch-read behavior and clearer PLC firmware-limit diagnostics

### v0.6.3
- ✅ Reliability-focused protocol and wrapper fixes
- ✅ Batch read/write/execute paths available for production usage
- ✅ Explicit unsupported gating for batch configuration APIs (`ConfigureBatchOperations`, `GetBatchConfig`)

### v0.2.0
- Individual tag operations
- Basic error handling
- Core data types

### v0.1.0
- Initial release
- Basic connectivity 
