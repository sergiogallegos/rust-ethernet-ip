# WinForms UDT Member Access Fix

## Issue

The WinForms example was trying to access UDT members using direct tag read/write methods like:
```csharp
_plcClient.ReadDint("gTestUDT.Member1_DINT");  // ❌ This doesn't work!
_plcClient.WriteDint("gTestUDT.Member1_DINT", 500);  // ❌ This doesn't work!
```

This fails because the C# wrapper's `ReadDint`, `WriteDint`, etc. methods expect simple tag names, not UDT member paths.

## Solution

UDT members must be accessed using the dedicated UDT methods:

### Reading UDT Members

**Before (Incorrect):**
```csharp
var value = _plcClient.ReadDint("gTestUDT.Member1_DINT");  // ❌ Fails
```

**After (Correct):**
```csharp
// Option 1: Use GetUdtMember helper
var memberValue = _plcClient.GetUdtMember("gTestUDT", "Member1_DINT");
var intValue = memberValue.As<int>();

// Option 2: Read whole UDT and access member
var udtValue = _plcClient.ReadUdt("gTestUDT");
var memberValue = udtValue.GetNestedValue("Member1_DINT");
var intValue = memberValue.As<int>();

// Option 3: Access via UdtMembers dictionary
var udtValue = _plcClient.ReadUdt("gTestUDT");
var memberValue = udtValue.UdtMembers["Member1_DINT"];
var intValue = memberValue.As<int>();
```

### Writing UDT Members

**Before (Incorrect):**
```csharp
_plcClient.WriteDint("gTestUDT.Member1_DINT", 500);  // ❌ Fails
```

**After (Correct):**
```csharp
// Option 1: Use SetUdtMember helper (Recommended)
_plcClient.SetUdtMember("gTestUDT", "Member1_DINT", PlcValue.Dint(500));

// Option 2: Read, modify, write back
var udtValue = _plcClient.ReadUdt("gTestUDT");
udtValue.UdtMembers["Member1_DINT"] = PlcValue.Dint(500);
_plcClient.WriteUdt("gTestUDT", udtValue);
```

## Updated WinForms Code

The `UdtMemberReadButton_Click` and `UdtMemberWriteButton_Click` methods have been updated to:

1. **Parse the path**: Split `"gTestUDT.Member1_DINT"` into `tagName="gTestUDT"` and `memberPath="Member1_DINT"`
2. **Use correct methods**: 
   - Reading: `GetUdtMember(tagName, memberPath)`
   - Writing: `SetUdtMember(tagName, memberPath, PlcValue)`

## Example Usage

### Reading UDT Members

```csharp
// Simple member
var member1 = _plcClient.GetUdtMember("gTestUDT", "Member1_DINT");
int value = member1.As<int>();

// Nested member (if UDT contains another UDT)
var nested = _plcClient.GetUdtMember("gTestUDT", "Status.Running");
bool isRunning = nested.As<bool>();

// Array member within UDT
var arrayElement = _plcClient.GetUdtMember("gTestUDT", "Array_DINT[5]");
int arrayValue = arrayElement.As<int>();
```

### Writing UDT Members

```csharp
// Simple member
_plcClient.SetUdtMember("gTestUDT", "Member1_DINT", PlcValue.Dint(500));

// Nested member
_plcClient.SetUdtMember("gTestUDT", "Status.Running", PlcValue.Bool(true));

// Array member within UDT
_plcClient.SetUdtMember("gTestUDT", "Array_DINT[5]", PlcValue.Dint(99));
```

## Notes

1. **Tag Existence**: The UDT tag (`gTestUDT`) must exist in the PLC. If it doesn't exist, you'll get an error.

2. **Member Names**: Member names are case-sensitive and must match exactly as defined in the PLC.

3. **Array Members**: Array members within UDTs can be accessed using bracket notation: `"Array_DINT[5]"`

4. **Performance**: `GetUdtMember` and `SetUdtMember` read/write the entire UDT, so they're slightly slower than direct tag access. For frequent access to the same UDT, consider reading the UDT once and accessing members from the returned `PlcValue`.

## Testing

After this fix, you should be able to:
- ✅ Read UDT members: `gTestUDT.Member1_DINT`
- ✅ Write UDT members: `gTestUDT.Member1_DINT = 500`
- ✅ Access array members: `gTestUDT.Array_DINT[5]`
- ✅ Access nested UDTs: `gTestUDT.Status.Running`

Make sure the tags from `PLC_TEST_TAG_DEFINITIONS.md` are created in your PLC before testing!

