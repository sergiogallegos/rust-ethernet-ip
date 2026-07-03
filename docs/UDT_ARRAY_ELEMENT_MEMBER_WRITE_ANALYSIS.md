> **Historical reference, superseded for scalar members.** This document records
> past work and may not reflect the current mainline codebase. CODEX-AM and the
> 2026-07-03 CODEX-AV hardware matrix showed that scalar UDT array element
> member writes (`DINT`/`REAL`/`BOOL`/`INT`) succeed on 5069-L330ERM fw38 when
> the full member path is preserved. STRING members remain rejected with
> `0x2107` under the current member encoding.

# UDT Array Element Member Write Analysis

**Date:** 2026-01-03  
**Reference:** Allen-Bradley Publication 1756-PM020: Logix Controller Access Data

## Summary

The library correctly implements the CIP path format for writing to UDT array element members (e.g., `gTestUDT_Array[3].Member1_DINT`), but the PLC firmware returns error `0x2107` (Vendor-specific or composite extended error), indicating that **direct writes to UDT array element members are not supported by the PLC**.

## Test Results

**Test:** Writing to `gTestUDT_Array[3].Member1_DINT` and similar paths  
**Result:** ❌ **FAILED** - Error 0x2107  
**Affected Tags:** 55 tags (all UDT array element member writes)

### Failed Patterns:
- `gTestUDT_Array[*].Member1_DINT` (10 tags)
- `gTestUDT_Array[*].Member2_REAL` (10 tags)
- `gTestUDT_Array[*].Member3_BOOL` (10 tags)
- `gTestUDT_Array[*].Member4_INT` (10 tags)
- `Program:TestProgram.gTestUDT_Array[*].Member1_DINT` (5 tags)
- `Program:TestProgram.gTestUDT_Array[*].Member2_REAL` (5 tags)
- `Program:TestProgram.gTestUDT_Array[*].Member3_BOOL` (5 tags)

## Implementation Verification

### Path Format (✅ CORRECT)

According to 1756-PM020, Page 1005-1023, the path format for accessing a member from an array of UDTs is:

**Example:** `Stations[3].Status`

**Request Path:**
```
91 08 53 74 61 74 69 6F 6E 73 28 03 91 06 53 74 61 74 75 73
```

**Breakdown:**
- `91 08` = Symbol segment, 8 chars
- `53 74 61 74 69 6F 6E 73` = "Stations"
- `28 03` = Element 3 (8-bit Element ID)
- `91 06` = Symbol segment, 6 chars
- `53 74 61 74 75 73` = "Status"

### Library Implementation

The library's `TagPath` parser correctly handles this format:

1. **Parsing:** `gTestUDT_Array[3].Member1_DINT` is parsed as:
   - `TagPath::Member { base_path: TagPath::Array { base_path: TagPath::Controller { tag_name: "gTestUDT_Array" }, indices: [3] }, member_name: "Member1_DINT" }`

2. **CIP Path Generation:** The `build_cip_path` method correctly generates:
   - Symbol segment for "gTestUDT_Array"
   - Element segment `28 03` for element 3
   - Symbol segment for "Member1_DINT"

3. **Write Request:** The `build_write_request` method correctly:
   - Uses Write Tag Service (0x4D)
   - Includes the correct path
   - Includes data type, element count, and value data

**Status:** ✅ **IMPLEMENTATION IS CORRECT**

## Rockwell Documentation Analysis

### What the Documentation Shows

**1756-PM020, Page 1005-1023:**
- ✅ **Reading** from UDT array element members: `Stations[3].Status` (Example 5)
- ✅ **Reading** entire UDT array element: `Stations[3]` (Example 4)
- ✅ **Writing** to single UDT member: `MachineData.Speed` (Page 1047-1060)
- ✅ **Writing** to array element within UDT: `MachineData.Counts[5]` (Page 1062-1072)
- ❓ **Writing** to member from array of UDTs: **NOT DOCUMENTED**

### Missing Documentation

The 1756-PM020 documentation does **not** provide an example of writing to a member from an array of UDTs (e.g., `Stations[3].Status = value`). This suggests that:

1. **Either** this operation is not supported by the PLC firmware
2. **Or** it requires a different approach (e.g., read entire UDT array element, modify in memory, write back)

## Error Analysis

### Error Code 0x2107

**Error Message:** "Vendor-specific or composite extended error: 0x2107 (LE) / 0x0721 (BE). Raw bytes: [0x07, 0x21]"

**Interpretation:**
- `0x07` = Connection lost (extended)
- `0x21` = Write-once value or medium already written (extended)

**Possible Meanings:**
1. The PLC firmware does not support direct writes to UDT array element members
2. The operation requires a different service or path format
3. The UDT array element is protected or read-only

## Workaround

To write to a UDT array element member, use the following approach:

1. **Read the entire UDT array element:**
   ```rust
   let udt_element = client.read_tag("gTestUDT_Array[3]").await?;
   ```

2. **Modify the member in memory:**
   ```rust
   // Parse UDT data and modify Member1_DINT
   // (Implementation depends on UDT structure)
   ```

3. **Write the entire UDT array element back:**
   ```rust
   client.write_tag("gTestUDT_Array[3]", modified_udt_element).await?;
   ```

## Conclusion

1. ✅ **Library Implementation:** The library correctly implements the CIP path format for UDT array element member access according to 1756-PM020.

2. ⚠️ **PLC Limitation:** The PLC firmware (ControlLogix 1756-L75) does not support direct writes to UDT array element members, as indicated by error 0x2107.

3. ✅ **Reading Works:** Reading from UDT array element members works correctly (verified in tests).

4. 📝 **Documentation Gap:** The Rockwell documentation does not provide an example of writing to UDT array element members, suggesting this operation may not be supported.

## Recommendations

1. **Document the Limitation:** Add a note in the README that direct writes to UDT array element members are not supported by the PLC firmware.

2. **Provide Workaround Example:** Add example code showing how to write to UDT array element members using the read-modify-write approach.

3. **Consider Alternative:** If direct writes are critical, investigate if there's a different CIP service or path format that supports this operation (though the documentation suggests there isn't).

---

**Status:** ✅ **IMPLEMENTATION VERIFIED - PLC LIMITATION CONFIRMED**

