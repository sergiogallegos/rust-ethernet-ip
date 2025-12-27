use rust_ethernet_ip::{EipClient, PlcValue};
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Testing REAL Tag Write and Read Operations");
    println!("===============================================");
    println!("PLC IP: 192.168.0.1:44818");
    println!("Testing REAL tags in API_Web program");
    println!("Writing 88.88 to REAL tags, then reading back to verify");
    println!();

    // Connect to PLC
    let mut client = match timeout(
        Duration::from_secs(10),
        EipClient::connect("192.168.0.1:44818"),
    )
    .await
    {
        Ok(Ok(client)) => client,
        Ok(Err(e)) => {
            eprintln!("❌ Failed to connect to PLC: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            eprintln!("❌ Connection timeout - PLC may not be available");
            return Err("Connection timeout".into());
        }
    };

    println!("✅ Connected to PLC successfully!\n");

    // List of REAL tags to test (from the API_Web program)
    let real_tags = vec![
        "Program:API_Web.out_FuseWeight2",
        "Program:API_Web.out_FuseWeight1",
        "Program:API_Web.out_FuseSandFillTime",
        "Program:API_Web.out_FuseResistance1",
    ];

    let test_value = 88.88;
    println!("🎯 Test Plan:");
    println!("1. Read initial values");
    println!("2. Write {} to all REAL tags", test_value);
    println!("3. Read back values to verify changes");
    println!();

    // Step 1: Read initial values
    println!("📖 Step 1: Reading initial values");
    println!("----------------------------------");
    let mut initial_values = std::collections::HashMap::new();

    for tag_name in &real_tags {
        let start = std::time::Instant::now();
        match client.read_tag(tag_name).await {
            Ok(value) => {
                let duration = start.elapsed();
                match value {
                    PlcValue::Real(real_val) => {
                        println!("✅ {}: {} (took {:?})", tag_name, real_val, duration);
                        initial_values.insert(tag_name, real_val);
                    }
                    other => {
                        println!(
                            "⚠️ {}: {:?} (unexpected type, took {:?})",
                            tag_name, other, duration
                        );
                    }
                }
            }
            Err(e) => {
                println!("❌ {}: Failed to read - {}", tag_name, e);
            }
        }
    }
    println!();

    // Step 2: Write test values
    println!("✏️ Step 2: Writing {} to all REAL tags", test_value);
    println!("--------------------------------------------");

    for tag_name in &real_tags {
        let start = std::time::Instant::now();
        match client.write_tag(tag_name, PlcValue::Real(test_value)).await {
            Ok(()) => {
                let duration = start.elapsed();
                println!("✅ {}: Write successful (took {:?})", tag_name, duration);
            }
            Err(e) => {
                println!("❌ {}: Write failed - {}", tag_name, e);
            }
        }
    }
    println!();

    // Small delay to ensure writes are processed
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Step 3: Read back values to verify changes
    println!("📖 Step 3: Reading back values to verify changes");
    println!("------------------------------------------------");

    let mut success_count = 0;
    let mut total_count = 0;

    for tag_name in &real_tags {
        let start = std::time::Instant::now();
        match client.read_tag(tag_name).await {
            Ok(value) => {
                let duration = start.elapsed();
                match value {
                    PlcValue::Real(real_val) => {
                        total_count += 1;

                        // Check if the value is close to our test value (allowing for small floating point differences)
                        let difference = (real_val - test_value).abs();
                        if difference < 0.01 {
                            println!("✅ {}: {} ✓ (took {:?})", tag_name, real_val, duration);
                            success_count += 1;
                        } else {
                            println!(
                                "❌ {}: {} ✗ Expected: {} (took {:?})",
                                tag_name, real_val, test_value, duration
                            );
                        }

                        // Show initial vs current comparison
                        if let Some(initial_val) = initial_values.get(tag_name) {
                            if (initial_val - real_val).abs() > 0.01 {
                                println!("   📊 Changed from {} to {}", initial_val, real_val);
                            } else {
                                println!("   📊 No change from initial value {}", initial_val);
                            }
                        }
                    }
                    other => {
                        println!(
                            "❌ {}: {:?} (unexpected type, took {:?})",
                            tag_name, other, duration
                        );
                    }
                }
            }
            Err(e) => {
                println!("❌ {}: Failed to read - {}", tag_name, e);
            }
        }
    }
    println!();

    // Summary
    println!("📊 Test Results Summary");
    println!("======================");
    println!("Total REAL tags tested: {}", real_tags.len());
    println!(
        "Successful write/read cycles: {}/{}",
        success_count, total_count
    );

    if success_count == total_count && total_count > 0 {
        println!("🎉 ALL TESTS PASSED! REAL tag write/read operations working perfectly!");
        println!(
            "✅ Successfully wrote {} to all REAL tags and verified the changes",
            test_value
        );
    } else if success_count > 0 {
        println!(
            "⚠️ PARTIAL SUCCESS: {}/{} REAL tags working correctly",
            success_count, total_count
        );
    } else {
        println!("❌ ALL TESTS FAILED: No REAL tags could be written/read successfully");
    }

    // Performance summary
    println!("\n⚡ Performance Notes:");
    println!("- Write operations should complete in <10ms");
    println!("- Read operations should complete in <5ms");
    println!("- Total test time should be <1 second for all operations");

    Ok(())
}
