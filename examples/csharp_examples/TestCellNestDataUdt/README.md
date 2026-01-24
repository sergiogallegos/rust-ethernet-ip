# TestCellNestDataUdt - C# Example

This C# example tests reading complex nested UDT array structures using the Rust EtherNet/IP C# wrapper.

## What it Tests

- **Array of UDT elements**: `Cell_NestData[90]`
- **Nested UDT members**: `Cell_NestData[90].PartData`
- **Nested array members**: `Cell_NestData[90].PartData.PlungerInsertion[0-3]`
- **Individual UDT members**: All PartData temperature, time, vision, and weight members
- **Top-level members**: ModelNumber, SerialNumber, LotNo, status fields, etc.

## Prerequisites

- .NET 9.0 SDK
- PLC accessible at `192.168.1.101:44818`
- `Cell_NestData` tag must exist in the PLC (array of 100 UDT elements)
- CPU slot 0 (CompactLogix) - adjust `CPU_SLOT` constant if different

## How to Run

```bash
cd examples/csharp_examples/TestCellNestDataUdt
dotnet run
```

## Configuration

Edit `Program.cs` to change:
- `PLC_ADDRESS`: Default is `"192.168.1.101:44818"`
- `CPU_SLOT`: Default is `0` (CompactLogix)

## Expected Output

The test will:
1. Connect to the PLC
2. Read the entire `Cell_NestData[90]` UDT
3. Read the nested `PartData` UDT
4. Read all individual PartData members (temperatures, times, etc.)
5. Read the nested `PlungerInsertion` array elements
6. Read other PartData members (vision, weight data)
7. Read top-level Cell_NestData members (strings, DINTs, BOOLs)

All successful reads will show ✅ with their values, and any errors will show ❌ with error messages.
