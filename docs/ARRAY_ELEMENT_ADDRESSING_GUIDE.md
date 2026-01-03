# Array Element Addressing Implementation Guide

Based on Allen-Bradley Publication 1756-PM020: Logix Controller Access Data

## Overview

This guide provides specific implementation details for correctly addressing array elements in CIP requests, based on the official Allen-Bradley documentation.

## Current Issues

The current implementation has the following problems:

1. **No Element Addressing in Path**: Arrays are addressed without using the `0x28` element segment
2. **Inefficient Writes**: Writing a single element requires reading the entire array
3. **Chunked Reading Always Starts from 0**: Cannot read specific ranges efficiently
4. **Complex Response Parsing**: Heuristics try to guess if element count field contains data

## Element Addressing Segments

### Element ID Segment Types

From Page 13 of 1756-PM020:

| Segment Type | Value | Format |
|--------------|-------|--------|
| 8-bit Element ID | `0x28` | Single byte value |
| 16-bit Element ID | `0x29` | 2 bytes (little-endian) |
| 32-bit Element ID | `0x2A` | 4 bytes (little-endian) |

**Byte Order:** All multi-byte values use **low byte first** (little-endian)

### Symbolic Segment

| Segment Type | Value | Format |
|--------------|-------|--------|
| ANSI Extended Symbolic | `0x91` | Length byte + ASCII tag name + padding |

**Format:**
```
[0x91] [Length] [Char1] [Char2] ... [CharN] [Padding?]
```

## Request Path Construction for Arrays

### For Single Element Access

**Example:** Reading `MyArray[5]` (from Page 603 of 1756-PM020)

**Correct Request Path:**
```
[0x91] [Length] [M] [y] [A] [r] [r] [a] [y] [Padding]
[0x28] [05]  // Element segment: 8-bit Element ID, index 5
```

**Complete Request:**
```
Service: 0x4C (Read Tag)
Path Size: (calculated from path length)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 05
Request Data: 01 00  // Number of elements to read (1)
```

**Current Implementation (WRONG):**
```
[0x91] [Length] [M] [y] [A] [r] [r] [a] [y] [Padding]
[01] [00]  // Just element count, no index in path!
```

### For Multiple Elements with Starting Index

**Example:** Reading elements 5-10 of array "MyArray" (6 elements total)

**Correct Request Path:**
```
[0x91] [Length] [M] [y] [A] [r] [r] [a] [y] [Padding]
[0x28] [05]  // Element segment: starting at index 5
```

**Complete Request:**
```
Service: 0x4C (Read Tag)
Path Size: (calculated)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 05
Request Data: 06 00  // Number of elements to read (6, starting from index 5)
```

**Key Points:**
- Element index goes in the **Request Path** using `0x28` segment
- Element count goes in the **Request Data**
- `0x28` uses single byte for index (8-bit Element ID)
- For indices > 255, use `0x29` (16-bit) or `0x2A` (32-bit)

## Using Fragmented Service for Arrays

### Read Tag Fragmented Service (0x52)

**Service Code:** `0x52` (Request), `0xD2` (Reply)

**Request Structure:**
```
[Service: 0x52]
[Path Size: N words]
[Request Path: 0x91 + tag name]
[Request Data: Element Count (2 bytes, little-endian)]
[Request Data: Byte Offset (4 bytes, little-endian)]
```

**Example: Reading elements 100-150 of a DINT array**

1. Calculate byte offset: 100 elements × 4 bytes = 400 bytes
2. Calculate element count: 51 elements (100-150 inclusive)

**Request:**
```
Service: 52
Path Size: 06 (6 words = 12 bytes)
Request Path: 91 0A 4D 79 41 72 72 61 79 00 00  // "MyArray"
Request Data: 33 00        // 51 elements (0x0033)
Request Data: 90 01 00 00  // Byte offset 400 (0x00000190)
```

**Response:**
```
Reply Service: D2
Reserved: 00
General Status: 00 (Success) or 06 (Reply Data Too Large)
Extended Status Size: 00
Reply Data: C4 00        // DINT type (0x00C4)
Reply Data: [Data for elements 100-150]
```

### Write Tag Fragmented Service (0x53)

**Service Code:** `0x53` (Request), `0xD3` (Reply)

**Request Structure:**
```
[Service: 0x53]
[Path Size: N words]
[Request Path: 0x91 + tag name]
[Request Data: Data Type (2 bytes)]
[Request Data: Total Element Count (2 bytes)]
[Request Data: Byte Offset (4 bytes)]
[Request Data: Actual data bytes]
```

**Key Points:**
- Total element count remains the same for all requests in the sequence
- Byte offset increases with each request
- Each request contains a chunk of data

## Implementation Recommendations

### 1. Fix Array Read Requests

**Current (WRONG):**
```rust
fn build_read_request_with_count(&self, tag_name: &str, element_count: u16) -> Vec<u8> {
    // ... builds path ...
    cip_request.extend_from_slice(&element_count.to_le_bytes()); // WRONG!
}
```

**Correct Approach:**

**Option A: Use Element Addressing in Path (for single/multiple elements)**

**Based on Page 603 of 1756-PM020:**

```rust
fn build_read_array_request(
    &self,
    tag_name: &str,
    start_index: u32,
    element_count: u16,
) -> Vec<u8> {
    let mut cip_request = Vec::new();
    
    // Service: Read Tag (0x4C)
    cip_request.push(0x4C);
    
    // Build base tag path (symbolic segment)
    let base_path = self.build_base_tag_path(tag_name);
    
    // Add element addressing segment
    let mut full_path = base_path.clone();
    
    // Determine element segment type based on index size
    if start_index <= 255 {
        // Use 8-bit Element ID (0x28)
        full_path.push(0x28);
        full_path.push(start_index as u8);
    } else if start_index <= 65535 {
        // Use 16-bit Element ID (0x29)
        full_path.push(0x29);
        full_path.push(0x00); // Padding byte
        full_path.extend_from_slice(&(start_index as u16).to_le_bytes());
    } else {
        // Use 32-bit Element ID (0x2A)
        full_path.push(0x2A);
        full_path.push(0x00); // Padding byte
        full_path.extend_from_slice(&start_index.to_le_bytes());
    }
    
    // Ensure path is word-aligned
    if full_path.len() % 2 != 0 {
        full_path.push(0x00);
    }
    
    // Path size (in words)
    let path_size = (full_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&full_path);
    
    // Request Data: Element count (NOT in path, but in Request Data)
    cip_request.extend_from_slice(&element_count.to_le_bytes());
    
    cip_request
}
```

**Option B: Use Fragmented Service (for large arrays or specific ranges)**
```rust
fn build_read_array_fragmented_request(
    &self,
    tag_name: &str,
    total_element_count: u16,
    byte_offset: u32,
) -> Vec<u8> {
    let mut cip_request = Vec::new();
    
    // Service: Read Tag Fragmented (0x52)
    cip_request.push(0x52);
    
    // Build base tag path
    let base_path = self.build_base_tag_path(tag_name);
    let path_size = (base_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&base_path);
    
    // Request Data: Element count
    cip_request.extend_from_slice(&total_element_count.to_le_bytes());
    
    // Request Data: Byte offset (4 bytes, little-endian)
    cip_request.extend_from_slice(&byte_offset.to_le_bytes());
    
    cip_request
}
```

### 2. Fix Array Write Requests

**Current (WRONG):**
- Reads entire array
- Modifies one element
- Writes entire array back

**Correct Approach:**

**Option A: Use Element Addressing (for single element)**

**Based on Page 603 of 1756-PM020:**

```rust
fn build_write_array_element_request(
    &self,
    tag_name: &str,
    index: u32,
    data_type: u16,
    value_bytes: &[u8],
) -> Vec<u8> {
    let mut cip_request = Vec::new();
    
    // Service: Write Tag (0x4D)
    cip_request.push(0x4D);
    
    // Build base tag path (symbolic segment)
    let base_path = self.build_base_tag_path(tag_name);
    
    // Add element addressing segment
    let mut full_path = base_path.clone();
    
    // Determine element segment type based on index size
    if index <= 255 {
        // Use 8-bit Element ID (0x28) - single byte
        full_path.push(0x28);
        full_path.push(index as u8);
    } else if index <= 65535 {
        // Use 16-bit Element ID (0x29)
        full_path.push(0x29);
        full_path.push(0x00); // Padding byte
        full_path.extend_from_slice(&(index as u16).to_le_bytes());
    } else {
        // Use 32-bit Element ID (0x2A)
        full_path.push(0x2A);
        full_path.push(0x00); // Padding byte
        full_path.extend_from_slice(&index.to_le_bytes());
    }
    
    // Ensure path is word-aligned
    if full_path.len() % 2 != 0 {
        full_path.push(0x00);
    }
    
    // Path size (in words)
    let path_size = (full_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&full_path);
    
    // Request Data: Data type, element count, and data
    cip_request.extend_from_slice(&data_type.to_le_bytes());
    cip_request.extend_from_slice(&1u16.to_le_bytes()); // Element count (1)
    cip_request.extend_from_slice(value_bytes);
    
    cip_request
}
```

**Option B: Use Fragmented Service (for large arrays)**
```rust
fn build_write_array_fragmented_request(
    &self,
    tag_name: &str,
    data_type: u16,
    total_element_count: u16,
    byte_offset: u32,
    data: &[u8],
) -> Vec<u8> {
    let mut cip_request = Vec::new();
    
    // Service: Write Tag Fragmented (0x53)
    cip_request.push(0x53);
    
    // Build base tag path
    let base_path = self.build_base_tag_path(tag_name);
    let path_size = (base_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&base_path);
    
    // Data type
    cip_request.extend_from_slice(&data_type.to_le_bytes());
    
    // Total element count
    cip_request.extend_from_slice(&total_element_count.to_le_bytes());
    
    // Byte offset
    cip_request.extend_from_slice(&byte_offset.to_le_bytes());
    
    // Data
    cip_request.extend_from_slice(data);
    
    cip_request
}
```

### 3. Fix Chunked Reading

**Current (WRONG):**
- Always reads from element 0
- Extracts portions from response

**Correct:**
```rust
async fn read_array_in_chunks(
    &mut self,
    base_array_name: &str,
    data_type: u16,
    start_index: u32,
    target_element_count: u32,
) -> crate::error::Result<Vec<u8>> {
    let element_size = get_element_size(data_type)?;
    let mut all_data = Vec::new();
    let mut current_byte_offset = start_index * element_size;
    let total_bytes = target_element_count * element_size;
    
    while current_byte_offset < total_bytes {
        // Use fragmented service to read from specific offset
        let request = self.build_read_array_fragmented_request(
            base_array_name,
            target_element_count as u16,
            current_byte_offset,
        );
        
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        
        // Check status
        if cip_data.len() >= 3 {
            let status = cip_data[2];
            if status == 0x06 {
                // Reply Data Too Large - extract what we got
                // Calculate how much data we received
                // Update offset for next request
            } else if status == 0x00 {
                // Success - last chunk
                // Extract remaining data
                break;
            } else {
                return Err(EtherNetIpError::Protocol(
                    format!("CIP Error: {}", status)
                ));
            }
        }
        
        // Extract data from response
        // Update current_byte_offset
    }
    
    Ok(all_data)
}
```

## Response Parsing

### Standard Read Tag Response

```
[Reply Service: 1 byte] [Reserved: 1 byte] [Status: 1 byte] [Status Size: 1 byte]
[Data Type: 2 bytes] [Element Count: 2 bytes] [Data: N bytes]
```

### Fragmented Read Response

```
[Reply Service: 1 byte] [Reserved: 1 byte] [Status: 1 byte] [Status Size: 1 byte]
[Data Type: 2 bytes] [Data: N bytes]
```

**Status Codes:**
- `0x00`: Success
- `0x06`: Reply Data Too Large (more data available)

## Testing Requirements

1. **Test Element Addressing:**
   - Read single array element: `ArrayName[5]`
   - Read multiple elements: `ArrayName[10-20]`
   - Verify path contains `0x28` segment

2. **Test Fragmented Service:**
   - Read large array (>500 bytes)
   - Read specific range (elements 100-200)
   - Verify byte offset calculation
   - Verify status code `06` handling

3. **Test Direct Writes:**
   - Write single element without reading entire array
   - Verify path contains element addressing

4. **Test Response Parsing:**
   - Verify consistent response format
   - Remove element count heuristics
   - Handle fragmented responses correctly

## Key Findings from 1756-PM020

### Element Addressing Format (Page 603)

**Single Element Access:**
- Path: `[0x91] [Tag Name] [0x28] [Index: 1 byte]`
- Request Data: `[Element Count: 2 bytes]` (typically 1 for single element)

**Multiple Elements from Starting Index:**
- Path: `[0x91] [Tag Name] [0x28] [StartIndex: 1 byte]`
- Request Data: `[Element Count: 2 bytes]` (number of elements to read starting from StartIndex)

**Example from Documentation:**
```
Access 5th element of array "count":
Path: 91 05 63 6F 75 6E 74 00 28 05
- 91 05 63 6F 75 6E 74 00 = "count" (symbolic segment)
- 28 05 = element 5 (8-bit Element ID)
```

### When to Use Which Service

1. **Read Tag (0x4C) with Element Addressing:**
   - For reading specific array elements or ranges
   - Element index in path (`0x28` segment)
   - Element count in Request Data
   - Use when data fits in single packet (~500 bytes)

2. **Read Tag Fragmented (0x52):**
   - For reading large arrays (>500 bytes)
   - Uses byte offset (4 bytes) in Request Data
   - Can read specific ranges by specifying offset
   - Returns status `0x06` when more data available

3. **Write Tag (0x4D) with Element Addressing:**
   - For writing specific array elements
   - Element index in path (`0x28` segment)
   - Element count in Request Data
   - Data in Request Data

4. **Write Tag Fragmented (0x53):**
   - For writing large arrays (>500 bytes)
   - Uses byte offset (4 bytes) in Request Data
   - Total element count remains same for all requests

---

**Status:** ✅ Element addressing format clarified from Page 603 of 1756-PM020

