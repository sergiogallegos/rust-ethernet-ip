# Rust EtherNet/IP for C#

`RustEtherNetIp` is the .NET wrapper for direct EtherNet/IP communication with
Allen-Bradley CompactLogix and ControlLogix controllers. It is designed for
industrial desktop applications, MES services, test stations, machine tools,
and data-collection services that need Logix tag access without an OPC layer.

## Release Status

- current published package: `1.2.1` on NuGet
- Previous published package: `1.2.0`
- Target framework: `.NET 10`
- Packaged native runtimes: `win-x64`, `linux-x64`, `osx-arm64`
- Native C ABI used by 1.2.1: version `3` (additive over `1.2.0`'s v2 — the
  new `eip_refresh_schema` export and `CAP_SCHEMA_REFRESH` capability bit;
  no existing export changed shape)

The 1.2.1 hardware gate exercised Rust, C#, Python, and C/C++ against a
ControlLogix 1756-L75 firmware 33: the schema-change gate (online array-shape
swap and offline UDT layout-edit), a four-binding full-coverage rerun
(2,304/2,304 reads, 2,285/2,285 writes, 0 anomalies), and a cross-binding
performance rerun, all zero-failure. The prior 1.2.0 gate exercised a
CompactLogix 5069-L330ERM firmware 38: each binding completed 2,338 reads and
2,319 writes plus read-back verification with zero unexpected anomalies.

## Install

```bash
dotnet add package RustEtherNetIp --version 1.2.1
```

Or add the package reference directly:

```xml
<PackageReference Include="RustEtherNetIp" Version="1.2.1" />
```

The NuGet package contains the managed assembly and its native Rust library.
When building from this repository, build the native library first:

```bash
cargo build --release --features ffi --locked
dotnet build csharp/RustEtherNetIp/RustEtherNetIp.csproj -c Release
```

## Start Here

Replace the example address and tag names with tags that exist in your Logix
project. Include port `44818` explicitly.

```csharp
using RustEtherNetIp;

using var plc = new EtherNetIpClient();
if (!plc.Connect("192.168.0.10:44818"))
    throw new InvalidOperationException(plc.LastConnectError);

int count = plc.ReadDint("ProductionCount");
float temperature = plc.ReadReal("TankTemperature");
bool running = plc.ReadBool("MachineRunning");
string recipe = plc.ReadString("RecipeName");

plc.WriteDint("ProductionSetpoint", 1250);
plc.WriteReal("TemperatureSetpoint", 72.5f);
plc.WriteBool("EnableCommand", true);
plc.WriteString("RecipeName", "PRODUCT_A");
```

`using` disposes the client and unregisters the EtherNet/IP session. Check the
result of `Connect`; read and write failures throw `PlcException` when native
error detail is available.

## Learning Examples

Copy-ready console programs live in
[`Examples/GettingStarted`](Examples/GettingStarted/README.md):

1. connection, scalar values, and STRINGs;
2. batch reads, writes, and mixed operations;
3. controller discovery and program-scoped tag paths;
4. routed ControlLogix connections;
5. health and diagnostics;
6. subscriptions and tag-group polling.

Each example reads configuration from environment variables so addresses and
tag names are not hard-coded into a production project.

For structure-specific guidance, see the maintained
[UDT and structure guide](UDT_README.md).

## Scalar Types

Use a typed method that matches the controller tag type.

| Logix type | Read | Write | C# type |
|---|---|---|---|
| `BOOL` | `ReadBool` | `WriteBool` | `bool` |
| `SINT` | `ReadSint` | `WriteSint` | `sbyte` |
| `INT` | `ReadInt` | `WriteInt` | `short` |
| `DINT` | `ReadDint` | `WriteDint` | `int` |
| `LINT` | `ReadLint` | `WriteLint` | `long` |
| `USINT` | `ReadUsint` | `WriteUsint` | `byte` |
| `UINT` | `ReadUint` | `WriteUint` | `ushort` |
| `UDINT` | `ReadUdint` | `WriteUdint` | `uint` |
| `ULINT` | `ReadUlint` | `WriteUlint` | `ulong` |
| `REAL` | `ReadReal` | `WriteReal` | `float` |
| `LREAL` | `ReadLreal` | `WriteLreal` | `double` |
| `STRING` or custom string structure | `ReadString` | `WriteString` | `string` |

A type mismatch normally appears as a `PlcException` containing the native CIP
reason. Do not catch a failure and retry with unrelated types in production;
know the Logix type or obtain its attributes first.

## Choose Single, Batch, or Whole-Structure Access

| Need | Best starting API | Why |
|---|---|---|
| One value on demand, a command, or an occasional setpoint | `ReadDint`, `ReadReal`, `WriteBool`, and the other typed methods | Clearest code and one result to handle |
| Several independent tags in the same scan | `ReadTagsBatch` / `WriteTagsBatch` | Reduces network round trips and reports a result for each tag |
| A mixed read/write transaction list | `ExecuteBatch` | Sends packet-size-aware groups; correlate results by tag and operation |
| One known UDT member | A typed method with the full member path | Avoids transferring or rebuilding the rest of the structure |
| A consistent snapshot of an entire UDT | `ReadUdt` or `ReadUdtChunked` | Returns the structure data in one logical operation; large replies are fragmented |
| Change an entire UDT | Usually do not; write known members individually | A whole write requires the exact controller template handle and binary layout |

A batch is not an atomic PLC transaction and can contain a mixture of successes
and failures. For one tag, the typed single-tag API is simpler. For a polling
screen, historian sample, or MES scan containing several independent values, a
batch normally provides the better network shape.

## STRING Support in 1.2.0

The 1.2.0 `WriteString` path discovers and uses the target structure handle.
This supersedes older documentation that described UDT STRING members as
firmware-blocked.

```csharp
// Built-in top-level STRING
plc.WriteString("RecipeName", "PRODUCT_A");

// Built-in or custom STRING member inside a UDT
plc.WriteString("Mixer.Description", "Primary mixer");

// STRING member inside an element of a UDT array
plc.WriteString("Motors[0].Description", "Infeed conveyor");

Console.WriteLine(plc.ReadString("Motors[0].Description"));
```

The Studio 5000 built-in `STRING` is a structure containing a 4-byte `LEN` and
an 82-byte `SINT DATA[82]` array (88 bytes total after alignment). Its text
capacity is therefore **82 bytes**, not an unlimited .NET string; non-ASCII
UTF-8 characters may consume more than one byte. A custom Logix string type,
for example `Str400` with `DATA[400]`, has the capacity declared by that data
array and its own structure handle.

Real hardware confirms built-in `STRING`, custom `Str82`, and custom `Str400`
members on 5069-L330ERM firmware 38. A measured unconnected CIP request on that
target fits about 494 bytes total, including service and symbolic-path overhead,
so there is no single universal “maximum text length” for one packet. Version
1.2.0 uses CIP Read/Write Tag Fragmented when a string or structure will not fit.
A 600-byte custom string is simulator-confirmed; qualify very large custom
strings on the exact controller and firmware before production use.

## Tag Paths

Pass Logix symbolic paths directly:

```csharp
int controllerTag = plc.ReadDint("ProductionCount");
int programTag = plc.ReadDint("Program:MainProgram.ProductionCount");
int arrayElement = plc.ReadDint("RecipeSteps[5]");
bool statusBit = plc.ReadBool("StatusWord.15");
float udtMember = plc.ReadReal("Mixer.Process.Temperature");
string arrayMember = plc.ReadString("Motors[2].Description");
```

The `Program:<program-name>.TagName` prefix is part of the tag path. Both
controller- and program-scoped paths work for typed reads, writes, and batches.
Scope changes only the symbolic path: a controller tag is visible to the whole
controller, while a program tag belongs to one Logix program. It does not
change which typed read/write method you call.

## Batch Operations

Use batches for a scan containing several tags. Always inspect each result;
partial failures do not make successful items invalid.

```csharp
var reads = plc.ReadTagsBatch(new[]
{
    "ProductionCount",
    "TankTemperature",
    "Program:MainProgram.MachineRunning"
});

foreach (var (tag, result) in reads)
{
    if (result.Success)
        Console.WriteLine($"{tag} = {result.Value}");
    else
        Console.Error.WriteLine($"{tag}: {result.ErrorMessage}");
}

var writes = plc.WriteTagsBatch(new Dictionary<string, object>
{
    ["ProductionSetpoint"] = 1250,
    ["TemperatureSetpoint"] = 72.5f,
    ["EnableCommand"] = true,
    ["RecipeName"] = "PRODUCT_A"
});
```

Mixed execution is available with `ExecuteBatch`:

```csharp
var results = plc.ExecuteBatch(new[]
{
    BatchOperation.Read("ProductionCount"),
    BatchOperation.Write("ProductionSetpoint", 1300),
    BatchOperation.Read("Program:MainProgram.MachineRunning")
});
```

Mixed operations may be regrouped for packet efficiency. Match returned items
by `TagName` and `IsWrite`, not only by array position. The legacy
`ConfigureBatchOperations` and `GetBatchConfig` methods intentionally throw
`NotSupportedException`; normal batch APIs use native packet-size-aware defaults.

## Discovery and Metadata

`DiscoverTagsDetailed` lists controller-scoped tags and their Logix type
information:

```csharp
foreach (var tag in plc.DiscoverTagsDetailed())
    Console.WriteLine($"{tag.Name,-40} {tag.DataTypeName,-10} {tag.Size,6} bytes");

TagAttributes attributes = plc.GetTagAttributes("ProductionCount");
Console.WriteLine(attributes);
```

The C# wrapper does not currently expose program-scoped enumeration as a separate
method. Program tags are fully accessible when their known path is supplied,
for example `Program:MainProgram.ProductionCount`. Program-scoped discovery is
currently a Rust-core API gap for the wrappers, not a reason to invent tag
names or imply that controller discovery returns program tags.

## ControlLogix Routing

For a ControlLogix CPU reached through an Ethernet module, connect to the
module and add the CPU backplane slot:

```csharp
var route = new RoutePath().AddSlot(0);

using var plc = new EtherNetIpClient();
if (!plc.ConnectWithRoute("192.168.0.20:44818", route))
    throw new InvalidOperationException(plc.LastConnectError);
```

Ordered multi-hop paths are also supported:

```csharp
var route = new RoutePath()
    .AddBackplane(port: 1, slot: 3)
    .AddEthernet(port: 2, address: "192.168.10.20")
    .AddBackplane(port: 1, slot: 0);
```

The hop order is significant. CompactLogix controllers with a built-in
Ethernet port normally use `Connect` without a route.

## Health and Diagnostics

```csharp
if (!plc.CheckHealth())
    Console.Error.WriteLine("PLC health check failed");

DiagnosticsSnapshot snapshot = plc.GetDiagnosticsSnapshotDetailed();
Console.WriteLine($"Reads: {snapshot.Operations.TotalReads}");
Console.WriteLine($"Failed reads: {snapshot.Operations.FailedReads}");
Console.WriteLine($"Average read latency: {snapshot.Performance.AvgReadLatencyMs:F2} ms");
Console.WriteLine($"Last error: {snapshot.Errors.LastErrorMessage}");
Console.WriteLine($"Schema generation: {snapshot.SchemaCache.Generation}");
```

Process CPU and memory fields are placeholders in this release. Operation,
connection, error, latency, and verified-health fields are the meaningful
driver metrics.

After an online tag replacement or Studio 5000 download, use this maintenance
sequence: pause application writes, complete the controller change, call
`plc.RefreshSchema()`, optionally rediscover tags and verify critical reads,
then resume writes. Refresh invalidates packed-BOOL classification, tag
metadata, STRING handles, and UDT definitions/templates without reconnecting.

## Polling and Subscriptions

Subscriptions are polling-based. Choose a rate appropriate for the PLC,
network, and number of tags.

```csharp
var subscription = plc.SubscribeToTag(
    "ProductionCount",
    new SubscriptionOptions(pollIntervalMs: 250));

subscription.ValueChanged += (_, change) =>
    Console.WriteLine($"{change.TagName}: {change.OldValue} -> {change.NewValue}");

// Later:
plc.UnsubscribeFromTag("ProductionCount");
```

For periodic multi-tag acquisition, register a tag group and use
`ReadTagGroupOnce` or `SubscribeToTagGroup`. Tag-group polling events distinguish
complete data, partial errors, and scan-level read failures.

## Async Methods and UI Applications

The async methods (`ReadDintAsync`, `WriteStringAsync`, batch async methods,
and others) wrap blocking native calls with `Task.Run`. They keep WPF/WinForms
UI threads responsive, but each in-flight call still occupies a thread-pool
thread. A client serializes native operations; avoid disposing it while other
operations are active and use a clear owner per controller connection.

## Error Handling

```csharp
try
{
    plc.WriteDint("ReadOnlyTag", 42);
}
catch (PlcException ex)
{
    Console.Error.WriteLine(ex.Message);
    Console.Error.WriteLine(ex.NativeError);
}
catch (InvalidOperationException ex)
{
    Console.Error.WriteLine(ex.Message);
}
```

There are no `TagNotFoundException` or `DataTypeMismatchException` classes in
the current wrapper. Native protocol detail is carried by `PlcException`.

## Current Boundaries

- Whole UDT-array-element reads work, including fragmented large structures.
- Whole UDT-array-element writes are not supported because the required
  structure symbol lookup for an indexed path is unavailable; write members
  individually instead.
- Offset-based UDT member APIs are obsolete in 1.2.0 and planned for removal in
  2.0. Use full symbolic member paths.
- Controller discovery is exposed; program-scoped enumeration is not yet
  exposed through C#, Python, or the C ABI.
- This project targets CompactLogix and ControlLogix EtherNet/IP tag access,
  not Modbus TCP or a general OPC server.

## Deployment and Evidence

- [Integration and deployment guide](../../docs/INTEGRATION_AND_DEPLOYMENT.md)
- [Programmer manual](../../docs/programmer_manual.md)
- [Hardware compatibility matrix](../../docs/HARDWARE_COMPATIBILITY.md)
- [1.2.0 cross-binding hardware gate](../../docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md)

## Support and License

Use [GitHub Issues](https://github.com/sergiogallegos/rust-ethernet-ip/issues)
for reproducible defects and
[GitHub Discussions](https://github.com/sergiogallegos/rust-ethernet-ip/discussions)
for integration questions. Contributions and additional real-controller
firmware results are welcome.

The wrapper is distributed under the repository's MIT license.
