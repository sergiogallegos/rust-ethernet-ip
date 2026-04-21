# Rust EtherNet/IP C# Wrapper

A high-performance C# wrapper for the Rust EtherNet/IP library, enabling communication with Allen-Bradley CompactLogix and ControlLogix PLCs. This wrapper provides both traditional individual tag operations and native batch operations for industrial automation applications.

## 🚀 **New: Batch Operations**

Batch operations can substantially reduce round trips for multi-tag scenarios, but the actual gain depends on controller model, route path, packet sizing, and tag mix.

Implementation note for `0.7.0`:
- `WriteTagsBatch(...)` and `ExecuteBatch(...)` use native typed FFI batch paths.
- `ReadTagsBatch(...)` uses the native batch-read path first, with fallback only when needed.

### Key Benefits

- **🚀 Performance**: Fewer round trips and better throughput for multi-tag workloads
- **📡 Network Efficiency**: 1 packet instead of N packets (50x reduction in network traffic)
- **💪 PLC Efficiency**: Lower CPU usage on the PLC
- **⚡ Throughput**: Perfect for data acquisition and coordinated control
- **🔧 Flexibility**: Mixed read/write operations in a single batch

## Features

### Core Capabilities
- **Complete Data Type Support**: All Allen-Bradley data types (BOOL, SINT, INT, DINT, LINT, USINT, UINT, UDINT, ULINT, REAL, LREAL, STRING, UDT)
- **Advanced Tag Addressing**: Program-scoped tags, arrays, bit operations, UDT members
- **High Performance**: 1,500+ reads/sec, 800+ writes/sec for individual operations
- **Batch Operations**: Native multi-tag read/write/mixed execution paths
- **Cross-Platform**: Windows, Linux, macOS support
- **Type Safety**: Strongly-typed API with comprehensive error handling

### Supported PLCs
- **CompactLogix**: L1x, L2x, L3x, L4x, L5x series
- **ControlLogix**: L6x, L7x, L8x series

## Quick Start

### Installation

1. Add the `RustEtherNetIp.dll` and `rust_ethernet_ip.dll` to your project
2. Reference the wrapper in your C# application

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

### STRING Tag Writing

**Cannot write directly to STRING tags** (e.g., `gTest_STRING`, `Program:TestProgram.gTest_STRING`).

**Root Cause:** PLC firmware limitation. On validated CompactLogix hardware this can surface either as batch-level `0x1E` (`Embedded service error`) or extended `0x2107`, depending on the request path.

**What Works:**
- ✅ Reading STRING tags: `gTest_STRING` (read successfully)
- ✅ Reading STRING members in UDTs: `gTestUDT.Member5_String` (read successfully)

**What Doesn't Work:**
- ❌ Writing simple STRING tags: `gTest_STRING` (write fails - PLC limitation)
- ❌ Writing program-scoped STRING tags: `Program:TestProgram.gTest_STRING` (write fails)
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

**Note:** For standalone STRING tags (not part of a UDT), there is no workaround at the communication library level.

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
- Standalone STRING tag writes have no workaround at the communication library level
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
- Check the troubleshooting section above
- Review the examples in `Program.cs`
- File issues on the GitHub repository

## Version History

### v0.7.0 (Current Stable)
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
