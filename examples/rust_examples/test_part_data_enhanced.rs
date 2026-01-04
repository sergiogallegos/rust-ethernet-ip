use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Part_Data UDT with Enhanced Parser");
    println!("===============================================");
    println!("PLC IP: 192.168.0.1");
    println!("UDT: Part_Data (Complex UDT with multiple members)");
    println!("Expected to have multiple members with different data types");
    println!();

    let plc_address = "192.168.0.1:44818";
    println!("📡 Connecting to PLC at {}", plc_address);

    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected successfully!\n");

    // Test 1: Try reading the entire Part_Data UDT
    println!("🎯 Test 1: Reading entire Part_Data UDT");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_tag("Part_Data").await {
        Ok(value) => {
            let duration = start_time.elapsed();
            println!("   ✅ SUCCESS: {:?} (took {:?})", value, duration);
            
            match value {
                PlcValue::Udt(udt_data) => {
                    println!("   📋 UDT Data:");
                    println!("      Symbol ID: {}", udt_data.symbol_id);
                    println!("      Data Size: {} bytes", udt_data.data.len());
                    println!("      Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                    if udt_data.data.len() > 32 {
                        println!("      ... ({} more bytes)", udt_data.data.len() - 32);
                    }
                }
                _ => {
                    println!("   ⚠️  Unexpected type: {:?}", value);
                }
            }
        }
        Err(e) => {
            let duration = start_time.elapsed();
            println!("   ❌ FAILED: {} (took {:?})", e, duration);
        }
    }

    // Test 2: Try chunked reading for large UDT
    println!("\n🎯 Test 2: Chunked Reading for Large UDT");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_udt_chunked("Part_Data").await {
        Ok(value) => {
            let duration = start_time.elapsed();
            println!("   ✅ SUCCESS: {:?} (took {:?})", value, duration);
            
            match value {
                PlcValue::Udt(udt_data) => {
                    println!("   📋 Chunked UDT Data:");
                    println!("      Symbol ID: {}", udt_data.symbol_id);
                    println!("      Data Size: {} bytes", udt_data.data.len());
                    println!("      Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                    if udt_data.data.len() > 32 {
                        println!("      ... ({} more bytes)", udt_data.data.len() - 32);
                    }
                }
                _ => {
                    println!("   ⚠️  Unexpected type: {:?}", value);
                }
            }
        }
        Err(e) => {
            let duration = start_time.elapsed();
            println!("   ❌ FAILED: {} (took {:?})", e, duration);
        }
    }

    // Test 3: Try to get UDT definition
    println!("\n🎯 Test 3: UDT Definition Discovery");
    println!("{}", "=".repeat(50));

    match client.get_udt_definition("Part_Data").await {
        Ok(definition) => {
            println!("   ✅ UDT Definition found!");
            println!("   📋 UDT Name: {}", definition.name);
            println!("   📊 Member Count: {}", definition.members.len());
            println!("   📏 Total Size: {} bytes", definition.members.iter().map(|m| m.size).sum::<u32>());
            
            println!("   📋 Members:");
            for member in &definition.members {
                println!("      - {}: {} (offset: {}, size: {})", 
                    member.name, 
                    member.data_type, 
                    member.offset, 
                    member.size
                );
            }
        }
        Err(e) => {
            println!("   ❌ UDT Definition not found: {}", e);
        }
    }

    println!("\n🎉 Part_Data UDT enhanced test completed!");
    Ok(())
}
