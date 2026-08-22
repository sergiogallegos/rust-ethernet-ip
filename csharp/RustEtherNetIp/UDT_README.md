# C# UDT and Structure Guide

This guide describes the maintained UDT workflows in `RustEtherNetIp` 1.2.0.
For installation, connections, batches, routing, and diagnostics, start with
the [main C# guide](README.md).

## Recommended Model

Use two different patterns deliberately:

- read a whole UDT when the application needs a structure snapshot;
- write individual members through their complete Logix symbolic paths.

```csharp
using RustEtherNetIp;

using var plc = new EtherNetIpClient();
if (!plc.Connect("192.168.0.10:44818"))
    throw new InvalidOperationException(plc.LastConnectError);

PlcValue mixer = plc.ReadUdtChunked("Mixer");

int speed = plc.ReadDint("Mixer.CommandSpeed");
bool enabled = plc.ReadBool("Mixer.Enabled");
string description = plc.ReadString("Mixer.Description");

plc.WriteDint("Mixer.CommandSpeed", 1250);
plc.WriteBool("Mixer.Enabled", true);
plc.WriteString("Mixer.Description", "Primary mixer");
```

Typed member writes avoid reconstructing an entire controller-defined binary
layout in application code.

## Nested and Array-Element Paths

Dot notation and array indices are part of the tag path:

```csharp
float temperature = plc.ReadReal("Line.Station1.Motor.Diagnostics.Temperature");
plc.WriteReal("Line.Station1.Motor.SpeedSetpoint", 60.0f);

int motorId = plc.ReadDint("Motors[0].MotorId");
plc.WriteBool("Motors[0].Enabled", true);
plc.WriteString("Motors[0].Description", "Infeed conveyor");
```

The 1.2.0 handle-aware string path supports built-in and custom STRING members.
Real hardware confirms built-in `STRING`, custom `Str82`, and custom `Str400`
members inside UDTs and UDT array elements on CompactLogix 5069-L330ERM
firmware 38.

## Whole-Structure Reads

```csharp
PlcValue value = plc.ReadUdt("Mixer");
PlcValue largeValue = plc.ReadUdtChunked("LargeRecipe");
```

`ReadUdtChunked` supports structures larger than one CIP response. Depending on
the target and available template information, a `PlcValue` may contain decoded
members or raw `UdtData`. Check its shape before navigating:

```csharp
if (value.UdtMembers is not null)
{
    PlcValue? nested = value.GetNestedValue("Process.Temperature");
    Console.WriteLine(nested);
}
else if (value.UdtData is not null)
{
    Console.WriteLine($"Raw structure bytes: {value.UdtData.Data.Length}");
}
```

`GetUdtMember(tag, memberPath)` is a convenience that reads the whole UDT and
navigates decoded members. For a single known PLC member, a direct typed read
is usually simpler and cheaper.

## Whole-Structure Writes

`WriteUdt` and `WriteUdtData` require structure data compatible with the exact
controller template and symbol ID. Do not invent a dictionary layout and
assume the PLC will derive byte offsets automatically.

Use whole-structure writes only when your application obtained and preserved a
compatible structure representation and you have validated the operation on
the target controller. For normal commands, recipes, and configuration updates,
write members individually.

Whole UDT-array-element writes such as `Motors[0]` are not supported in 1.2.0
because the indexed-path structure symbol lookup is unavailable. Whole-element
reads work; update `Motors[0].MemberName` paths individually.

## `SetUdtMember` Versus Direct Writes

`SetUdtMember` performs a whole-UDT read/modify/write and requires decoded
members:

```csharp
plc.SetUdtMember("Mixer", "Enabled", PlcValue.Bool(true));
```

Prefer a direct typed write when the member type is known:

```csharp
plc.WriteBool("Mixer.Enabled", true);
plc.WriteString("Mixer.Description", "Primary mixer");
```

Direct member writes reduce network and serialization work and avoid changing
unrelated structure bytes.

## Retired Offset APIs

These methods are obsolete in 1.2.0 because they never had a reliable native
payload contract:

- `ReadUdtMemberByOffset`
- `WriteUdtMemberByOffset`

They are retained only for 1.x source compatibility and return unsupported
behavior. Use symbolic member paths or a whole-structure read.

## Batch Member Access

Known member paths can be batched like scalar tags:

```csharp
var snapshot = plc.ReadTagsBatch(new[]
{
    "Mixer.CommandSpeed",
    "Mixer.Enabled",
    "Mixer.Description",
    "Motors[0].SpeedFeedback"
});

var writes = plc.WriteTagsBatch(new Dictionary<string, object>
{
    ["Mixer.CommandSpeed"] = 1250,
    ["Mixer.Enabled"] = true,
    ["Mixer.Description"] = "Primary mixer"
});
```

Inspect every returned result because a batch can contain both successful and
failed member operations.

## Evidence and Remaining Boundary

The [1.2.0 cross-binding hardware gate](../../docs/validation/2026-07-08_release-1.2.0-gate_cross-binding_5069-L330ERM_fw38.md)
validated scalar and STRING member paths across controller/program scopes and
UDT array elements. It also validated fragmented reads of an approximately
658-byte structure.

That evidence applies to the recorded processor, firmware, and tag shapes; it
is not a claim that every arbitrary UDT layout can be safely synthesized or
written as a whole structure.
