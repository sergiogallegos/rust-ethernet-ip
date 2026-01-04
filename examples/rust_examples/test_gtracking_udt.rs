use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Instant;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing gTracking UDT with Enhanced Parser");
    println!("==============================================");
    println!("PLC IP: 192.168.0.1");
    println!("UDT: gTracking (Controller-scoped UDT with multiple members)");
    println!();

    let plc_address = "192.168.0.1:44818";
    println!("📡 Connecting to PLC at {}", plc_address);

    let mut client = EipClient::connect(plc_address).await?;
    println!("✅ Connected successfully!\n");

    // Test 1: Try reading the entire gTracking UDT
    println!("🎯 Test 1: Reading entire gTracking UDT");
    println!("{}", "=".repeat(50));

    let start_time = Instant::now();
    
    match client.read_tag("gTracking").await {
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
    
    match client.read_udt_chunked("gTracking").await {
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

    // Test 3: Try to discover common tracking UDT members
    println!("\n🎯 Test 3: Member Discovery");
    println!("{}", "=".repeat(50));

    let common_tracking_members = vec![
        "gTracking.PartCount",
        "gTracking.TotalParts",
        "gTracking.CurrentPart",
        "gTracking.ProductionRate",
        "gTracking.ShiftNumber",
        "gTracking.LineSpeed",
        "gTracking.QualityCount",
        "gTracking.RejectCount",
        "gTracking.StartTime",
        "gTracking.EndTime",
        "gTracking.Status",
        "gTracking.Mode",
        "gTracking.Operator",
        "gTracking.BatchNumber",
        "gTracking.LotNumber",
        "gTracking.SerialNumber",
        "gTracking.Timestamp",
        "gTracking.CycleTime",
        "gTracking.Downtime",
        "gTracking.Efficiency"
    ];

    let mut successful_members = 0;

    for member_path in &common_tracking_members {
        match client.read_tag(member_path).await {
            Ok(value) => {
                println!("   ✅ {}: {:?}", member_path, value);
                successful_members += 1;
            }
            Err(_) => {
                // Member not found or not accessible
            }
        }
    }

    if successful_members > 0 {
        println!("\n   📊 Member Discovery Results:");
        println!("      - Found {} accessible members", successful_members);
        println!("      - Total members tested: {}", common_tracking_members.len());
    } else {
        println!("   ⚠️  No individual members could be accessed");
        println!("   💡 This might indicate the UDT structure is different than expected");
    }

    // Test 4: Try UDT definition discovery
    println!("\n🎯 Test 4: UDT Definition Discovery");
    println!("{}", "=".repeat(50));

    match client.get_udt_definition("gTracking").await {
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

    println!("\n🎉 gTracking UDT test completed!");
    Ok(())
}
