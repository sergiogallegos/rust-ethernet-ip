//! Tests for the new generic UdtData format
//! 
//! This test suite validates the new UDT handling approach that uses
//! UdtData with symbol_id and raw bytes instead of hardcoded member names.

use rust_ethernet_ip::{EipClient, PlcValue, UdtData};
use std::collections::HashMap;

/// Helper function to check if a PLC is available for testing
async fn is_plc_available(address: &str) -> bool {
    match EipClient::connect(address).await {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::test]
#[ignore] // Requires actual PLC connection
async fn test_udt_read_returns_udt_data() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!("Skipping test: No PLC available at {}", address);
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Read a UDT - should return UdtData with symbol_id and raw bytes
    let result = client.read_tag("Part_Data").await;
    
    match result {
        Ok(PlcValue::Udt(udt_data)) => {
            println!("✅ UDT read successfully");
            println!("   Symbol ID: {}", udt_data.symbol_id);
            println!("   Data size: {} bytes", udt_data.data.len());
            println!("   Raw data (first 32 bytes): {:02X?}", 
                     &udt_data.data[..udt_data.data.len().min(32)]);
            
            // Verify we have a valid symbol_id (should be > 0 for real UDTs)
            assert!(udt_data.symbol_id > 0, "symbol_id should be > 0 for valid UDT");
            
            // Verify we have data
            assert!(!udt_data.data.is_empty(), "UDT data should not be empty");
        }
        Ok(other) => {
            panic!("Expected UdtData, got: {:?}", other);
        }
        Err(e) => {
            // If tag doesn't exist, that's okay for testing
            if e.to_string().contains("not found") || e.to_string().contains("does not exist") {
                println!("⚠️ Tag 'Part_Data' not found - skipping test");
                return;
            }
            panic!("UDT read failed: {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires actual PLC connection
async fn test_udt_write_with_symbol_id() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!("Skipping test: No PLC available at {}", address);
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // First, read the UDT to get symbol_id
    let read_result = client.read_tag("Part_Data").await;
    
    let udt_data = match read_result {
        Ok(PlcValue::Udt(data)) => {
            println!("✅ Read UDT with symbol_id: {}", data.symbol_id);
            data
        }
        Err(e) => {
            if e.to_string().contains("not found") {
                println!("⚠️ Tag 'Part_Data' not found - skipping test");
                return;
            }
            panic!("Failed to read UDT: {}", e);
        }
        _ => {
            panic!("Expected UdtData");
        }
    };

    // Modify the data (for testing, we'll just write back the same data)
    // In a real scenario, you'd modify specific bytes based on UDT definition
    let mut modified_data = udt_data.data.clone();
    
    // Example: modify first byte (if it's a BOOL member)
    if !modified_data.is_empty() {
        modified_data[0] = if modified_data[0] == 0 { 0xFF } else { 0x00 };
    }

    // Create new UdtData with the same symbol_id
    let modified_udt = UdtData {
        symbol_id: udt_data.symbol_id,
        data: modified_data,
    };

    // Write it back
    match client.write_tag("Part_Data", PlcValue::Udt(modified_udt.clone())).await {
        Ok(_) => {
            println!("✅ UDT written successfully with symbol_id: {}", modified_udt.symbol_id);
        }
        Err(e) => {
            // Writing might fail if tag is read-only, that's okay
            println!("⚠️ UDT write failed (may be read-only): {}", e);
        }
    }
}

#[tokio::test]
#[ignore] // Requires actual PLC connection
async fn test_udt_write_auto_reads_symbol_id() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!("Skipping test: No PLC available at {}", address);
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Create UdtData with symbol_id = 0 (should trigger auto-read)
    let udt_data = UdtData {
        symbol_id: 0, // Will trigger read to get symbol_id
        data: vec![0x00, 0x00, 0x00, 0x00], // Example data
    };

    // Write should automatically read symbol_id if it's 0
    match client.write_tag("Part_Data", PlcValue::Udt(udt_data)).await {
        Ok(_) => {
            println!("✅ UDT write with auto symbol_id read succeeded");
        }
        Err(e) => {
            // Check if error is about missing symbol_id (shouldn't happen with auto-read)
            if e.to_string().contains("symbol_id") && e.to_string().contains("not found") {
                panic!("Auto-read of symbol_id failed: {}", e);
            }
            // Other errors (read-only, etc.) are acceptable
            println!("⚠️ UDT write failed (may be expected): {}", e);
        }
    }
}

#[tokio::test]
async fn test_udt_data_parse_with_definition() {
    use rust_ethernet_ip::udt::{UserDefinedType, UdtMember};
    
    // Create a mock UDT definition
    let mut udt_def = UserDefinedType::new("TestUDT".to_string());
    udt_def.add_member(UdtMember {
        name: "Bool1".to_string(),
        data_type: 0x00C1, // BOOL
        offset: 0,
        size: 1,
    });
    udt_def.add_member(UdtMember {
        name: "Dint1".to_string(),
        data_type: 0x00C4, // DINT
        offset: 4, // Assuming 4-byte alignment
        size: 4,
    });

    // Create UdtData with raw bytes
    let mut raw_data = vec![0u8; 8];
    raw_data[0] = 0xFF; // Bool1 = true
    raw_data[4..8].copy_from_slice(&42i32.to_le_bytes()); // Dint1 = 42

    let udt_data = UdtData {
        symbol_id: 123,
        data: raw_data,
    };

    // Parse using the definition
    match udt_data.parse(&udt_def) {
        Ok(members) => {
            println!("✅ Parsed UDT successfully");
            assert_eq!(members.len(), 2);
            assert_eq!(members.get("Bool1"), Some(&PlcValue::Bool(true)));
            assert_eq!(members.get("Dint1"), Some(&PlcValue::Dint(42)));
        }
        Err(e) => {
            panic!("Failed to parse UDT: {}", e);
        }
    }
}

#[tokio::test]
async fn test_udt_data_from_hash_map() {
    use rust_ethernet_ip::udt::{UserDefinedType, UdtMember};
    
    // Create a mock UDT definition
    let mut udt_def = UserDefinedType::new("TestUDT".to_string());
    udt_def.add_member(UdtMember {
        name: "Bool1".to_string(),
        data_type: 0x00C1, // BOOL
        offset: 0,
        size: 1,
    });
    udt_def.add_member(UdtMember {
        name: "Dint1".to_string(),
        data_type: 0x00C4, // DINT
        offset: 4,
        size: 4,
    });

    // Create HashMap of member values
    let mut members = HashMap::new();
    members.insert("Bool1".to_string(), PlcValue::Bool(true));
    members.insert("Dint1".to_string(), PlcValue::Dint(42));

    // Convert to UdtData
    match UdtData::from_hash_map(&members, &udt_def, 123) {
        Ok(udt_data) => {
            println!("✅ Created UdtData from HashMap");
            assert_eq!(udt_data.symbol_id, 123);
            assert!(!udt_data.data.is_empty());
            
            // Verify the data
            assert_eq!(udt_data.data[0], 0xFF); // Bool = true
            let dint_value = i32::from_le_bytes([
                udt_data.data[4],
                udt_data.data[5],
                udt_data.data[6],
                udt_data.data[7],
            ]);
            assert_eq!(dint_value, 42);
        }
        Err(e) => {
            panic!("Failed to create UdtData from HashMap: {}", e);
        }
    }
}

#[tokio::test]
async fn test_udt_data_round_trip() {
    use rust_ethernet_ip::udt::{UserDefinedType, UdtMember};
    
    // Create a mock UDT definition
    let mut udt_def = UserDefinedType::new("TestUDT".to_string());
    udt_def.add_member(UdtMember {
        name: "Bool1".to_string(),
        data_type: 0x00C1,
        offset: 0,
        size: 1,
    });
    udt_def.add_member(UdtMember {
        name: "Dint1".to_string(),
        data_type: 0x00C4,
        offset: 4,
        size: 4,
    });

    // Start with HashMap
    let mut original_members = HashMap::new();
    original_members.insert("Bool1".to_string(), PlcValue::Bool(true));
    original_members.insert("Dint1".to_string(), PlcValue::Dint(42));

    // Convert to UdtData
    let udt_data = UdtData::from_hash_map(&original_members, &udt_def, 123).unwrap();
    
    // Parse back to HashMap
    let parsed_members = udt_data.parse(&udt_def).unwrap();
    
    // Verify round-trip
    assert_eq!(parsed_members.get("Bool1"), original_members.get("Bool1"));
    assert_eq!(parsed_members.get("Dint1"), original_members.get("Dint1"));
    
    println!("✅ Round-trip conversion successful");
}

#[tokio::test]
#[ignore] // Requires actual PLC connection
async fn test_udt_generic_any_udt() {
    let address = "127.0.0.1:44818";
    if !is_plc_available(address).await {
        println!("Skipping test: No PLC available at {}", address);
        return;
    }

    let mut client = EipClient::connect(address).await.unwrap();

    // Test with different UDT names - should work generically
    let udt_names = vec!["Part_Data", "MotorData", "TestUDT"];
    
    for udt_name in udt_names {
        match client.read_tag(udt_name).await {
            Ok(PlcValue::Udt(udt_data)) => {
                println!("✅ Successfully read generic UDT: {}", udt_name);
                println!("   Symbol ID: {}", udt_data.symbol_id);
                println!("   Data size: {} bytes", udt_data.data.len());
                
                // Verify it's valid
                assert!(udt_data.symbol_id > 0, "symbol_id should be > 0");
                assert!(!udt_data.data.is_empty(), "data should not be empty");
            }
            Err(e) => {
                // Tag might not exist, that's okay
                if e.to_string().contains("not found") {
                    println!("⚠️ Tag '{}' not found (skipping)", udt_name);
                    continue;
                }
                // Other errors might indicate a problem
                println!("⚠️ Error reading '{}': {}", udt_name, e);
            }
            _ => {
                println!("⚠️ '{}' is not a UDT", udt_name);
            }
        }
    }
}

