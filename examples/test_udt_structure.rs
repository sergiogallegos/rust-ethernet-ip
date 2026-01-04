// examples/test_udt_structure.rs
// =========================================================================
//
// UDT Structure Reading Test
//
// This example tests reading a UDT (User Defined Type) structure tag:
// - FIS_RT (controller-scoped UDT structure)
//
// Usage:
//   cargo run --example test_udt_structure -- <PLC_IP:PORT>
//
// Example:
//   cargo run --example test_udt_structure -- 192.168.0.1:44818
//
// =========================================================================

use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧪 UDT Structure Reading Test");
    println!("===============================\n");

    // Parse command line arguments
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("Usage: cargo run --example test_udt_structure -- <PLC_IP:PORT>");
        println!("\nExample:");
        println!("  cargo run --example test_udt_structure -- 192.168.0.1:44818");
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

    // Test reading the UDT structure
    let udt_tag = "FIS_RT";

    println!("📊 Test: Reading UDT Structure");
    println!("-------------------------------\n");

    println!("  Reading {} (controller-scoped UDT)...", udt_tag);

    match timeout(Duration::from_secs(10), client.read_tag(udt_tag)).await {
        Ok(Ok(value)) => {
            println!("  ✅ Successfully read UDT structure!");
            println!("\n  Structure value:");
            println!("  {:?}", value);

            // If it's a UDT, show UdtData information
            if let PlcValue::Udt(udt_data) = &value {
                println!("\n  UDT Data:");
                println!("    Symbol ID: {}", udt_data.symbol_id);
                println!("    Data Size: {} bytes", udt_data.data.len());
                println!("    Data Preview: {:02X?}", &udt_data.data[..udt_data.data.len().min(32)]);
                if udt_data.data.len() > 32 {
                    println!("    ... ({} more bytes)", udt_data.data.len() - 32);
                }
            }
        }
        Ok(Err(e)) => {
            println!("  ❌ Error reading UDT: {}", e);
            return Err(e.into());
        }
        Err(_) => {
            println!("  ❌ Timeout reading UDT");
            return Err("Timeout".into());
        }
    }

    // Test reading individual UDT members
    println!("\n📊 Test: Reading Individual UDT Members");
    println!("----------------------------------------\n");

    let udt_members = vec![
        "FIS_RT.State",
        "FIS_RT.State_Prev",
        "FIS_RT.Busy",
        "FIS_RT.Done",
        "FIS_RT.Error",
        "FIS_RT.Error_Code",
        "FIS_RT.Retry_Count",
        "FIS_RT.Connected",
        "FIS_RT.BCNF_Pass",
        "FIS_RT.BCNF_Fail",
        "FIS_RT.BACK_Pass",
        "FIS_RT.BACK_Fail",
        "FIS_RT.TX_Buffer",
        "FIS_RT.RX_Buffer",
        "FIS_RT.TX_Len",
        "FIS_RT.RX_Len",
        "FIS_RT.Sequence",
    ];

    let mut success_count = 0;
    let mut fail_count = 0;

    for member_path in &udt_members {
        print!("  Reading {}... ", member_path);

        match timeout(Duration::from_secs(5), client.read_tag(member_path)).await {
            Ok(Ok(value)) => {
                println!("✅ {:?}", value);
                success_count += 1;
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

    println!("\n📈 Test Summary");
    println!("===============");
    println!("  UDT structure read: ✅");
    println!(
        "  Individual members: {} successful, {} failed",
        success_count, fail_count
    );

    if success_count == udt_members.len() {
        println!("\n🎉 All UDT operations completed successfully!");
    } else if fail_count > 0 {
        println!("\n⚠️  Some UDT member reads failed.");
    }

    Ok(())
}
