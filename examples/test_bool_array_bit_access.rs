// examples/test_bool_array_bit_access.rs
// =========================================================================
//
// BOOL Array Bit Access Test
//
// This example tests reading BOOL array elements using bit notation
// instead of array indexing, since BOOL arrays are stored as DWORDs:
// - gArrayBoolTest.0, gArrayBoolTest.1, etc. (bit notation)
// vs
// - gArrayBoolTest[0], gArrayBoolTest[1] (array indexing - fails)
//
// Usage:
//   cargo run --example test_bool_array_bit_access -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_bool_array_bit_access -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{Duration, timeout};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 BOOL Array Bit Access Test");
    println!("==============================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_bool_array_bit_access -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_bool_array_bit_access -- 192.168.0.1:44818");
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

    // Test 1: Read base array
    println!("📊 Test 1: Reading Base BOOL Array");
    println!("-----------------------------------\n");

    println!("  Reading {} (base array)...", array_name);
    match timeout(Duration::from_secs(5), client.read_tag(array_name)).await {
        Ok(Ok(value)) => {
            println!("  ✅ Base array read: {:?}", value);
            if let PlcValue::Udint(dword) = value {
                println!("     DWORD value: 0x{:08X} (binary: {:032b})", dword, dword);
                println!("     This represents 32 BOOL values as bits");
            }
        }
        Ok(Err(e)) => {
            println!("  ❌ Base array read failed: {}", e);
        }
        Err(_) => {
            println!("  ❌ Base array read timeout");
        }
    }

    // Test 2: Try array indexing (should fail)
    println!("\n📊 Test 2: Array Indexing (gArrayBoolTest[0], etc.)");
    println!("---------------------------------------------------\n");

    let mut array_index_success = 0;
    let mut array_index_fail = 0;

    for index in 0..5 {
        let tag_path = format!("{}[{}]", array_name, index);
        print!("  Reading {}... ", tag_path);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                array_index_success += 1;
            }
            Ok(Err(e)) => {
                println!("❌ {}", e);
                array_index_fail += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                array_index_fail += 1;
            }
        }
    }

    // Test 3: Try bit notation (gArrayBoolTest.0, etc.)
    println!("\n📊 Test 3: Bit Notation (gArrayBoolTest.0, etc.)");
    println!("------------------------------------------------\n");

    let mut bit_notation_success = 0;
    let mut bit_notation_fail = 0;

    for bit_index in 0..19 {
        let tag_path = format!("{}.{}", array_name, bit_index);
        print!("  Reading {}... ", tag_path);

        match timeout(Duration::from_secs(5), client.read_tag(&tag_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                bit_notation_success += 1;
            }
            Ok(Err(e)) => {
                println!("❌ {}", e);
                bit_notation_fail += 1;
            }
            Err(_) => {
                println!("❌ Timeout");
                bit_notation_fail += 1;
            }
        }
    }

    println!("\n📈 Test Summary");
    println!("===============");
    println!("  Base array read: ✅");
    println!(
        "  Array indexing ([]): {} successful, {} failed",
        array_index_success, array_index_fail
    );
    println!(
        "  Bit notation (.): {} successful, {} failed",
        bit_notation_success, bit_notation_fail
    );

    if bit_notation_success > 0 {
        println!("\n💡 BOOL arrays may need to be accessed using bit notation (.0, .1, etc.)");
        println!("   instead of array indexing ([0], [1], etc.)");
    }

    Ok(())
}
