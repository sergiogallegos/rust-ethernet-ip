> **Historical reference.** This document records past work and may not reflect the current 1.0.0 codebase.

# Array Implementation Fixes - Based on 1756-PM020

This document provides the corrected implementation for array read/write operations based on the official Allen-Bradley documentation (1756-PM020, Pages 603-919).

**Reference:** `docs/Logix_5000_Controllers_Data_Access.md` - Comprehensive CIP Addressing Examples section

## Critical Finding: Element Addressing Format

**From 1756-PM020 (Pages 603-919):**

### Element ID Segment Selection

| Element Value Range | Segment Type | Format |
|---------------------|--------------|--------|
| 0-255 | 8-bit Element ID | `0x28` + 1 byte value |
| 256-65535 | 16-bit Element ID | `0x29 0x00` + 2 bytes (low, high) |
| 65536+ | 32-bit Element ID | `0x2A 0x00` + 4 bytes (lowest to highest) |

### Example: Access Element 5 of Array "count"

**Request Path:**
```
91 05 63 6F 75 6E 74 00 28 05
```

**Breakdown:**
- `91` = ANSI Extended Symbol Segment identifier
- `05` = Length of tag name (5 characters)
- `63 6F 75 6E 74` = ASCII for "count"
- `00` = Pad byte (names with odd length need padding to word boundary)
- `28` = 8-bit Element ID segment identifier
- `05` = Element index = 5

### Example: Access Element 300 (16-bit Element ID)

**Request Path:**
```
91 0A 4C 61 72 67 65 41 72 72 61 79 29 00 2C 01
```

**Breakdown:**
- `91 0A` = Symbol segment, 10 chars
- `4C 61 72 67 65 41 72 72 61 79` = "LargeArray"
- `29` = 16-bit Element ID segment identifier
- `00` = Pad byte
- `2C 01` = Element index = 0x012C = 300 (little-endian)

### Key Insights:
- Element index goes in the **Request Path** using appropriate Element ID segment
- Element count goes in the **Request Data** (NOT in the path)
- Path must be word-aligned (even number of bytes)

## Current Implementation Problems

### Problem 1: Missing Element Index in Path

**Current (WRONG):**
```rust
// build_read_request_with_count() - Line 5069
Request Path: [0x91] [Tag Name]
Request Data: [Element Count]  // Missing starting index!
```

**Correct:**
```rust
Request Path: [0x91] [Tag Name] [0x28] [StartIndex]
Request Data: [Element Count]
```

### Problem 2: No Support for Reading Specific Ranges

**Current:** Always reads from element 0, then extracts portions

**Correct:** Use element addressing to read from any starting index

### Problem 3: Inefficient Writes

**Current:** Reads entire array, modifies element, writes back

**Correct:** Write directly to element using element addressing in path

## Corrected Implementation

### 1. Fix Array Read Request

**File:** `src/lib.rs`

**Replace `build_read_request_with_count()`:**

```rust
/// Builds a CIP Read Tag Service request with element addressing
/// 
/// For arrays, the element index is specified in the Request Path using
/// the 0x28 Element ID segment. The element count is in Request Data.
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
    let mut full_path = base_path;
    
    // Add element addressing segment
    // Based on 1756-PM020: Use appropriate segment type based on index value
    if start_index <= 255 {
        // 8-bit Element ID: 0x28 + index
        full_path.push(0x28);
        full_path.push(start_index as u8);
    } else if start_index <= 65535 {
        // 16-bit Element ID: 0x29, 0x00, low_byte, high_byte
        full_path.push(0x29);
        full_path.push(0x00);  // Padding byte
        full_path.extend_from_slice(&(start_index as u16).to_le_bytes());
    } else {
        // 32-bit Element ID: 0x2A, 0x00, byte0, byte1, byte2, byte3
        full_path.push(0x2A);
        full_path.push(0x00);  // Padding byte
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
    
    // Request Data: Element count (NOT in path!)
    cip_request.extend_from_slice(&element_count.to_le_bytes());
    
    cip_request
}

/// Builds base tag path without array element addressing
fn build_base_tag_path(&self, tag_name: &str) -> Vec<u8> {
    // Parse tag path but strip array indices
    match TagPath::parse(tag_name) {
        Ok(path) => {
            // If it's an array path, get just the base
            let base_path = match &path {
                TagPath::Array { base_path, .. } => base_path.as_ref(),
                _ => &path,
            };
            base_path.to_cip_path().unwrap_or_else(|_| {
                // Fallback: simple symbol segment
                let mut path = Vec::new();
                path.push(0x91);
                path.push(tag_name.len() as u8);
                path.extend_from_slice(tag_name.as_bytes());
                if path.len() % 2 != 0 {
                    path.push(0x00);
                }
                path
            })
        }
        Err(_) => {
            // Fallback: simple symbol segment
            let mut path = Vec::new();
            path.push(0x91);
            path.push(tag_name.len() as u8);
            path.extend_from_slice(tag_name.as_bytes());
            if path.len() % 2 != 0 {
                path.push(0x00);
            }
            path
        }
    }
}
```

### 2. Fix Array Write Request

**Replace `build_write_array_request()`:**

```rust
/// Builds a CIP Write Tag Service request with element addressing
fn build_write_array_request(
    &self,
    tag_name: &str,
    start_index: u32,
    element_count: u16,
    data_type: u16,
    data: &[u8],
) -> crate::error::Result<Vec<u8>> {
    let mut cip_request = Vec::new();
    
    // Service: Write Tag (0x4D)
    cip_request.push(0x4D);
    
    // Build base tag path
    let base_path = self.build_base_tag_path(tag_name);
    let mut full_path = base_path;
    
    // Add element addressing segment
    if start_index <= 255 {
        full_path.push(0x28);
        full_path.push(start_index as u8);
    } else if start_index <= 65535 {
        full_path.push(0x29);
        full_path.push(0x00);
        full_path.extend_from_slice(&(start_index as u16).to_le_bytes());
    } else {
        full_path.push(0x2A);
        full_path.push(0x00);
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
    
    // Request Data: Data type, element count, and data
    cip_request.extend_from_slice(&data_type.to_le_bytes());
    cip_request.extend_from_slice(&element_count.to_le_bytes());
    cip_request.extend_from_slice(data);
    
    Ok(cip_request)
}

/// Write a single array element directly (no need to read entire array)
async fn write_array_element_direct(
    &mut self,
    base_array_name: &str,
    index: u32,
    value: PlcValue,
) -> crate::error::Result<()> {
    let data_type = value.get_data_type();
    let value_bytes = value.to_bytes();
    
    // Write directly to the element using element addressing
    let request = self.build_write_array_request(
        base_array_name,
        index,
        1, // Write 1 element
        data_type,
        &value_bytes,
    )?;
    
    let response = self.send_cip_request(&request).await?;
    let cip_data = self.extract_cip_from_response(&response)?;
    
    // Check for errors
    if cip_data.len() >= 3 && cip_data[2] != 0x00 {
        let error_msg = self.get_cip_error_message(cip_data[2]);
        return Err(EtherNetIpError::Protocol(format!(
            "CIP Error: {}", error_msg
        )));
    }
    
    Ok(())
}
```

### 3. Fix Chunked Reading

**Replace `read_array_in_chunks()`:**

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
    let mut current_index = start_index;
    const MAX_ELEMENTS_PER_REQUEST: u16 = 50; // Conservative limit
    
    while current_index < start_index + target_element_count {
        let remaining = (start_index + target_element_count) - current_index;
        let chunk_count = (remaining.min(MAX_ELEMENTS_PER_REQUEST as u32)) as u16;
        
        // Use element addressing to read from current_index
        let request = self.build_read_array_request(
            base_array_name,
            current_index,
            chunk_count,
        );
        
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        
        // Check for errors
        if cip_data.len() >= 3 {
            let status = cip_data[2];
            if status != 0x00 && status != 0x06 {
                let error_msg = self.get_cip_error_message(status);
                return Err(EtherNetIpError::Protocol(format!(
                    "CIP Error: {}", error_msg
                )));
            }
        }
        
        // Extract data (starts at offset 8: service, status, data_type, element_count)
        let data_start = 8;
        if cip_data.len() < data_start {
            return Err(EtherNetIpError::Protocol(
                "Response too short".to_string()
            ));
        }
        
        // Calculate how much data we got
        let available = cip_data.len() - data_start;
        let elements_received = (available / element_size).min(chunk_count as usize);
        let data_bytes = elements_received * element_size;
        
        if available >= data_bytes {
            all_data.extend_from_slice(&cip_data[data_start..data_start + data_bytes]);
        } else {
            // Got less than expected - might be end of array
            all_data.extend_from_slice(&cip_data[data_start..]);
            break;
        }
        
        current_index += elements_received as u32;
        
        // If status was 0x00 (Success), we're done
        if cip_data.len() >= 3 && cip_data[2] == 0x00 {
            break;
        }
    }
    
    Ok(all_data)
}
```

### 4. Update Array Element Read Workaround

**Replace `read_array_element_workaround()`:**

```rust
async fn read_array_element_workaround(
    &mut self,
    base_array_name: &str,
    index: u32,
) -> crate::error::Result<PlcValue> {
    // Use element addressing to read directly from index
    let request = self.build_read_array_request(
        base_array_name,
        index,
        1, // Read 1 element
    );
    
    let response = self.send_cip_request(&request).await?;
    let cip_data = self.extract_cip_from_response(&response)?;
    
    // Parse response (should be consistent format now)
    self.parse_cip_response(&cip_data)
}
```

### 5. Update Array Element Write Workaround

**Replace `write_array_element_workaround()`:**

```rust
async fn write_array_element_workaround(
    &mut self,
    base_array_name: &str,
    index: u32,
    value: PlcValue,
) -> crate::error::Result<()> {
    // Use direct element write - no need to read entire array!
    self.write_array_element_direct(base_array_name, index, value).await
}
```

## Testing the Fixes

### Test Case 1: Read Single Array Element

```rust
// Read MyArray[5]
let value = client.read_tag("MyArray[5]").await?;
```

**Expected Request (from 1756-PM020 Example):**
```
Service: 4C
Path Size: 05 (5 words = 10 bytes)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 05
  - 91 07 = Symbol segment, 7 chars
  - 4D 79 41 72 72 61 79 00 = "MyArray" + pad
  - 28 05 = Element 5
Request Data: 01 00 (Read 1 element)
```

**Expected Response:**
```
Reply Service: CC
Reserved: 00
General Status: 00 (Success)
Extended Status Size: 00
Reply Data: C4 00 (DINT type) + 4-byte value
```

### Test Case 2: Read Array Range

```rust
// Read elements 10-20 of MyArray (11 elements)
// This should use element addressing with start_index=10, count=11
```

**Expected Request (from 1756-PM020 Example, Page 840-851):**
```
Service: 4C
Path Size: 05 (5 words = 10 bytes)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 0A
  - 91 07 = Symbol segment, 7 chars
  - 4D 79 41 72 72 61 79 00 = "MyArray" + pad
  - 28 0A = Element 10 (0x0A)
Request Data: 0B 00 (Read 5 elements - note: example shows 05 00 for 5 elements)
```

**Response contains:** Tag type + 5 consecutive DINT values (20 bytes of data)

### Test Case 3: Write Single Array Element

```rust
// Write to MyArray[10] with value 0x12345678
client.write_tag("MyArray[10]", PlcValue::Dint(0x12345678)).await?;
```

**Expected Request (from 1756-PM020 Example, Page 855-867):**
```
Service: 4D
Path Size: 05 (5 words = 10 bytes)
Request Path: 91 07 4D 79 41 72 72 61 79 00 28 0A
  - 91 07 = Symbol segment, 7 chars
  - 4D 79 41 72 72 61 79 00 = "MyArray" + pad
  - 28 0A = Element 10
Request Data: 
  - C4 00 = DINT type (0x00C4)
  - 01 00 = Write 1 element
  - 78 56 34 12 = Value 0x12345678 (little-endian)
```

**Expected Response:**
```
Reply Service: CD
General Status: 00 (Success)
```

### Test Case 4: Read Element > 255 (16-bit Element ID)

```rust
// Read element 300 of LargeArray
let value = client.read_tag("LargeArray[300]").await?;
```

**Expected Request Path:**
```
91 0A 4C 61 72 67 65 41 72 72 61 79 29 00 2C 01
  - 91 0A = Symbol segment, 10 chars
  - 4C 61 72 67 65 41 72 72 61 79 = "LargeArray"
  - 29 = 16-bit Element ID segment
  - 00 = Pad byte
  - 2C 01 = Element index 300 (0x012C, little-endian)
```

## Migration Notes

1. **Backward Compatibility:** The old `build_read_request_with_count()` can be kept for backward compatibility but should be deprecated
2. **TagPath Integration:** The `TagPath` implementation already uses `0x28` for single element access - this is correct!
3. **Multiple Elements:** For reading multiple elements, use element addressing with start_index in path and count in Request Data
4. **Large Arrays:** For arrays >500 bytes, consider using Fragmented Service (0x52) with byte offset

---

**Status:** ✅ Implementation details clarified from 1756-PM020 Page 603

