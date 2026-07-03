> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# Library Limitations

## Overview

The Rust EtherNet/IP library has certain limitations and controller-specific wire-format requirements. These limitations are documented here for all wrapper users.

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

### 2. STRING Members in UDTs Are Current-Encoding Blocked

**Observed Error Code:** CIP Error 0xFF/0x2107 (Read/Write Tag data-type mismatch)

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

### 3. UDT Array Element Members

Scalar UDT array element members were revalidated on 2026-07-03 against 5069-L330ERM fw38. DINT, REAL, BOOL, and INT member writes succeeded in controller and program scopes when the full member path was preserved.

**Affected Operations:**
- Writing scalar members of UDT array elements (e.g., `gTestUDT_Array[0].Member1_DINT`) is supported on the validated controller/firmware.
- Writing STRING members of UDT array elements (e.g., `gTestUDT_Array[0].Member5_String`) still rejects with `0x2107` under the current member encoding.

**What Works:**
- ✅ Reading UDT array element members
- ✅ Writing entire UDT array elements (e.g., `gTestUDT_Array[0]`)
- ✅ Writing UDT members for non-array UDTs (e.g., `gTestUDT.Member1_DINT`)
- ✅ Writing scalar UDT array element members on 5069-L330ERM fw38

**What Doesn't Work:**
- ❌ Writing UDT array element STRING members directly under the current member encoding

**Workaround:**
For STRING members, read the entire UDT array element, modify the member in memory, then write the entire UDT array element back.

## Test Results

Based on comprehensive testing with 392 tags:

- ✅ **335/392 tags** successfully read and written
- Superseded by the 2026-07-03 CODEX-AV matrix for UDT-array-element members:
  scalar array-element member writes are writeable on 5069-L330ERM fw38; STRING
  members remain current-encoding blocked.

## Error Handling

When encountering Error 0x2107, check the tag path to determine which limitation applies:

1. **Malformed/simple STRING request:** `gTest_STRING` with the wrong type bytes → request data-type mismatch
2. **STRING member in UDT:** `gTestUDT.Member5_String` → current member encoding rejected
3. **UDT array element scalar member:** `gTestUDT_Array[0].Member1_DINT` → should write on validated firmware; investigate path/encoding if it returns `0x2107`
4. **UDT array element STRING member:** `gTestUDT_Array[0].Member5_String` → current member encoding rejected

## Implementation Notes

All wrappers (C#, Go, Python) should:
1. Document these limitations in their API documentation
2. Provide clear error messages when these limitations are encountered
3. Suggest appropriate workarounds in error messages
4. Display limitations notices in example applications

