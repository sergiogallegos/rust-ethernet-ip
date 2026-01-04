use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Part_Data UDT with Chunked Reading");
    println!("===============================================");
    println!("PLC IP: 192.168.0.1");
    println!("Tag: Part_Data (Large UDT with multiple members)");
    println!();

    let plc_address = "192.168.0.1:44818";
    println!("📡 Connecting to PLC at {}", plc_address);

    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected successfully!\n");

    // Test reading the Part_Data UDT with chunked reading
    println!("🎯 Testing: Part_Data UDT (Chunked Reading)");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_udt_chunked("Part_Data").await {
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
                PlcValue::Dint(actual_value) => {
                    println!("   📊 Direct DINT value: {}", actual_value);
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

    println!("\n🎉 Part_Data UDT chunked reading test completed!");
    Ok(())
}
