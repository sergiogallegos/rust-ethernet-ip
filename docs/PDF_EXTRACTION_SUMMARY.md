> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# PDF Extraction Summary - 1756-PM020

This document summarizes the key information extracted from Allen-Bradley Publication 1756-PM020: Logix Controller Access Data, specifically pages 13-29 and 63.

## Pages Documented

- **Page 13**: Logical Segments (Element ID segments 0x28, 0x29, 0x2A)
- **Page 14**: Symbolic Segments (0x91 ANSI Extended Symbolic)
- **Page 15**: CIP Service Request/Response Format
- **Page 18**: Read Tag Service - Example 1 (Symbolic Segment Addressing)
- **Page 20**: Read Tag Fragmented Service (0x52) - Example with byte offset
- **Page 27**: Write Tag Service - Example (Symbolic Segment Addressing)
- **Page 29**: Write Tag Fragmented Service (0x53)
- **Page 63**: CIP Addressing Examples (pending - need Example 1 details)

## Key Findings

### 1. Element Addressing Segments

**Element ID Segments:**
- `0x28`: 8-bit Element ID (single byte value)
- `0x29`: 16-bit Element ID (2 bytes, little-endian)
- `0x2A`: 32-bit Element ID (4 bytes, little-endian)

**Important:** All values use **low byte first** (little-endian) byte order.

### 2. Symbolic Segments

- `0x91`: ANSI Extended Symbolic Segment
- Format: `[0x91] [Length] [Tag Name ASCII] [Padding]`
- Used for tag names in Request Path

### 3. Service Codes

| Service | Request | Reply |
|---------|---------|-------|
| Read Tag | `0x4C` | `0xCC` |
| Write Tag | `0x4D` | `0xCD` |
| Read Tag Fragmented | `0x52` | `0xD2` |
| Write Tag Fragmented | `0x53` | `0xD3` |

### 4. Fragmented Service Details

**Read Tag Fragmented Service (0x52):**
- Used for reading large data (>500 bytes)
- Request Data includes:
  - Element Count (2 bytes, little-endian)
  - Byte Offset (4 bytes, little-endian)
- Response Status:
  - `0x00`: Success (last chunk)
  - `0x06`: Reply Data Too Large (more data available)

**Write Tag Fragmented Service (0x53):**
- Used for writing large data (>500 bytes)
- Request Data includes:
  - Data Type (2 bytes)
  - Total Element Count (2 bytes) - same for all requests
  - Byte Offset (4 bytes) - increases with each request
  - Actual data bytes

### 5. Request/Response Format

**Request:**
```
[Service: 1 byte]
[Path Size: 1 byte (words)]
[Request Path: Variable]
[Request Data: Variable (if needed)]
```

**Response:**
```
[Reply Service: 1 byte]
[Reserved: 1 byte]
[General Status: 1 byte]
[Extended Status Size: 1 byte]
[Extended Status: Variable (if size > 0)]
[Reply Data: Variable (if service returns data)]
```

## Critical Information for Array Fixes

### Current Implementation Issues Confirmed

1. **Missing Element Addressing:** Current code doesn't use `0x28` segment in Request Path
2. **Wrong Service for Large Arrays:** Should use `0x52` (Fragmented) instead of `0x4C` (Read Tag)
3. **No Byte Offset Support:** Chunked reading always starts from 0, should use byte offset
4. **Inefficient Writes:** Should use element addressing or fragmented service

### Recommended Solutions

1. **For Single Element:** Use `0x28` element segment in Request Path
2. **For Multiple Elements (Small):** Use `0x28` with index + count (format TBD)
3. **For Large Arrays:** Use Fragmented Service (0x52) with byte offset
4. **For Specific Ranges:** Use Fragmented Service with calculated byte offset

## Open Questions

1. **Element Addressing Format:** What is the exact format for `0x28` segment when specifying index + count?
   - Need to see Page 63 Example 1 for array element addressing
   - Current TagPath uses `[0x28] [0x04] [Index: 4 bytes]` for single element
   - Need format for: `[0x28] [Format] [Index] [Count]`?

2. **When to Use Which Service:**
   - When should we use Read Tag (0x4C) vs Read Tag Fragmented (0x52)?
   - Can fragmented service be used for small arrays?
   - Is there a size threshold?

3. **Element Count in Request Data:**
   - For Read Tag (0x4C): Element count is in Request Data
   - For Read Tag Fragmented (0x52): Element count is in Request Data, plus byte offset
   - For array element addressing: Is count in path or Request Data?

## Next Steps

1. **Get Page 63 Example 1:** This should show array element addressing with `0x28` segment
2. **Verify Element Addressing Format:** Confirm how index + count are encoded
3. **Test Fragmented Service:** Implement and test with actual PLC
4. **Update Implementation:** Fix array read/write to use proper addressing

## Related Documents

- `CIP_PROTOCOL_REFERENCE_1756-PM020.md` - Complete protocol reference
- `ARRAY_ELEMENT_ADDRESSING_GUIDE.md` - Implementation guide for arrays
- `ARRAY_IMPLEMENTATION_REVIEW.md` - Current issues analysis
- `ARRAY_FIX_IMPLEMENTATION_PLAN.md` - Fix implementation plan

---

**Last Updated:** Based on PDF pages 13-29, 63 (partial)
**Status:** Awaiting additional pages for complete array element addressing format

