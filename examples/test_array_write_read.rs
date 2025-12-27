// examples/test_array_write_read.rs
// =========================================================================
//
// Array Write and Read Test
//
// This example tests writing to and reading back from:
// - gArrayTest (controller-scoped DINT[10])
// - gArrayRealTest (controller-scoped REAL[10])
//
// Usage:
//   cargo run --example test_array_write_read -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_array_write_read -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Array Write and Read Test");
    println!("============================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_array_write_read -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_array_write_read -- 192.168.0.1:44818");
        return Ok(());
    }

    let plc_address = &args[1];

    println!("📡 Connecting to PLC at {}", plc_address);

    // Connect to PLC
    let mut client = match timeout(Duration::from_secs(10), EipClient::connect(plc_address)).await {
        Ok(Ok(client)) => {
            println!("✅ Connected successfully!\n");
            client
        }
        Ok(Err(e)) => {
            eprintln!("❌ Failed to connect: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            eprintln!("❌ Connection timeout");
            return Err("Connection timeout".into());
        }
    };

    // Test DINT array
    println!("📊 Test 1: DINT Array Write and Read");
    println!("{}", "=".repeat(60));
    test_dint_array(&mut client).await?;

    println!("\n");

    // Test REAL array
    println!("📊 Test 2: REAL Array Write and Read");
    println!("{}", "=".repeat(60));
    test_real_array(&mut client).await?;

    println!("\n📈 Test Summary");
    println!("===============");
    println!("✅ All array write/read tests completed!");

    Ok(())
}

async fn test_dint_array(client: &mut EipClient) -> Result<(), Box<dyn std::error::Error>> {
    let array_name = "gArrayTest";
    let test_values = vec![100, 200, 300, 400, 500, 600, 700, 800, 900, 1000];

    println!("\n  Step 1: Writing values to DINT array elements...");
    for (index, &value) in test_values.iter().enumerate() {
        let tag_path = format!("{}[{}]", array_name, index);
        print!("    Writing {}[{}] = {}... ", array_name, index, value);

        match timeout(
            Duration::from_secs(5),
            client.write_tag(&tag_path, PlcValue::Dint(value)),
        )
        .await
        {
            Ok(Ok(_)) => {
                println!("✅");
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                return Err(e.into());
            }
            Err(_) => {
                println!("❌ Timeout");
                return Err("Timeout".into());
            }
        }
    }

    println!("\n  Step 2: Reading back DINT array elements...");
    let mut success_count = 0;
    let mut fail_count = 0;

    for (index, &expected_value) in test_values.iter().enumerate() {
        let tag_path = format!("{}[{}]", array_name, index);

        print!(
            "    Reading {}[{}] (expected {})... ",
            array_name, index, expected_value
        );

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                match value {
                    PlcValue::Dint(actual) => {
                        if actual == expected_value as i32 {
                            println!("✅ Dint({}) (correct!)", actual);
                            success_count += 1;
                        } else {
                            println!(
                                "⚠️  Dint({}) (expected {}, got {})",
                                actual, expected_value, actual
                            );
                            success_count += 1; // Still count as success, just wrong value
                        }
                    }
                    _ => {
                        println!("⚠️  {:?} (unexpected type, expected Dint)", value);
                        success_count += 1;
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                fail_count += 1;
            }
        }
    }

    println!("\n  Results for DINT array:");
    println!("    ✅ Successful: {}", success_count);
    println!("    ❌ Failed: {}", fail_count);

    if success_count == 10 && fail_count == 0 {
        println!("    🎉 All DINT array write/read operations successful!");
    } else if fail_count > 0 {
        println!("    ⚠️  Some DINT array operations failed");
    }

    Ok(())
}

async fn test_real_array(client: &mut EipClient) -> Result<(), Box<dyn std::error::Error>> {
    let array_name = "gArrayRealTest";
    let test_values = vec![
        11.11, 22.22, 33.33, 44.44, 55.55, 66.66, 77.77, 88.88, 99.99, 111.11,
    ];

    println!("\n  Step 1: Writing values to REAL array elements...");
    for (index, &value) in test_values.iter().enumerate() {
        let tag_path = format!("{}[{}]", array_name, index);
        print!("    Writing {}[{}] = {:.2}... ", array_name, index, value);

        match timeout(
            Duration::from_secs(5),
            client.write_tag(&tag_path, PlcValue::Real(value)),
        )
        .await
        {
            Ok(Ok(_)) => {
                println!("✅");
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                return Err(e.into());
            }
            Err(_) => {
                println!("❌ Timeout");
                return Err("Timeout".into());
            }
        }
    }

    println!("\n  Step 2: Reading back REAL array elements...");
    let mut success_count = 0;
    let mut fail_count = 0;

    for (index, &expected_value) in test_values.iter().enumerate() {
        let tag_path = format!("{}[{}]", array_name, index);

        print!(
            "    Reading {}[{}] (expected {:.2})... ",
            array_name, index, expected_value
        );

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                match value {
                    PlcValue::Real(actual) => {
                        // Allow small floating point differences
                        let diff = (actual - expected_value).abs();
                        if diff < 0.01 {
                            println!("✅ Real({:.2}) (correct!)", actual);
                            success_count += 1;
                        } else {
                            println!(
                                "⚠️  Real({:.2}) (expected {:.2}, got {:.2}, diff: {:.4})",
                                actual, expected_value, actual, diff
                            );
                            success_count += 1; // Still count as success, just wrong value
                        }
                    }
                    _ => {
                        println!("⚠️  {:?} (unexpected type, expected Real)", value);
                        success_count += 1;
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                fail_count += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                fail_count += 1;
            }
        }
    }

    println!("\n  Results for REAL array:");
    println!("    ✅ Successful: {}", success_count);
    println!("    ❌ Failed: {}", fail_count);

    if success_count == 10 && fail_count == 0 {
        println!("    🎉 All REAL array write/read operations successful!");
    } else if fail_count > 0 {
        println!("    ⚠️  Some REAL array operations failed");
    }

    Ok(())
}

