use rust_ethernet_ip::EipClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== Real UDT Test with PLC ===");
    println!("Testing UDT reading directly from Rust library");
    println!();

    let plc_address = "192.168.0.1:44818";
    
    // Create and connect to PLC
    println!("🔌 Connecting to PLC...");
    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected to PLC successfully!");
    
    // Test different UDT tag path formats
    let udt_variations = vec![
        "Part_Data",
        "Part_Data[0]",
        "Program:MainProgram.Part_Data",
        "Program:MainProgram.Part_Data[0]",
        "Controller:Part_Data",
        "Controller:Part_Data[0]",
        "PC_Database",
        "PC_Database[0]",
        "Program:MainProgram.PC_Database",
        "Program:MainProgram.PC_Database[0]",
    ];
    
    println!("\n🔍 Testing UDT tag path variations:");
    println!("{}", "=".repeat(60));
    
    for (i, tag_path) in udt_variations.iter().enumerate() {
        println!("\n{}. Testing: {}", i + 1, tag_path);
        println!("   {}", "-".repeat(50));
        
        match test_udt_reading(&mut client, tag_path).await {
            Ok(result) => {
                println!("   ✅ SUCCESS!");
                println!("   📊 UDT Data: {:?}", result);
                println!("   🎯 Working tag path: {}", tag_path);
                return Ok(());
            }
            Err(e) => {
                println!("   ❌ FAILED: {}", e);
            }
        }
    }
    
    println!("\n❌ None of the UDT tag paths worked!");
    println!("🔍 Let's try to discover what tags actually exist...");
    
    // Try to discover tags
    test_tag_discovery(&mut client).await?;
    
    Ok(())
}

async fn test_udt_reading(client: &mut EipClient, tag_path: &str) -> Result<rust_ethernet_ip::PlcValue, Box<dyn std::error::Error>> {
    println!("   🔍 Attempting to read UDT: {}", tag_path);
    
    // Try to read the tag
    let result = client.read_tag(tag_path).await?;
    
    // Check if it's a UDT
    match &result {
        rust_ethernet_ip::PlcValue::Udt(udt_data) => {
            println!("   📋 UDT Data:");
            println!("      Symbol ID: {}", udt_data.symbol_id);
            println!("      Data Size: {} bytes", udt_data.data.len());
            println!("      Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
            if udt_data.data.len() > 32 {
                println!("      ... ({} more bytes)", udt_data.data.len() - 32);
            }
            Ok(result)
        }
        other => {
            println!("   ⚠️  Tag found but not a UDT: {:?}", other);
            Err(format!("Tag '{}' is not a UDT type", tag_path).into())
        }
    }
}

async fn test_tag_discovery(client: &mut EipClient) -> Result<(), Box<dyn std::error::Error>> {
    println!("\n🔍 Tag Discovery Test");
    println!("{}", "=".repeat(40));
    
    // Try to discover tags with different patterns
    let discovery_patterns = vec![
        "Part",
        "P",
        "Data", 
        "UDT",
        "PC",
        "Database",
    ];
    
    for pattern in discovery_patterns {
        println!("\n🔍 Searching for tags containing: '{}'", pattern);
        
        // Try to read a tag with this pattern
        let test_tag = format!("{}", pattern);
        match client.read_tag(&test_tag).await {
            Ok(value) => {
                println!("   ✅ Found tag '{}': {:?}", test_tag, value);
            }
            Err(e) => {
                println!("   ❌ Tag '{}' not found: {}", test_tag, e);
            }
        }
    }
    
    Ok(())
}
