use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing Program Tag: out_FusePartStatus");
    println!("=============================================");
    println!("PLC IP: 192.168.0.1");
    println!("Program: API_Web");
    println!("Tag: out_FusePartStatus");
    println!();

    let plc_address = "192.168.0.1:44818";
    println!("📡 Connecting to PLC at {}", plc_address);

    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected successfully!\n");

    // Test reading the out_FusePartStatus program tag
    println!("🎯 Testing: out_FusePartStatus (Program Tag)");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_tag("Program:API_Web.out_FusePartStatus").await {
        Ok(value) => {
            let duration = start_time.elapsed();
            println!("   ✅ SUCCESS: {:?} (took {:?})", value, duration);
            
            match value {
                PlcValue::Bool(actual_value) => {
                    println!("   📊 Boolean value: {}", actual_value);
                }
                PlcValue::Dint(actual_value) => {
                    println!("   📊 DINT value: {}", actual_value);
                }
                PlcValue::Real(actual_value) => {
                    println!("   📊 REAL value: {}", actual_value);
                }
                PlcValue::String(actual_value) => {
                    println!("   📊 STRING value: {}", actual_value);
                }
                PlcValue::Udt(udt_data) => {
                    println!("   📊 UDT Data:");
                    println!("      Symbol ID: {}", udt_data.symbol_id);
                    println!("      Data Size: {} bytes", udt_data.data.len());
                    println!("      Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                    if udt_data.data.len() > 32 {
                        println!("      ... ({} more bytes)", udt_data.data.len() - 32);
                    }
                }
                _ => {
                    println!("   📊 Other type: {:?}", value);
                }
            }
        }
        Err(e) => {
            let duration = start_time.elapsed();
            println!("   ❌ FAILED: {} (took {:?})", e, duration);
            
            // Try alternative approaches
            println!("   🔍 Trying alternative approaches...");
            
            let alternative_paths = vec![
                "API_Web.out_FusePartStatus",
                "Controller:API_Web.out_FusePartStatus",
                "out_FusePartStatus",
            ];
            
            for alt_path in alternative_paths {
                println!("   🔍 Trying: {}", alt_path);
                match client.read_tag(alt_path).await {
                    Ok(value) => {
                        println!("   ✅ Alternative path worked: {:?}", value);
                        break;
                    }
                    Err(e) => {
                        println!("   ❌ Alternative path failed: {}", e);
                    }
                }
            }
        }
    }

    println!("\n🎉 out_FusePartStatus program tag test completed!");
    Ok(())
}
