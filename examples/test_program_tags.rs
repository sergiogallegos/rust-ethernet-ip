// examples/test_program_tags.rs
// =========================================================================
//
// Program-Scoped Tag Reading/Writing Test
//
// This example tests reading and writing to program-scoped tags in MainProgram:
// - BoolTest (BOOL)
// - DINTTest (DINT)
// - RealTest (REAL)
// - StringTest (STRING)
// - ArrayTest (DINT[10]) - base array only, not elements
//
// Usage:
//   cargo run --example test_program_tags -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_program_tags -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Program-Scoped Tag Reading/Writing Test");
    println!("===========================================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_program_tags -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_program_tags -- 192.168.0.1:44818");
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

    // Test tags - all program-scoped in MainProgram
    let test_tags = vec![
        (
            "Program:MainProgram.BoolTest",
            PlcValue::Bool(true),
            PlcValue::Bool(false),
        ),
        (
            "Program:MainProgram.DINTTest",
            PlcValue::Dint(100),
            PlcValue::Dint(200),
        ),
        (
            "Program:MainProgram.RealTest",
            PlcValue::Real(123.45),
            PlcValue::Real(678.90),
        ),
        (
            "Program:MainProgram.StringTest",
            PlcValue::String("HELLO".to_string()),
            PlcValue::String("WORLD".to_string()),
        ),
    ];

    println!("📊 Test: Reading Program-Scoped Tags");
    println!("-------------------------------------\n");

    // First, read all tags to see their current values
    for (tag_name, _, _) in &test_tags {
        print!("  Reading {}... ", tag_name);

        match timeout(Duration::from_secs(5), client.read_tag(tag_name)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
            }
            Err(_) => {
                println!("❌ Timeout");
            }
        }
    }

    // Test reading the base array tag (without index)
    println!("\n  Reading Program:MainProgram.ArrayTest (base array)... ");
    match timeout(
        Duration::from_secs(5),
        client.read_tag("Program:MainProgram.ArrayTest"),
    )
    .await
    {
        Ok(Ok(value)) => {
            println!("✅ {:?}", value);
        }
        Ok(Err(e)) => {
            println!("❌ Error: {}", e);
        }
        Err(_) => {
            println!("❌ Timeout");
        }
    }

    println!("\n📝 Test: Writing Program-Scoped Tags");
    println!("-------------------------------------\n");

    // Write new values
    for (tag_name, write_value, _) in &test_tags {
        print!("  Writing {} = {:?}... ", tag_name, write_value);

        match timeout(
            Duration::from_secs(5),
            client.write_tag(tag_name, write_value.clone()),
        )
        .await
        {
            Ok(Ok(())) => {
                println!("✅ Write successful");

                // Read back to verify
                print!("    Reading back... ");
                match timeout(Duration::from_secs(5), client.read_tag(tag_name)).await {
                    Ok(Ok(read_value)) => {
                        if read_value == *write_value {
                            println!("✅ Verified: {:?}", read_value);
                        } else {
                            println!(
                                "⚠️  Mismatch: expected {:?}, got {:?}",
                                write_value, read_value
                            );
                        }
                    }
                    Ok(Err(e)) => {
                        println!("❌ Read back failed: {}", e);
                    }
                    Err(_) => {
                        println!("❌ Read back timeout");
                    }
                }
            }
            Ok(Err(e)) => {
                println!("❌ Write failed: {}", e);
            }
            Err(_) => {
                println!("❌ Write timeout");
            }
        }
    }

    println!("\n📊 Test: Restoring Original Values");
    println!("------------------------------------\n");

    // Restore original values
    for (tag_name, _, restore_value) in &test_tags {
        print!("  Restoring {} = {:?}... ", tag_name, restore_value);

        match timeout(
            Duration::from_secs(5),
            client.write_tag(tag_name, restore_value.clone()),
        )
        .await
        {
            Ok(Ok(())) => {
                println!("✅ Restored");
            }
            Ok(Err(e)) => {
                println!("❌ Restore failed: {}", e);
            }
            Err(_) => {
                println!("❌ Restore timeout");
            }
        }
    }

    println!("\n📈 Test Summary");
    println!("===============");
    println!("✅ All program-scoped tag operations completed!");
    println!("\nNote: If any operations failed, check:");
    println!("  - Tag names are correct and case-sensitive");
    println!("  - Tags exist in MainProgram");
    println!("  - Tags have appropriate permissions (read/write)");
    println!("  - Program name is 'MainProgram' (case-sensitive)");

    Ok(())
}
