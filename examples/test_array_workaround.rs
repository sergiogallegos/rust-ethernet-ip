// examples/test_array_workaround.rs
// =========================================================================
//
// Array Element Workaround Test
//
// This example tests the workaround for reading array elements:
// - gArrayBoolTest (controller-scoped BOOL[32] array)
// - Program:MainProgram.ArrayTest (program-scoped DINT[10] array)
//
// Usage:
//   cargo run --example test_array_workaround -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_array_workaround -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 Array Element Workaround Test");
    println!("=================================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_array_workaround -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_array_workaround -- 192.168.0.1:44818");
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

    // Test 1: Controller-scoped BOOL array (gArrayBoolTest)
    println!("📊 Test 1: Controller-Scoped BOOL Array (gArrayBoolTest)");
    println!("--------------------------------------------------------\n");

    // First, read the base array to see what we get
    println!("  Reading base array 'gArrayBoolTest'...");
    match timeout(Duration::from_secs(5), client.read_tag("gArrayBoolTest")).await {
        Ok(Ok(value)) => {
            println!("  ✅ Base array read: {:?}", value);
            if let PlcValue::Udint(dword) = value {
                println!("     DWORD value: 0x{:08X} (binary: {:032b})", dword, dword);
            }
        }
        Ok(Err(e)) => {
            println!("  ❌ Base array read failed: {}", e);
        }
        Err(_) => {
            println!("  ❌ Base array read timeout");
        }
    }

    // Now test reading individual elements
    println!("\n  Reading individual BOOL array elements (indices 0-18):");
    let mut bool_success = 0;
    let mut bool_fail = 0;

    for index in 0..19 {
        let tag_path = format!("gArrayBoolTest[{}]", index);
        print!("    Reading {}... ", tag_path);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                bool_success += 1;
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                bool_fail += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                bool_fail += 1;
            }
        }
    }

    println!(
        "\n  BOOL Array Results: {} successful, {} failed",
        bool_success, bool_fail
    );

    // Test 2: Program-scoped DINT array (Program:MainProgram.ArrayTest)
    println!("\n📊 Test 2: Program-Scoped DINT Array (Program:MainProgram.ArrayTest)");
    println!("-------------------------------------------------------------------\n");

    // First, read the base array
    println!("  Reading base array 'Program:MainProgram.ArrayTest'...");
    match timeout(
        Duration::from_secs(5),
        client.read_tag("Program:MainProgram.ArrayTest"),
    )
    .await
    {
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

    // Now test reading individual elements
    println!("\n  Reading individual DINT array elements (indices 0-9):");
    let mut dint_success = 0;
    let mut dint_fail = 0;

    for index in 0..10 {
        let tag_path = format!("Program:MainProgram.ArrayTest[{}]", index);
        print!("    Reading {}... ", tag_path);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                dint_success += 1;
            }
            Ok(Err(e)) => {
                println!("❌ Error: {}", e);
                dint_fail += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                dint_fail += 1;
            }
        }
    }

    println!(
        "\n  DINT Array Results: {} successful, {} failed",
        dint_success, dint_fail
    );

    // Summary
    println!("\n📈 Test Summary");
    println!("===============");
    println!("  Controller-scoped BOOL array (gArrayBoolTest):");
    println!("    ✅ Successful: {}", bool_success);
    println!("    ❌ Failed: {}", bool_fail);
    println!("\n  Program-scoped DINT array (Program:MainProgram.ArrayTest):");
    println!("    ✅ Successful: {}", dint_success);
    println!("    ❌ Failed: {}", dint_fail);

    if bool_success > 0 && dint_success > 0 {
        println!("\n🎉 Array element workaround is working!");
    } else if bool_success == 0 && dint_success == 0 {
        println!("\n⚠️  Array element workaround needs investigation.");
    } else {
        println!("\n⚠️  Partial success - some arrays work, others don't.");
    }

    Ok(())
}
