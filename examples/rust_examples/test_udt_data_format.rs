//! Example: Testing the new generic UDT data format
//!
//! This example demonstrates how to use the new UdtData format which is
//! generic and works with any UDT without requiring hardcoded member names.

use rust_ethernet_ip::{EipClient, PlcValue, UdtData};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to PLC
    let address = std::env::var("PLC_ADDRESS")
        .unwrap_or_else(|_| "192.168.1.100:44818".to_string());
    
    println!("🔌 Connecting to PLC at {}...", address);
    let mut client = match EipClient::connect(&address).await {
        Ok(client) => {
            println!("✅ Connected successfully!");
            client
        }
        Err(e) => {
            eprintln!("❌ Connection failed: {}", e);
            eprintln!("💡 Set PLC_ADDRESS environment variable or modify the address in code");
            return Err(e.into());
        }
    };

    // Example 1: Read a UDT generically (works with any UDT)
    println!("\n📖 Example 1: Reading UDT generically");
    let udt_name = std::env::var("UDT_TAG_NAME")
        .unwrap_or_else(|_| "Part_Data".to_string());
    
    match client.read_tag(&udt_name).await {
        Ok(PlcValue::Udt(udt_data)) => {
            println!("✅ Successfully read UDT: {}", udt_name);
            println!("   Symbol ID (template instance ID): {}", udt_data.symbol_id);
            println!("   Data size: {} bytes", udt_data.data.len());
            println!("   Raw data (first 32 bytes): {:02X?}", 
                     &udt_data.data[..udt_data.data.len().min(32)]);
            
            // The data is now in raw bytes - you can parse it when you have the UDT definition
            if udt_data.symbol_id > 0 {
                println!("   ✅ Valid symbol_id - ready for writing");
            } else {
                println!("   ⚠️ symbol_id is 0 - may need to read tag attributes");
            }
        }
        Ok(other) => {
            println!("⚠️ Tag '{}' is not a UDT, got: {:?}", udt_name, other);
        }
        Err(e) => {
            println!("❌ Failed to read UDT '{}': {}", udt_name, e);
            println!("💡 Make sure the tag exists and is a UDT type");
        }
    }

    // Example 2: Parse UDT with definition (when you know the structure)
    println!("\n📖 Example 2: Parsing UDT with definition");
    match client.read_tag(&udt_name).await {
        Ok(PlcValue::Udt(udt_data)) => {
            // Get UDT definition from PLC
            match client.get_udt_definition(&udt_name).await {
                Ok(definition) => {
                    println!("✅ Got UDT definition with {} members", definition.members.len());
                    
                    // Parse raw bytes into member values
                    match udt_data.parse(&definition) {
                        Ok(members) => {
                            println!("✅ Parsed UDT into {} members:", members.len());
                            for (name, value) in &members {
                                println!("   {}: {:?}", name, value);
                            }
                        }
                        Err(e) => {
                            println!("⚠️ Failed to parse UDT: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Could not get UDT definition: {}", e);
                    println!("💡 The UDT definition parsing may need improvement");
                }
            }
        }
        Err(e) => {
            println!("⚠️ Could not read UDT for parsing example: {}", e);
        }
        _ => {}
    }

    // Example 3: Write UDT (requires symbol_id)
    println!("\n📝 Example 3: Writing UDT");
    
    // First, read to get symbol_id
    match client.read_tag(&udt_name).await {
        Ok(PlcValue::Udt(udt_data)) => {
            println!("✅ Read UDT with symbol_id: {}", udt_data.symbol_id);
            
            // Modify the data (example: flip first byte if it's a BOOL)
            let mut modified_data = udt_data.data.clone();
            if !modified_data.is_empty() {
                let original = modified_data[0];
                modified_data[0] = if original == 0 { 0xFF } else { 0x00 };
                println!("   Modified first byte: 0x{:02X} -> 0x{:02X}", original, modified_data[0]);
            }
            
            // Create new UdtData with same symbol_id
            let modified_udt = UdtData {
                symbol_id: udt_data.symbol_id,
                data: modified_data,
            };
            
            // Write it back
            match client.write_tag(&udt_name, PlcValue::Udt(modified_udt.clone())).await {
                Ok(_) => {
                    println!("✅ Successfully wrote UDT with symbol_id: {}", modified_udt.symbol_id);
                }
                Err(e) => {
                    println!("⚠️ Write failed (tag may be read-only): {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ Could not read UDT for write example: {}", e);
        }
        _ => {}
    }

    // Example 4: Write UDT with auto-read of symbol_id
    println!("\n📝 Example 4: Writing UDT with auto-read symbol_id");
    
    // Create UdtData with symbol_id = 0 (will trigger auto-read)
    let udt_data = UdtData {
        symbol_id: 0, // Will be read automatically
        data: vec![0x00, 0x00, 0x00, 0x00], // Example data
    };
    
    match client.write_tag(&udt_name, PlcValue::Udt(udt_data)).await {
        Ok(_) => {
            println!("✅ Successfully wrote UDT (symbol_id was auto-read)");
        }
        Err(e) => {
            if e.to_string().contains("symbol_id") {
                println!("❌ Auto-read of symbol_id failed: {}", e);
            } else {
                println!("⚠️ Write failed (may be expected): {}", e);
            }
        }
    }

    // Example 5: Convert between HashMap and UdtData
    println!("\n🔄 Example 5: Converting between HashMap and UdtData");
    
    match client.get_udt_definition(&udt_name).await {
        Ok(definition) => {
            // Create HashMap of member values
            let mut members = HashMap::new();
            members.insert("ExampleMember".to_string(), PlcValue::Dint(42));
            
            // Convert to UdtData
            match UdtData::from_hash_map(&members, &definition, 123) {
                Ok(udt_data) => {
                    println!("✅ Converted HashMap to UdtData");
                    println!("   Symbol ID: {}", udt_data.symbol_id);
                    println!("   Data size: {} bytes", udt_data.data.len());
                    
                    // Convert back to HashMap
                    match udt_data.parse(&definition) {
                        Ok(parsed_members) => {
                            println!("✅ Converted UdtData back to HashMap");
                            println!("   Members: {:?}", parsed_members.keys().collect::<Vec<_>>());
                        }
                        Err(e) => {
                            println!("⚠️ Failed to parse back: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("⚠️ Failed to convert HashMap to UdtData: {}", e);
                }
            }
        }
        Err(e) => {
            println!("⚠️ Could not get UDT definition for conversion example: {}", e);
        }
    }

    println!("\n✅ Examples completed!");
    println!("\n💡 Key points:");
    println!("   - UDTs are now stored as raw bytes with symbol_id");
    println!("   - No hardcoded member names required");
    println!("   - Works generically with any UDT");
    println!("   - Use get_udt_definition() and parse() when you need member access");
    println!("   - symbol_id is required for writing (auto-read if 0)");

    Ok(())
}

