# rust-ethernet-ip Programmer Manual

Practical manual for developers integrating the library in Rust projects or through the C# wrapper.

- Last updated for release: `1.2.1`
- Release date: `2026-08-22`

- Current published stable line: `1.2.1` (crates.io + NuGet + PyPI)
- Previous stable line: `1.2.0`
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
rust-ethernet-ip = "1.2.1"
tokio = { version = "1", features = ["full"] }
```

`1.2.1` is the current published stable release on crates.io, NuGet, and PyPI. The crate re-exports four publishable sibling crates (`rust-ethernet-ip-{types,protocol,tag-path,udt}`); source builds from `main` track the same line.

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
| Read/Write Core | `read_tag`, `write_tag`, `read_bit`, `write_bit`, `write_string_tag` |
| Arrays | `read_array_range`, `build_read_array_request`, `build_write_array_request_with_index` |
| UDT | `read_udt_chunked`, `write_udt_member`, `write_udt_array_member`, `discover_udt_members`, `get_udt_definition`, `get_udt_definition_cached` |
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
write_string_tag(tag_name, value)

// Arrays / tag path building helpers
read_array_range(base_array_name, start_index, element_count)
build_read_array_request(base_tag_name, start_index, element_count, is_multi_dimensional)
build_write_array_request_with_index(tag_name, index, value_data_type, value_data)
build_element_id_segment(index)
build_base_tag_path(tag_name)
build_list_tags_request()

// UDT helpers
read_udt_chunked(tag_name)
write_udt_member(udt_name, member_name, value)
write_udt_array_member(array_name, index, member_name, value)

// Deprecated compatibility stubs retained until 2.0:
write_string(tag_name, value)
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
TagGroupFailureDiagnostic { category, retriable, status_code }

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
| Tag Group Polling | `UpsertTagGroup`, `RemoveTagGroup`, `ListTagGroups`, `ReadTagGroupOnce`, `SubscribeToTagGroup`, `TagGroup.PollingEvent` (`Data` / `PartialError` / `ReadFailure`) |
| Subscriptions | `SubscribeToTag`, `UnsubscribeFromTag`, `UnsubscribeFromAllTags` |
| Health / Utility | `CheckHealth`, `CheckHealthDetailed`, `SetMaxPacketSize` |

### Batch Notes

- `WriteTagsBatch(...)` and `ExecuteBatch(...)` are native typed FFI-backed.
- `ReadTagsBatch(...)` uses the native batch-read FFI path first.
- On the validated CompactLogix `5069-L320ERMS3` / firmware `35`, mixed native batch reads now include controller BOOL array elements correctly.
- `ConfigureBatchOperations(...)` and `GetBatchConfig()` are intentionally unsupported in wrapper/runtime right now.
- Rust `execute_batch(...)` may regroup mixed operations for packet optimization, so callers should correlate results by operation metadata rather than assuming strict mixed-input ordering.

### CompactLogix Validation Snapshot

- Real hardware validated: `5069-L320ERMS3`, firmware `35`
- Validated areas: primitive reads/writes, program-scoped tags, route-path connection, subscriptions, UDT reads, batch read/write/mixed execute, C# wrapper parity
- Remaining observed firmware limits on that target:
  - historical direct standalone/UDT `STRING` write failures before the standard STRING encoding fix
  - historical direct writes to UDT array element members before CODEX-AV revalidated scalar member paths

See:
- `docs/validation/2026-04-07_real_plc_5069-L320ERMS3_fw35.md`
- `docs/validation/2026-04-07_csharp_wrapper_real_plc_5069-L320ERMS3_fw35.md`

### Tag Group Event Handling (Rust + C#)

Use the event kind as your first branch, then inspect details:

- `Data`: all configured tags read successfully in that cycle.
- `PartialError`: at least one tag failed, but others succeeded.
- `ReadFailure`: full cycle failed (transport/session/scan-level issue).

Rust pattern:

```rust
while let Some(event) = subscription.wait_for_update().await {
    match event.kind {
        TagGroupEventKind::Data => {}
        TagGroupEventKind::PartialError => {
            // inspect event.snapshot.values[*].error
        }
        TagGroupEventKind::ReadFailure => {
            // inspect event.error and event.failure (category/retriable/status_code)
        }
    }
}
```

C# pattern:

```csharp
group.PollingEvent += (_, evt) =>
{
    switch (evt.Kind)
    {
        case TagGroupEventKind.Data:
            break;
        case TagGroupEventKind.PartialError:
            // inspect evt.Errors
            break;
        case TagGroupEventKind.ReadFailure:
            // inspect evt.ErrorMessage and evt.Failure
            break;
    }
};
```

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
WriteUdtMember
GetUdtDefinition
GetTagAttributes
GetTagMetadata

// Deprecated compatibility stubs retained until 2.0:
ReadUdtMemberByOffset
WriteUdtMemberByOffset

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
TagGroup.PollingEvent
TagGroupEventKind (Data, PartialError, ReadFailure)
TagGroupFailureDiagnostic (Category, Retriable, StatusCode)

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

- Standalone standard `STRING` tags are writeable with the validated Logix structure encoding.
- Scalar UDT array element members are writeable on 5069-L330ERM fw38 when the full member path is preserved.
- `STRING` members inside UDTs and UDT array elements reject with `0x2107` under the current member encoding.

Recommended workaround pattern for rejected STRING members in both Rust and C#: read full UDT/element, modify in memory, write full structure back.

## Production Checklist

- Validate all critical tag paths against real PLC programs.
- Run sustained read/write soak tests in your target network topology.
- Capture error handling for disconnects, timeouts, and type mismatches.
- Lock down network access (firewall/VLAN) because EtherNet/IP has limited built-in security.
- Pin stable crate versions in production deployments (`1.2.1` is the current published stable release).
