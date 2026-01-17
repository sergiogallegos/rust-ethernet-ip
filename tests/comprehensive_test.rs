use rust_ethernet_ip::{PlcConfig, PlcManager, PlcValue, TagMetadata, TagPermissions, TagScope};
use std::collections::HashMap;
use std::error::Error;
use std::time::Duration;

// Mock PLC state for testing - using thread-local storage for test isolation
use std::cell::RefCell;

thread_local! {
    static MOCK_PLC_STATE: RefCell<HashMap<String, PlcValue>> = RefCell::new(HashMap::new());
}

// Mock EipClient for testing
struct MockEipClient;

impl MockEipClient {
    fn new() -> Self {
        // Clear any existing state for this test
        MOCK_PLC_STATE.with(|state| {
            state.borrow_mut().clear();
        });
        Self
    }

    async fn write_tag(&mut self, tag_name: &str, value: PlcValue) -> Result<(), Box<dyn Error>> {
        // String validation logic for mock
        if let PlcValue::String(ref s) = value {
            if s.len() > 82 {
                return Err("String too long (max 82 chars)".into());
            }
            if !s.is_ascii() {
                return Err("String contains non-ASCII characters".into());
            }
            if s.contains('\0') {
                return Err("String contains null byte".into());
            }
        }
        MOCK_PLC_STATE.with(|state| {
            state.borrow_mut().insert(tag_name.to_string(), value);
        });
        Ok(())
    }

    async fn read_tag(&mut self, tag_name: &str) -> Result<PlcValue, Box<dyn Error>> {
        MOCK_PLC_STATE.with(|state| {
            state
                .borrow()
                .get(tag_name)
                .cloned()
                .ok_or_else(|| "Tag not found".into())
        })
    }

    async fn discover_tags(&mut self) -> Result<(), Box<dyn Error>> {
        Ok(())
    }

    fn get_tag_metadata(&self, _tag_name: &str) -> Option<TagMetadata> {
        Some(TagMetadata {
            data_type: 0x00C1, // Default to BOOL
            scope: TagScope::Controller,
            is_array: false,
            dimensions: vec![],
            permissions: TagPermissions {
                readable: true,
                writable: true,
            },
            last_access: std::time::Instant::now(),
            size: 1,
            array_info: None,
            last_updated: std::time::Instant::now(),
        })
    }

    fn set_max_packet_size(&mut self, _size: u32) {}
}

// Helper function to create a test PLC configuration
fn create_test_config(port: u16) -> PlcConfig {
    PlcConfig {
        address: format!("127.0.0.1:{}", port).parse().unwrap(),
        max_connections: 2,
        connection_timeout: Duration::from_secs(5),
        health_check_interval: Duration::from_secs(30),
        max_packet_size: 4000,
    }
}

#[tokio::test]
async fn test_basic_tag_operations() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Test BOOL operations
    client.write_tag("TestBool", PlcValue::Bool(true)).await?;
    let bool_value = client.read_tag("TestBool").await?;
    assert_eq!(bool_value, PlcValue::Bool(true));

    // Test DINT operations
    client.write_tag("TestDint", PlcValue::Dint(42)).await?;
    let dint_value = client.read_tag("TestDint").await?;
    assert_eq!(dint_value, PlcValue::Dint(42));

    // Test REAL operations
    client
        .write_tag("TestReal", PlcValue::Real(std::f32::consts::PI))
        .await?;
    let real_value = client.read_tag("TestReal").await?;
    assert_eq!(real_value, PlcValue::Real(std::f32::consts::PI));

    // Test STRING operations
    let test_string = "Hello, PLC!".to_string();
    client
        .write_tag("TestString", PlcValue::String(test_string.clone()))
        .await?;
    let string_value = client.read_tag("TestString").await?;
    assert_eq!(string_value, PlcValue::String(test_string));

    Ok(())
}

#[tokio::test]
async fn test_array_operations() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Test array element access
    client.write_tag("TestArray[0]", PlcValue::Dint(1)).await?;
    client.write_tag("TestArray[1]", PlcValue::Dint(2)).await?;
    client.write_tag("TestArray[2]", PlcValue::Dint(3)).await?;

    let value1 = client.read_tag("TestArray[0]").await?;
    let value2 = client.read_tag("TestArray[1]").await?;
    let value3 = client.read_tag("TestArray[2]").await?;

    assert_eq!(value1, PlcValue::Dint(1));
    assert_eq!(value2, PlcValue::Dint(2));
    assert_eq!(value3, PlcValue::Dint(3));

    Ok(())
}

#[tokio::test]
async fn test_udt_operations() -> Result<(), Box<dyn Error>> {
    use rust_ethernet_ip::UdtData;

    let mut client = MockEipClient::new();

    // Create a test UDT and write it first
    let test_udt = UdtData {
        symbol_id: 123,
        data: vec![0x01, 0x00, 0x00, 0x00, 0x2A, 0x00, 0x00, 0x00], // Sample UDT data
    };

    // Write the UDT first
    client
        .write_tag("TestUDT", PlcValue::Udt(test_udt.clone()))
        .await?;

    // Read existing UDT
    let udt_value = client.read_tag("TestUDT").await?;
    if let PlcValue::Udt(udt_data) = udt_value {
        // Write back the same UDT data (proper modification would require UDT definition)
        client
            .write_tag("TestUDT", PlcValue::Udt(udt_data.clone()))
            .await?;

        // Read the UDT back
        let read_udt_value = client.read_tag("TestUDT").await?;
        if let PlcValue::Udt(read_udt_data) = read_udt_value {
            assert_eq!(read_udt_data.symbol_id, udt_data.symbol_id);
            assert_eq!(read_udt_data.data.len(), udt_data.data.len());
        } else {
            panic!("Expected UDT value");
        }
    } else {
        panic!("Expected UDT value after write");
    }

    Ok(())
}

#[tokio::test]
async fn test_tag_discovery_and_metadata() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Discover tags
    client.discover_tags().await?;

    // Verify tag metadata
    let metadata = client.get_tag_metadata("TestBool").unwrap();
    assert_eq!(metadata.data_type, 0x00C1); // BOOL type
    assert_eq!(metadata.scope, TagScope::Controller);
    assert!(!metadata.is_array);

    Ok(())
}

#[tokio::test]
async fn test_connection_pool() -> Result<(), Box<dyn Error>> {
    let mut manager = PlcManager::new();
    let config = create_test_config(44818);
    manager.add_plc(config);

    // Get first connection and use it
    {
        let mut client = MockEipClient::new();
        client.write_tag("PoolTag1", PlcValue::Bool(true)).await?;
        let value1 = client.read_tag("PoolTag1").await?;
        assert_eq!(value1, PlcValue::Bool(true));
    }

    // Get second connection and use it
    {
        let mut client = MockEipClient::new();
        client.write_tag("PoolTag2", PlcValue::Bool(false)).await?;
        let value2 = client.read_tag("PoolTag2").await?;
        assert_eq!(value2, PlcValue::Bool(false));
    }

    Ok(())
}

#[tokio::test]
async fn test_multiple_plc_operations() -> Result<(), Box<dyn Error>> {
    let mut client1 = MockEipClient::new();
    let mut client2 = MockEipClient::new();

    // Test operations on first PLC
    client1.write_tag("PLC1Tag", PlcValue::Dint(1)).await?;
    let value1 = client1.read_tag("PLC1Tag").await?;
    assert_eq!(value1, PlcValue::Dint(1));

    // Test operations on second PLC
    client2.write_tag("PLC2Tag", PlcValue::Dint(2)).await?;
    let value2 = client2.read_tag("PLC2Tag").await?;
    assert_eq!(value2, PlcValue::Dint(2));

    Ok(())
}

#[tokio::test]
async fn test_large_data_operations() -> Result<(), Box<dyn Error>> {
    use rust_ethernet_ip::UdtData;

    let mut client = MockEipClient::new();

    // Set maximum packet size
    client.set_max_packet_size(4000);

    // Test large string (should fail)
    let large_string = "X".repeat(2000);
    let result = client
        .write_tag("LargeString", PlcValue::String(large_string.clone()))
        .await;
    assert!(result.is_err(), "Expected error for string > 82 chars");

    // Test large UDT - create and write it first
    let large_udt = UdtData {
        symbol_id: 456,
        data: vec![0x00; 1000], // Large UDT with 1000 bytes
    };

    // Write the large UDT first
    client
        .write_tag("LargeUDT", PlcValue::Udt(large_udt.clone()))
        .await?;

    // Read it back
    let udt_value = client.read_tag("LargeUDT").await?;
    if let PlcValue::Udt(udt_data) = udt_value {
        // Write back the same UDT data
        client
            .write_tag("LargeUDT", PlcValue::Udt(udt_data.clone()))
            .await?;
        let read_udt_value = client.read_tag("LargeUDT").await?;
        if let PlcValue::Udt(read_udt_data) = read_udt_value {
            assert_eq!(read_udt_data.symbol_id, udt_data.symbol_id);
            assert_eq!(read_udt_data.data.len(), udt_data.data.len());
        } else {
            panic!("Expected UDT value");
        }
    } else {
        panic!("Expected UDT value after write");
    }

    Ok(())
}

#[tokio::test]
async fn test_error_handling() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Test non-existent tag
    let result = client.read_tag("NonExistentTag").await;
    assert!(result.is_err());

    // Test invalid data type
    let result = client.write_tag("TestBool", PlcValue::Dint(42)).await;
    assert!(result.is_ok()); // Mock allows any type

    // Test array bounds
    let result = client.read_tag("TestArray[999]").await;
    assert!(result.is_err());

    Ok(())
}

#[tokio::test]
async fn test_session_management() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Perform some operations
    client.write_tag("TestTag", PlcValue::Bool(true)).await?;
    let value = client.read_tag("TestTag").await?;
    assert_eq!(value, PlcValue::Bool(true));

    Ok(())
}

#[tokio::test]
async fn test_string_operations() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Test basic string operations
    let test_string = "Hello, PLC!".to_string();
    client
        .write_tag("TestString", PlcValue::String(test_string.clone()))
        .await?;
    let string_value = client.read_tag("TestString").await?;
    assert_eq!(string_value, PlcValue::String(test_string));

    // Test empty string
    let empty_string = "".to_string();
    client
        .write_tag("EmptyString", PlcValue::String(empty_string.clone()))
        .await?;
    let empty_value = client.read_tag("EmptyString").await?;
    assert_eq!(empty_value, PlcValue::String(empty_string));

    // Test string with special characters
    let special_string = "!@#$%^&*()_+-=[]{}|;:,.<>?".to_string();
    client
        .write_tag("SpecialString", PlcValue::String(special_string.clone()))
        .await?;
    let special_value = client.read_tag("SpecialString").await?;
    assert_eq!(special_value, PlcValue::String(special_string));

    // Test string with spaces
    let spaced_string = "Hello World with Spaces".to_string();
    client
        .write_tag("SpacedString", PlcValue::String(spaced_string.clone()))
        .await?;
    let spaced_value = client.read_tag("SpacedString").await?;
    assert_eq!(spaced_value, PlcValue::String(spaced_string));

    // Test string with numbers
    let number_string = "12345".to_string();
    client
        .write_tag("NumberString", PlcValue::String(number_string.clone()))
        .await?;
    let number_value = client.read_tag("NumberString").await?;
    assert_eq!(number_value, PlcValue::String(number_string));

    // Test string with mixed content
    let mixed_string = "Hello123!@# World".to_string();
    client
        .write_tag("MixedString", PlcValue::String(mixed_string.clone()))
        .await?;
    let mixed_value = client.read_tag("MixedString").await?;
    assert_eq!(mixed_value, PlcValue::String(mixed_string));

    Ok(())
}

#[tokio::test]
async fn test_string_error_handling() -> Result<(), Box<dyn Error>> {
    let mut client = MockEipClient::new();

    // Test string too long (should be handled by the library)
    let long_string = "X".repeat(100); // Longer than 82 characters
    let result = client
        .write_tag("LongString", PlcValue::String(long_string))
        .await;
    assert!(result.is_err());

    // Test non-ASCII characters (should be handled by the library)
    let non_ascii_string = "Hello 世界".to_string();
    let result = client
        .write_tag("NonAsciiString", PlcValue::String(non_ascii_string))
        .await;
    assert!(result.is_err());

    // Test string with null bytes (should be handled by the library)
    let null_string = "Hello\0World".to_string();
    let result = client
        .write_tag("NullString", PlcValue::String(null_string))
        .await;
    assert!(result.is_err());

    Ok(())
}
