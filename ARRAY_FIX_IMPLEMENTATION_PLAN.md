# Array Read/Write Fix Implementation Plan

## Problem Summary

The current array implementation has several issues:
1. **No proper element addressing**: Arrays are read/written without using CIP element addressing (0x28 segment)
2. **Inefficient writes**: Writing a single element requires reading the entire array
3. **Chunked reading always starts from 0**: Can't read specific ranges efficiently
4. **Complex heuristics**: Code tries to guess if element count field contains data or count

## Root Cause

According to CIP specification (1756-PM020), arrays should use **element addressing segments** in the path:
- Element Symbol: 0x28 (8-bit Element ID)
- Format/Size: 0x02 (for 16-bit addressing) or 0x04 (for 32-bit)
- Index: Starting element index (2 or 4 bytes, little-endian)
- Count: Number of elements (2 bytes, little-endian) - **Format TBD**

**Note:** From 1756-PM020 Page 13:
- `0x28` is for 8-bit Element ID (single byte value)
- `0x29` is for 16-bit Element ID (2 bytes, little-endian)
- `0x2A` is for 32-bit Element ID (4 bytes, little-endian)

The exact format for array element addressing with index + count needs verification from additional documentation sections (Page 63 Example 1).

**Alternative:** Use Read Tag Fragmented Service (0x52) with byte offset for large arrays or specific ranges.

The current implementation just appends element count to the request, which doesn't properly address array elements.

## Solution: Implement Proper Element Addressing

### Phase 1: Fix Array Read Requests

**File**: `src/lib.rs`

**Current Code** (lines 5069-5111):
```rust
fn build_read_request_with_count(&self, tag_name: &str, element_count: u16) -> Vec<u8> {
    // ... builds path ...
    // Element count (little-endian)  <-- WRONG: Just appends count
    cip_request.extend_from_slice(&element_count.to_le_bytes());
}
```

**Fixed Code**:
```rust
fn build_read_array_request(
    &self,
    tag_name: &str,
    start_index: u16,
    element_count: u16,
) -> Vec<u8> {
    let mut cip_request = Vec::new();
    
    // Service: Read Tag (0x4C)
    cip_request.push(0x4C);
    
    // Build base tag path (without array indices)
    let base_path = self.build_base_tag_path(tag_name);
    let path_size = (base_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&base_path);
    
    // Element addressing segment (CIP spec)
    cip_request.push(0x28);  // Element symbol
    cip_request.push(0x02);  // Format: 16-bit (2 bytes)
    cip_request.extend_from_slice(&start_index.to_le_bytes());   // Starting index
    cip_request.extend_from_slice(&element_count.to_le_bytes()); // Count
    
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
                path
            })
        }
        Err(_) => {
            // Fallback: simple symbol segment
            let mut path = Vec::new();
            path.push(0x91);
            path.push(tag_name.len() as u8);
            path.extend_from_slice(tag_name.as_bytes());
            path
        }
    }
}
```

### Phase 2: Fix Array Write Requests

**Current Code** (lines 3116-3141):
```rust
fn build_write_array_request(
    &self,
    tag_name: &str,
    data_type: u16,
    element_count: u16,
    data: &[u8],
) -> Vec<u8> {
    // ... builds path ...
    // Data type and element count  <-- WRONG: No element addressing
    cip_request.extend_from_slice(&data_type.to_le_bytes());
    cip_request.extend_from_slice(&element_count.to_le_bytes());
}
```

**Fixed Code**:
```rust
fn build_write_array_request(
    &self,
    tag_name: &str,
    start_index: u16,
    element_count: u16,
    data_type: u16,
    data: &[u8],
) -> crate::error::Result<Vec<u8>> {
    let mut cip_request = Vec::new();
    
    // Service: Write Tag (0x4D)
    cip_request.push(0x4D);
    
    // Build base tag path
    let base_path = self.build_base_tag_path(tag_name);
    let path_size = (base_path.len() / 2) as u8;
    cip_request.push(path_size);
    cip_request.extend_from_slice(&base_path);
    
    // Element addressing segment
    cip_request.push(0x28);  // Element symbol
    cip_request.push(0x02);  // Format: 16-bit
    cip_request.extend_from_slice(&start_index.to_le_bytes());
    cip_request.extend_from_slice(&element_count.to_le_bytes());
    
    // Data type and data
    cip_request.extend_from_slice(&data_type.to_le_bytes());
    cip_request.extend_from_slice(data);
    
    Ok(cip_request)
}

/// Write a single array element efficiently
async fn write_array_element_direct(
    &mut self,
    base_array_name: &str,
    index: u32,
    value: PlcValue,
) -> crate::error::Result<()> {
    let data_type = value.get_data_type();
    let value_bytes = value.to_bytes();
    
    // Write directly to the element
    let request = self.build_write_array_request(
        base_array_name,
        index as u16,
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

### Phase 3: Fix Chunked Reading

**Current Code** (lines 2188-2486):
- Always reads from element 0
- Extracts portions from the response

**Fixed Code**:
```rust
async fn read_array_in_chunks(
    &mut self,
    base_array_name: &str,
    data_type: u16,
    start_index: u32,
    target_element_count: u32,
) -> crate::error::Result<Vec<u8>> {
    let element_size = match data_type {
        0x00C1 => 1, 0x00C2 => 1, 0x00C3 => 2, 0x00C4 => 4,
        0x00C5 => 8, 0x00C6 => 1, 0x00C7 => 2, 0x00C8 => 4,
        0x00C9 => 8, 0x00CA => 4, 0x00CB => 8,
        _ => return Err(EtherNetIpError::Protocol(
            format!("Unsupported array data type: 0x{:04X}", data_type)
        )),
    };
    
    let mut all_data = Vec::new();
    let mut current_index = start_index;
    const MAX_ELEMENTS_PER_REQUEST: u16 = 50;
    
    while current_index < start_index + target_element_count {
        let remaining = (start_index + target_element_count) - current_index;
        let chunk_count = (remaining.min(MAX_ELEMENTS_PER_REQUEST as u32)) as u16;
        
        // Use element addressing to read from current_index
        let request = self.build_read_array_request(
            base_array_name,
            current_index as u16,
            chunk_count,
        );
        
        let response = self.send_cip_request(&request).await?;
        let cip_data = self.extract_cip_from_response(&response)?;
        
        // Extract data (format should be consistent now)
        if cip_data.len() < 8 {
            return Err(EtherNetIpError::Protocol(
                "Response too short".to_string()
            ));
        }
        
        // Check for errors
        if cip_data[2] != 0x00 {
            let error_msg = self.get_cip_error_message(cip_data[2]);
            return Err(EtherNetIpError::Protocol(format!(
                "CIP Error: {}", error_msg
            )));
        }
        
        // Extract data (starts at offset 8: service, status, data_type, element_count)
        let data_start = 8;
        if cip_data.len() < data_start + (chunk_count as usize * element_size) {
            // Got less data than requested - might be end of array
            let available = cip_data.len() - data_start;
            let elements_received = available / element_size;
            all_data.extend_from_slice(&cip_data[data_start..data_start + (elements_received * element_size)]);
            break; // Reached end of array
        } else {
            all_data.extend_from_slice(&cip_data[data_start..data_start + (chunk_count as usize * element_size)]);
        }
        
        current_index += chunk_count as u32;
    }
    
    Ok(all_data)
}
```

### Phase 4: Simplify Response Parsing

With proper element addressing, the response format should be consistent:
```
[Service: 1 byte] [Reserved: 1 byte] [Status: 1 byte] [Status Size: 1 byte]
[Data Type: 2 bytes] [Element Count: 2 bytes] [Data: N bytes]
```

We can remove the complex heuristics for detecting if element count field contains data.

## Testing Plan

1. **Unit Tests**:
   - Test `build_read_array_request()` generates correct CIP path
   - Test `build_write_array_request()` generates correct CIP path
   - Test element addressing segment format

2. **Integration Tests**:
   - Read array element [5] directly
   - Read array range [10-20]
   - Write array element [5] without reading entire array
   - Read large array (>50 elements) with chunking
   - Write large array with chunking

3. **Regression Tests**:
   - Ensure existing array reads still work
   - Ensure BOOL array handling still works
   - Ensure multi-dimensional arrays still work

## Migration Strategy

1. **Add new methods** alongside existing ones (backward compatibility)
2. **Update internal calls** to use new methods
3. **Deprecate old methods** with warnings
4. **Remove old methods** in next major version

## Priority

- **HIGH**: Fix read requests (affects all array reads)
- **HIGH**: Fix write requests (affects all array writes)
- **MEDIUM**: Fix chunked reading (improves efficiency)
- **LOW**: Simplify response parsing (code cleanup)

## Estimated Impact

- **Performance**: 10-100x improvement for large array operations
- **Reliability**: Eliminates heuristics that can fail
- **Correctness**: Aligns with CIP specification
- **Maintainability**: Simpler, more understandable code

