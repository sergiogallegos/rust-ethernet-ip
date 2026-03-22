# rust-ethernet-ip Programmer Manual

Practical manual for developers integrating the library in Rust projects or through the C# wrapper.

- Last updated: `2026-03-22`
- Source baseline commit: `8f502c9`

- Stable crate line: `0.6.3`
- Current hardening line: `0.7.0` (unreleased)
- Target PLCs: CompactLogix / ControlLogix

## Contents

1. [Integration Paths](#integration-paths)
2. [Common Concepts](#common-concepts)
3. [Rust Track (Native)](#rust-track-native)
4. [Rust API Catalog](#rust-api-catalog-source-derived)
5. [C# Track (Wrapper)](#c-track-wrapper)
6. [C# API Catalog](#c-api-catalog-source-derived)
7. [Capability Matrix](#capability-matrix)
8. [Known PLC/Firmware Limitations](#known-plcfirmware-limitations)
9. [Production Checklist](#production-checklist)

## Integration Paths

### Path A: Pure Rust

- Use crate `rust-ethernet-ip` directly.
- Async API based on `tokio`.
- Best for Linux/Windows/macOS services and tooling.

### Path B: C# Wrapper

- Use `RustEtherNetIp.dll` wrapper + native library.
- Synchronous typed API for .NET desktop/server apps.
- Best for WPF/WinForms/ASP.NET projects.

## Common Concepts

- Address format: `192.168.1.100:44818`.
- Route path for backplane/slot routing (ControlLogix): slot path through `RoutePath`.
- Supported data types: `BOOL SINT INT DINT LINT USINT UINT UDINT ULINT REAL LREAL STRING UDT`.
- Tag path support: program scope, arrays, bits, and nested UDT members.
- Batch operations: multi-tag read/write/mixed for better throughput and fewer packets.

```text
Program:MainProgram.Motor.Status
DataArray[5]
StatusWord.15
MixerRecipe.Stage[2].TemperatureSetpoint
```

## Rust Track (Native)

### Install

```toml
[dependencies]
rust-ethernet-ip = "0.6.3"
tokio = { version = "1", features = ["full"] }
```

### Quick Start

```rust
use rust_ethernet_ip::{EipClient, PlcValue, RoutePath};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = EipClient::connect("192.168.1.100:44818").await?;

    // Optional routed connect
    let route = RoutePath::new().add_slot(0);
    let mut routed = EipClient::with_route_path("192.168.1.100:44818", route).await?;

    let running = client.read_tag("Program:Main.MotorRunning").await?;
    client.write_tag("Program:Main.SetPoint", PlcValue::Dint(1500)).await?;

    let tags = vec!["Program:Main.Temp", "Program:Main.Pressure"];
    let batch = routed.read_tags_batch(&tags).await?;

    println!("running={running:?}, batch={batch:?}");
    Ok(())
}
```

### Primary API Surface (Rust)

| Category | Main Functions |
|---|---|
| Connection & Session | `EipClient::new`, `EipClient::connect`, `EipClient::with_route_path`, `set_route_path`, `clear_route_path`, `unregister_session` |
| Read/Write Core | `read_tag`, `write_tag`, `read_bit`, `write_bit`, `write_string` |
| Arrays | `read_array_range`, `build_read_array_request`, `build_write_array_request_with_index` |
| UDT | `read_udt_chunked`, `read_udt_member_by_offset`, `write_udt_member_by_offset`, `discover_udt_members`, `get_udt_definition`, `get_udt_definition_cached` |
| Tag Discovery / Metadata | `discover_tags`, `discover_tags_detailed`, `discover_program_tags`, `get_tag_metadata`, `get_tag_attributes`, `list_cached_tag_attributes`, `clear_caches` |
| Batch Operations | `read_tags_batch`, `write_tags_batch`, `execute_batch`, `configure_batch_operations`, `get_batch_config` |
| Tag Group Polling | `upsert_tag_group`, `remove_tag_group`, `list_tag_groups`, `read_tag_group_once`, `subscribe_tag_group` |
| Health / Diagnostics | `check_health`, `check_health_detailed`, `send_cip_request` |
| Subscriptions | `subscribe_to_tag`, `subscribe_to_tags` |

### Rust API Catalog (Source-Derived)

Derived from `src/lib.rs` on March 22, 2026. Focused on developer-facing integration APIs.

```text
// EipClient connection/session
new(addr)
connect(addr)
with_route_path(addr, route)
connect_with_stream(stream, route)
set_route_path(route)
get_route_path()
clear_route_path()
unregister_session()

// Tag discovery / metadata / caches
discover_tags()
discover_tags_detailed()
discover_program_tags(program_name)
get_tag_metadata(tag_name)
get_tag_attributes(tag_name)
discover_udt_members(tag_name)
get_udt_definition(udt_name)
get_udt_definition_cached(udt_name)
list_udt_definitions()
list_cached_tag_attributes()
clear_caches()

// Core reads/writes
read_tag(tag_name)
write_tag(tag_name, value)
read_bit(tag_base, bit_index)
write_bit(tag_base, bit_index, value)
write_string(tag_name, value)

// Arrays / tag path building helpers
read_array_range(base_array_name, start_index, element_count)
build_read_array_request(base_tag_name, start_index, element_count, is_multi_dimensional)
build_write_array_request_with_index(tag_name, index, value_data_type, value_data)
build_element_id_segment(index)
build_base_tag_path(tag_name)
build_list_tags_request()

// UDT helpers
read_udt_chunked(tag_name)
read_udt_member_by_offset(udt_name, member_offset, member_size, data_type)
write_udt_member_by_offset(udt_name, member_offset, member_size, data_type, value)

// Batch operations
execute_batch(operations)
read_tags_batch(tag_names)
write_tags_batch(tag_values)
configure_batch_operations(config)
get_batch_config()

// Tag-group polling
upsert_tag_group(group_name, tags, update_rate_ms)
remove_tag_group(group_name)
list_tag_groups()
read_tag_group_once(group_name)
subscribe_tag_group(group_name)

// Tag-group event classification
TagGroupEventKind::Data
TagGroupEventKind::PartialError
TagGroupEventKind::ReadFailure

// Health / diagnostics / low-level
check_health()
check_health_detailed()
send_cip_request(cip_request)

// Subscriptions
subscribe_to_tag(tag_name, options)
subscribe_to_tags(tag_names, options)
```

## C# Track (Wrapper)

### Install / Reference

- Add wrapper assembly `RustEtherNetIp.dll`.
- Ensure native library `rust_ethernet_ip.dll` is deployed with app output.

### Quick Start

```csharp
using RustEtherNetIp;

using var client = new EtherNetIpClient();
if (client.Connect("192.168.1.100:44818"))
{
    bool running = client.ReadBool("Program:Main.MotorRunning");
    int count = client.ReadDint("Program:Main.ProductionCount");

    client.WriteBool("Program:Main.Start", true);
    client.WriteDint("Program:Main.SetPoint", 1500);
}
```

### Primary API Surface (C#)

| Category | Main Methods |
|---|---|
| Connection | `Connect`, `ConnectWithRoute`, `Disconnect`, `IsConnected` |
| Typed Read/Write | `ReadBool/WriteBool`, `ReadSint/WriteSint`, `ReadInt/WriteInt`, `ReadDint/WriteDint`, `ReadLint/WriteLint`, `ReadUsint/WriteUsint`, `ReadUint/WriteUint`, `ReadUdint/WriteUdint`, `ReadUlint/WriteUlint`, `ReadReal/WriteReal`, `ReadLreal/WriteLreal`, `ReadString/WriteString` |
| UDT | `ReadUdt`, `WriteUdt`, `ReadUdtChunked`, `GetUdtMember`, `SetUdtMember`, `WriteUdtMember`, `ReadUdtMemberByOffset`, `WriteUdtMemberByOffset`, `GetUdtDefinition` |
| Tag Discovery / Metadata | `DiscoverTags`, `DiscoverTagsDetailed`, `GetTagMetadata`, `GetTagAttributes` |
| Batch Operations | `ReadTagsBatch`, `WriteTagsBatch`, `ExecuteBatch` |
| Tag Group Polling | `UpsertTagGroup`, `RemoveTagGroup`, `ListTagGroups`, `ReadTagGroupOnce`, `SubscribeToTagGroup` |
| Subscriptions | `SubscribeToTag`, `UnsubscribeFromTag`, `UnsubscribeFromAllTags` |
| Health / Utility | `CheckHealth`, `CheckHealthDetailed`, `SetMaxPacketSize` |

### Batch Notes (Current 0.7.0 Hardening Line)

- `WriteTagsBatch(...)` and `ExecuteBatch(...)` are native typed FFI-backed.
- `ReadTagsBatch(...)` currently uses sequential type-probing fallback.
- `ConfigureBatchOperations(...)` and `GetBatchConfig()` are intentionally unsupported in wrapper/runtime right now.

### C# API Catalog (Source-Derived)

Derived from `csharp/RustEtherNetIp/EthernetNetIpClient.cs` on March 22, 2026.

```text
// Connection / lifecycle
ConnectWithRoute(string address, RoutePath routePath)
Connect(string address)
Disconnect()
Dispose()
ConnectToPlc(string address)
TryConnectToPlc(string address, int maxRetries = 3, int retryDelayMs = 1000)

// Typed read/write (BOOL..STRING)
ReadBool/WriteBool
ReadSint/WriteSint
ReadInt/WriteInt
ReadDint/WriteDint
ReadLint/WriteLint
ReadUsint/WriteUsint
ReadUint/WriteUint
ReadUdint/WriteUdint
ReadUlint/WriteUlint
ReadReal/WriteReal
ReadLreal/WriteLreal
ReadString/WriteString
WriteStringAsUdt

// UDT / metadata
ReadUdt
WriteUdt (PlcValue and Dictionary<string, object> overloads)
WriteUdtData
ReadUdtAsDictionary
GetUdtMember
SetUdtMember
ReadUdtChunked
ReadUdtMemberByOffset
WriteUdtMemberByOffset
WriteUdtMember
GetUdtDefinition
GetTagAttributes
GetTagMetadata

// Discovery / arrays / rich reads
DiscoverTags
DiscoverTagsDetailed
ReadTagWithDetails
ReadTags
ReadTagsWithDetails
ReadArrayRange
ReadDintArrayRange
ReadRealArrayRange
WriteTags

// Batch operations
ReadTagsBatch
WriteTagsBatch
ExecuteBatch
ConfigureBatchOperations
GetBatchConfig

// Tag-group polling
UpsertTagGroup
RemoveTagGroup
ListTagGroups
ReadTagGroupOnce
SubscribeToTagGroup

// Health / utility / subscriptions
SetMaxPacketSize
CheckHealth
CheckHealthDetailed
SubscribeToTag
UnsubscribeFromTag
UnsubscribeFromAllTags
```

## Capability Matrix

| Capability | Rust | C# Wrapper |
|---|---|---|
| Direct PLC connection | Yes | Yes |
| Route path connection | Yes | Yes |
| All core AB data types | Yes | Yes |
| Program tags / arrays / bits / UDT members | Yes | Yes |
| UDT discovery + definitions | Yes | Yes |
| Batch read/write/mixed | Yes | Yes |
| Batch config API | Yes | Not yet (throws `NotSupportedException`) |
| Health checks | Yes | Yes |
| Subscriptions | Yes | Yes |

## Known PLC/Firmware Limitations

- Some controllers reject direct writes to standalone `STRING` tags.
- Some controllers reject direct writes to UDT array element members (for example `MyUdtArray[0].Member1`).

Recommended workaround pattern in both Rust and C#: read full UDT/element, modify in memory, write full structure back.

## Production Checklist

- Validate all critical tag paths against real PLC programs.
- Run sustained read/write soak tests in your target network topology.
- Capture error handling for disconnects, timeouts, and type mismatches.
- Lock down network access (firewall/VLAN) because EtherNet/IP has limited built-in security.
- Pin stable crate versions in production deployments (`0.6.3` currently published).
