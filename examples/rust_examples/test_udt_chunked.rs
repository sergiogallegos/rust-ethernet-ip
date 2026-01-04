use rust_ethernet_ip::EipClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== UDT Chunked Reading Test ===");
    println!("Testing chunked reading for large UDT");
    println!();

    let plc_address = "192.168.0.1:44818";
    
    // Create and connect to PLC
    println!("🔌 Connecting to PLC...");
    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected to PLC successfully!");
    
    // Test chunked reading for Part_Data UDT
    println!("\n🔍 Testing chunked reading for Part_Data UDT:");
    println!("{}", "=".repeat(60));
    
    match test_udt_chunked_reading(&mut client, "Part_Data").await {
        Ok(udt_value) => {
            println!("✅ SUCCESS! UDT read with chunked method");
            println!("📊 UDT Value: {:?}", udt_value);
            
            // Check if it's a UDT and display members
            match &udt_value {
                rust_ethernet_ip::PlcValue::Udt(udt_data) => {
                    println!("📋 UDT Data:");
                    println!("   Symbol ID: {}", udt_data.symbol_id);
                    println!("   Data Size: {} bytes", udt_data.data.len());
                    println!("   Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                    if udt_data.data.len() > 32 {
                        println!("   ... ({} more bytes)", udt_data.data.len() - 32);
                    }
                }
                other => {
                    println!("⚠️  Tag found but not a UDT: {:?}", other);
                }
            }
        }
        Err(e) => {
            println!("❌ FAILED: {}", e);
            println!("🔍 Let's try to discover what UDTs actually exist...");
            test_udt_discovery(&mut client).await?;
        }
    }
    
    Ok(())
}

async fn test_udt_chunked_reading(client: &mut EipClient, udt_name: &str) -> Result<rust_ethernet_ip::PlcValue, Box<dyn std::error::Error>> {
    println!("🔍 Attempting chunked read for UDT: {}", udt_name);
    
    // Try chunked reading
    match client.read_udt_chunked(udt_name).await {
        Ok(udt_value) => {
            println!("   ✅ Chunked reading successful!");
            Ok(udt_value)
        }
        Err(e) => {
            println!("   ❌ Chunked reading failed: {}", e);
            Err(e.into())
        }
    }
}

async fn test_udt_discovery(client: &mut EipClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 UDT Discovery Test");
    println!("{}", "=".repeat(40));
    
    // Try to get UDT definition
    println!("🔍 Attempting to get UDT definition for Part_Data...");
    match client.get_udt_definition("Part_Data").await {
        Ok(definition) => {
            println!("   ✅ UDT Definition found!");
            println!("   📋 UDT Name: {}", definition.name);
            println!("   📊 Member Count: {}", definition.members.len());
            println!("   📏 Total Size: {} bytes", definition.members.iter().map(|m| m.size).sum::<u32>());
            
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
    
    // Try to get tag attributes
    println!("\n🔍 Attempting to get tag attributes for Part_Data...");
    match client.get_tag_attributes("Part_Data").await {
        Ok(attributes) => {
            println!("   ✅ Tag Attributes found!");
            println!("   📋 Tag Name: {}", attributes.name);
            println!("   📊 Data Type: {} (0x{:04X})", attributes.data_type_name, attributes.data_type);
            println!("   📏 Size: {} bytes", attributes.size);
            println!("   🎯 Template ID: {:?}", attributes.template_instance_id);
        }
        Err(e) => {
            println!("   ❌ Tag Attributes not found: {}", e);
        }
    }
    
    Ok(())
}
