// udt_enhanced_tests.rs - Comprehensive unit tests for enhanced UDT functionality
// ============================================================================
//
// This test module covers all the new UDT features implemented for CompactLogix L320ERS2:
// - Chunked reading for large UDTs
// - Individual UDT member access
// - Complete data type support including STRING
// - UDT writing functionality
// - Error recovery and retry logic

use rust_ethernet_ip::udt::{UdtMember, UserDefinedType};
use rust_ethernet_ip::{PlcValue, UdtData};

/// Mock EipClient for testing UDT functionality
struct MockEipClient {
    udt_data: Vec<u8>,
    should_fail_partial_transfer: bool,
    should_fail_chunked: bool,
}

impl MockEipClient {
    fn new() -> Self {
        Self {
            udt_data: create_test_udt_data(),
            should_fail_partial_transfer: false,
            should_fail_chunked: false,
        }
    }

    fn set_partial_transfer_failure(&mut self, fail: bool) {
        self.should_fail_partial_transfer = fail;
    }

    fn set_chunked_failure(&mut self, fail: bool) {
        self.should_fail_chunked = fail;
    }

    async fn read_tag(&mut self, tag_name: &str) -> Result<PlcValue, Box<dyn std::error::Error>> {
        if tag_name.ends_with("UDT")
            || tag_name == "Part_Data"
            || tag_name == "MyUDT"
            || tag_name == "TestUDT"
        {
            if self.should_fail_partial_transfer {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Protocol error: CIP Error 0x06: Partial transfer",
                )));
            }

            // Parse UDT data
            let udt = create_test_udt_definition();
            let result = udt.to_hash_map(&self.udt_data)?;
            let udt_data_value = UdtData::from_hash_map(&result, &udt, 0)?;
            Ok(PlcValue::Udt(udt_data_value))
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Tag not found",
            )))
        }
    }

    async fn read_udt_chunked(
        &mut self,
        tag_name: &str,
    ) -> Result<PlcValue, Box<dyn std::error::Error>> {
        if tag_name.ends_with("UDT")
            || tag_name == "Part_Data"
            || tag_name == "MyUDT"
            || tag_name == "TestUDT"
        {
            if self.should_fail_chunked {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "UDT too large even for smallest chunk size",
                )));
            }

            // Simulate chunked reading
            let mut all_data = Vec::new();
            let chunk_size = 200; // Smaller chunks for testing

            for chunk_start in (0..self.udt_data.len()).step_by(chunk_size) {
                let chunk_end = (chunk_start + chunk_size).min(self.udt_data.len());
                all_data.extend_from_slice(&self.udt_data[chunk_start..chunk_end]);
            }

            let udt = create_test_udt_definition();
            let result = udt.to_hash_map(&all_data)?;
            let udt_data_value = UdtData::from_hash_map(&result, &udt, 0)?;
            Ok(PlcValue::Udt(udt_data_value))
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Tag not found",
            )))
        }
    }

    async fn read_udt_member(
        &mut self,
        udt_name: &str,
        member_name: &str,
    ) -> Result<PlcValue, Box<dyn std::error::Error>> {
        if udt_name == "TestUDT" || udt_name == "MyUDT" || udt_name == "Part_Data" {
            let udt = create_test_udt_definition();
            // Check if member exists first
            if !udt.members.iter().any(|m| m.name == member_name) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Member not found",
                )));
            }
            udt.read_member(&self.udt_data, member_name).map_err(|e| {
                Box::new(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )) as Box<dyn std::error::Error>
            })
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "UDT not found",
            )))
        }
    }

    async fn write_udt_member(
        &mut self,
        udt_name: &str,
        member_name: &str,
        value: PlcValue,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if udt_name == "TestUDT" || udt_name == "MyUDT" || udt_name == "Part_Data" {
            let udt = create_test_udt_definition();
            // Check if member exists first
            if !udt.members.iter().any(|m| m.name == member_name) {
                return Err(Box::new(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Member not found",
                )));
            }
            udt.write_member(&mut self.udt_data, member_name, &value)
                .map_err(|e| {
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    )) as Box<dyn std::error::Error>
                })
        } else {
            Err(Box::new(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "UDT not found",
            )))
        }
    }
}

/// Creates test UDT data with known values
fn create_test_udt_data() -> Vec<u8> {
    let mut data = vec![0u8; 96]; // Total size for test UDT

    // BOOL member (offset 0)
    data[0] = 0xFF; // bool1 = true (PLC standard: 0xFF = true, 0x00 = false)

    // INT member (offset 2)
    data[2..4].copy_from_slice(&1234i16.to_le_bytes()); // int1 = 1234

    // DINT member (offset 4)
    data[4..8].copy_from_slice(&123456i32.to_le_bytes()); // dint1 = 123456

    // REAL member (offset 8)
    data[8..12].copy_from_slice(&3.14159f32.to_le_bytes()); // real1 = 3.14159

    // STRING member (offset 12)
    let string_value = "Hello UDT!";
    let length = string_value.len() as u16;
    data[12..14].copy_from_slice(&length.to_le_bytes());
    data[14..14 + string_value.len()].copy_from_slice(string_value.as_bytes());

    data
}

/// Creates a generic test UDT definition
fn create_test_udt_definition() -> UserDefinedType {
    let mut udt = UserDefinedType::new("TestUDT".to_string());

    // Add various data types at different offsets
    udt.add_member(UdtMember {
        name: "bool1".to_string(),
        data_type: 0x00C1,
        offset: 0,
        size: 1,
    });
    udt.add_member(UdtMember {
        name: "int1".to_string(),
        data_type: 0x00C2,
        offset: 2,
        size: 2,
    });
    udt.add_member(UdtMember {
        name: "dint1".to_string(),
        data_type: 0x00C4,
        offset: 4,
        size: 4,
    });
    udt.add_member(UdtMember {
        name: "real1".to_string(),
        data_type: 0x00CA,
        offset: 8,
        size: 4,
    });
    udt.add_member(UdtMember {
        name: "string1".to_string(),
        data_type: 0x00CE,
        offset: 12,
        size: 84,
    });

    udt
}

#[tokio::test]
async fn test_udt_data_type_parsing() {
    let udt = create_test_udt_definition();
    let test_data = create_test_udt_data();

    // Test BOOL parsing
    let bool_member = UdtMember {
        name: "test_bool".to_string(),
        data_type: 0x00C1,
        offset: 0,
        size: 1,
    };
    let bool_value = udt
        .parse_member_value(&bool_member, &test_data[0..1])
        .unwrap();
    assert_eq!(bool_value, PlcValue::Bool(true));

    // Test REAL parsing
    let real_member = UdtMember {
        name: "test_real".to_string(),
        data_type: 0x00CA,
        offset: 0,
        size: 4,
    };
    let real_value = udt
        .parse_member_value(&real_member, &test_data[8..12])
        .unwrap();
    assert_eq!(real_value, PlcValue::Real(3.14159));

    // Test STRING parsing
    let string_member = UdtMember {
        name: "test_string".to_string(),
        data_type: 0x00CE,
        offset: 0,
        size: 84,
    };
    let string_value = udt
        .parse_member_value(&string_member, &test_data[12..96])
        .unwrap();
    assert_eq!(string_value, PlcValue::String("Hello UDT!".to_string()));
}

#[tokio::test]
async fn test_udt_data_type_serialization() {
    let udt = create_test_udt_definition();

    // Test BOOL serialization
    let bool_member = UdtMember {
        name: "test_bool".to_string(),
        data_type: 0x00C1,
        offset: 0,
        size: 1,
    };
    let bool_data = udt
        .serialize_member_value(&bool_member, &PlcValue::Bool(true))
        .unwrap();
    assert_eq!(bool_data, vec![0xFF]); // PLC standard: 0xFF = true, 0x00 = false

    // Test REAL serialization
    let real_member = UdtMember {
        name: "test_real".to_string(),
        data_type: 0x00CA,
        offset: 0,
        size: 4,
    };
    let real_data = udt
        .serialize_member_value(&real_member, &PlcValue::Real(3.14))
        .unwrap();
    assert_eq!(real_data, 3.14f32.to_le_bytes().to_vec());

    // Test STRING serialization
    let string_member = UdtMember {
        name: "test_string".to_string(),
        data_type: 0x00CE,
        offset: 0,
        size: 84,
    };
    let string_data = udt
        .serialize_member_value(&string_member, &PlcValue::String("Hello".to_string()))
        .unwrap();
    assert_eq!(string_data.len(), 8); // 2 bytes length + 5 bytes data + 1 byte padding
    assert_eq!(&string_data[0..2], &(5u16).to_le_bytes());
    assert_eq!(&string_data[2..7], b"Hello");
}

#[tokio::test]
async fn test_udt_member_access() {
    let udt = create_test_udt_definition();
    let test_data = create_test_udt_data();

    // Test reading BOOL member
    let bool_value = udt.read_member(&test_data, "bool1").unwrap();
    assert_eq!(bool_value, PlcValue::Bool(true));

    // Test reading INT member
    let int_value = udt.read_member(&test_data, "int1").unwrap();
    assert_eq!(int_value, PlcValue::Int(1234));

    // Test reading DINT member
    let dint_value = udt.read_member(&test_data, "dint1").unwrap();
    assert_eq!(dint_value, PlcValue::Dint(123456));

    // Test reading REAL member
    let real_value = udt.read_member(&test_data, "real1").unwrap();
    assert_eq!(real_value, PlcValue::Real(3.14159));

    // Test reading STRING member
    let string_value = udt.read_member(&test_data, "string1").unwrap();
    assert_eq!(string_value, PlcValue::String("Hello UDT!".to_string()));

    // Test reading non-existent member
    let result = udt.read_member(&test_data, "NonExistentMember");
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_member_writing() {
    let udt = create_test_udt_definition();
    let mut test_data = create_test_udt_data();

    // Test writing BOOL member
    udt.write_member(&mut test_data, "bool1", &PlcValue::Bool(false))
        .unwrap();
    let bool_value = udt.read_member(&test_data, "bool1").unwrap();
    assert_eq!(bool_value, PlcValue::Bool(false));

    // Test writing REAL member
    udt.write_member(&mut test_data, "real1", &PlcValue::Real(99.9))
        .unwrap();
    let real_value = udt.read_member(&test_data, "real1").unwrap();
    assert_eq!(real_value, PlcValue::Real(99.9));

    // Test writing STRING member
    udt.write_member(
        &mut test_data,
        "string1",
        &PlcValue::String("NEW123".to_string()),
    )
    .unwrap();
    let string_value = udt.read_member(&test_data, "string1").unwrap();
    assert_eq!(string_value, PlcValue::String("NEW123".to_string()));

    // Test writing to non-existent member
    let result = udt.write_member(&mut test_data, "NonExistentMember", &PlcValue::Bool(true));
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_hash_map_conversion() {
    let udt = create_test_udt_definition();
    let test_data = create_test_udt_data();

    // Test converting UDT data to HashMap
    let hash_map = udt.to_hash_map(&test_data).unwrap();

    // Verify some key members
    assert_eq!(hash_map.get("bool1"), Some(&PlcValue::Bool(true)));
    assert_eq!(hash_map.get("int1"), Some(&PlcValue::Int(1234)));
    assert_eq!(hash_map.get("real1"), Some(&PlcValue::Real(3.14159)));
    assert_eq!(
        hash_map.get("string1"),
        Some(&PlcValue::String("Hello UDT!".to_string()))
    );

    // Test converting HashMap back to UDT data
    let mut new_data = vec![0u8; udt.size as usize];
    for (member_name, value) in &hash_map {
        udt.write_member(&mut new_data, member_name, value).unwrap();
    }

    // Verify the data matches
    assert_eq!(test_data, new_data);
}

#[tokio::test]
async fn test_udt_member_metadata() {
    let udt = create_test_udt_definition();

    // Test getting member size
    assert_eq!(udt.get_member_size("bool1"), Some(1));
    assert_eq!(udt.get_member_size("real1"), Some(4));
    assert_eq!(udt.get_member_size("string1"), Some(84));
    assert_eq!(udt.get_member_size("NonExistentMember"), None);

    // Test getting member data type
    assert_eq!(udt.get_member_data_type("bool1"), Some(0x00C1));
    assert_eq!(udt.get_member_data_type("real1"), Some(0x00CA));
    assert_eq!(udt.get_member_data_type("string1"), Some(0x00CE));
    assert_eq!(udt.get_member_data_type("NonExistentMember"), None);
}

#[tokio::test]
async fn test_udt_chunked_reading() {
    let mut client = MockEipClient::new();

    // Test successful chunked reading
    let result = client.read_udt_chunked("Part_Data").await;
    assert!(result.is_ok());

    if let Ok(PlcValue::Udt(udt_data)) = result {
        // UdtData now contains raw bytes, not parsed members
        assert!(udt_data.symbol_id >= 0);
        assert!(!udt_data.data.is_empty());
    }

    // Test chunked reading failure
    client.set_chunked_failure(true);
    let result = client.read_udt_chunked("Part_Data").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_member_reading() {
    let mut client = MockEipClient::new();

    // Test reading individual UDT members
    let bool_value = client.read_udt_member("TestUDT", "bool1").await.unwrap();
    assert_eq!(bool_value, PlcValue::Bool(true));

    let int_value = client.read_udt_member("TestUDT", "int1").await.unwrap();
    assert_eq!(int_value, PlcValue::Int(1234));

    let real_value = client.read_udt_member("TestUDT", "real1").await.unwrap();
    assert_eq!(real_value, PlcValue::Real(3.14159));

    let string_value = client.read_udt_member("TestUDT", "string1").await.unwrap();
    assert_eq!(string_value, PlcValue::String("Hello UDT!".to_string()));

    // Test reading non-existent member
    let result = client.read_udt_member("TestUDT", "NonExistentMember").await;
    assert!(result.is_err());

    // Test reading from non-existent UDT
    let result = client.read_udt_member("NonExistentUDT", "bool1").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_member_writing_client() {
    let mut client = MockEipClient::new();

    // Test writing individual UDT members
    client
        .write_udt_member("TestUDT", "bool1", PlcValue::Bool(false))
        .await
        .unwrap();
    let bool_value = client.read_udt_member("TestUDT", "bool1").await.unwrap();
    assert_eq!(bool_value, PlcValue::Bool(false));

    client
        .write_udt_member("TestUDT", "real1", PlcValue::Real(99.9))
        .await
        .unwrap();
    let real_value = client.read_udt_member("TestUDT", "real1").await.unwrap();
    assert_eq!(real_value, PlcValue::Real(99.9));

    client
        .write_udt_member("TestUDT", "string1", PlcValue::String("NEW123".to_string()))
        .await
        .unwrap();
    let string_value = client.read_udt_member("TestUDT", "string1").await.unwrap();
    assert_eq!(string_value, PlcValue::String("NEW123".to_string()));

    // Test writing to non-existent member
    let result = client
        .write_udt_member("TestUDT", "NonExistentMember", PlcValue::Bool(true))
        .await;
    assert!(result.is_err());

    // Test writing to non-existent UDT
    let result = client
        .write_udt_member("NonExistentUDT", "bool1", PlcValue::Bool(true))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_error_handling() {
    let udt = create_test_udt_definition();

    // Test data type mismatch
    let bool_member = UdtMember {
        name: "test_bool".to_string(),
        data_type: 0x00C1,
        offset: 0,
        size: 1,
    };
    let result = udt.serialize_member_value(&bool_member, &PlcValue::Real(3.14));
    assert!(result.is_err());

    // Test insufficient data
    let real_member = UdtMember {
        name: "test_real".to_string(),
        data_type: 0x00CA,
        offset: 0,
        size: 4,
    };
    let result = udt.parse_member_value(&real_member, &[1, 2]); // Only 2 bytes, need 4
    assert!(result.is_err());

    // Test STRING data too short
    let string_member = UdtMember {
        name: "test_string".to_string(),
        data_type: 0x00CE,
        offset: 0,
        size: 84,
    };
    let result = udt.parse_member_value(&string_member, &[1]); // Only 1 byte, need at least 2
    assert!(result.is_err());
}

#[tokio::test]
async fn test_udt_comprehensive_workflow() {
    let mut client = MockEipClient::new();

    // 1. Read entire UDT
    let udt_value = client.read_tag("Part_Data").await.unwrap();
    if let PlcValue::Udt(udt_data) = udt_value {
        // UdtData now contains raw bytes, not parsed members
        assert!(udt_data.symbol_id >= 0);
        assert!(!udt_data.data.is_empty());
    }

    // 2. Read individual members
    let bool_value = client.read_udt_member("TestUDT", "bool1").await.unwrap();
    assert_eq!(bool_value, PlcValue::Bool(true));

    let int_value = client.read_udt_member("TestUDT", "int1").await.unwrap();
    assert_eq!(int_value, PlcValue::Int(1234));

    // 3. Write individual members
    client
        .write_udt_member("TestUDT", "bool1", PlcValue::Bool(false))
        .await
        .unwrap();
    client
        .write_udt_member("TestUDT", "real1", PlcValue::Real(99.9))
        .await
        .unwrap();
    client
        .write_udt_member(
            "TestUDT",
            "string1",
            PlcValue::String("UPDATED123".to_string()),
        )
        .await
        .unwrap();

    // 4. Verify changes
    let bool_value = client.read_udt_member("TestUDT", "bool1").await.unwrap();
    assert_eq!(bool_value, PlcValue::Bool(false));

    let real_value = client.read_udt_member("TestUDT", "real1").await.unwrap();
    assert_eq!(real_value, PlcValue::Real(99.9));

    let string_value = client.read_udt_member("TestUDT", "string1").await.unwrap();
    assert_eq!(string_value, PlcValue::String("UPDATED123".to_string()));
}

#[tokio::test]
async fn test_udt_performance() {
    let udt = create_test_udt_definition();
    let test_data = create_test_udt_data();

    // Test parsing performance
    let start = std::time::Instant::now();
    for _ in 0..1000 {
        let _ = udt.to_hash_map(&test_data).unwrap();
    }
    let duration = start.elapsed();

    // Should complete 1000 parses in less than 100ms
    assert!(
        duration.as_millis() < 100,
        "UDT parsing too slow: {:?}",
        duration
    );

    // Test member access performance
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = udt.read_member(&test_data, "bool1").unwrap();
    }
    let duration = start.elapsed();

    // Should complete 10000 member reads in less than 50ms
    assert!(
        duration.as_millis() < 50,
        "UDT member access too slow: {:?}",
        duration
    );
}

#[tokio::test]
async fn test_udt_edge_cases() {
    let udt = create_test_udt_definition();

    // Test empty UDT data
    let empty_data = vec![0u8; 0];
    let result = udt.to_hash_map(&empty_data);
    assert!(result.is_err());

    // Test partial UDT data
    let partial_data = vec![0u8; 10]; // Only enough for BOOL members
    let result = udt.to_hash_map(&partial_data);
    assert!(result.is_ok()); // Should succeed but only parse available members

    // Test STRING with maximum length
    let max_string = "X".repeat(82);
    let string_member = UdtMember {
        name: "test_string".to_string(),
        data_type: 0x00CE,
        offset: 0,
        size: 84,
    };
    let string_data = udt
        .serialize_member_value(&string_member, &PlcValue::String(max_string))
        .unwrap();
    assert_eq!(string_data.len(), 84); // Should be exactly 84 bytes (2 length + 82 data)

    // Test STRING with length exceeding maximum
    let too_long_string = "X".repeat(100);
    let string_data = udt
        .serialize_member_value(&string_member, &PlcValue::String(too_long_string))
        .unwrap();
    assert_eq!(string_data.len(), 84); // Should be truncated to 84 bytes
    let length = u16::from_le_bytes([string_data[0], string_data[1]]);
    assert_eq!(length, 82); // Should be truncated to max length
}
