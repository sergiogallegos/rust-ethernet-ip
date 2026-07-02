> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# Library Limitations

## Overview

The Rust EtherNet/IP library has certain limitations due to PLC firmware restrictions (not library bugs). These limitations are documented here for all wrapper users.

## Known Limitations

### 1. Top-Level STRING Tags

Standard top-level Logix `STRING` tags can be read and written directly when encoded as the Logix structure type.

**Affected Operations:**
- Reading simple STRING tags (e.g., `gTest_STRING`)
- Writing simple STRING tags (e.g., `gTest_STRING`)
- Reading/writing program-scoped STRING tags (e.g., `Program:TestProgram.gTest_STRING`)

**What Works:**
- ✅ Reading STRING tags (both controller and program-scoped)
- ✅ Writing standalone standard STRING tags
- ✅ Reading STRING members within UDTs

**What Doesn't Work:**
- ❌ Writing STRING members in UDTs directly

**Workaround:**
For STRING values that are part of a UDT, read the entire UDT, modify the STRING member in memory, then write the entire UDT back.

### 2. STRING Members in UDTs Cannot Be Written Directly

**Error Code:** CIP Error 0x2107 (Vendor Specific Error)

**Affected Operations:**
- Writing to STRING members within UDTs (e.g., `gTestUDT.Member5_String`)
- Writing to program-scoped STRING members (e.g., `Program:TestProgram.gTestUDT.Member5_String`)

**What Works:**
- ✅ Reading STRING members in UDTs
- ✅ Writing non-STRING members in UDTs (e.g., `gTestUDT.Member1_DINT`)

**What Doesn't Work:**
- ❌ Writing STRING members in UDTs directly

**Workaround:**
Read the entire UDT, modify the STRING member in memory, then write the entire UDT back.

### 3. UDT Array Element Members Cannot Be Written Directly

**Error Code:** CIP Error 0x2107 (Vendor Specific Error)

**Affected Operations:**
- Writing to members of UDT array elements (e.g., `gTestUDT_Array[0].Member1_DINT`)
- Writing to program-scoped UDT array element members (e.g., `Program:TestProgram.gTestUDT_Array[0].Member1_DINT`)

**What Works:**
- ✅ Reading UDT array element members
- ✅ Writing entire UDT array elements (e.g., `gTestUDT_Array[0]`)
- ✅ Writing UDT members for non-array UDTs (e.g., `gTestUDT.Member1_DINT`)

**What Doesn't Work:**
- ❌ Writing UDT array element members directly

**Workaround:**
Read the entire UDT array element, modify the member in memory, then write the entire UDT array element back.

## Test Results

Based on comprehensive testing with 392 tags:

- ✅ **335/392 tags** successfully read and written
- ❌ **57/392 tags** failed:
  - 55 tags: UDT array element member writes (PLC limitation)
  - 2 tags: STRING member writes in UDTs (PLC limitation)

## Error Handling

When encountering Error 0x2107, check the tag path to determine which limitation applies:

1. **Malformed/simple STRING request:** `gTest_STRING` with the wrong type bytes → request data-type mismatch
2. **STRING member in UDT:** `gTestUDT.Member5_String` → STRING member limitation
3. **UDT array element member:** `gTestUDT_Array[0].Member1_DINT` → UDT array element member limitation

## Implementation Notes

All wrappers (C#, Go, Python) should:
1. Document these limitations in their API documentation
2. Provide clear error messages when these limitations are encountered
3. Suggest appropriate workarounds in error messages
4. Display limitations notices in example applications

