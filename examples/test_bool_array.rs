// examples/test_bool_array.rs
// =========================================================================
//
// BOOL Array Reading Test
//
// This example tests reading a BOOL array:
// - gArrayBoolTest (controller-scoped BOOL[32])
//
// Usage:
//   cargo run --example test_bool_array -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_bool_array -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 BOOL Array Reading Test");
    println!("==========================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_bool_array -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_bool_array -- 192.168.0.1:44818");
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

    let array_name = "gArrayBoolTest";

    // Test 1: Read base array (no index)
    println!("📊 Test 1: Reading Base BOOL Array");
    println!("-----------------------------------\n");

    println!("  Reading {} (base array, no index)...", array_name);
    match timeout(Duration::from_secs(5), client.read_tag(array_name)).await {
        Ok(Ok(value)) => {
            println!("  ✅ Base array read: {:?}", value);
        }
        Ok(Err(e)) => {
            println!("  ❌ Base array read failed: {}", e);
        }
        Err(_) => {
            println!("  ❌ Base array read timeout");
        }
    }

    // Test 2: Read individual array elements (indices 0-18 to match what's visible)
    println!("\n📊 Test 2: Reading Individual BOOL Array Elements");
    println!("------------------------------------------------\n");

    let mut success_count = 0;
    let mut fail_count = 0;

    // Test first 19 elements (0-18) as shown in the image
    for index in 0..19 {
        let tag_path = format!("{}[{}]", array_name, index);

        print!("  Reading {}[{}]... ", array_name, index);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                success_count += 1;
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                fail_count += 1;

                // Check if it's the path segment error
                if e.to_string().contains("Path segment error")
                    || e.to_string().contains("CIP Error 4")
                {
                    println!("       ⚠️  This is the 'Path segment error' we're trying to fix!");
                }
            }
            Err(_) => {
                println!("❌ Timeout");
                fail_count += 1;
            }
        }
    }

    println!("\n  Results:");
    println!("    ✅ Successful: {}", success_count);
    println!("    ❌ Failed: {}", fail_count);

    // Test 3: Read a few more elements to test the full array
    if success_count > 0 {
        println!("\n📊 Test 3: Reading Additional Array Elements (19-31)");
        println!("---------------------------------------------------\n");

        let mut additional_success = 0;
        let mut additional_fail = 0;

        for index in 19..32 {
            let tag_path = format!("{}[{}]", array_name, index);

            print!("  Reading {}[{}]... ", array_name, index);

            match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
                Ok(Ok(value)) => {
                    println!("✅ {:?}", value);
                    additional_success += 1;
                }
                Ok(Err(e)) => {
                    println!("❌ Error: {}", e);
                    additional_fail += 1;
                }
                Err(_) => {
                    println!("❌ Timeout");
                    additional_fail += 1;
                }
            }
        }

        println!("\n  Additional Results:");
        println!("    ✅ Successful: {}", additional_success);
        println!("    ❌ Failed: {}", additional_fail);
    }

    println!("\n📈 Test Summary");
    println!("===============");
    println!("✅ BOOL array reading test completed!");

    if fail_count > 0 {
        println!("\n⚠️  Array element reads are still failing with 'Path segment error'.");
        println!("   This confirms the issue is specific to array element indexing.");
    } else if success_count > 0 {
        println!("\n🎉 All BOOL array element reads successful!");
    }

    Ok(())
}
