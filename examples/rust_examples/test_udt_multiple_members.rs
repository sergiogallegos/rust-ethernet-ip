use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing UDT with Multiple Members");
    println!("=====================================");
    println!("PLC IP: 192.168.0.1");
    println!("UDT: TestTagUDT (UDT_Test type)");
    println!("Expected members:");
    println!("  - TestTagUDT.TestTagUDT (DINT, value: 99)");
    println!("  - TestTagUDT.TestTagUDT2 (DINT, value: 88)");
    println!("  - TestTagUDT.TestTagUDT3 (REAL, value: 12.12)");
    println!();

    let plc_address = "192.168.0.1:44818";
    println!("📡 Connecting to PLC at {}", plc_address);

    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected successfully!\n");

    // Test 1: Read the entire UDT
    println!("🎯 Test 1: Reading entire UDT");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_tag("TestTagUDT").await {
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
                    println!("\n   💡 To access specific members, use UDT definition parsing");
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

    // Test 2: Try reading individual UDT members
    println!("\n🎯 Test 2: Reading individual UDT members");
    println!("{}", "=".repeat(50));

    let individual_members = vec![
        "TestTagUDT.TestTagUDT",
        "TestTagUDT.TestTagUDT2", 
        "TestTagUDT.TestTagUDT3",
    ];

    for member in individual_members {
        println!("\n🔍 Testing: {}", member);
        
        let start_time = Instant::now();
        
        match client.read_tag(member).await {
            Ok(value) => {
                let duration = start_time.elapsed();
                println!("   ✅ SUCCESS: {:?} (took {:?})", value, duration);
            }
            Err(e) => {
                let duration = start_time.elapsed();
                println!("   ❌ FAILED: {} (took {:?})", e, duration);
            }
        }
    }

    println!("\n🎉 UDT multiple members test completed!");
    Ok(())
}
