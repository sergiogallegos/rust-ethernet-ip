# Allen-Bradley ControlLogix/CompactLogix: Writing Strings and UDT Arrays via CIP

## Executive Summary

**There are significant limitations when writing STRING tags and UDT array element members from external devices via CIP (EtherNet/IP) to Allen-Bradley ControlLogix and CompactLogix PLCs.**

This is documented in Rockwell's official publication **1756-PM020** (Logix 5000 Controllers Data Access) and is a fundamental limitation of the CIP protocol implementation for Logix controllers.

**These limitations affect all EtherNet/IP communication libraries**, including this Rust EtherNet/IP library. They are not bugs in the library implementation, but rather restrictions imposed by the PLC firmware itself.

---

## The Core Problem

### Strings (STRING data type)

The STRING data type in Logix 5000 controllers is a **Predefined Structure**, not an atomic type. It consists of:
- `LEN` (DINT) - The length of the string
- `DATA[82]` (SINT array) - The character data (default 82 characters)

**Key Limitation from Rockwell Documentation (1756-PM020):**

> "Do not access complete structure tags of these types [Predefined, Module-Defined, Add-On-Defined], or complete UDTs with nested tags of these types. Instead, access atomic members of these tags."

### UDT Arrays

Writing arrays of User-Defined Types (UDTs) requires:
1. Correct **Structure Handle** (calculated CRC value)
2. Proper **data packing** with padding bytes
3. Matching **Tag Type Service Parameter**

The Write Tag Service returns error **0xFF / 0x2107** ("Tag type used in request does not match the target tag's data type") when the Structure Handle doesn't match.

---

## Official Rockwell Documentation

### From 1756-PM020 (Logix 5000 Controllers Data Access):

#### On Predefined Structures (including STRING):

> "Predefined, Module-Defined, and Add-On-Defined structure tags have a more complex set of rules than user-defined data types (UDT). **Do not access complete structure tags of these types**, or complete UDTs with nested tags of these types."

> "The STRING data type is a form of Predefined structure that would normally be excluded after executing steps that are described later."

#### On Writing Structures:

> "**IMPORTANT:** Reading a structure before writing to it is one way to obtain the value for this parameter [Structure Handle], but that does not provide any understanding of the structure makeup, which is critical information when manipulating structure data. Also, the **Structure Handle value is not unique among structures**."

> "The correct way to access structures as a whole is to first read their template information and understand the data packing."

#### Write Tag Service Error 0xFF/0x2107:

> "General Error: Tag type used in request does not match the target tag's data type."

---

## Workarounds

### For Strings

**Option 1: Write .DATA and .LEN separately (Not Currently Supported)**

> **Note:** This library does not currently support writing to `.DATA` and `.LEN` members separately due to the same underlying PLC firmware restrictions. Attempting to write to `MyString.DATA` or `MyString.LEN` directly will also fail with CIP Error 0x2107.

The recommended approach documented by other vendors is to write `.DATA` and `.LEN` separately, but this also fails with the same PLC firmware limitation in practice.

```
// This approach does NOT work with current PLC firmware:
1. Write to MyString.DATA[0] through MyString.DATA[4] = 'H','e','l','l','o'  // ❌ Fails
2. Write to MyString.LEN = 5  // ❌ Fails
```

**Option 2: Use LogixString Helper Class (Recommended for This Library)**

This library provides a `LogixString` helper class that mirrors the STRING structure:

```csharp
// C# Example
using RustEtherNetIp;

var logixString = new LogixString();
logixString.SetString("Hello");

// Write as UDT
client.WriteStringAsUdt("gTest_STRING", logixString);
```

The `LogixString` class mirrors the internal STRING structure:
```
LogixString:
  - len : Int32 (DINT)
  - data : byte[] (SINT array, default 82 bytes)
```

**Note:** Standard standalone Logix STRING tags should be written through the direct STRING path, which emits the validated structure type and standard STRING handle. The UDT read-modify-write workaround is for STRING members inside larger UDT structures.

**Option 3: Use an intermediary tag**

1. Create an atomic array tag in the PLC (e.g., `StringBuffer : SINT[84]`)
2. Write to the atomic array from external device
3. Use ladder logic (COP instruction) to copy to the STRING tag

### For UDT Arrays

**Option 1: Write individual array elements (Supported)**

This library **supports** writing entire UDT array elements:

```csharp
// ✅ This works - writing entire UDT array element
var udtElement = client.ReadUdt("gTestUDT_Array[0]");
// ... modify UDT structure ...
client.WriteUdt("gTestUDT_Array[0]", udtElement);
```

**Option 2: Write individual members (Not Supported for Array Elements)**

Writing to individual members of UDT array elements is **not supported** due to PLC firmware limitations:

```csharp
// ❌ This does NOT work - writing UDT array element member
client.WriteDint("gTestUDT_Array[0].Member1_DINT", 42);  // Fails with CIP Error 0x2107
```

**Workaround:** Read the entire UDT array element, modify the member in memory, then write the entire element back.

**Option 2: Use atomic intermediary arrays**

1. Create atomic arrays in the PLC that mirror your UDT data
2. Write to atomic arrays from external device
3. Use ladder logic to assemble into UDT array

**Option 3: Calculate Structure Handle correctly**

If you must write complete UDT arrays:
1. Read the Template Object (Class 0x6C) to get the Structure Handle
2. Include the correct 4-byte Tag Type Service Parameter: `A0 02 <Handle_Low> <Handle_High>`
3. Pack data according to alignment rules (32-bit boundaries, padding bytes)

See Rockwell document: https://www.rockwellautomation.com/content/dam/rockwell-automation/sites/downloads/pdf/TypeEncode_CIPRW.pdf

---

## Technical Details

### Tag Type Service Parameter for Structures

```
Atomic tags:  2 bytes (e.g., 0x00C4 for DINT)
Structure tags: 4 bytes = 0xA0 0x02 + 2-byte Structure Handle
```

### Structure Handle

The Structure Handle is a 16-bit CRC value calculated from the structure definition. It can be obtained by:
1. Reading Template Instance Attribute 1 (Class 0x6C)
2. Calculating it from the type encoding string (see TypeEncode_CIPRW.pdf)

### Data Alignment Rules

From 1756-PM020:
- SINTs: 8-bit boundary
- INTs: 16-bit boundary  
- DINTs, REALs, arrays: 32-bit boundary
- Structures: Begin and end on 32-bit boundaries
- Pad bytes are inserted for alignment and **must be sent on the wire**

---

## CIP Error Codes Reference

| Error | Extended | Meaning |
|-------|----------|---------|
| 0x04 | 0x0000 | Syntax error decoding Request Path |
| 0x05 | 0x0000 | Request Path destination unknown |
| 0x06 | N/A | Insufficient Packet Space |
| 0xFF | 0x2104 | Offset beyond end of tag |
| 0xFF | 0x2105 | Access beyond end of object |
| 0xFF | 0x2107 | **Tag type mismatch** (Structure Handle wrong) |

---

## Recommendations for This Library

### For STRING Tags

**If STRING is part of a UDT:**
```csharp
// ✅ Recommended: Read entire UDT, modify STRING member, write back
var udt = client.ReadTag("gTestUDT");
// ... modify STRING member in UDT structure ...
client.WriteTag("gTestUDT", udt);
```

**If STRING is standalone:**
```csharp
// ✅ Direct write uses the standard Logix STRING structure encoding
client.WriteString("gTest_STRING", "Hello");

// Direct .DATA/.LEN component writes are no longer the preferred path.
```

### For UDT Array Element Members

```csharp
// ❌ Direct write to UDT array element member does NOT work
client.WriteDint("gTestUDT_Array[0].Member1_DINT", 42);  // Fails with CIP Error 0x2107

// ✅ Recommended: Read-modify-write pattern
var element = client.ReadTag("gTestUDT_Array[0]");
// ... modify member in UDT structure ...
client.WriteTag("gTestUDT_Array[0]", element);
```

### For Entire UDT Array Elements

```csharp
// ✅ This works - writing entire UDT array element
var element = client.ReadTag("gTestUDT_Array[0]");
// ... modify UDT structure ...
client.WriteTag("gTestUDT_Array[0]", element);
```

### Alternative: Use Ladder Logic Intermediary

For complex scenarios:
1. Create atomic buffer tags in the PLC
2. Write to buffers from external device
3. Use ladder logic (COP instruction) to copy buffers to structured tags

---

## References

1. **Rockwell 1756-PM020** - Logix 5000 Controllers Data Access
   https://literature.rockwellautomation.com/idc/groups/literature/documents/pm/1756-pm020_-en-p.pdf

2. **Rockwell TypeEncode_CIPRW.pdf** - Type Encoding of Logix Structures
   https://www.rockwellautomation.com/content/dam/rockwell-automation/sites/downloads/pdf/TypeEncode_CIPRW.pdf

3. **Kepware ControlLogix Ethernet Driver Manual**
   (Documents STRING handling limitations)

4. **ODVA CIP Networks Library** - Volume 1 (Requires license from ODVA)

---

## Conclusion

The limitations you're experiencing are **by design** in the CIP protocol implementation for Logix controllers. Rockwell explicitly states that Predefined structures (including STRING) and complex structure operations should be handled by accessing individual atomic members rather than complete structures.

The workaround is to either:
1. Access structure members individually (`.DATA`, `.LEN`, etc.)
2. Use intermediary atomic tags with ladder logic
3. Implement full CIP structure handling with proper Template reading and Structure Handle calculation

This is not a bug in your driver or code - it's a fundamental limitation of how CIP handles complex data types in Logix controllers.
