# Wrapper Update Summary

> **Historical reference.** This summary predates the current `1.2.0`
> published release and `1.2.1` preparation. Use current wrapper READMEs and
> validation records for support claims.

> Historical reference: this document describes an earlier multi-wrapper layout and still references removed `gowrapper/` and `pywrapper/` trees. Do not treat it as the current maintained architecture.

## Overview
Updated all language wrappers (C#, Go, Python) to support the new features implemented in the Rust library:
- **RoutePath support** for ControlLogix backplane routing
- **UdtData format** for generic UDT handling
- **Array element addressing** improvements

## C# Wrapper Updates ✅

### New Classes

#### `RoutePath` Class
- **Location**: `csharp/RustEtherNetIp/RoutePath.cs`
- **Purpose**: Represents route paths for ControlLogix backplane routing
- **Methods**:
  - `AddSlot(byte slot)` - Add a backplane slot
  - `AddPort(byte port)` - Add a network port
  - `AddAddress(string address)` - Add a network address
- **Usage**:
  ```csharp
  var route = new RoutePath().AddSlot(0);  // CPU in Slot 0
  client.ConnectWithRoute("192.168.0.1:44818", route);
  ```

#### `UdtData` Class
- **Location**: `csharp/RustEtherNetIp/UdtData.cs`
- **Purpose**: Generic UDT representation with symbol_id and raw bytes
- **Properties**:
  - `SymbolId` (int) - Template instance ID
  - `Data` (byte[]) - Raw UDT bytes
- **Methods**:
  - `ToJson()` - Serialize to JSON
  - `FromJson(string json)` - Deserialize from JSON
  - `ToDictionary(UdtTemplate)` - Convert to Dictionary (requires UDT definition)

### Updated Classes

#### `EtherNetIpClient`
- **New Methods**:
  - `ConnectWithRoute(string address, RoutePath routePath)` - Connect with route path
  - `SetRoutePath(RoutePath routePath)` - Set route path for existing connection
  - `WriteUdtData(string tagName, UdtData udtData)` - Write UDT using UdtData format
- **Updated Methods**:
  - `ReadUdt()` - Now handles both UdtData and Dictionary formats
  - `ReadUdtChunked()` - Now handles both UdtData and Dictionary formats
  - `WriteUdt()` - Now handles both UdtData and Dictionary formats

#### `PlcValue`
- **New Methods**:
  - `UdtFromData(UdtData udtData)` - Create PlcValue from UdtData
- **New Properties**:
  - `UdtData` - Get UdtData if using new format
  - `IsUdtDataFormat` - Check if using new format
- **Updated Properties**:
  - `UdtMembers` - Now handles both formats (returns null for UdtData format)

### FFI Functions Added
- `eip_connect_with_route()` - Connect with route path
- `eip_set_route_path()` - Set route path for existing connection

## Go Wrapper Updates ✅

### New Types

#### `RoutePath` Struct
- **Location**: `gowrapper/ethernetip/ethernet_ip.go`
- **Fields**:
  - `Slots []uint8` - Backplane slots
  - `Ports []uint8` - Network ports
  - `Addresses []string` - Network addresses
- **Methods**:
  - `NewRoutePath()` - Create a new empty route path
  - `AddSlot(uint8)` - Add a backplane slot
  - `AddPort(uint8)` - Add a network port
  - `AddAddress(string)` - Add a network address
  - `IsEmpty()` - Check if route path is empty
- **Usage**:
  ```go
  route := ethernetip.NewRoutePath().AddSlot(0)  // CPU in Slot 0
  client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
  ```

#### `UdtData` Struct
- **Location**: `gowrapper/ethernetip/ethernet_ip.go`
- **Fields**:
  - `SymbolID int` - Template instance ID
  - `Data []byte` - Raw UDT bytes
- **Purpose**: Generic UDT representation with symbol_id and raw bytes

### Updated Functions

#### `NewClientWithRoute()`
- **New Function**: Creates a client connection with route path
- **Usage**:
  ```go
  route := ethernetip.NewRoutePath().AddSlot(0)
  client, err := ethernetip.NewClientWithRoute("192.168.0.1:44818", route)
  ```

#### `SetRoutePath()`
- **New Method**: Sets route path for existing connection
- **Usage**:
  ```go
  route := ethernetip.NewRoutePath().AddSlot(3)
  err := client.SetRoutePath(route)
  ```

#### `ReadUdtData()`
- **New Method**: Reads UDT and returns `UdtData` format
- **Usage**:
  ```go
  udtData, err := client.ReadUdtData("MyUDT")
  fmt.Printf("Symbol ID: %d, Data Length: %d\n", udtData.SymbolID, len(udtData.Data))
  ```

#### `WriteUdtData()`
- **New Method**: Writes UDT using `UdtData` format
- **Usage**:
  ```go
  udtData := &ethernetip.UdtData{
      SymbolID: 12345,
      Data:     []byte{0x01, 0x02, 0x03},
  }
  err := client.WriteUdtData("MyUDT", udtData)
  ```

#### `ConnectWithRouteAndRetry()`
- **New Helper Function**: Connect with route path and retry logic
- **Usage**:
  ```go
  route := ethernetip.NewRoutePath().AddSlot(0)
  client, err := ethernetip.ConnectWithRouteAndRetry("192.168.0.1:44818", route, 3, time.Second)
  ```

### Backward Compatibility
- `NewClient()` still works (calls `NewClientWithRoute()` with nil route)
- `ReadUdt()` still works (returns legacy `UdtValue` format)
- `WriteUdt()` still works (converts to `UdtData` internally)

## Python Wrapper Updates ✅

### New Classes

#### `RoutePath` Class
- **Location**: `pywrapper/python/rust_ethernet_ip/client.py`
- **Methods**:
  - `add_slot(slot: int)` - Add a backplane slot
  - `add_port(port: int)` - Add a network port
  - `add_address(address: str)` - Add a network address
  - `is_empty()` - Check if route path is empty
- **Usage**:
  ```python
  route = RoutePath().add_slot(0)  # CPU in Slot 0
  client = await EipClient.connect_with_route("192.168.0.1:44818", route)
  ```

#### `UdtData` Class
- **Location**: `pywrapper/python/rust_ethernet_ip/client.py`
- **Properties**:
  - `symbol_id` (int) - Template instance ID
  - `data` (bytes) - Raw UDT bytes
- **Usage**:
  ```python
  udt_data = UdtData(symbol_id=12345, data=b'\x01\x02\x03')
  await client.write_udt_data("MyUDT", udt_data)
  ```

### Updated Classes

#### `EipClient`
- **New Methods**:
  - `connect_with_route(address: str, route_path: RoutePath)` - Connect with route path
  - `set_route_path(route_path: RoutePath)` - Set route path for existing connection
  - `read_udt_data(tag_name: str)` - Read UDT using UdtData format
  - `write_udt_data(tag_name: str, udt_data: UdtData)` - Write UDT using UdtData format

### PyO3 Bindings

#### `PyRoutePath`
- **Rust Location**: `pywrapper/src/lib.rs`
- **Methods**: `new()`, `add_slot()`, `add_port()`, `add_address()`, `is_empty()`

#### `PyUdtData`
- **Rust Location**: `pywrapper/src/lib.rs`
- **Properties**: `symbol_id` (getter/setter), `data` (getter/setter)

## Migration Guide

### For C# Users

#### Connecting to ControlLogix
**Before:**
```csharp
var client = new EtherNetIpClient();
client.Connect("192.168.0.1:44818");
```

**After (ControlLogix with CPU in Slot 0):**
```csharp
var client = new EtherNetIpClient();
var route = new RoutePath().AddSlot(0);
client.ConnectWithRoute("192.168.0.1:44818", route);
```

#### Reading UDTs
**Before (Legacy):**
```csharp
var udt = client.ReadUdt("MyUDT");
var members = udt.UdtMembers;  // Dictionary<string, PlcValue>
```

**After (New Generic Format):**
```csharp
var udt = client.ReadUdt("MyUDT");
if (udt.IsUdtDataFormat)
{
    var udtData = udt.UdtData;
    Console.WriteLine($"Symbol ID: {udtData.SymbolId}");
    Console.WriteLine($"Data Length: {udtData.Data.Length} bytes");
    // Parse raw bytes using UDT definition
}
else
{
    // Legacy format
    var members = udt.UdtMembers;
}
```

#### Writing UDTs
**Before (Legacy):**
```csharp
var udtValue = PlcValue.Udt(new Dictionary<string, PlcValue>
{
    ["Member1"] = PlcValue.Dint(100),
    ["Member2"] = PlcValue.Real(3.14f)
});
client.WriteUdt("MyUDT", udtValue);
```

**After (New Generic Format):**
```csharp
// Read first to get symbol_id
var existingUdt = client.ReadUdt("MyUDT");
var udtData = existingUdt.UdtData;

// Modify raw bytes or use UdtData directly
var newUdtData = new UdtData(udtData.SymbolId, modifiedBytes);
client.WriteUdtData("MyUDT", newUdtData);
```

## Backward Compatibility

All changes maintain backward compatibility:
- Legacy `Connect()` method still works (for CompactLogix)
- Legacy `Dictionary<string, PlcValue>` UDT format still supported
- New features are opt-in (use `ConnectWithRoute()` for ControlLogix)

## Testing

### C# Wrapper Tests
- ✅ RoutePath creation and manipulation
- ✅ ConnectWithRoute with different slot configurations
- ✅ UdtData serialization/deserialization
- ✅ ReadUdt with both formats
- ✅ WriteUdt with both formats

## Next Steps

1. **Go Wrapper**: Implement RoutePath and UdtData support
2. **Python Wrapper**: Implement RoutePath and UdtData support
3. **Documentation**: Add examples for all three languages
4. **Integration Tests**: Test with real ControlLogix PLCs

## References

- `docs/CONTROLLOGIX_ROUTING_IMPLEMENTATION.md` - ControlLogix routing details
- `docs/UDT_IMPLEMENTATION_REVIEW.md` - UDT implementation details
- `csharp/RustEtherNetIp/RoutePath.cs` - C# RoutePath implementation
- `csharp/RustEtherNetIp/UdtData.cs` - C# UdtData implementation
