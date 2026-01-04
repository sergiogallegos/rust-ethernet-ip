/// Unit tests for CIP element addressing generation
///
/// These tests verify that the element addressing functions correctly generate
/// CIP segments according to the 1756-PM020 specification.
///
/// Reference: 1756-PM020, Pages 603-611, 870-890 (Element ID Segments)
///
/// Note: These tests require a connection to test the EipClient helper methods.
/// They are marked with #[ignore] and will only run when explicitly requested.
/// To run: `cargo test -- --ignored`
use rust_ethernet_ip::{EipClient, TagPath};

// ============================================================================
// Tests that don't require a connection - using public TagPath API
// ============================================================================

#[test]
fn test_tagpath_element_addressing_8bit() {
    // Test that TagPath correctly generates 8-bit element segments
    // Reference: 1756-PM020, Page 607, Example 1

    let path = TagPath::parse("MyArray[5]").unwrap();
    let cip_path = path.to_cip_path().unwrap();

    // Find the element segment (should be 0x28 for 8-bit)
    let mut found_element = false;
    for i in 0..cip_path.len().saturating_sub(1) {
        if cip_path[i] == 0x28 {
            found_element = true;
            assert_eq!(cip_path[i + 1], 5, "Element index should be 5");
            break;
        }
    }
    assert!(
        found_element,
        "Path should contain 8-bit element segment (0x28)"
    );
}

#[test]
fn test_tagpath_element_addressing_16bit() {
    // Test that TagPath correctly generates 16-bit element segments
    // Reference: 1756-PM020, Page 666-684, Example 3

    let path = TagPath::parse("MyArray[1000]").unwrap();
    let cip_path = path.to_cip_path().unwrap();

    // Find the element segment (should be 0x29 for 16-bit)
    let mut found_element = false;
    for i in 0..cip_path.len().saturating_sub(3) {
        if cip_path[i] == 0x29 {
            found_element = true;
            assert_eq!(
                cip_path[i + 1],
                0x00,
                "16-bit segment should have padding byte"
            );
            let index = u16::from_le_bytes([cip_path[i + 2], cip_path[i + 3]]);
            assert_eq!(index, 1000, "Element index should be 1000");
            break;
        }
    }
    assert!(
        found_element,
        "Path should contain 16-bit element segment (0x29)"
    );
}

#[test]
fn test_tagpath_element_addressing_32bit() {
    // Test that TagPath correctly generates 32-bit element segments
    // Reference: 1756-PM020, Page 144-146

    let path = TagPath::parse("MyArray[100000]").unwrap();
    let cip_path = path.to_cip_path().unwrap();

    // Find the element segment (should be 0x2A for 32-bit)
    let mut found_element = false;
    for i in 0..cip_path.len().saturating_sub(5) {
        if cip_path[i] == 0x2A {
            found_element = true;
            assert_eq!(
                cip_path[i + 1],
                0x00,
                "32-bit segment should have padding byte"
            );
            let index = u32::from_le_bytes([
                cip_path[i + 2],
                cip_path[i + 3],
                cip_path[i + 4],
                cip_path[i + 5],
            ]);
            assert_eq!(index, 100000, "Element index should be 100000");
            break;
        }
    }
    assert!(
        found_element,
        "Path should contain 32-bit element segment (0x2A)"
    );
}

#[test]
fn test_tagpath_element_addressing_boundaries() {
    // Test boundary conditions for element segment selection

    // Boundary: 255 (8-bit max)
    let path_255 = TagPath::parse("MyArray[255]").unwrap();
    let cip_path_255 = path_255.to_cip_path().unwrap();
    assert!(
        cip_path_255.contains(&0x28),
        "Index 255 should use 8-bit segment"
    );

    // Boundary: 256 (16-bit start)
    let path_256 = TagPath::parse("MyArray[256]").unwrap();
    let cip_path_256 = path_256.to_cip_path().unwrap();
    assert!(
        cip_path_256.contains(&0x29),
        "Index 256 should use 16-bit segment"
    );

    // Boundary: 65535 (16-bit max)
    let path_65535 = TagPath::parse("MyArray[65535]").unwrap();
    let cip_path_65535 = path_65535.to_cip_path().unwrap();
    assert!(
        cip_path_65535.contains(&0x29),
        "Index 65535 should use 16-bit segment"
    );

    // Boundary: 65536 (32-bit start)
    let path_65536 = TagPath::parse("MyArray[65536]").unwrap();
    let cip_path_65536 = path_65536.to_cip_path().unwrap();
    assert!(
        cip_path_65536.contains(&0x2A),
        "Index 65536 should use 32-bit segment"
    );
}

// ============================================================================
// Tests that require EipClient connection (marked with #[ignore])
// ============================================================================

/// Helper function to create a test EipClient
/// Note: Requires a PLC connection or will be skipped
async fn create_test_client() -> Option<EipClient> {
    // Try to connect to localhost - if it fails, tests will be skipped
    EipClient::connect("127.0.0.1:44818").await.ok()
}

#[tokio::test]
#[ignore] // Requires EipClient instance - run with `cargo test -- --ignored`
async fn test_element_id_segment_8bit() {
    // Test 8-bit Element ID segment (0x28) for indices 0-255
    // Reference: 1756-PM020, Page 607, Example 1

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test index 0
    let segment = client.build_element_id_segment(0);
    assert_eq!(
        segment,
        vec![0x28, 0x00],
        "Index 0 should use 8-bit segment"
    );

    // Test index 5
    let segment = client.build_element_id_segment(5);
    assert_eq!(
        segment,
        vec![0x28, 0x05],
        "Index 5 should use 8-bit segment"
    );

    // Test index 255 (max 8-bit)
    let segment = client.build_element_id_segment(255);
    assert_eq!(
        segment,
        vec![0x28, 0xFF],
        "Index 255 should use 8-bit segment"
    );

    // Verify length is 2 bytes
    assert_eq!(segment.len(), 2, "8-bit segment should be 2 bytes");
}

#[tokio::test]
#[ignore]
async fn test_element_id_segment_16bit() {
    // Test 16-bit Element ID segment (0x29) for indices 256-65535
    // Reference: 1756-PM020, Page 666-684, Example 3

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test index 256 (first 16-bit)
    let segment = client.build_element_id_segment(256);
    assert_eq!(
        segment,
        vec![0x29, 0x00, 0x00, 0x01],
        "Index 256 should use 16-bit segment"
    );

    // Test index 1000
    let segment = client.build_element_id_segment(1000);
    let expected = {
        let mut v = vec![0x29, 0x00];
        v.extend_from_slice(&1000u16.to_le_bytes());
        v
    };
    assert_eq!(segment, expected, "Index 1000 should use 16-bit segment");
    assert_eq!(segment.len(), 4, "16-bit segment should be 4 bytes");

    // Test index 65535 (max 16-bit)
    let segment = client.build_element_id_segment(65535);
    let expected = {
        let mut v = vec![0x29, 0x00];
        v.extend_from_slice(&65535u16.to_le_bytes());
        v
    };
    assert_eq!(segment, expected, "Index 65535 should use 16-bit segment");
    assert_eq!(segment.len(), 4, "16-bit segment should be 4 bytes");
}

#[tokio::test]
#[ignore]
async fn test_element_id_segment_32bit() {
    // Test 32-bit Element ID segment (0x2A) for indices > 65535
    // Reference: 1756-PM020, Page 144-146 (Element ID Segments table)

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test index 65536 (first 32-bit)
    let segment = client.build_element_id_segment(65536);
    let expected = {
        let mut v = vec![0x2A, 0x00];
        v.extend_from_slice(&65536u32.to_le_bytes());
        v
    };
    assert_eq!(segment, expected, "Index 65536 should use 32-bit segment");
    assert_eq!(segment.len(), 6, "32-bit segment should be 6 bytes");

    // Test index 100000
    let segment = client.build_element_id_segment(100000);
    let expected = {
        let mut v = vec![0x2A, 0x00];
        v.extend_from_slice(&100000u32.to_le_bytes());
        v
    };
    assert_eq!(segment, expected, "Index 100000 should use 32-bit segment");
    assert_eq!(segment.len(), 6, "32-bit segment should be 6 bytes");

    // Test index u32::MAX
    let segment = client.build_element_id_segment(u32::MAX);
    let expected = {
        let mut v = vec![0x2A, 0x00];
        v.extend_from_slice(&u32::MAX.to_le_bytes());
        v
    };
    assert_eq!(
        segment, expected,
        "Index u32::MAX should use 32-bit segment"
    );
    assert_eq!(segment.len(), 6, "32-bit segment should be 6 bytes");
}

#[tokio::test]
#[ignore]
async fn test_element_id_segment_boundaries() {
    // Test boundary conditions between segment types
    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Boundary: 255 (8-bit max)
    let segment_255 = client.build_element_id_segment(255);
    assert_eq!(segment_255[0], 0x28, "Index 255 should use 8-bit segment");

    // Boundary: 256 (16-bit start)
    let segment_256 = client.build_element_id_segment(256);
    assert_eq!(segment_256[0], 0x29, "Index 256 should use 16-bit segment");

    // Boundary: 65535 (16-bit max)
    let segment_65535 = client.build_element_id_segment(65535);
    assert_eq!(
        segment_65535[0], 0x29,
        "Index 65535 should use 16-bit segment"
    );

    // Boundary: 65536 (32-bit start)
    let segment_65536 = client.build_element_id_segment(65536);
    assert_eq!(
        segment_65536[0], 0x2A,
        "Index 65536 should use 32-bit segment"
    );
}

#[tokio::test]
#[ignore]
async fn test_build_read_array_request_single_element() {
    // Test building a read request for a single array element
    // Reference: 1756-PM020, Page 815-837

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Read element 5 from "MyArray"
    let request = client.build_read_array_request("MyArray", 5, 1);

    // Verify service code
    assert_eq!(request[0], 0x4C, "Service should be Read Tag (0x4C)");

    // Verify path size is present
    let path_size = request[1] as usize;
    assert!(path_size > 0, "Path size should be non-zero");

    // Verify path starts with symbol segment
    let path_start = 2;
    assert_eq!(
        request[path_start], 0x91,
        "Path should start with ANSI Extended Symbol (0x91)"
    );

    // Verify element count in request data
    let path_end = 2 + (path_size * 2);
    let element_count = u16::from_le_bytes([request[path_end], request[path_end + 1]]);
    assert_eq!(element_count, 1, "Element count should be 1");
}

#[tokio::test]
#[ignore]
async fn test_build_read_array_request_range() {
    // Test building a read request for a range of array elements
    // Reference: 1756-PM020, Page 840-851

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Read 5 elements starting at index 10 from "MyArray"
    let request = client.build_read_array_request("MyArray", 10, 5);

    // Verify service code
    assert_eq!(request[0], 0x4C, "Service should be Read Tag (0x4C)");

    // Verify element count in request data
    let path_size = request[1] as usize;
    let path_end = 2 + (path_size * 2);
    let element_count = u16::from_le_bytes([request[path_end], request[path_end + 1]]);
    assert_eq!(element_count, 5, "Element count should be 5");
}

#[tokio::test]
#[ignore]
async fn test_build_read_array_request_16bit_index() {
    // Test building a read request with 16-bit element index
    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Read element 1000 from "MyArray" (uses 16-bit segment)
    let request = client.build_read_array_request("MyArray", 1000, 1);

    // Verify service code
    assert_eq!(request[0], 0x4C, "Service should be Read Tag (0x4C)");

    // Verify path contains 16-bit element segment (0x29)
    let path_start = 2;
    let path_size = request[1] as usize;
    let path = &request[path_start..path_start + (path_size * 2)];

    // Find the element segment (should be 0x29 for 16-bit)
    let mut found_29 = false;
    for i in 0..path.len().saturating_sub(3) {
        if path[i] == 0x29 {
            found_29 = true;
            // Verify it's followed by 0x00 and then the index bytes
            assert_eq!(path[i + 1], 0x00, "16-bit segment should have padding byte");
            let index = u16::from_le_bytes([path[i + 2], path[i + 3]]);
            assert_eq!(index, 1000, "Index should be 1000");
            break;
        }
    }
    assert!(
        found_29,
        "Path should contain 16-bit element segment (0x29)"
    );
}

#[tokio::test]
#[ignore]
async fn test_build_read_array_request_32bit_index() {
    // Test building a read request with 32-bit element index
    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Read element 100000 from "MyArray" (uses 32-bit segment)
    let request = client.build_read_array_request("MyArray", 100000, 1);

    // Verify service code
    assert_eq!(request[0], 0x4C, "Service should be Read Tag (0x4C)");

    // Verify path contains 32-bit element segment (0x2A)
    let path_start = 2;
    let path_size = request[1] as usize;
    let path = &request[path_start..path_start + (path_size * 2)];

    // Find the element segment (should be 0x2A for 32-bit)
    let mut found_2a = false;
    for i in 0..path.len().saturating_sub(5) {
        if path[i] == 0x2A {
            found_2a = true;
            // Verify it's followed by 0x00 and then the index bytes
            assert_eq!(path[i + 1], 0x00, "32-bit segment should have padding byte");
            let index = u32::from_le_bytes([path[i + 2], path[i + 3], path[i + 4], path[i + 5]]);
            assert_eq!(index, 100000, "Index should be 100000");
            break;
        }
    }
    assert!(
        found_2a,
        "Path should contain 32-bit element segment (0x2A)"
    );
}

#[tokio::test]
#[ignore]
async fn test_build_base_tag_path() {
    // Test extracting base tag path from array notation
    // Reference: 1756-PM020, Page 894-909

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test simple array
    let path = client.build_base_tag_path("MyArray[5]");
    assert_eq!(
        path[0], 0x91,
        "Path should start with ANSI Extended Symbol (0x91)"
    );
    assert_eq!(path[1], 7, "Length should be 7 for 'MyArray'");
    assert_eq!(&path[2..9], b"MyArray", "Path should contain 'MyArray'");

    // Test simple tag (no array)
    let path = client.build_base_tag_path("MyTag");
    assert_eq!(
        path[0], 0x91,
        "Path should start with ANSI Extended Symbol (0x91)"
    );
    assert_eq!(path[1], 5, "Length should be 5 for 'MyTag'");
    assert_eq!(&path[2..7], b"MyTag", "Path should contain 'MyTag'");
}

#[tokio::test]
#[ignore]
async fn test_build_write_array_request_with_index() {
    // Test building a write request for array elements with element addressing
    // Reference: 1756-PM020, Page 855-867

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Write element 5 of DINT array "MyArray" with value 0x12345678
    let data_type = 0x00C4; // DINT
    let value = 0x12345678u32;
    let request = client
        .build_write_array_request_with_index("MyArray", 5, 1, data_type, &value.to_le_bytes())
        .unwrap();

    // Verify service code
    assert_eq!(request[0], 0x4D, "Service should be Write Tag (0x4D)");

    // Verify path size is present
    let path_size = request[1] as usize;
    assert!(path_size > 0, "Path size should be non-zero");

    // Verify data type and element count in request data
    let path_end = 2 + (path_size * 2);
    let req_data_type = u16::from_le_bytes([request[path_end], request[path_end + 1]]);
    assert_eq!(
        req_data_type, data_type,
        "Data type should be DINT (0x00C4)"
    );

    let element_count = u16::from_le_bytes([request[path_end + 2], request[path_end + 3]]);
    assert_eq!(element_count, 1, "Element count should be 1");

    // Verify value bytes
    let value_start = path_end + 4;
    let written_value = u32::from_le_bytes([
        request[value_start],
        request[value_start + 1],
        request[value_start + 2],
        request[value_start + 3],
    ]);
    assert_eq!(written_value, value, "Value should match");
}

#[tokio::test]
#[ignore]
async fn test_build_write_array_request_range() {
    // Test building a write request for a range of array elements
    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Write 3 DINT elements starting at index 10
    let data_type = 0x00C4; // DINT
    let values = vec![0x11111111u32, 0x22222222u32, 0x33333333u32];
    let mut data = Vec::new();
    for v in &values {
        data.extend_from_slice(&v.to_le_bytes());
    }

    let request = client
        .build_write_array_request_with_index("MyArray", 10, 3, data_type, &data)
        .unwrap();

    // Verify service code
    assert_eq!(request[0], 0x4D, "Service should be Write Tag (0x4D)");

    // Verify element count
    let path_size = request[1] as usize;
    let path_end = 2 + (path_size * 2);
    let element_count = u16::from_le_bytes([request[path_end + 2], request[path_end + 3]]);
    assert_eq!(element_count, 3, "Element count should be 3");

    // Verify data length (3 DINTs = 12 bytes)
    let data_start = path_end + 4;
    assert_eq!(
        request.len() - data_start,
        12,
        "Data should be 12 bytes (3 DINTs)"
    );
}

#[tokio::test]
#[ignore]
async fn test_path_word_alignment() {
    // Test that paths are properly word-aligned (even byte count)
    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test with odd-length tag name (should be padded)
    let path = client.build_base_tag_path("Tag");
    assert_eq!(
        path.len() % 2,
        0,
        "Path should be word-aligned (even length)"
    );

    // Test with even-length tag name
    let path = client.build_base_tag_path("MyTag");
    assert_eq!(
        path.len() % 2,
        0,
        "Path should be word-aligned (even length)"
    );

    // Test read request path alignment
    let request = client.build_read_array_request("Tag", 5, 1);
    let path_size = request[1] as usize;
    let path_length = path_size * 2;
    assert_eq!(path_length % 2, 0, "Path length should be word-aligned");
}

#[tokio::test]
#[ignore]
async fn test_element_id_segment_little_endian() {
    // Verify that multi-byte indices use little-endian byte order
    // Reference: 1756-PM020 specifies little-endian for all multi-byte values

    let client = match create_test_client().await {
        Some(c) => c,
        None => {
            println!("Skipping test - no PLC connection available");
            return;
        }
    };

    // Test 16-bit: 0x1234 should be [0x34, 0x12]
    let segment = client.build_element_id_segment(0x1234);
    assert_eq!(segment[0], 0x29, "Should use 16-bit segment");
    assert_eq!(segment[2], 0x34, "Low byte should be first (little-endian)");
    assert_eq!(
        segment[3], 0x12,
        "High byte should be second (little-endian)"
    );

    // Test 32-bit: 0x12345678 should be [0x78, 0x56, 0x34, 0x12]
    let segment = client.build_element_id_segment(0x12345678);
    assert_eq!(segment[0], 0x2A, "Should use 32-bit segment");
    assert_eq!(segment[2], 0x78, "Byte 0 should be 0x78 (little-endian)");
    assert_eq!(segment[3], 0x56, "Byte 1 should be 0x56 (little-endian)");
    assert_eq!(segment[4], 0x34, "Byte 2 should be 0x34 (little-endian)");
    assert_eq!(segment[5], 0x12, "Byte 3 should be 0x12 (little-endian)");
}
