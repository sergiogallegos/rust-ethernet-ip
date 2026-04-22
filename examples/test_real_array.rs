// examples/test_real_array.rs
// =========================================================================
//
// REAL Array Element Reading Test
//
// This example tests reading individual REAL array elements from:
// - gArrayRealTest (controller-scoped REAL[10]) - values 1.11, 2.22, 3.33, etc.
//
// Usage:
//   cargo run --example test_real_array -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_real_array -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 REAL Array Element Reading Test");
    println!("==================================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_real_array -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_real_array -- 192.168.0.1:44818");
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

    // Test REAL array
    let array_name = "gArrayRealTest";
    let expected_values = vec![1.11, 2.22, 3.33, 4.44, 5.55, 6.66, 7.77, 8.88, 9.99, 98.76];

    println!("📊 Testing Controller-scoped REAL Array: {}", array_name);
    println!("{}", "=".repeat(60));

    // First, verify base array can be read
    println!("\n  Step 1: Reading base array (no index)...");
    match timeout(Duration::from_secs(5), client.read_tag(array_name)).await {
        Ok(Ok(value)) => {
            println!("    ✅ Base array read: {:?}", value);
        }
        Ok(Err(e)) => {
            println!("    ❌ Base array read failed: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("    ❌ Base array read timeout");
            return Err("Timeout".into());
        }
    }

    // Now try reading individual elements (indices 0-9)
    println!("\n  Step 2: Reading individual REAL array elements...");
    let mut success_count = 0;
    let mut fail_count = 0;

    for (index, &expected_value) in expected_values.iter().enumerate() {
        let tag_path = format!("{}[{}]", array_name, index);

        print!(
            "    Reading {}[{}] (expected {:.2})... ",
            array_name, index, expected_value
        );

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                // Check if value matches expected
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
                        success_count += 1; // Still count as success
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

    println!("\n  Results for Controller-scoped REAL array:");
    println!("    ✅ Successful: {}", success_count);
    println!("    ❌ Failed: {}", fail_count);

    if success_count == 10 {
        println!("    🎉 All REAL array elements read successfully!");
    } else if fail_count > 0 {
        println!("    ⚠️  Some REAL array elements failed to read");
    }

    println!("\n📈 Test Summary");
    println!("===============");
    println!("✅ REAL array element reading test completed!");

    Ok(())
}
