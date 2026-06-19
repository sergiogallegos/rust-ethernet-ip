> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# Implementation Reference - 1756-PM020

This document provides code snippets and implementation patterns directly from the Allen-Bradley documentation for reference during implementation.

## Element ID Segment Size Selection

**From 1756-PM020, Page 870-890:**

```rust
fn build_element_id_segment(index: u32) -> Vec<u8> {
    let mut segment = Vec::new();
    
    if index <= 255 {
        // 8-bit Element ID: 0x28 + index
        segment.push(0x28);
        segment.push(index as u8);
    } else if index <= 65535 {
        // 16-bit Element ID: 0x29, 0x00, low_byte, high_byte
        segment.push(0x29);
        segment.push(0x00);
        segment.extend_from_slice(&(index as u16).to_le_bytes());
    } else {
        // 32-bit Element ID: 0x2A, 0x00, byte0, byte1, byte2, byte3
        segment.push(0x2A);
        segment.push(0x00);
        segment.extend_from_slice(&index.to_le_bytes());
    }
    
    segment
}
```

## ANSI Extended Symbol Segment Construction

**From 1756-PM020, Page 894-909:**

```rust
fn build_symbol_segment(tag_name: &str) -> Vec<u8> {
    let name_bytes = tag_name.as_bytes();
    let length = name_bytes.len();
    
    // Start with segment type and length
    let mut segment = vec![0x91, length as u8];
    segment.extend_from_slice(name_bytes);
    
    // Pad to word boundary if odd length
    if length % 2 == 1 {
        segment.push(0x00);
    }
    
    segment
}
```

**Examples:**
| Tag Name | Length | Segment Bytes |
|----------|--------|---------------|
| "rate" | 4 | `91 04 72 61 74 65` |
| "count" | 5 | `91 05 63 6F 75 6E 74 00` (padded) |
| "TotalCount" | 10 | `91 0A 54 6F 74 61 6C 43 6F 75 6E 74` |

## Complete Read Tag Request for Array Element

**From 1756-PM020, Page 815-837:**

```rust
fn build_read_array_element_request(
    tag_name: &str,
    element_index: u32,
) -> Vec<u8> {
    let mut request = Vec::new();
    
    // Request Service: Read Tag (0x4C)
    request.push(0x4C);
    
    // Build Request Path
    let mut path = build_symbol_segment(tag_name);
    path.extend_from_slice(&build_element_id_segment(element_index));
    
    // Ensure path is word-aligned
    if path.len() % 2 != 0 {
        path.push(0x00);
    }
    
    // Request Path Size (in words)
    let path_size = (path.len() / 2) as u8;
    request.push(path_size);
    request.extend_from_slice(&path);
    
    // Request Data: Element count (1)
    request.extend_from_slice(&1u16.to_le_bytes());
    
    request
}
```

**Example: Read `MyArray[10]` (DINT array)**

```
Service: 4C
Path Size: 05 (5 words = 10 bytes)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 0A
Request Data: 01 00
```

## Complete Write Tag Request for Array Element

**From 1756-PM020, Page 855-867:**

```rust
fn build_write_array_element_request(
    tag_name: &str,
    element_index: u32,
    data_type: u16,
    value_bytes: &[u8],
) -> Vec<u8> {
    let mut request = Vec::new();
    
    // Request Service: Write Tag (0x4D)
    request.push(0x4D);
    
    // Build Request Path
    let mut path = build_symbol_segment(tag_name);
    path.extend_from_slice(&build_element_id_segment(element_index));
    
    // Ensure path is word-aligned
    if path.len() % 2 != 0 {
        path.push(0x00);
    }
    
    // Request Path Size (in words)
    let path_size = (path.len() / 2) as u8;
    request.push(path_size);
    request.extend_from_slice(&path);
    
    // Request Data: Data type, element count, and data
    request.extend_from_slice(&data_type.to_le_bytes());
    request.extend_from_slice(&1u16.to_le_bytes()); // Write 1 element
    request.extend_from_slice(value_bytes);
    
    request
}
```

**Example: Write value 0x12345678 to `MyArray[10]`**

```
Service: 4D
Path Size: 05
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 0A
Request Data: 
  - C4 00 (DINT type)
  - 01 00 (1 element)
  - 78 56 34 12 (value, little-endian)
```

## Reading Multiple Array Elements Starting at Index

**From 1756-PM020, Page 840-851:**

```rust
fn build_read_array_range_request(
    tag_name: &str,
    start_index: u32,
    element_count: u16,
) -> Vec<u8> {
    let mut request = Vec::new();
    
    // Request Service: Read Tag (0x4C)
    request.push(0x4C);
    
    // Build Request Path with starting element index
    let mut path = build_symbol_segment(tag_name);
    path.extend_from_slice(&build_element_id_segment(start_index));
    
    // Ensure path is word-aligned
    if path.len() % 2 != 0 {
        path.push(0x00);
    }
    
    // Request Path Size (in words)
    let path_size = (path.len() / 2) as u8;
    request.push(path_size);
    request.extend_from_slice(&path);
    
    // Request Data: Element count (NOT in path!)
    request.extend_from_slice(&element_count.to_le_bytes());
    
    request
}
```

**Example: Read 5 elements starting at `MyArray[10]`**

```
Service: 4C
Path Size: 05
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 0A
Request Data: 05 00 (Read 5 elements)
```

**Response contains:** Tag type + 5 consecutive DINT values (20 bytes of data)

## Multi-Dimensional Array Access

**From 1756-PM020, Page 756-794:**

### 2D Array Access

**Example: Access element [2,5] of 2D array `Grid`**

```
Request Path: 91 04 47 72 69 64 28 02 28 05
  - 91 04 = Symbol segment, 4 chars
  - 47 72 69 64 = "Grid"
  - 28 02 = First dimension = 2
  - 28 05 = Second dimension = 5
```

**Implementation:**
```rust
fn build_2d_array_path(
    tag_name: &str,
    dim0: u32,
    dim1: u32,
) -> Vec<u8> {
    let mut path = build_symbol_segment(tag_name);
    path.extend_from_slice(&build_element_id_segment(dim0));
    path.extend_from_slice(&build_element_id_segment(dim1));
    
    if path.len() % 2 != 0 {
        path.push(0x00);
    }
    
    path
}
```

### 3D Array Access

**Example: Access element [1,2,3] of 3D array `Cube`**

```
Request Path: 91 04 43 75 62 65 28 01 28 02 28 03
  - 91 04 = Symbol segment, 4 chars
  - 43 75 62 65 = "Cube"
  - 28 01 = Dim 0 = 1
  - 28 02 = Dim 1 = 2
  - 28 03 = Dim 2 = 3
```

## UDT Member Access

**From 1756-PM020, Page 920-1095:**

### Access Single Member

**Example: Read member `Speed` from UDT tag `MachineData`**

```
Request Path: 91 0B 4D 61 63 68 69 6E 65 44 61 74 61 00 91 05 53 70 65 65 64 00
  - 91 0B = Symbol segment, 11 chars
  - 4D 61 63 68 69 6E 65 44 61 74 61 00 = "MachineData" + pad
  - 91 05 = Symbol segment, 5 chars
  - 53 70 65 65 64 00 = "Speed" + pad
```

**Implementation:**
```rust
fn build_udt_member_path(
    udt_tag_name: &str,
    member_name: &str,
) -> Vec<u8> {
    let mut path = build_symbol_segment(udt_tag_name);
    path.extend_from_slice(&build_symbol_segment(member_name));
    
    if path.len() % 2 != 0 {
        path.push(0x00);
    }
    
    path
}
```

### Access Array Member Within UDT

**Example: Read element 5 of array member `Counts` from UDT tag `MachineData`**

```
Request Path: 91 0B 4D 61 63 68 69 6E 65 44 61 74 61 00 91 06 43 6F 75 6E 74 73 28 05
  - 91 0B ... 00 = "MachineData"
  - 91 06 = Symbol segment, 6 chars
  - 43 6F 75 6E 74 73 = "Counts"
  - 28 05 = Element 5
```

This accesses `MachineData.Counts[5]`

### Access Member from Array of UDTs

**Example: Read member `Status` from element 3 of UDT array `Stations`**

```
Request Path: 91 08 53 74 61 74 69 6F 6E 73 28 03 91 06 53 74 61 74 75 73
  - 91 08 = Symbol segment, 8 chars
  - 53 74 61 74 69 6F 6E 73 = "Stations"
  - 28 03 = Element 3
  - 91 06 = Symbol segment, 6 chars
  - 53 74 61 74 75 73 = "Status"
```

This accesses `Stations[3].Status`

## Response Parsing

**From 1756-PM020, Page 828-837:**

### Read Tag Response Format

```
Reply Service: CC (0x4C with MSB set)
Reserved: 00
General Status: 00 (Success) or error code
Extended Status Size: 00 (or number of 16-bit words)
Extended Status: (if Extended Status Size > 0)
Reply Data:
  - Tag Type: 2 bytes (e.g., C4 00 for DINT)
  - Data: Variable length based on type and element count
```

**Parsing:**
```rust
fn parse_read_tag_response(response: &[u8]) -> Result<(u8, u16, Vec<u8>), Error> {
    if response.len() < 8 {
        return Err(Error::InvalidResponse);
    }
    
    let status = response[2];
    let extended_status_size = response[3];
    let data_start = 4 + (extended_status_size as usize * 2);
    
    if response.len() < data_start + 2 {
        return Err(Error::InvalidResponse);
    }
    
    let tag_type = u16::from_le_bytes([response[data_start], response[data_start + 1]]);
    let data = response[data_start + 2..].to_vec();
    
    Ok((status, tag_type, data))
}
```

---

**Status:** ✅ Complete reference implementation patterns from 1756-PM020

