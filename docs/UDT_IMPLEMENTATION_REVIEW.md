> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# UDT Implementation Review - 1756-PM020 Compliance

**Date:** 2025-01-XX  
**Reference:** Allen-Bradley Publication 1756-PM020: Logix Controller Access Data

## Summary

The UDT implementation has been reviewed against the 1756-PM020 specification. The implementation correctly follows the generic UDT approach recommended by contributors, using `UdtData` with `symbol_id` and raw bytes instead of hardcoded member names.

## ✅ Correctly Implemented

### 1. Generic UDT Data Structure

**Implementation:** `UdtData` struct with `symbol_id` and raw `data: Vec<u8>`

**Status:** ✅ **CORRECT**

- Stores raw bytes without requiring member name knowledge
- Includes `symbol_id` (template instance ID) required for writes
- Matches contributor recommendation: "in order to write a UDT, you typically need to read it first to get the symbol_id"

**Reference:** 1756-PM020, Pages 920-1095 (Access User-Defined Structures)

### 2. Symbol ID (Template Instance ID) Handling

**Implementation:** 
- Reads `template_instance_id` from tag attributes (Template Attribute 1)
- Uses `symbol_id` for UDT writes
- Auto-reads tag attributes if `symbol_id` is 0 during write

**Status:** ✅ **CORRECT**

**Reference:** 1756-PM020, Page 946 (Structure Handle from Template Instance Attribute 1)

### 3. Structure Tag Type for Writes

**Implementation:** Fixed to use `0x02A0 + symbol_id` for UDT writes

**Status:** ✅ **FIXED**

**Previous Issue:** Used `0x00A0` (incorrect placeholder)  
**Correct Format:** `0x02A0 + Structure Handle` (Structure Handle = template_instance_id)

**Reference:** 1756-PM020, Page 1080 (UDT Data Layout Considerations)

**Code Location:** `src/lib.rs::build_write_request()` - Now correctly calculates:
```rust
let data_type = if let PlcValue::Udt(udt_data) = value {
    0x02A0u16.wrapping_add(udt_data.symbol_id as u16)
} else {
    value.get_data_type()
};
```

### 4. UDT Reading

**Implementation:** 
- Reads entire UDT structure as raw bytes
- Retrieves `symbol_id` from tag attributes
- Returns `PlcValue::Udt(UdtData { symbol_id, data })`

**Status:** ✅ **CORRECT**

**Reference:** 1756-PM020, Pages 926-945 (Read Entire UDT Structure)

### 5. UDT Member Access

**Implementation:** Supports member access via `TagPath` parsing (e.g., `MachineData.Speed`)

**Status:** ✅ **CORRECT**

**Reference:** 1756-PM020, Pages 950-967 (Read Single Member of UDT)

### 6. Array Members Within UDTs

**Implementation:** Supports array member access (e.g., `MachineData.Counts[5]`)

**Status:** ✅ **CORRECT**

**Reference:** 1756-PM020, Pages 971-988 (Read Array Member Within UDT)

### 7. Arrays of UDTs

**Implementation:** Supports array element access (e.g., `Stations[3]`, `Stations[3].Status`)

**Status:** ✅ **CORRECT**

**Reference:** 1756-PM020, Pages 992-1023 (Array of UDTs Access)

## 📋 Implementation Details

### UDT Read Flow

1. **Detect UDT Tag:** Check if `data_type == 0x00A0` or `0x02A0`
2. **Get Tag Attributes:** Retrieve `template_instance_id` (symbol_id)
3. **Read Raw Data:** Read entire structure as raw bytes
4. **Return UdtData:** `PlcValue::Udt(UdtData { symbol_id, data })`

**Reference:** 1756-PM020, Pages 926-945

### UDT Write Flow

1. **Check symbol_id:** If 0, read tag attributes to get it
2. **Build Request:** Use Structure Tag Type `0x02A0 + symbol_id`
3. **Send Write:** Include raw UDT bytes in request data

**Reference:** 1756-PM020, Page 1080

### Structure Tag Type Format

**Read Response:**
```
Reply Data: A0 02 xx xx
  - 0x02A0 = Base Structure Tag Type
  - xx xx = Structure Handle (template_instance_id)
```

**Write Request:**
```
Request Data: (0x02A0 + Structure Handle) 01 00 [UDT bytes...]
  - Data Type = 0x02A0 + Structure Handle
  - Element Count = 1
  - Data = Raw UDT bytes
```

**Reference:** 1756-PM020, Pages 943, 1080

## ⚠️ Considerations

### Data Alignment

**Note:** The implementation relies on the PLC to handle data alignment and padding. When parsing UDT data, members are aligned per their data type:
- SINT: 1-byte boundary
- INT: 2-byte boundary
- DINT/REAL: 4-byte boundary
- LINT: 8-byte boundary

**Reference:** 1756-PM020, Page 1082-1088

### BOOL Mapping

**Note:** BOOLs are packed into hidden SINT host members (up to 8 BOOLs per SINT). The raw byte representation reflects this packing.

**Reference:** 1756-PM020, Page 1090

### Template Structure Size

**Note:** Use Template Attribute 5 (Template Structure Size) to know the exact byte count for UDT structures.

**Reference:** 1756-PM020, Page 1092

## 🔧 Code Locations

### Key Methods

1. **`read_tag()`** - `src/lib.rs:1457`
   - Detects UDT tags and reads with symbol_id

2. **`write_tag()`** - `src/lib.rs:3230`
   - Handles UDT writes with symbol_id validation

3. **`build_write_request()`** - `src/lib.rs:3392`
   - Builds write requests with correct Structure Tag Type

4. **`UdtData`** - `src/lib.rs:752`
   - Generic UDT data structure

5. **`read_udt_member_discovery()`** - `src/lib.rs:2479`
   - Generic UDT reading without member knowledge

## ✅ Compliance Status

| Feature | Status | 1756-PM020 Reference |
|---------|--------|---------------------|
| Generic UDT Structure | ✅ Correct | Pages 920-1095 |
| Symbol ID Handling | ✅ Correct | Page 946 |
| Structure Tag Type (Read) | ✅ Correct | Page 943 |
| Structure Tag Type (Write) | ✅ **FIXED** | Page 1080 |
| UDT Member Access | ✅ Correct | Pages 950-967 |
| Array Members in UDTs | ✅ Correct | Pages 971-988 |
| Arrays of UDTs | ✅ Correct | Pages 992-1023 |
| Nested UDT Access | ✅ Correct | Pages 1027-1043 |

## 📝 Changes Made

1. **Fixed UDT Write Data Type:** Changed from `0x00A0` to `0x02A0 + symbol_id` in `build_write_request()`
   - **File:** `src/lib.rs:3415-3422`
   - **Reference:** 1756-PM020, Page 1080

2. **Added Documentation:** Added 1756-PM020 page references to UDT write methods
   - **File:** `src/lib.rs:3388-3392`

## 🎯 Conclusion

The UDT implementation is **compliant** with the 1756-PM020 specification. The generic approach using `UdtData` with `symbol_id` and raw bytes is correct and matches the contributor's recommendation. The Structure Tag Type calculation for writes has been fixed to use `0x02A0 + Structure Handle` as required by the specification.

---

**Status:** ✅ **REVIEW COMPLETE - IMPLEMENTATION COMPLIANT**

