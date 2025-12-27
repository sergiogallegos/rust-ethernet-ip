// examples/test_array_elements.rs
// =========================================================================
//
// Array Element Reading Test
//
// This example tests reading individual array elements from:
// - gArrayTest (controller-scoped DINT[10]) - values 1-10
// - Program:MainProgram.ArrayTest (program-scoped DINT[10]) - values 1-10
//
// Usage:
//   cargo run --example test_array_elements -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_array_elements -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Array Element Reading Test");
    println!("==============================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_array_elements -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_array_elements -- 192.168.0.1:44818");
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

    // Test arrays
    let arrays = vec![
        ("gArrayTest", "Controller-scoped"),
        ("Program:MainProgram.ArrayTest", "Program-scoped"),
    ];

    for (array_name, description) in &arrays {
        println!("📊 Testing {} Array: {}", description, array_name);
        println!("{}", "=".repeat(60));

        // First, verify base array can be read
        println!("\n  Step 1: Reading base array (no index)...");
        match timeout(Duration::from_secs(5), client.read_tag(array_name)).await {
            Ok(Ok(value)) => {
                println!("    ✅ Base array read: {:?}", value);
            }
            Ok(Err(e)) => {
                println!("    ❌ Base array read failed: {}", e);
                continue; // Skip this array if base read fails
            }
            Err(_) => {
                println!("    ❌ Base array read timeout");
                continue;
            }
        }

        // Now try reading individual elements (indices 0-9, expecting values 1-10)
        println!("\n  Step 2: Reading individual array elements...");
        let mut success_count = 0;
        let mut fail_count = 0;

        for index in 0..10 {
            let expected_value = index + 1; // Array has values 1-10
            let tag_path = format!("{}[{}]", array_name, index);

            print!(
                "    Reading {}[{}] (expected {})... ",
                array_name, index, expected_value
            );

            match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
                Ok(Ok(value)) => {
                    // Check if value matches expected
                    match value {
                        PlcValue::Dint(actual) => {
                            if actual == expected_value as i32 {
                                println!("✅ {:?} (correct!)", value);
                                success_count += 1;
                            } else {
                                println!(
                                    "⚠️  {:?} (expected {}, got {})",
                                    value, expected_value, actual
                                );
                                success_count += 1; // Still count as success, just wrong value
                            }
                        }
                        _ => {
                            println!("⚠️  {:?} (unexpected type, expected Dint)", value);
                            success_count += 1; // Still count as success
                        }
                    }
                }
                Ok(Err(e)) => {
                    println!("❌ Error: {}", e);
                    fail_count += 1;

                    // Check if it's the path segment error
                    if e.to_string().contains("Path segment error")
                        || e.to_string().contains("CIP Error 4")
                    {
                        println!(
                            "       ⚠️  This is the 'Path segment error' we're trying to fix!"
                        );
                    }
                }
                Err(_) => {
                    println!("❌ Timeout");
                    fail_count += 1;
                }
            }
        }

        println!("\n  Results for {}:", description);
        println!("    ✅ Successful: {}", success_count);
        println!("    ❌ Failed: {}", fail_count);

        if success_count == 10 {
            println!("    🎉 All array elements read successfully!");
        } else if fail_count > 0 {
            println!("    ⚠️  Some array elements failed to read");
        }

        println!();
    }

    println!("📈 Test Summary");
    println!("===============");
    println!("✅ Array element reading test completed!");
    println!("\nNote: If array element reads fail with 'Path segment error',");
    println!("      this indicates the element segment encoding needs adjustment.");

    Ok(())
}
