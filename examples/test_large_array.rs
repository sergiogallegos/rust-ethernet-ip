use rust_ethernet_ip::{EipClient, PlcValue};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: {} <plc_address:port>", args[0]);
        eprintln!("Example: {} 192.168.0.1:44818", args[0]);
        std::process::exit(1);
    }

    let address = &args[1];
    println!("🧪 Large Array (50 elements) Read and Write Test");
    println!("{}", "=".repeat(60));
    println!();

    println!("📡 Connecting to PLC at {}", address);
    let mut client = EipClient::connect(address).await?;
    println!("✅ Connected successfully!");
    println!();

    let array_name = "gArrayRealTest";
    let array_size = 50;

    println!(
        "📊 Test: REAL Array ({} elements) Write and Read",
        array_size
    );
    println!("{}", "=".repeat(60));
    println!();

    // Step 1: Write values to array elements
    println!("  Step 1: Writing values to REAL array elements...");
    let mut write_success = 0;
    let mut write_failed = 0;

    for i in 0..array_size {
        let value = (i as f32) * 1.11 + 1.0; // Values: 1.0, 2.11, 3.22, 4.33, ...
        let tag_name = format!("{}[{}]", array_name, i);

        print!("    Writing {} = {:.2}... ", tag_name, value);
        match client.write_tag(&tag_name, PlcValue::Real(value)).await {
            Ok(_) => {
                println!("✅");
                write_success += 1;
            }
            Err(e) => {
                println!("❌ Error: {}", e);
                write_failed += 1;
            }
        }
    }

    println!();
    println!("  Write Results:");
    println!("    ✅ Successful: {}", write_success);
    println!("    ❌ Failed: {}", write_failed);
    println!();

    // Step 2: Read values back and verify
    println!("  Step 2: Reading values back from REAL array elements...");
    let mut read_success = 0;
    let mut read_failed = 0;
    let mut read_mismatch = 0;

    for i in 0..array_size {
        let expected_value = (i as f32) * 1.11 + 1.0;
        let tag_name = format!("{}[{}]", array_name, i);

        print!(
            "    Reading {} (expected {:.2})... ",
            tag_name, expected_value
        );
        match client.read_tag(&tag_name).await {
            Ok(value) => match value {
                PlcValue::Real(actual) => {
                    let diff = (actual - expected_value).abs();
                    if diff < 0.01 {
                        println!("✅ Got {:.2}", actual);
                        read_success += 1;
                    } else {
                        println!(
                            "⚠️  Got {:.2} (expected {:.2}, diff: {:.2})",
                            actual, expected_value, diff
                        );
                        read_mismatch += 1;
                    }
                }
                _ => {
                    println!("❌ Wrong type: {:?}", value);
                    read_failed += 1;
                }
            },
            Err(e) => {
                println!("❌ Error: {}", e);
                read_failed += 1;
            }
        }
    }

    println!();
    println!("  Read Results:");
    println!("    ✅ Successful: {}", read_success);
    println!("    ⚠️  Mismatch: {}", read_mismatch);
    println!("    ❌ Failed: {}", read_failed);
    println!();

    // Summary
    println!("📈 Test Summary");
    println!("{}", "=".repeat(60));
    if write_success == array_size && read_success == array_size {
        println!(
            "🎉 All {} array operations completed successfully!",
            array_size * 2
        );
    } else {
        println!("⚠️  Some array operations had issues:");
        if write_failed > 0 {
            println!("   - {} write operations failed", write_failed);
        }
        if read_failed > 0 {
            println!("   - {} read operations failed", read_failed);
        }
        if read_mismatch > 0 {
            println!("   - {} read values didn't match expected", read_mismatch);
        }
    }

    Ok(())
}
